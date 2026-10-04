//! Compilation du catalogue en artefact statique (ADR 0027, ADR 0029).
//!
//! L'artefact reprend l'arborescence du préfixe `staging/` du bucket :
//!
//! - `r/<sha256>.json` : ressources (cartes, recettes, exclusions) ;
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

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::chargement::{Catalogue, Table};
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
    let mut chemins = vec![&manifeste["jaquette"], &manifeste["exclusions"]];
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
        }))
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
        let relations = tries(&self.c.relations)
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
            .collect();
        carte.insert("relations".into(), relations);
        if unite["nature"] == "mot" {
            let compositions = tries(&self.c.compositions)
                .into_iter()
                .filter(|(_, c)| c["mot_id"] == uid && retenu(c))
                .map(|(_, c)| c.clone())
                .collect();
            carte.insert("compositions".into(), compositions);
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
        });
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

        let parutions = json!({
            "format": FORMAT,
            "version": self.version,
            "essai": self.essai,
            "referentiels": self.referentiels(),
            "fascicules": calendrier,
        });
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
