//! Compilation du catalogue en artefact statique (ADR 0027, ADR 0029).
//!
//! L'artefact reprend l'arborescence du préfixe `staging/` du bucket :
//!
//! - `r/<sha256>.json` : ressources (cartes, recettes, exclusions, trompe-l'œil) ;
//! - `a/<sha256>.<ext>` : médias (silhouettes des cartes et des langues, jaquette) ;
//! - `f/<sha256>.json` : un manifeste par fascicule ;
//! - `parutions.json` : calendrier des parutions, seul fichier modifiable. Il
//!   porte les référentiels et, pour chaque fascicule, sa date et son manifeste.
//!
//! Le calendrier et l'index ne citent que des manifestes ; seul le manifeste cite
//! ses ressources (voir `ressources`). À la date d'effet, la tâche de parution
//! lit le manifeste dans `staging/`, copie ses ressources puis le manifeste dans
//! `public/`, et écrit `public/index.json` en dernier.
//!
//! Chaque fichier haché est nommé d'après l'empreinte SHA-256 de son contenu :
//! un fichier ne change jamais, et un contenu identique garde le même chemin.

use std::cell::OnceCell;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

use crate::chargement::{Catalogue, Table};
use crate::homographes;
use crate::json::{CANONIQUE, a_cle, ecrire, liste, texte, vrai};
use crate::medias::{self, CleMedia};

pub const FORMAT: i64 = 1;

/// Champs éditoriaux jamais publiés (modèle de données, carte de répartition).
pub const CHAMPS_PRIVES: [&str; 2] = ["note_interne", "etat_relecture"];

/// JSON canonique : clés triées, sans espace, UTF-8. Deux contenus égaux ont la même empreinte.
#[must_use]
pub fn serialiser(objet: &Value) -> Vec<u8> {
    ecrire(objet, CANONIQUE).into_bytes()
}

#[must_use]
pub fn empreinte(contenu: &[u8]) -> String {
    Sha256::digest(contenu)
        .iter()
        .fold(String::with_capacity(64), |mut s, o| {
            let _ = write!(s, "{o:02x}");
            s
        })
}

#[must_use]
pub fn sans_prive(objet: &Value) -> Value {
    match objet {
        Value::Object(o) => Value::Object(
            o.iter()
                .filter(|(c, _)| !CHAMPS_PRIVES.contains(&c.as_str()))
                .map(|(c, v)| (c.clone(), sans_prive(v)))
                .collect(),
        ),
        Value::Array(a) => Value::Array(a.iter().map(sans_prive).collect()),
        v => v.clone(),
    }
}

/// Un fait non retenu n'est jamais publié, même en mode d'essai.
fn retenu(objet: &Value) -> bool {
    objet["fait"]["confiance"] != "non_retenue"
}

fn tries(table: &Table) -> Vec<(&String, &Value)> {
    let mut paires: Vec<_> = table.iter().collect();
    paires.sort_by(|a, b| a.0.cmp(b.0));
    paires
}

#[derive(Default)]
pub struct Artefact {
    pub fichiers: BTreeMap<String, Vec<u8>>,
}

impl Artefact {
    pub fn deposer(&mut self, dossier: &str, objet: &Value) -> String {
        let contenu = serialiser(objet);
        let chemin = format!("{dossier}/{}.json", empreinte(&contenu));
        self.fichiers.insert(chemin.clone(), contenu);
        chemin
    }

    /// Nom tiré des seuls octets : jamais le mot, ni l'identifiant d'une carte (ADR 0025).
    pub fn deposer_media(&mut self, contenu: Vec<u8>, extension: &str) -> String {
        let chemin = format!("a/{}{extension}", empreinte(&contenu));
        self.fichiers.insert(chemin.clone(), contenu);
        chemin
    }
}

/// Chemins des ressources citées par un manifeste, triés et sans doublon.
#[must_use]
pub fn ressources(manifeste: &Value) -> Vec<String> {
    let mut chemins = vec![
        &manifeste["jaquette"],
        &manifeste["exclusions"],
        &manifeste["trompes_l_oeil"],
    ];
    for carte in liste(&manifeste["cartes"]) {
        chemins.extend([&carte["carte"], &carte["silhouette"]]);
    }
    chemins.extend(
        liste(&manifeste["langues"])
            .iter()
            .map(|l| &l["silhouette"]),
    );
    chemins.extend(liste(&manifeste["recettes"]).iter().map(|r| &r["chemin"]));
    chemins
        .into_iter()
        .filter(|c| vrai(c))
        .map(|c| texte(c).to_owned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

struct Compilateur<'a> {
    c: &'a Catalogue,
    essai: bool,
    version: &'a str,
    artefact: Artefact,
    brique_par_unite: HashMap<&'a str, &'a Value>,
    medias: HashMap<CleMedia, &'a Value>,
    classes: OnceCell<(HashMap<String, String>, HashMap<String, String>)>,
}

/// Même texte pour deux cartes : même classe, donc jamais deux réponses justes (ADR 0044).
// ponytail: `casefold` de Python replié à la main (ß seul) ; un autre repli spécial (ς, ŉ…) demanderait une table.
fn texte_normalise(texte: &str) -> Option<String> {
    if texte.trim().is_empty() {
        return None;
    }
    let bas = texte
        .nfc()
        .collect::<String>()
        .to_lowercase()
        .replace('ß', "ss");
    Some(bas.split_whitespace().collect::<Vec<_>>().join(" "))
}

/// Identifiant → classe : le plus petit identifiant parmi les textes égaux (aucun texte publié).
fn classes(textes: &BTreeMap<&str, &str>) -> HashMap<String, String> {
    let mut premier: HashMap<String, &str> = HashMap::new();
    let mut classes = HashMap::new();
    for (ident, texte) in textes {
        if let Some(norme) = texte_normalise(texte) {
            let classe = premier.entry(norme).or_insert(ident);
            classes.insert((*ident).to_owned(), (*classe).to_owned());
        }
    }
    classes
}

impl<'a> Compilateur<'a> {
    fn new(c: &'a Catalogue, essai: bool, version: &'a str) -> Self {
        let brique_par_unite = c
            .briques
            .values()
            .map(|b| (texte(&b["unite_id"]), b))
            .collect();
        Self {
            c,
            essai,
            version,
            artefact: Artefact::default(),
            brique_par_unite,
            medias: medias::par_cible(c),
            classes: OnceCell::new(),
        }
    }

    // ------------------------------------------------------------------ médias
    fn media(
        &mut self,
        type_media: &str,
        nature: &str,
        identifiant: &str,
    ) -> Result<Value, String> {
        let Some(media) = self.medias.get(&(
            type_media.to_owned(),
            nature.to_owned(),
            identifiant.to_owned(),
        )) else {
            return Ok(Value::Null);
        };
        let fichier = medias::chemin(self.c, media);
        let contenu = fs::read(&fichier)
            .ok()
            .filter(|_| fichier.is_file())
            .ok_or_else(|| {
                format!(
                    "Média {} : fichier {}/{} introuvable.",
                    texte(&media["id"]),
                    medias::DOSSIER,
                    texte(&media["fichier"])
                )
            })?;
        let extension = fichier
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default();
        Ok(Value::String(
            self.artefact.deposer_media(contenu, &extension),
        ))
    }

    // ---------------------------------------------------------------- sources
    fn sources_citees(objet: &Value, trouvees: &mut BTreeSet<String>) {
        match objet {
            Value::Object(o) => {
                if let Some(Value::String(s)) = o.get("source_id") {
                    trouvees.insert(s.clone());
                }
                o.values().for_each(|v| Self::sources_citees(v, trouvees));
            }
            Value::Array(a) => a.iter().for_each(|v| Self::sources_citees(v, trouvees)),
            _ => {}
        }
    }

    fn avec_sources(&self, mut carte: Value) -> Value {
        let mut ids = BTreeSet::new();
        Self::sources_citees(&carte, &mut ids);
        let sources: Map<String, Value> = ids
            .into_iter()
            .filter_map(|sid| self.c.sources.get(&sid).map(|s| (sid, sans_prive(s))))
            .collect();
        carte["sources"] = Value::Object(sources);
        carte
    }

    // ------------------------------------------------------------------ cartes
    fn etymon(&self, uid: &str) -> Value {
        let unite = &self.c.unites[uid];
        let canonique = liste(&unite["formes"])
            .iter()
            .find(|f| f["role"] == "canonique")
            .cloned()
            .unwrap_or(Value::Null);
        sans_prive(&json!({
            "id": uid,
            "nature": unite["nature"],
            "langue_id": unite["langue_id"],
            "forme": canonique,
            "sens": liste(&unite["sens"]).iter().filter(|s| retenu(s)).collect::<Vec<_>>(),
            // Maillons suivants de la chaîne ; le graphe est acyclique (contrôle `valider`).
            "relations": self.relations_entrantes(uid),
            "compositions": self.compositions_retenues(uid),
        }))
    }

    fn relations_entrantes(&self, uid: &str) -> Vec<Value> {
        tries(&self.c.relations)
            .into_iter()
            .filter(|(_, r)| {
                r["unite_cible_id"] == uid
                    && retenu(r)
                    && self.c.unites.contains_key(texte(&r["unite_source_id"]))
            })
            .map(|(_, r)| {
                let mut r = r.clone();
                r["etymon"] = self.etymon(texte(&r["unite_source_id"]));
                r
            })
            .collect()
    }

    fn compositions_retenues(&self, uid: &str) -> Vec<Value> {
        tries(&self.c.compositions)
            .into_iter()
            .filter(|(_, c)| c["mot_id"] == uid && retenu(c))
            .map(|(_, c)| c.clone())
            .collect()
    }

    fn carte(&self, uid: &str) -> Value {
        let unite = &self.c.unites[uid];
        let mut carte: Map<String, Value> = unite
            .as_object()
            .into_iter()
            .flatten()
            .filter(|(c, _)| c.as_str() != "formes" && c.as_str() != "sens")
            .map(|(c, v)| (c.clone(), v.clone()))
            .collect();
        carte.insert(
            "formes".into(),
            liste(&unite["formes"])
                .iter()
                .filter(|f| retenu(f))
                .cloned()
                .collect(),
        );
        carte.insert(
            "sens".into(),
            liste(&unite["sens"])
                .iter()
                .filter(|s| retenu(s))
                .cloned()
                .collect(),
        );
        let relations = self.relations_entrantes(uid);
        carte.insert("relations".into(), relations.into());
        if !unite.get("attestation").is_some_and(retenu) {
            carte.remove("attestation");
        }
        if unite["nature"] == "mot" {
            carte.insert(
                "compositions".into(),
                self.compositions_retenues(uid).into(),
            );
        }
        if let Some(brique) = self.brique_par_unite.get(uid) {
            let extrait: Map<String, Value> =
                ["id", "forme_affichee_id", "sens_affiche_id", "rarete"]
                    .into_iter()
                    .map(|c| (c.to_owned(), brique[c].clone()))
                    .collect();
            carte.insert("brique".into(), Value::Object(extrait));
        }
        self.avec_sources(sans_prive(&Value::Object(carte)))
    }

    // ---------------------------------------------------------------- recettes
    fn resultat(&self, recette: &Value) -> &'a str {
        texte(&self.c.compositions[texte(&recette["composition_id"])]["mot_id"])
    }

    fn recette(&self, rid: &str) -> Value {
        let mut recette = sans_prive(&self.c.recettes[rid]);
        recette["resultat_id"] = Value::String(self.resultat(&recette).to_owned());
        recette
    }

    // --------------------------------------------------------------- ricochets
    fn composition_preferee(&self, mot_id: &str) -> Option<&'a Value> {
        let candidates: Vec<&Value> = tries(&self.c.compositions)
            .into_iter()
            .map(|(_, c)| c)
            .filter(|c| c["mot_id"] == mot_id && vrai(&c["preferee"]) && retenu(c))
            .collect();
        candidates
            .iter()
            .find(|c| c["type_analyse"] == "synchronique")
            .or(candidates.first())
            .copied()
    }

    /// (classe de sens par sens_id de brique, classe de sens littéral par mot), sur tout le catalogue.
    fn classes_ricochets(&self) -> &(HashMap<String, String>, HashMap<String, String>) {
        self.classes.get_or_init(|| {
            let glose: BTreeMap<&str, &str> = self
                .c
                .sens
                .iter()
                .map(|(sid, (_, s))| (sid.as_str(), texte(&s["libelles"]["fr"]["glose"])))
                .collect();
            let litteral: BTreeMap<&str, &str> = self
                .c
                .unites
                .iter()
                .filter(|(_, u)| u["nature"] == "mot")
                .map(|(mid, _)| {
                    let cmp = self.composition_preferee(mid);
                    (
                        mid.as_str(),
                        cmp.map_or("", |c| texte(&c["libelles"]["fr"]["sens_litteral"])),
                    )
                })
                .collect();
            (classes(&glose), classes(&litteral))
        })
    }

    /// Frontières entre les parties, si les segments pavent la forme résultat sans trou ; sinon aucune.
    fn coupes(&self, composition: &Value) -> Vec<i64> {
        let Some((_, forme)) = self.c.formes.get(texte(&composition["forme_resultat_id"])) else {
            return Vec::new();
        };
        let longueur = texte(&forme["texte"]).nfc().count();
        let mut parties: Vec<&Value> = liste(&composition["parties"]).iter().collect();
        parties.sort_by_key(|p| p["rang"].as_i64());
        let mut debut = 0;
        for partie in &parties {
            match (
                partie["segment_debut"].as_i64(),
                partie["segment_fin"].as_i64(),
            ) {
                (Some(d), Some(fin)) if d == debut && fin > debut => debut = fin,
                _ => return Vec::new(),
            }
        }
        if usize::try_from(debut) != Ok(longueur) {
            return Vec::new();
        }
        parties
            .iter()
            .take(parties.len().saturating_sub(1))
            .filter_map(|p| p["segment_fin"].as_i64())
            .collect()
    }

    /// Les deux briques de la composition préférée, préfixe puis suffixe, quand le mot s'assemble
    /// (ADR 0067) ; vide sinon. Les deux briques doivent figurer parmi celles des ricochets.
    fn assemblage(&self, mot_id: &str, briques: &[Value]) -> Vec<String> {
        let Some(composition) = self.composition_preferee(mot_id) else {
            return Vec::new();
        };
        let mut parties: Vec<&Value> = liste(&composition["parties"]).iter().collect();
        parties.sort_by_key(|p| p["rang"].as_i64());
        let [devant, derriere] = parties.as_slice() else {
            return Vec::new();
        };
        let choisie = |partie: &Value, position: &str| {
            let brique = self.brique_par_unite.get(texte(&partie["unite_id"]))?;
            briques
                .iter()
                .find(|b| b["id"] == brique["id"] && b["position"] == position)
                .map(|b| texte(&b["id"]).to_owned())
        };
        match (choisie(devant, "prefixe"), choisie(derriere, "suffixe")) {
            (Some(prefixe), Some(suffixe)) => vec![prefixe, suffixe],
            _ => Vec::new(),
        }
    }

    /// De quoi juger un ricochet sans lire un texte (ADR 0044) : sens affiché et classes de sens des
    /// briques, classe du sens littéral et coupes des mots. Jamais de légendaire ni de mot qui en dépend.
    fn ricochets(&self, resultats: &BTreeSet<&str>, visibles: &[&String]) -> Value {
        let (classes_sens, classes_litteral) = self.classes_ricochets();
        let mut tries_visibles: Vec<&str> = visibles.iter().map(|u| u.as_str()).collect();
        tries_visibles.sort_unstable();
        let mut briques = Vec::new();
        for uid in &tries_visibles {
            let Some(brique) = self.brique_par_unite.get(uid) else {
                continue;
            };
            let nature = texte(&self.c.unites[*uid]["nature"]);
            let sens = texte(&brique["sens_affiche_id"]);
            if matches!(nature, "prefixe" | "suffixe")
                && let Some(classe) = classes_sens.get(sens)
            {
                briques.push(json!({"id": brique["id"], "sens_id": sens, "classe_sens": classe, "position": nature}));
            }
        }
        briques.sort_by(|a, b| texte(&a["id"]).cmp(texte(&b["id"])));
        let mut mots = Vec::new();
        for uid in tries_visibles {
            if !resultats.contains(uid) || self.c.unites[uid]["nature"] != "mot" {
                continue;
            }
            let coupes = self
                .composition_preferee(uid)
                .map(|c| self.coupes(c))
                .unwrap_or_default();
            let assemblage = self.assemblage(uid, &briques);
            let mut mot = json!({"id": uid, "coupes": coupes});
            if !assemblage.is_empty() {
                mot["assemblage"] = json!(assemblage);
            }
            let classe = classes_litteral.get(uid);
            if let Some(classe) = classe {
                mot["classe_litteral"] = json!(classe);
            }
            if classe.is_some()
                || !assemblage.is_empty()
                || !mot["coupes"].as_array().is_some_and(Vec::is_empty)
            {
                mots.push(mot);
            }
        }
        json!({"briques": briques, "mots": mots})
    }

    // --------------------------------------------------------------- fascicule
    fn legendaire(&self, uid: &str) -> bool {
        self.brique_par_unite
            .get(uid)
            .is_some_and(|b| b["rarete"] == "legendaire")
    }

    fn depend_de_legendaire(&self, uid: &str, recettes: &[&String]) -> bool {
        recettes
            .iter()
            .map(|rid| &self.c.recettes[rid.as_str()])
            .filter(|r| self.resultat(r) == uid)
            .any(|r| {
                liste(&r["emplacements"]).iter().any(|e| {
                    liste(&e["briques_acceptees"])
                        .iter()
                        .filter_map(|b| self.c.briques.get(texte(b)))
                        .any(|b| self.legendaire(texte(&b["unite_id"])))
                })
            })
    }

    /// Unités déclarées par ce fascicule et par ceux qui le précèdent (parution, puis numéro).
    fn unites_jusqua(&self, fid: &str) -> BTreeSet<&'a str> {
        let c = self.c;
        let cle = |f: &Value| (texte(&f["parution"]).to_owned(), f["numero"].as_i64());
        let borne = cle(&c.fascicules[fid]);
        c.fascicules
            .values()
            .filter(|f| (self.essai || !vrai(&f["test"])) && cle(f) <= borne)
            .flat_map(|f| liste(&f["unites"]).iter().map(|u| texte(&u["unite_id"])))
            .collect()
    }

    /// Fausses fusions du fascicule (ADR 0052), sans fait ni note : seulement ce que le joueur lit.
    /// Une brique légendaire n'y figure jamais : un trompe-l'œil trahirait son existence.
    fn trompes_l_oeil(&self, fid: &str) -> Vec<Value> {
        tries(&self.c.trompes_l_oeil)
            .into_iter()
            .filter(|(_, t)| {
                t["fascicule_id"] == fid
                    && retenu(t)
                    && (self.essai || t["fait"]["etat_relecture"] == "relu")
                    && !liste(&t["combinaison"]).iter().any(|b| {
                        self.c
                            .briques
                            .get(texte(b))
                            .is_some_and(|b| b["rarete"] == "legendaire")
                    })
            })
            .map(|(tid, t)| {
                let mut publie = json!({
                    "id": tid,
                    "combinaison": t["combinaison"],
                    "motif": t["motif"],
                    "explication_courte": t["explication_courte"],
                    "explication": t["explication"],
                });
                if a_cle(t, "mot_id") {
                    publie["mot_id"] = t["mot_id"].clone();
                }
                publie
            })
            .collect()
    }

    /// Groupes de briques de même forme que ce fascicule fait connaître (ADR 0052) : tous les membres
    /// déjà publiés, hors légendaires, pour qu'au moins l'un d'eux soit nouveau.
    fn homographes(&self, fid: &str) -> Vec<Vec<String>> {
        let c = self.c;
        let connues = self.unites_jusqua(fid);
        let nouvelles: BTreeSet<&str> = liste(&c.fascicules[fid]["unites"])
            .iter()
            .map(|u| texte(&u["unite_id"]))
            .collect();
        let publiee = |bid: &String| {
            c.briques.get(bid).is_some_and(|b| {
                b["rarete"] != "legendaire" && connues.contains(texte(&b["unite_id"]))
            })
        };
        homographes::groupes(c)
            .into_iter()
            .map(|g| g.into_iter().filter(|b| publiee(b)).collect::<Vec<_>>())
            .filter(|g| {
                g.len() > 1
                    && g.iter()
                        .any(|b| nouvelles.contains(texte(&c.briques[b]["unite_id"])))
            })
            .collect()
    }

    fn fascicule(&mut self, fid: &str) -> Result<Value, String> {
        let c = self.c;
        let fsc = &c.fascicules[fid];
        let entrees = liste(&fsc["unites"]);
        let uids: Vec<String> = entrees
            .iter()
            .map(|u| texte(&u["unite_id"]).to_owned())
            .collect();
        let resultats: BTreeSet<&str> = entrees
            .iter()
            .filter(|u| u["role"] == "resultat")
            .map(|u| texte(&u["unite_id"]))
            .collect();

        let mut cartes = Vec::new();
        for entree in entrees {
            let uid = texte(&entree["unite_id"]);
            let chemin = self.artefact.deposer("r", &self.carte(uid));
            let silhouette = self.media("silhouette", "unite", uid)?;
            let mut carte = json!({"id": uid, "role": entree["role"], "carte": chemin, "silhouette": silhouette});
            if a_cle(entree, "difficulte") {
                carte["difficulte"] = entree["difficulte"].clone();
            }
            cartes.push(carte);
        }
        let mut langues = Vec::new();
        for lid in medias::langues_citees(c, &uids) {
            let silhouette = self.media("silhouette", "langue", &lid)?;
            langues.push(json!({"id": lid, "silhouette": silhouette}));
        }

        let mut ids_recettes: Vec<&String> = c
            .recettes
            .iter()
            .filter(|(_, r)| resultats.contains(self.resultat(r)))
            .map(|(rid, _)| rid)
            .collect();
        ids_recettes.sort();
        let mut recettes = Vec::new();
        for rid in &ids_recettes {
            let chemin = self.artefact.deposer("r", &self.recette(rid));
            recettes.push(json!({"id": rid, "chemin": chemin}));
        }

        let exclusions: Vec<Value> = tries(&c.exclusions)
            .into_iter()
            .filter(|(_, e)| e["fascicule_id"] == fid && (self.essai || e["etat_relecture"] == "relu"))
            .map(|(eid, e)| json!({"id": eid, "combinaison": e["combinaison"], "raison": e["raison"]}))
            .collect();
        let chemin_exclusions = self
            .artefact
            .deposer("r", &json!({"exclusions": exclusions}));

        let trompes_l_oeil = self.trompes_l_oeil(fid);
        let homographes = self.homographes(fid);

        // Compteurs hors légendaires et hors mots qui en dépendent (ADR 0015).
        let visibles: Vec<&String> = uids
            .iter()
            .filter(|u| !self.legendaire(u) && !self.depend_de_legendaire(u, &ids_recettes))
            .collect();
        let nature = |u: &str| texte(&c.unites[u]["nature"]);
        let compte = |n: &str| visibles.iter().filter(|u| nature(u) == n).count();
        let compteurs = json!({
            "mots": visibles.iter().filter(|u| resultats.contains(u.as_str()) && nature(u) == "mot").count(),
            "prefixes": compte("prefixe"),
            "suffixes": compte("suffixe"),
            "langues": visibles.iter().map(|u| texte(&c.unites[u.as_str()]["langue_id"])).collect::<BTreeSet<_>>().len(),
        });

        // Seuls les préfixes et les suffixes se tirent, jamais un mot (ADR 0035).
        let mut briques: Vec<Value> = uids
            .iter()
            .filter(|u| matches!(nature(u), "prefixe" | "suffixe"))
            .filter_map(|u| self.brique_par_unite.get(u.as_str()).map(|b| json!({"id": b["id"], "unite_id": u, "nature": nature(u), "rarete": b["rarete"]})))
            .collect();
        briques.sort_by(|a, b| texte(&a["id"]).cmp(texte(&b["id"])));

        // Mot posable comme brique (résultat ou ingrédient) : elle ne s'use jamais, et la découverte
        // d'un mot résultat en offre une (il ne sort d'aucun pli).
        let mut mots_briques: Vec<Value> = uids
            .iter()
            .filter(|u| nature(u) == "mot")
            .filter_map(|u| {
                self.brique_par_unite
                    .get(u.as_str())
                    .map(|b| json!({"mot_id": u, "brique_id": b["id"]}))
            })
            .collect();
        mots_briques.sort_by(|a, b| texte(&a["mot_id"]).cmp(texte(&b["mot_id"])));

        let mut jalons: Vec<&Value> = liste(&fsc["jalons"]).iter().collect();
        jalons.sort_by_key(|j| j["decouvertes"].as_i64());
        let jalons: Vec<Value> = jalons
            .into_iter()
            .map(|j| {
                let mut pli: Vec<&Value> = liste(&j["pli"]).iter().collect();
                pli.sort_by(|a, b| texte(a).cmp(texte(b)));
                json!({"id": j["id"], "decouvertes": j["decouvertes"], "pli": pli, "sabliers": j["sabliers"]})
            })
            .collect();
        let mut graines: Vec<&Value> = liste(&fsc["graines"]).iter().collect();
        graines.sort_by(|a, b| texte(&a["brique_id"]).cmp(texte(&b["brique_id"])));

        let jaquette = self.media("jaquette", "fascicule", fid)?;
        let mut manifeste = json!({
            "format": FORMAT,
            // Identifiant opaque : les données du joueur désignent le fascicule par lui.
            "id": fid,
            "numero": fsc["numero"],
            "nom": fsc["nom"],
            "parution": fsc["parution"],
            "jaquette": jaquette,
            "compteurs": compteurs,
            // Briques tirables et paramètres d'équilibrage du pli (ADR 0009, 0011).
            "pli": {"briques": briques, "equilibrage": c.equilibrage},
            "cartes": cartes,
            "langues": langues,
            "recettes": recettes,
            "exclusions": chemin_exclusions,
            // Jalons, récompenses déterministes affichées à l'avance (ADR 0038).
            "jalons": jalons,
            // Réserve de départ, accordée une fois à chaque joueur (ADR 0030).
            "graines": graines,
            "mots_briques": mots_briques,
            // Données de jugement des ricochets, sans aucun texte (ADR 0044).
            "ricochets": self.ricochets(&resultats, &visibles),
        });
        // Absents quand ils sont vides : un fascicule sans fausse piste garde son empreinte.
        if !trompes_l_oeil.is_empty() {
            manifeste["trompes_l_oeil"] = json!(
                self.artefact
                    .deposer("r", &json!({"trompes_l_oeil": trompes_l_oeil}))
            );
        }
        if !homographes.is_empty() {
            manifeste["homographes"] = json!(homographes);
        }
        if vrai(&fsc["test"]) {
            manifeste["test"] = Value::Bool(true);
        }
        let chemin_manifeste = self.artefact.deposer("f", &manifeste);
        Ok(
            json!({"numero": fsc["numero"], "parution": fsc["parution"], "manifeste": chemin_manifeste}),
        )
    }

    // ----------------------------------------------------------------- global
    fn referentiels(&self) -> Value {
        let publie = |t: &Table| {
            tries(t)
                .into_iter()
                .map(|(_, v)| sans_prive(v))
                .collect::<Vec<_>>()
        };
        json!({
            "langues": publie(&self.c.langues),
            "systemes_ecriture": publie(&self.c.ecritures),
            "conventions": publie(&self.c.conventions),
            // Noms des relations et des transformations, pour la section « En savoir plus ».
            "types_relation": publie(&self.c.types_relation),
            "types_transformation": publie(&self.c.types_transformation),
        })
    }

    fn compiler(mut self) -> Result<Artefact, String> {
        let c = self.c;
        let mut ordre: Vec<&String> = c
            .fascicules
            .iter()
            .filter(|(_, f)| self.essai || !vrai(&f["test"]))
            .map(|(fid, _)| fid)
            .collect();
        ordre.sort_by_key(|i| {
            (
                texte(&c.fascicules[i.as_str()]["parution"]),
                c.fascicules[i.as_str()]["numero"].as_i64(),
            )
        });
        let calendrier = ordre
            .into_iter()
            .map(|fid| self.fascicule(fid))
            .collect::<Result<Vec<_>, _>>()?;

        let mut parutions = json!({
            "format": FORMAT,
            "version": self.version,
            "essai": self.essai,
            "referentiels": self.referentiels(),
            "fascicules": calendrier,
        });
        // Couche ludique commune (ADR 0045, 0046) : absente sans `ludique/quetes.json`.
        if let Some(quetes) = &c.quetes {
            parutions["quetes"] = quetes["quetes"].clone();
            if !quetes["cadeau_de_bienvenue"].is_null() {
                parutions["cadeau_de_bienvenue"] = quetes["cadeau_de_bienvenue"].clone();
            }
        }
        self.artefact
            .fichiers
            .insert("parutions.json".into(), serialiser(&parutions));
        Ok(self.artefact)
    }
}

pub fn compiler(catalogue: &Catalogue, essai: bool, version: &str) -> Result<Artefact, String> {
    Compilateur::new(catalogue, essai, version).compiler()
}

pub fn ecrire_artefact(artefact: &Artefact, sortie: &Path) -> std::io::Result<()> {
    if sortie.exists() {
        fs::remove_dir_all(sortie)?;
    }
    for (chemin, contenu) in &artefact.fichiers {
        let cible = sortie.join(chemin);
        if let Some(parent) = cible.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(cible, contenu)?;
    }
    Ok(())
}

fn lire_json(fichier: &Path) -> Value {
    fs::read_to_string(fichier)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(Value::Null)
}

/// Recalcule l'empreinte de chaque fichier haché et contrôle chaque chemin cité,
/// du calendrier vers les manifestes, puis des manifestes vers les ressources.
#[must_use]
pub fn verifier(sortie: &Path) -> Vec<String> {
    let mut problemes = Vec::new();
    let mut fichiers = Vec::new();
    for dossier in ["a", "f", "r"] {
        for entree in fs::read_dir(sortie.join(dossier))
            .into_iter()
            .flatten()
            .flatten()
        {
            let nom = entree.file_name().to_string_lossy().into_owned();
            if dossier == "a" || Path::new(&nom).extension().is_some_and(|e| e == "json") {
                fichiers.push(format!("{dossier}/{nom}"));
            }
        }
    }
    fichiers.sort();
    for relatif in fichiers {
        let fichier = sortie.join(&relatif);
        let attendu = fichier
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        if fs::read(&fichier).map(|c| empreinte(&c)).ok().as_deref() != Some(attendu.as_str()) {
            problemes.push(format!("Empreinte fausse : {relatif}"));
        }
    }
    let parutions = lire_json(&sortie.join("parutions.json"));
    for fsc in liste(&parutions["fascicules"]) {
        let manifeste = texte(&fsc["manifeste"]);
        let fichier = sortie.join(manifeste);
        if !fichier.is_file() {
            problemes.push(format!(
                "Manifeste absent : {manifeste} (fascicule {})",
                fsc["numero"]
            ));
            continue;
        }
        for chemin in ressources(&lire_json(&fichier)) {
            if !sortie.join(&chemin).is_file() {
                problemes.push(format!(
                    "Ressource absente : {chemin} (fascicule {})",
                    fsc["numero"]
                ));
            }
        }
    }
    problemes
}
