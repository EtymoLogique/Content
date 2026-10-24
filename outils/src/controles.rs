//! Contrôles d'intégrité du catalogue (modèle de données, « Contraintes d'intégrité » ; ADR 0005, 0013, 0014).

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::Path;

use chrono::{Datelike, Utc};
use indexmap::IndexMap;
use serde_json::Value;
use unicode_normalization::UnicodeNormalization;

use crate::chargement::{Catalogue, Table};
use crate::homographes;
use crate::json::{a_cle, entier, liste, texte, vrai};
use crate::medias;
use crate::rapport::Rapport;

pub const LOCALE_REFERENCE: &str = "fr";
/// Préfixe d’URL de la seule source de vérité (ADR 0060).
const SOURCE_WIKIDATA: &str = "https://www.wikidata.org/";
const TRAITS_UNION: [char; 6] = [
    '-', '\u{2010}', '\u{2011}', '\u{2012}', '\u{2013}', '\u{2014}',
];
pub const NIVEAUX_DIFFICULTE: [&str; 3] = ["facile", "moyen", "difficile"]; // ADR 0047
pub const TAILLE_FASCICULE: (usize, usize) = (20, 30); // ADR 0022, paramètre d'équilibrage
/// ADR 0054 : les dix premiers fascicules publient plus de 30 mots, chaque semaine, les quatre premiers ensemble ;
/// ADR 0059 : ils ne sont pas tenus à la règle des tiers.
pub const FASCICULES_DE_LANCEMENT: i64 = 10;
/// Règle d'amorçage (game-design) : trois découvertes au moins sans ouvrir de pli.
pub const MIN_RECETTES_AMORCAGE: usize = 3;
/// La table de fusion accepte exactement deux briques : jamais plus de deux éléments à la fois.
pub const BRIQUES_PAR_FUSION: usize = 2;

type Aretes = BTreeMap<String, BTreeSet<String>>;

fn nfc(texte: &str) -> String {
    texte.nfc().collect()
}

/// Sens de rang le plus petit, le premier en cas d'égalité.
fn premier_sens(unite: &Value) -> Option<&Value> {
    let mut premier: Option<&Value> = None;
    for sens in liste(&unite["sens"]) {
        if premier.is_none_or(|p| entier(&sens["rang"]) < entier(&p["rang"])) {
            premier = Some(sens);
        }
    }
    premier
}

/// Cycles d'un graphe orienté, chacun signalé une fois (parcours en profondeur itératif).
fn cycles(aretes: &Aretes) -> Vec<Vec<String>> {
    #[derive(Clone, Copy, PartialEq)]
    enum Couleur {
        Blanc,
        Gris,
        Noir,
    }
    let voisins = |n: &str| -> Vec<String> {
        aretes
            .get(n)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    };
    let mut couleur: HashMap<String, Couleur> = HashMap::new();
    let teinte =
        |c: &HashMap<String, Couleur>, n: &str| c.get(n).copied().unwrap_or(Couleur::Blanc);
    let mut trouves = Vec::new();
    for depart in aretes.keys() {
        if teinte(&couleur, depart) != Couleur::Blanc {
            continue;
        }
        let mut pile: Vec<(String, Vec<String>, usize)> =
            vec![(depart.clone(), voisins(depart), 0)];
        let mut chemin = vec![depart.clone()];
        couleur.insert(depart.clone(), Couleur::Gris);
        while let Some((noeud, suivants, i)) = pile.last_mut() {
            let Some(suivant) = suivants.get(*i).cloned() else {
                couleur.insert(noeud.clone(), Couleur::Noir);
                pile.pop();
                chemin.pop();
                continue;
            };
            *i += 1;
            match teinte(&couleur, &suivant) {
                Couleur::Gris => {
                    let debut = chemin
                        .iter()
                        .position(|n| *n == suivant)
                        .unwrap_or_default();
                    let mut cycle = chemin[debut..].to_vec();
                    cycle.push(suivant);
                    trouves.push(cycle);
                }
                Couleur::Blanc => {
                    couleur.insert(suivant.clone(), Couleur::Gris);
                    chemin.push(suivant.clone());
                    let v = voisins(&suivant);
                    pile.push((suivant, v, 0));
                }
                Couleur::Noir => {}
            }
        }
    }
    trouves
}

pub struct Controleur<'a> {
    c: &'a Catalogue,
    r: &'a mut Rapport,
    publication: bool,
    suites_connues: HashMap<Vec<String>, String>,
}

impl<'a> Controleur<'a> {
    pub fn new(catalogue: &'a Catalogue, rapport: &'a mut Rapport, publication: bool) -> Self {
        Self {
            c: catalogue,
            r: rapport,
            publication,
            suites_connues: HashMap::new(),
        }
    }

    // ------------------------------------------------------------------ outils

    fn lieu(&self, objet: Option<&str>, fichier_de: Option<&str>) -> Option<&'a str> {
        self.c
            .fichier(fichier_de.filter(|f| !f.is_empty()).or(objet))
    }

    fn erreur(
        &mut self,
        regle: &str,
        message: impl Into<String>,
        objet: &str,
        fichier_de: Option<&str>,
    ) {
        let fichier = self.lieu(Some(objet), fichier_de);
        self.r.erreur(regle, message, fichier, Some(objet));
    }

    fn avert(
        &mut self,
        regle: &str,
        message: impl Into<String>,
        objet: &str,
        fichier_de: Option<&str>,
    ) {
        let fichier = self.lieu(Some(objet), fichier_de);
        self.r.avertissement(regle, message, fichier, Some(objet));
    }

    /// Erreur en publication, avertissement sinon.
    fn signaler(&mut self, regle: &str, message: impl Into<String>, objet: &str) {
        if self.publication {
            self.erreur(regle, message, objet, None);
        } else {
            self.avert(regle, message, objet, None);
        }
    }

    /// Résout une référence ; signale une référence cassée, sauf vers un fichier déjà signalé invalide.
    fn reference(
        &mut self,
        table: &'a Table,
        cible: Option<&str>,
        objet: &str,
        champ: &str,
        nature: &str,
        fichier_de: Option<&str>,
    ) -> Option<&'a Value> {
        let cible = cible?;
        if let Some(trouve) = table.get(cible) {
            return Some(trouve);
        }
        if !self.c.ids_invalides.contains(cible) {
            self.erreur(
                "reference.cassee",
                format!("{champ} : {nature} {cible} introuvable."),
                objet,
                fichier_de,
            );
        }
        None
    }

    fn forme_de(&self, unite_id: &str, forme_id: &str) -> Option<&'a Value> {
        self.c
            .formes
            .get(forme_id)
            .filter(|(u, _)| u == unite_id)
            .map(|(_, f)| f)
    }

    fn sens_de(&self, unite_id: &str, sens_id: &str) -> Option<&'a Value> {
        self.c
            .sens
            .get(sens_id)
            .filter(|(u, _)| u == unite_id)
            .map(|(_, s)| s)
    }

    fn canonique(unite: &Value) -> Option<&Value> {
        let canoniques: Vec<&Value> = liste(&unite["formes"])
            .iter()
            .filter(|f| f["role"] == "canonique")
            .collect();
        if canoniques.len() == 1 {
            Some(canoniques[0])
        } else {
            None
        }
    }

    fn ancetres(&self, langue_id: &str) -> Vec<String> {
        let mut chaine: Vec<String> = Vec::new();
        let mut courant = self.c.langues.get(langue_id);
        while let Some(langue) = courant {
            let Some(parent) = langue["langue_parente_id"]
                .as_str()
                .filter(|p| !p.is_empty() && !chaine.iter().any(|c| c == p))
            else {
                break;
            };
            chaine.push(parent.to_owned());
            courant = self.c.langues.get(parent);
        }
        chaine
    }

    // --------------------------------------------------------------- contrôles

    pub fn executer(&mut self) {
        self.langues();
        self.conventions();
        self.sources_des_faits();
        self.unites();
        self.homographes();
        self.compositions();
        self.relations();
        self.briques();
        self.recettes();
        self.fascicules();
        self.quetes();
        self.exclusions();
        self.trompes_l_oeil();
        self.medias();
        if self.publication {
            self.publiable();
        }
    }

    fn langues(&mut self) {
        let c = self.c;
        let mut codes: HashMap<&str, &str> = HashMap::new();
        for (lid, langue) in &c.langues {
            self.reference(
                &c.langues,
                langue["langue_parente_id"].as_str(),
                lid,
                "langue_parente_id",
                "langue",
                None,
            );
            self.reference(
                &c.ecritures,
                langue["systeme_ecriture_defaut_id"].as_str(),
                lid,
                "systeme_ecriture_defaut_id",
                "écriture",
                None,
            );
            let code = texte(&langue["code_interne"]);
            if let Some(autre) = codes.get(code) {
                self.erreur(
                    "langue.code_duplique",
                    format!("Code interne déjà utilisé par {autre}."),
                    lid,
                    None,
                );
            }
            codes.insert(code, lid);
            if let (Some(debut), Some(fin)) = (
                langue["periode_debut"].as_f64(),
                langue["periode_fin"].as_f64(),
            ) && debut > fin
            {
                self.erreur(
                    "langue.periode",
                    "La période commence après sa fin.",
                    lid,
                    None,
                );
            }
        }
        let mut filiation = Aretes::new();
        for (lid, langue) in &c.langues {
            if vrai(&langue["langue_parente_id"]) {
                filiation
                    .entry(lid.clone())
                    .or_default()
                    .insert(texte(&langue["langue_parente_id"]).to_owned());
            }
        }
        for cycle in cycles(&filiation) {
            self.erreur(
                "langue.filiation_cyclique",
                format!("Filiation cyclique : {}.", cycle.join(" → ")),
                &cycle[0],
                None,
            );
        }
        let mut iso: HashMap<&str, &str> = HashMap::new();
        for (eid, ecriture) in &c.ecritures {
            let code = texte(&ecriture["code_iso15924"]);
            if let Some(autre) = iso.get(code) {
                self.erreur(
                    "ecriture.code_duplique",
                    format!("Code ISO 15924 déjà utilisé par {autre}."),
                    eid,
                    None,
                );
            }
            iso.insert(code, eid);
        }
    }

    fn conventions(&mut self) {
        let c = self.c;
        let mut codes: HashMap<&str, &str> = HashMap::new();
        for (cid, convention) in &c.conventions {
            self.reference(
                &c.ecritures,
                convention["systeme_ecriture_id"].as_str(),
                cid,
                "systeme_ecriture_id",
                "écriture",
                None,
            );
            let code = texte(&convention["code"]);
            if let Some(autre) = codes.get(code) {
                self.erreur(
                    "convention.code_duplique",
                    format!("Code déjà utilisé par {autre}."),
                    cid,
                    None,
                );
            }
            codes.insert(code, cid);
        }
    }

    /// (identifiant du fait, identifiant du fichier, fait) pour toutes les affirmations.
    fn faits(&self) -> Vec<(&'a str, &'a str, &'a Value)> {
        let c = self.c;
        let mut faits = Vec::new();
        for (uid, unite) in &c.unites {
            faits.push((uid.as_str(), uid.as_str(), &unite["fait"]));
            for forme in liste(&unite["formes"]) {
                faits.push((texte(&forme["id"]), uid.as_str(), &forme["fait"]));
            }
            for sens in liste(&unite["sens"]) {
                faits.push((texte(&sens["id"]), uid.as_str(), &sens["fait"]));
            }
            if let Some(attestation) = unite.get("attestation") {
                faits.push((
                    texte(&attestation["id"]),
                    uid.as_str(),
                    &attestation["fait"],
                ));
            }
        }
        for (cid, composition) in &c.compositions {
            faits.push((cid.as_str(), cid.as_str(), &composition["fait"]));
        }
        for (rid, relation) in &c.relations {
            faits.push((rid.as_str(), rid.as_str(), &relation["fait"]));
        }
        for (tid, trompe) in &c.trompes_l_oeil {
            faits.push((tid.as_str(), tid.as_str(), &trompe["fait"]));
        }
        faits
    }

    fn sources_des_faits(&mut self) {
        let c = self.c;
        for (fid, fichier_de, fait) in self.faits() {
            let mut vues = HashSet::new();
            for lien in liste(&fait["sources"]) {
                let source = texte(&lien["source_id"]);
                self.reference(
                    &c.sources,
                    lien["source_id"].as_str(),
                    fid,
                    "sources",
                    "source",
                    Some(fichier_de),
                );
                if !vues.insert(source) {
                    self.erreur(
                        "fait.source_dupliquee",
                        format!("Source {source} citée deux fois."),
                        fid,
                        Some(fichier_de),
                    );
                }
            }
        }
    }

    fn unites(&mut self) {
        let c = self.c;
        for (uid, unite) in &c.unites {
            let langue = self.reference(
                &c.langues,
                unite["langue_id"].as_str(),
                uid,
                "langue_id",
                "langue",
                None,
            );
            let canoniques: Vec<&Value> = liste(&unite["formes"])
                .iter()
                .filter(|f| f["role"] == "canonique")
                .collect();
            if canoniques.len() != 1 {
                self.erreur(
                    "unite.forme_canonique",
                    format!(
                        "Une unité a exactement une forme canonique ({} trouvée(s)).",
                        canoniques.len()
                    ),
                    uid,
                    None,
                );
            }
            if let Some(langue) = langue
                && canoniques.len() == 1
                && canoniques[0]["systeme_ecriture_id"] != langue["systeme_ecriture_defaut_id"]
            {
                self.erreur(
                    "unite.ecriture_incompatible",
                    "La forme canonique doit utiliser le système d’écriture par défaut de la langue.",
                    texte(&canoniques[0]["id"]),
                    Some(uid),
                );
            }
            for forme in liste(&unite["formes"]) {
                let fid = texte(&forme["id"]);
                let contenu = texte(&forme["texte"]);
                if contenu.starts_with(TRAITS_UNION) || contenu.ends_with(TRAITS_UNION) {
                    self.erreur(
                        "forme.trait_union",
                        "Le texte d’une forme est stocké sans trait d’union.",
                        fid,
                        Some(uid),
                    );
                }
                if texte(&forme["cle_normalisee"]) != nfc(contenu) {
                    self.erreur(
                        "forme.cle_normalisee",
                        "cle_normalisee doit être la forme NFC du texte.",
                        fid,
                        Some(uid),
                    );
                }
                self.reference(
                    &c.ecritures,
                    forme["systeme_ecriture_id"].as_str(),
                    fid,
                    "systeme_ecriture_id",
                    "écriture",
                    Some(uid),
                );
                let mut conventions_vues = HashSet::new();
                for trl in liste(&forme["translitterations"]) {
                    let convention_id = texte(&trl["convention_id"]);
                    let convention = self.reference(
                        &c.conventions,
                        trl["convention_id"].as_str(),
                        fid,
                        "convention_id",
                        "convention",
                        Some(uid),
                    );
                    if let Some(convention) = convention
                        && convention["systeme_ecriture_id"] != forme["systeme_ecriture_id"]
                    {
                        self.erreur(
                            "translitteration.ecriture",
                            format!("La convention {convention_id} ne s’applique pas à l’écriture de cette forme."),
                            fid,
                            Some(uid),
                        );
                    }
                    if !conventions_vues.insert(convention_id) {
                        self.erreur(
                            "translitteration.doublon",
                            "Deux translittérations pour une même convention.",
                            fid,
                            Some(uid),
                        );
                    }
                }
            }
            if let Some(attestation) = unite.get("attestation") {
                self.attestation(uid, unite, langue, attestation);
            }
            let rangs: Vec<String> = liste(&unite["sens"])
                .iter()
                .map(|s| s["rang"].to_string())
                .collect();
            if rangs.iter().collect::<HashSet<_>>().len() != rangs.len() {
                self.erreur(
                    "sens.rang_duplique",
                    "Deux sens ont le même rang.",
                    uid,
                    None,
                );
            }
        }
    }

    /// ADR 0056 : la première attestation d'un mot français, jamais dans le futur.
    fn attestation(
        &mut self,
        uid: &str,
        unite: &Value,
        langue: Option<&Value>,
        attestation: &Value,
    ) {
        let aid = texte(&attestation["id"]);
        if unite["nature"] != "mot" || langue.is_some_and(|l| l["code_interne"] != "fr") {
            self.erreur(
                "attestation.mot_francais",
                "Seul un mot français porte une première attestation (ADR 0056).",
                aid,
                Some(uid),
            );
        }
        let annee = attestation["annee"].as_i64().unwrap_or_default();
        if annee > i64::from(Utc::now().year()) {
            self.erreur(
                "attestation.future",
                format!("{annee} : une attestation n’est jamais dans le futur."),
                aid,
                Some(uid),
            );
        }
        if let Some(fin) = attestation["annee_fin"].as_i64()
            && fin <= annee
        {
            self.erreur(
                "attestation.intervalle",
                format!("L’intervalle {annee}-{fin} doit finir après son début."),
                aid,
                Some(uid),
            );
        }
    }

    fn sens_affiche(&self, uid: &str) -> Option<&'a Value> {
        for brique in self.c.briques.values() {
            if brique["unite_id"] == uid {
                return self.sens_de(uid, texte(&brique["sens_affiche_id"]));
            }
        }
        premier_sens(self.c.unites.get(uid)?)
    }

    fn etymons(&self, uid: &str) -> BTreeSet<&'a str> {
        self.c
            .relations
            .values()
            .filter(|r| r["unite_cible_id"] == uid)
            .map(|r| texte(&r["unite_source_id"]))
            .collect()
    }

    fn homographes(&mut self) {
        let c = self.c;
        let mut groupes: IndexMap<(&str, &str, &str), Vec<&str>> = IndexMap::new();
        for (uid, unite) in &c.unites {
            if let Some(canonique) = Self::canonique(unite) {
                groupes
                    .entry((
                        texte(&unite["langue_id"]),
                        texte(&unite["nature"]),
                        texte(&canonique["cle_normalisee"]),
                    ))
                    .or_default()
                    .push(uid);
            }
        }
        for membres in groupes.values() {
            let mut membres = membres.clone();
            membres.sort_unstable();
            for (i, a) in membres.iter().enumerate() {
                for b in &membres[i + 1..] {
                    let (sens_a, sens_b) = (self.sens_affiche(a), self.sens_affiche(b));
                    let libelles =
                        |s: Option<&'a Value>| s.map_or(&Value::Null, |s| &s["libelles"]);
                    let (libelles_a, libelles_b) = (libelles(sens_a), libelles(sens_b));
                    let locales: BTreeSet<&String> = libelles_a
                        .as_object()
                        .into_iter()
                        .chain(libelles_b.as_object())
                        .flat_map(|o| o.keys())
                        .collect();
                    for locale in locales {
                        if libelles_a[locale.as_str()]["glose"]
                            == libelles_b[locale.as_str()]["glose"]
                        {
                            self.erreur(
                                "homographe.glose_identique",
                                format!("Homographe de {b} : les gloses affichées doivent différer (locale {locale})."),
                                a,
                                None,
                            );
                        }
                    }
                    if self.etymons(a) == self.etymons(b) {
                        self.erreur(
                            "homographe.etymon_identique",
                            format!("Homographe de {b} : les étymons doivent différer."),
                            a,
                            None,
                        );
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    fn compositions(&mut self) {
        let c = self.c;
        let mut preferees: IndexMap<&str, Vec<&str>> = IndexMap::new();
        let mut graphe = Aretes::new();
        for (cid, cmp) in &c.compositions {
            let mot_id = texte(&cmp["mot_id"]);
            let mot = self.reference(
                &c.unites,
                cmp["mot_id"].as_str(),
                cid,
                "mot_id",
                "unité",
                None,
            );
            let mut forme_resultat = None;
            if let Some(mot) = mot {
                if mot["nature"] != "mot" {
                    self.erreur(
                        "composition.resultat_non_mot",
                        "Le résultat d’une composition est un mot.",
                        cid,
                        None,
                    );
                }
                forme_resultat = self.forme_de(mot_id, texte(&cmp["forme_resultat_id"]));
                if forme_resultat.is_none() {
                    self.erreur(
                        "composition.forme_resultat",
                        "forme_resultat_id n’est pas une forme du mot résultat.",
                        cid,
                        None,
                    );
                }
            }
            if vrai(&cmp["preferee"]) {
                preferees.entry(mot_id).or_default().push(cid);
            }

            let mut parties: Vec<&Value> = liste(&cmp["parties"]).iter().collect();
            parties.sort_by_key(|p| entier(&p["rang"]));
            let rangs: Vec<i64> = parties.iter().map(|p| entier(&p["rang"])).collect();
            if rangs != (1..=i64::try_from(parties.len()).unwrap_or_default()).collect::<Vec<_>>() {
                self.erreur(
                    "composition.rangs",
                    "Les rangs des parties sont uniques et continus à partir de 1.",
                    cid,
                    None,
                );
            }
            let dernier = i64::try_from(parties.len()).unwrap_or_default();
            let mut segments: Vec<(i64, i64, i64)> = Vec::new();
            for partie in &parties {
                let pu = texte(&partie["unite_id"]);
                let rang = entier(&partie["rang"]);
                let unite = self.reference(
                    &c.unites,
                    partie["unite_id"].as_str(),
                    cid,
                    &format!("partie {rang}"),
                    "unité",
                    None,
                );
                graphe
                    .entry(mot_id.to_owned())
                    .or_default()
                    .insert(pu.to_owned());
                if pu == mot_id {
                    self.erreur(
                        "composition.partie_resultat",
                        "Une partie ne peut pas être le résultat lui-même.",
                        cid,
                        None,
                    );
                }
                if let Some(unite) = unite {
                    if self.sens_de(pu, texte(&partie["sens_id"])).is_none() {
                        self.erreur(
                            "composition.sens_partie",
                            format!("Partie {rang} : le sens n’appartient pas à l’unité."),
                            cid,
                            None,
                        );
                    }
                    if self.forme_de(pu, texte(&partie["forme_id"])).is_none() {
                        self.erreur(
                            "composition.forme_partie",
                            format!("Partie {rang} : la forme n’appartient pas à l’unité."),
                            cid,
                            None,
                        );
                    }
                    if unite["nature"] == "prefixe" && rang == dernier {
                        self.erreur(
                            "composition.position",
                            "Un préfixe n’apparaît pas au dernier rang.",
                            cid,
                            None,
                        );
                    }
                    if unite["nature"] == "suffixe" && rang == 1 {
                        self.erreur(
                            "composition.position",
                            "Un suffixe n’apparaît pas au premier rang.",
                            cid,
                            None,
                        );
                    }
                }
                let (debut, fin) = (
                    partie["segment_debut"].as_i64(),
                    partie["segment_fin"].as_i64(),
                );
                match (debut, fin) {
                    (Some(debut), Some(fin)) => segments.push((debut, fin, rang)),
                    (None, None) => {}
                    _ => self.erreur(
                        "composition.segment",
                        format!("Partie {rang} : segment_debut et segment_fin vont ensemble."),
                        cid,
                        None,
                    ),
                }
            }
            if let Some(forme_resultat) = forme_resultat {
                let longueur = i64::try_from(nfc(texte(&forme_resultat["texte"])).chars().count())
                    .unwrap_or_default();
                for &(debut, fin, rang) in &segments {
                    if !(0 <= debut && debut < fin && fin <= longueur) {
                        self.erreur("composition.segment", format!("Partie {rang} : segment [{debut}, {fin}) hors de la forme résultat."), cid, None);
                    }
                }
                segments.sort_unstable();
                for paire in segments.windows(2) {
                    let ((_, f1, r1), (d2, _, r2)) = (paire[0], paire[1]);
                    if d2 < f1 {
                        self.erreur(
                            "composition.segment",
                            format!("Les segments des parties {r1} et {r2} se chevauchent."),
                            cid,
                            None,
                        );
                    }
                }
                // Les segments pavent le mot : c’est d’eux que viennent les coupes du ricochet « Découper » (ADR 0044).
                if segments.len() == parties.len() {
                    let mut position = 0;
                    for &(debut, fin, rang) in &segments {
                        if debut > position {
                            self.erreur(
                                "composition.pavage",
                                format!("Un trou précède la partie {rang} : les lettres [{position}, {debut}) n’appartiennent à aucune partie."),
                                cid,
                                None,
                            );
                        }
                        position = position.max(fin);
                    }
                    if position < longueur {
                        self.erreur(
                            "composition.pavage",
                            format!("Les lettres [{position}, {longueur}) du mot n’appartiennent à aucune partie."),
                            cid,
                            None,
                        );
                    }
                }
            }
            if segments.len() < parties.len() {
                self.erreur(
                    "composition.segments_manquants",
                    "Chaque partie porte son segment (segment_debut et segment_fin) : sans eux, le ricochet « Découper » ne peut pas couper le mot.",
                    cid,
                    None,
                );
            }

            let mut ordres: Vec<&Value> = Vec::new();
            for trf in liste(&cmp["transformations"]) {
                let type_code = texte(&trf["type_code"]);
                if !c.types_transformation.contains_key(type_code) {
                    self.erreur(
                        "transformation.type_inconnu",
                        format!("Type de transformation « {type_code} » inconnu."),
                        cid,
                        None,
                    );
                }
                if !trf["rang_partie"].is_null() && !rangs.contains(&entier(&trf["rang_partie"])) {
                    self.erreur(
                        "transformation.rang",
                        format!(
                            "La transformation {} vise une partie absente.",
                            texte(&trf["id"])
                        ),
                        cid,
                        None,
                    );
                }
                if ordres.contains(&&trf["ordre"]) {
                    self.erreur(
                        "transformation.ordre",
                        "Deux transformations ont le même ordre.",
                        cid,
                        None,
                    );
                }
                ordres.push(&trf["ordre"]);
            }
        }

        for (mot_id, cids) in &preferees {
            if cids.len() > 1 {
                let mut tries = cids.clone();
                tries.sort_unstable();
                self.erreur(
                    "composition.preferee",
                    format!(
                        "Plusieurs compositions préférées pour {mot_id} : {}.",
                        tries.join(", ")
                    ),
                    cids[0],
                    None,
                );
            }
        }
        for cycle in cycles(&graphe) {
            self.erreur(
                "composition.cycle",
                format!("Composition cyclique : {}.", cycle.join(" → ")),
                &cycle[0],
                None,
            );
        }
    }

    fn relations(&mut self) {
        let c = self.c;
        let mut vues: HashMap<(&str, &str, &str), &str> = HashMap::new();
        let mut graphe = Aretes::new();
        for (rid, rel) in &c.relations {
            let (src, cib, typ) = (
                texte(&rel["unite_source_id"]),
                texte(&rel["unite_cible_id"]),
                texte(&rel["type_code"]),
            );
            let source = self.reference(
                &c.unites,
                rel["unite_source_id"].as_str(),
                rid,
                "unite_source_id",
                "unité",
                None,
            );
            let cible = self.reference(
                &c.unites,
                rel["unite_cible_id"].as_str(),
                rid,
                "unite_cible_id",
                "unité",
                None,
            );
            let type_rel = c.types_relation.get(typ);
            if type_rel.is_none() {
                self.erreur(
                    "relation.type_inconnu",
                    format!("Type de relation « {typ} » inconnu."),
                    rid,
                    None,
                );
            }
            if rel["unite_source_id"] == rel["unite_cible_id"] {
                self.erreur(
                    "relation.boucle",
                    "La source et la cible d’une relation sont différentes.",
                    rid,
                    None,
                );
            }
            if let Some(autre) = vues.get(&(src, cib, typ)) {
                self.erreur(
                    "relation.doublon",
                    format!("Relation identique à {autre}."),
                    rid,
                    None,
                );
            }
            vues.insert((src, cib, typ), rid);
            if type_rel.is_some_and(|t| vrai(&t["acyclique"])) {
                graphe
                    .entry(src.to_owned())
                    .or_default()
                    .insert(cib.to_owned());
            }
            if let (Some(source), Some(cible)) = (source, cible) {
                if typ == "emprunt" && source["langue_id"] == cible["langue_id"] {
                    self.erreur(
                        "relation.emprunt",
                        "Un emprunt relie deux langues différentes.",
                        rid,
                        None,
                    );
                }
                if typ == "heritage"
                    && !self
                        .ancetres(texte(&cible["langue_id"]))
                        .iter()
                        .any(|l| l == texte(&source["langue_id"]))
                {
                    self.erreur(
                        "relation.heritage",
                        "Un héritage suit la filiation des langues.",
                        rid,
                        None,
                    );
                }
            }
        }
        for cycle in cycles(&graphe) {
            self.erreur(
                "relation.cycle",
                format!("Relations acycliques en cycle : {}.", cycle.join(" → ")),
                &cycle[0],
                None,
            );
        }
    }

    fn briques(&mut self) {
        let c = self.c;
        let mut par_unite: HashMap<&str, &str> = HashMap::new();
        for (bid, brique) in &c.briques {
            let uid = texte(&brique["unite_id"]);
            let unite = self.reference(
                &c.unites,
                brique["unite_id"].as_str(),
                bid,
                "unite_id",
                "unité",
                None,
            );
            if let Some(autre) = par_unite.get(uid) {
                self.erreur(
                    "brique.unicite",
                    format!("L’unité {uid} a déjà la brique {autre}."),
                    bid,
                    None,
                );
            }
            par_unite.insert(uid, bid);
            if unite.is_none() {
                continue;
            }
            match self.forme_de(uid, texte(&brique["forme_affichee_id"])) {
                None => self.erreur(
                    "brique.forme",
                    "La forme affichée n’appartient pas à l’unité.",
                    bid,
                    None,
                ),
                Some(forme) if forme["role"] != "canonique" => {
                    self.erreur(
                        "brique.forme",
                        "Une brique affiche la forme canonique de son unité.",
                        bid,
                        None,
                    );
                }
                Some(_) => {}
            }
            match self.sens_de(uid, texte(&brique["sens_affiche_id"])) {
                None => self.erreur(
                    "brique.sens",
                    "Le sens affiché n’appartient pas à l’unité.",
                    bid,
                    None,
                ),
                Some(sens) if !vrai(&sens["libelles"][LOCALE_REFERENCE]["glose"]) => {
                    self.erreur(
                        "brique.glose",
                        "Le sens affiché par une brique a une glose dans la locale de référence.",
                        bid,
                        None,
                    );
                }
                Some(_) => {}
            }
        }
    }

    fn brique_de(&self, uid: &str) -> Option<&'a str> {
        self.c
            .briques
            .iter()
            .find(|(_, b)| b["unite_id"] == uid)
            .map(|(bid, _)| bid.as_str())
    }

    /// Suites de briques jouables par une recette, dans l'ordre des emplacements.
    pub(crate) fn suites(emplacements: &[Value]) -> BTreeSet<Vec<String>> {
        let mut ordonnes: Vec<&Value> = emplacements.iter().collect();
        ordonnes.sort_by_key(|e| entier(&e["rang"]));
        let mut suites: Vec<Vec<String>> = vec![Vec::new()];
        for emplacement in ordonnes {
            suites = suites
                .iter()
                .flat_map(|debut| {
                    liste(&emplacement["briques_acceptees"])
                        .iter()
                        .map(move |b| {
                            let mut suite = debut.clone();
                            suite.push(texte(b).to_owned());
                            suite
                        })
                })
                .collect();
        }
        suites.into_iter().collect()
    }

    #[allow(clippy::too_many_lines)]
    fn recettes(&mut self) {
        let c = self.c;
        let mut suites: HashMap<Vec<String>, String> = HashMap::new();
        let mots_en_fascicule: HashSet<&str> = c
            .fascicules
            .values()
            .flat_map(|f| liste(&f["unites"]))
            .filter(|u| u["role"] == "resultat")
            .map(|u| texte(&u["unite_id"]))
            .collect();
        for (rid, recette) in &c.recettes {
            let cmp = self.reference(
                &c.compositions,
                recette["composition_id"].as_str(),
                rid,
                "composition_id",
                "composition",
                None,
            );
            let emplacements = liste(&recette["emplacements"]);
            for emp in emplacements {
                let champ = format!("emplacement {}", entier(&emp["rang"]));
                for bid in liste(&emp["briques_acceptees"]) {
                    self.reference(&c.briques, bid.as_str(), rid, &champ, "brique", None);
                }
            }
            if emplacements.len() != BRIQUES_PAR_FUSION {
                self.erreur(
                    "recette.deux_briques",
                    format!(
                        "Une recette se joue avec exactement {BRIQUES_PAR_FUSION} briques ({} emplacements).",
                        emplacements.len()
                    ),
                    rid,
                    None,
                );
            }
            let mut rangs: Vec<i64> = emplacements.iter().map(|e| entier(&e["rang"])).collect();
            rangs.sort_unstable();
            if rangs != (1..=i64::try_from(rangs.len()).unwrap_or_default()).collect::<Vec<_>>() {
                self.erreur(
                    "recette.rangs",
                    "Les rangs des emplacements sont uniques et continus à partir de 1.",
                    rid,
                    None,
                );
            }
            for suite in Self::suites(emplacements) {
                if let Some(autre) = suites.get(&suite) {
                    let message = format!("La même suite de briques mène aussi à {autre}.");
                    self.erreur("recette.ambigue", message, rid, None);
                }
                suites.entry(suite).or_insert_with(|| rid.clone());
            }
            let Some(cmp) = cmp else { continue };
            if cmp["fait"]["confiance"] == "non_retenue" {
                self.erreur(
                    "recette.non_retenue",
                    "Une analyse non retenue n’est jamais jouée comme solution.",
                    rid,
                    None,
                );
            }
            let mut parties: BTreeMap<i64, &Value> = BTreeMap::new();
            for partie in liste(&cmp["parties"]) {
                parties.insert(entier(&partie["rang"]), partie);
            }
            if parties.keys().copied().collect::<Vec<_>>() != rangs {
                self.erreur(
                    "recette.emplacements",
                    "Les emplacements suivent les parties de la composition.",
                    rid,
                    None,
                );
            }
            for emp in emplacements {
                let rang = entier(&emp["rang"]);
                let Some(partie) = parties.get(&rang) else {
                    continue;
                };
                let unite_id = texte(&partie["unite_id"]);
                match self.brique_de(unite_id) {
                    None => self.erreur(
                        "recette.brique_absente",
                        format!("Emplacement {rang} : l’unité {unite_id} n’a pas de brique."),
                        rid,
                        None,
                    ),
                    Some(attendue) if !liste(&emp["briques_acceptees"]).iter().any(|b| b == attendue) => self.erreur(
                        "recette.brique_partie",
                        format!("Emplacement {rang} : la brique {attendue} de la partie n’est pas acceptée."),
                        rid,
                        None,
                    ),
                    Some(_) => {}
                }
            }
            let mot_id = texte(&cmp["mot_id"]);
            if let Some(sens) = c.unites.get(mot_id).and_then(premier_sens)
                && !vrai(&sens["libelles"][LOCALE_REFERENCE]["definition"])
            {
                self.erreur(
                    "recette.definition",
                    "Le mot résultat d’une recette a une définition dans la locale de référence.",
                    rid,
                    None,
                );
            }
            if !mots_en_fascicule.contains(mot_id) {
                self.avert(
                    "recette.hors_fascicule",
                    format!("Le mot {mot_id} n’est le résultat d’aucun fascicule."),
                    rid,
                    None,
                );
            }
        }
        self.suites_connues = suites;
    }

    fn recettes_du_mot(&self, mot_id: &str) -> Vec<&'a Value> {
        let c = self.c;
        c.recettes
            .values()
            .filter(|r| {
                c.compositions
                    .get(texte(&r["composition_id"]))
                    .is_some_and(|cmp| cmp["mot_id"] == mot_id)
            })
            .collect()
    }

    #[allow(clippy::too_many_lines)]
    fn fascicules(&mut self) {
        let c = self.c;
        let mut numeros: HashMap<(bool, i64), &str> = HashMap::new();
        let mut jalons: HashMap<String, String> = HashMap::new();
        // Identifiants de palier déjà pris : ceux des quêtes au long cours, puis ceux des collections.
        let mut collections: HashMap<String, String> = c
            .quetes
            .iter()
            .flat_map(|q| liste(&q["quetes"]))
            .flat_map(|q| liste(&q["paliers"]))
            .map(|p| {
                (
                    texte(&p["id"]).to_owned(),
                    crate::chargement::QUETES.to_owned(),
                )
            })
            .collect();
        let mut resultats: HashMap<(bool, &str), &str> = HashMap::new();
        for (fid, fsc) in &c.fascicules {
            let test = vrai(&fsc["test"]);
            let cle = (test, entier(&fsc["numero"]));
            if let Some(autre) = numeros.get(&cle) {
                self.erreur(
                    "fascicule.numero",
                    format!("Numéro déjà utilisé par {autre}."),
                    fid,
                    None,
                );
            }
            numeros.insert(cle, fid);
            let mut vus: HashSet<&str> = HashSet::new();
            for ligne in liste(&fsc["unites"]) {
                let uid = texte(&ligne["unite_id"]);
                if !vus.insert(uid) {
                    self.erreur(
                        "fascicule.unite_dupliquee",
                        format!("L’unité {uid} est déclarée deux fois."),
                        fid,
                        None,
                    );
                }
                let unite = self.reference(
                    &c.unites,
                    ligne["unite_id"].as_str(),
                    fid,
                    "unites",
                    "unité",
                    None,
                );
                if ligne["role"] == "resultat" {
                    if let Some(autre) = resultats.get(&(test, uid)) {
                        self.erreur(
                            "fascicule.unicite",
                            format!("Le mot {uid} est déjà le résultat de {autre}."),
                            fid,
                            None,
                        );
                    }
                    resultats.insert((test, uid), fid);
                    match unite {
                        Some(u) if u["nature"] != "mot" => {
                            self.erreur(
                                "fascicule.resultat_non_mot",
                                format!("Le résultat {uid} doit être un mot."),
                                fid,
                                None,
                            );
                        }
                        Some(_) if self.recettes_du_mot(uid).is_empty() => {
                            self.erreur(
                                "fascicule.resultat_sans_recette",
                                format!("Le résultat {uid} n’a pas de recette."),
                                fid,
                                None,
                            );
                        }
                        _ => {}
                    }
                } else if unite.is_some() && self.brique_de(uid).is_none() {
                    self.erreur(
                        "fascicule.ingredient_sans_brique",
                        format!("L’ingrédient {uid} n’a pas de brique."),
                        fid,
                        None,
                    );
                }
            }
            self.graines(fid, fsc, &vus);
            self.jalons(fid, fsc, &vus, &mut jalons);
            self.collections(fid, fsc, &mut collections);
        }

        for (fid, fsc) in &c.fascicules {
            let test = vrai(&fsc["test"]);
            let mut declares: IndexMap<&str, &str> = IndexMap::new();
            for ligne in liste(&fsc["unites"]) {
                declares.insert(texte(&ligne["unite_id"]), texte(&ligne["role"]));
            }
            let mut utilises: HashSet<&str> = HashSet::new();
            let mut nb_mots = 0;
            for (uid, role) in &declares {
                if *role != "resultat" {
                    continue;
                }
                nb_mots += 1;
                for recette in self.recettes_du_mot(uid) {
                    let cmp = &c.compositions[texte(&recette["composition_id"])];
                    for partie in liste(&cmp["parties"]) {
                        let pu = texte(&partie["unite_id"]);
                        utilises.insert(pu);
                        if !declares.contains_key(pu) {
                            self.erreur(
                                "fascicule.autonomie",
                                format!(
                                    "L’ingrédient {pu} de {} n’est pas déclaré.",
                                    texte(&recette["id"])
                                ),
                                fid,
                                None,
                            );
                            continue;
                        }
                        if let Some(origine) = resultats.get(&(test, pu))
                            && entier(&c.fascicules[*origine]["numero"]) > entier(&fsc["numero"])
                        {
                            self.erreur(
                                "fascicule.autonomie",
                                format!("L’ingrédient {pu} n’est publié que par un fascicule suivant ({origine})."),
                                fid,
                                None,
                            );
                        }
                    }
                }
            }
            for (uid, role) in &declares {
                if *role == "ingredient" && !utilises.contains(uid) {
                    self.avert(
                        "fascicule.ingredient_inutile",
                        format!("L’ingrédient {uid} ne sert à aucune recette du fascicule."),
                        fid,
                        None,
                    );
                }
            }
            let (minimum, maximum) = TAILLE_FASCICULE;
            if !test
                && entier(&fsc["numero"]) > FASCICULES_DE_LANCEMENT
                && !(minimum..=maximum).contains(&nb_mots)
            {
                self.avert(
                    "fascicule.taille",
                    format!("{nb_mots} mots : un fascicule en publie de {minimum} à {maximum}."),
                    fid,
                    None,
                );
            }
            if !test {
                // ADR 0059 : la règle des tiers vaut à partir du onzième fascicule.
                if entier(&fsc["numero"]) > FASCICULES_DE_LANCEMENT {
                    self.tiers(fid, fsc);
                }
                let rarete: HashMap<&str, &str> = c
                    .briques
                    .values()
                    .map(|b| (texte(&b["unite_id"]), texte(&b["rarete"])))
                    .collect();
                if !declares
                    .keys()
                    .any(|uid| rarete.get(uid) == Some(&"legendaire"))
                {
                    self.signaler(
                        "fascicule.legendaire_absente",
                        "Aucune brique légendaire : la chance de légendaire du pli (ADR 0043) n’aurait rien à donner.",
                        fid,
                    );
                }
            }
        }

        for test in [false, true] {
            let mut premiers: Vec<(i64, &str)> = c
                .fascicules
                .iter()
                .filter(|(_, f)| f["test"] == test)
                .map(|(fid, f)| (entier(&f["numero"]), fid.as_str()))
                .collect();
            premiers.sort_unstable();
            if let Some((_, premier)) = premiers.first()
                && !vrai(&c.fascicules[*premier]["graines"])
            {
                self.avert(
                    "fascicule.amorcage",
                    "Le premier fascicule ne déclare aucune graine : pas de réserve de départ.",
                    premier,
                    None,
                );
            }
        }

        let mut dates: Vec<(i64, &str, &str)> = c
            .fascicules
            .iter()
            .filter(|(_, f)| !vrai(&f["test"]))
            .map(|(fid, f)| (entier(&f["numero"]), texte(&f["parution"]), fid.as_str()))
            .collect();
        dates.sort_unstable();
        for paire in dates.windows(2) {
            let ((n1, d1, _), (n2, d2, f2)) = (paire[0], paire[1]);
            // Deux fascicules de lancement peuvent paraître le même jour (ADR 0054).
            if d2 < d1 || (d2 == d1 && n2 > FASCICULES_DE_LANCEMENT) {
                self.avert(
                    "fascicule.chronologie",
                    format!(
                        "Le fascicule {n2} paraît avant ou en même temps que le fascicule {n1}."
                    ),
                    f2,
                    None,
                );
            }
        }
    }

    /// Jalons (ADR 0038) : identifiants uniques, découvertes strictement croissantes,
    /// pli fait des préfixes et suffixes du fascicule ; au moins deux, car le
    /// deuxième débloque le fascicule suivant (ADR 0068).
    fn jalons(
        &mut self,
        fid: &str,
        fsc: &Value,
        declares: &HashSet<&str>,
        vus: &mut HashMap<String, String>,
    ) {
        let c = self.c;
        if liste(&fsc["jalons"]).len() < 2 {
            self.erreur(
                "fascicule.jalon_deblocage",
                format!(
                    "Le fascicule {fid} doit déclarer au moins deux jalons : le deuxième débloque le fascicule suivant (ADR 0068)."
                ),
                fid,
                None,
            );
        }
        let mut precedent = 0;
        for jalon in liste(&fsc["jalons"]) {
            let jid = texte(&jalon["id"]);
            if let Some(autre) = vus.get(jid) {
                self.erreur(
                    "fascicule.jalon_duplique",
                    format!("Le jalon {jid} est déjà déclaré par {autre}."),
                    fid,
                    None,
                );
            }
            vus.insert(jid.to_owned(), fid.to_owned());
            let decouvertes = entier(&jalon["decouvertes"]);
            if decouvertes <= precedent {
                self.erreur(
                    "fascicule.jalon_ordre",
                    format!("Le jalon {jid} doit demander plus de {precedent} découvertes."),
                    fid,
                    None,
                );
            }
            precedent = decouvertes;
            for bid in liste(&jalon["pli"]) {
                let Some(brique) =
                    self.reference(&c.briques, bid.as_str(), fid, "jalons", "brique", None)
                else {
                    continue;
                };
                let bid = texte(bid);
                let unite_id = texte(&brique["unite_id"]);
                if !declares.contains(unite_id) {
                    self.erreur(
                        "fascicule.jalon_hors_fascicule",
                        format!(
                            "La brique {bid} du jalon {jid} n’est pas une brique du fascicule."
                        ),
                        fid,
                        None,
                    );
                } else if !c
                    .unites
                    .get(unite_id)
                    .is_some_and(|u| u["nature"] == "prefixe" || u["nature"] == "suffixe")
                {
                    self.erreur(
                        "fascicule.jalon_mot",
                        format!("La brique {bid} du jalon {jid} est un mot : un pli ne donne que des affixes."),
                        fid,
                        None,
                    );
                }
                if brique["rarete"] == "legendaire" {
                    self.erreur(
                        "fascicule.jalon_legendaire",
                        format!("La brique {bid} du jalon {jid} est légendaire : un pli de jalon n’en donne jamais (ADR 0043)."),
                        fid,
                        None,
                    );
                }
            }
        }
    }

    /// Quêtes de collection (App#90) : un identifiant de palier jamais repris, un
    /// critère au plus une fois par fascicule, au moins trois membres hors
    /// légendaires, et une récompense.
    fn collections(&mut self, fid: &str, fsc: &Value, vus: &mut HashMap<String, String>) {
        let mut criteres: HashSet<&str> = HashSet::new();
        for quete in liste(&fsc["collections"]) {
            let id = texte(&quete["id"]);
            let critere = texte(&quete["critere"]);
            if let Some(autre) = vus.get(id) {
                self.erreur(
                    "fascicule.collection_doublon",
                    format!("Le palier {id} est déjà déclaré par {autre}."),
                    fid,
                    None,
                );
            }
            vus.insert(id.to_owned(), fid.to_owned());
            if !criteres.insert(critere) {
                self.erreur(
                    "fascicule.collection_doublon",
                    format!("La quête de collection « {critere} » est déclarée deux fois."),
                    fid,
                    None,
                );
            }
            let membres = crate::collections::membres(self.c, fid, critere).len();
            if membres < crate::collections::MEMBRES_MIN {
                self.erreur(
                    "fascicule.collection_trop_petite",
                    format!(
                        "La quête de collection « {critere} » ne compte que {membres} carte(s), hors légendaires : il en faut au moins {}.",
                        crate::collections::MEMBRES_MIN
                    ),
                    fid,
                    None,
                );
            }
            if quete["encre"].as_i64().unwrap_or(0) == 0
                && quete["sabliers"].as_i64().unwrap_or(0) == 0
            {
                self.erreur(
                    "fascicule.collection_sans_recompense",
                    format!("La quête de collection {id} ne donne rien."),
                    fid,
                    None,
                );
            }
        }
    }

    /// Quêtes au long cours et cadeau de bienvenue (ADR 0045, 0046) : références,
    /// paliers croissants, aucune légendaire comptée ni donnée.
    fn quetes(&mut self) {
        let c = self.c;
        let Some(quetes) = c.quetes.as_ref() else {
            return;
        };
        let fichier = Some(crate::chargement::QUETES);
        let mut ids: HashSet<&str> = HashSet::new();
        for quete in liste(&quetes["quetes"]) {
            let qid = texte(&quete["id"]);
            if !ids.insert(qid) {
                self.r.erreur(
                    "quete.doublon",
                    format!("La quête {qid} est déclarée deux fois."),
                    fichier,
                    Some(qid),
                );
            }
            match texte(&quete["sorte"]) {
                "langue" => {
                    let langue = texte(&quete["langue_id"]);
                    if !c.langues.contains_key(langue) {
                        self.r.erreur(
                            "reference.cassee",
                            format!("langue_id : langue {langue} introuvable."),
                            fichier,
                            Some(qid),
                        );
                    }
                }
                "transformation" => {
                    let code = texte(&quete["type_code"]);
                    if !c.types_transformation.contains_key(code) {
                        self.r.erreur(
                            "reference.cassee",
                            format!("type_code : type de transformation {code} introuvable."),
                            fichier,
                            Some(qid),
                        );
                    }
                }
                _ => {}
            }
            let mut precedent = 0;
            let legendaire = texte(&quete["sorte"]) == "premiere_legendaire";
            for palier in liste(&quete["paliers"]) {
                let pid = texte(&palier["id"]);
                if !ids.insert(pid) {
                    self.r.erreur(
                        "quete.doublon",
                        format!("Le palier {pid} est déclaré deux fois."),
                        fichier,
                        Some(pid),
                    );
                }
                let seuil = palier["seuil"].as_i64().unwrap_or(0);
                if seuil <= precedent {
                    self.r.erreur(
                        "quete.paliers_non_croissants",
                        format!("{qid} : les seuils des paliers doivent croître strictement ({seuil} après {precedent})."),
                        fichier,
                        Some(pid),
                    );
                }
                precedent = seuil;
                let encre = palier["encre"].as_i64().unwrap_or(0);
                let sabliers = palier["sabliers"].as_i64().unwrap_or(0);
                if encre + sabliers == 0 {
                    self.r.erreur(
                        "quete.palier_sans_recompense",
                        format!("{qid} : le palier {pid} ne donne rien."),
                        fichier,
                        Some(pid),
                    );
                }
                // Aucune quête ne fixe un nombre de légendaires : il trahirait combien il en existe.
                if legendaire && seuil != 1 {
                    self.r.erreur(
                        "quete.legendaires_comptees",
                        format!("{qid} : seule « Trouvez votre première légendaire » est permise, à un seul palier de seuil 1."),
                        fichier,
                        Some(pid),
                    );
                }
            }
        }
        self.cadeau(quetes);
    }

    /// Le cadeau de bienvenue ne donne que des briques de fascicules, jamais une légendaire (ADR 0043, 0046).
    fn cadeau(&mut self, quetes: &Value) {
        let c = self.c;
        let fichier = Some(crate::chargement::QUETES);
        let dans_un_fascicule: HashSet<&str> = c
            .fascicules
            .values()
            .flat_map(|f| liste(&f["unites"]))
            .map(|u| texte(&u["unite_id"]))
            .collect();
        for brique in liste(&quetes["cadeau_de_bienvenue"]["plis"]) {
            let bid = texte(brique);
            let Some(objet) = c.briques.get(bid) else {
                self.r.erreur(
                    "reference.cassee",
                    format!("cadeau_de_bienvenue : brique {bid} introuvable."),
                    fichier,
                    Some(bid),
                );
                continue;
            };
            if objet["rarete"] == "legendaire" {
                self.r.erreur(
                    "cadeau.legendaire",
                    format!("La brique {bid} est légendaire : le cadeau n'en donne jamais."),
                    fichier,
                    Some(bid),
                );
            }
            if !dans_un_fascicule.contains(texte(&objet["unite_id"])) {
                self.r.erreur(
                    "cadeau.hors_fascicule",
                    format!("La brique {bid} n'appartient à aucun fascicule."),
                    fichier,
                    Some(bid),
                );
            }
        }
    }

    /// Un tiers de mots faciles, moyens et difficiles, à un mot près ; légendaires hors compteurs (ADR 0047).
    fn tiers(&mut self, fid: &str, fsc: &Value) {
        let c = self.c;
        let legendaires: HashSet<&str> = c
            .briques
            .values()
            .filter(|b| b["rarete"] == "legendaire")
            .map(|b| texte(&b["id"]))
            .collect();
        let mut compte = [0_usize; 3];
        for ligne in liste(&fsc["unites"]) {
            if ligne["role"] != "resultat" || !a_cle(ligne, "difficulte") {
                continue;
            }
            let secret = self
                .recettes_du_mot(texte(&ligne["unite_id"]))
                .iter()
                .flat_map(|r| liste(&r["emplacements"]))
                .flat_map(|e| liste(&e["briques_acceptees"]))
                .any(|b| legendaires.contains(texte(b)));
            if let Some(i) = NIVEAUX_DIFFICULTE
                .iter()
                .position(|n| ligne["difficulte"] == *n)
                && !secret
            {
                compte[i] += 1;
            }
        }
        #[allow(clippy::cast_precision_loss)]
        let tiers = compte.iter().sum::<usize>() as f64 / 3.0;
        #[allow(clippy::cast_precision_loss)]
        if compte.iter().any(|&n| (n as f64 - tiers).abs() > 1.0) {
            let detail: Vec<String> = NIVEAUX_DIFFICULTE
                .iter()
                .zip(compte)
                .map(|(niveau, n)| format!("{n} {niveau}s"))
                .collect();
            self.avert(
                "fascicule.tiers",
                format!(
                    "{} : un tiers de chaque niveau, à un mot près (légendaires exclus).",
                    detail.join(", ")
                ),
                fid,
                None,
            );
        }
    }

    /// Réserve de départ (ADR 0030) : briques du fascicule, et au moins trois
    /// recettes réalisables avec elles seules, toutes ensemble (règle d'amorçage).
    fn graines(&mut self, fid: &str, fsc: &Value, declares: &HashSet<&str>) {
        let c = self.c;
        let mut graines: HashMap<&str, i64> = HashMap::new();
        for graine in liste(&fsc["graines"]) {
            let bid = texte(&graine["brique_id"]);
            if graines.contains_key(bid) {
                self.erreur(
                    "fascicule.graine_dupliquee",
                    format!("La brique {bid} est semée deux fois."),
                    fid,
                    None,
                );
            }
            graines.insert(bid, entier(&graine["exemplaires"]));
            let brique = self.reference(
                &c.briques,
                graine["brique_id"].as_str(),
                fid,
                "graines",
                "brique",
                None,
            );
            if let Some(brique) = brique {
                if !declares.contains(texte(&brique["unite_id"])) {
                    self.erreur(
                        "fascicule.graine_hors_fascicule",
                        format!("La graine {bid} n’est pas une brique du fascicule."),
                        fid,
                        None,
                    );
                }
                if brique["rarete"] == "legendaire" {
                    self.erreur(
                        "fascicule.graine_legendaire",
                        format!("La graine {bid} est légendaire : la réserve de départ n’en donne jamais (ADR 0043)."),
                        fid,
                        None,
                    );
                }
            }
        }
        if graines.is_empty() {
            return;
        }
        let mut besoins: HashMap<&str, i64> = HashMap::new();
        let mut realisables = 0;
        let mut resultats: Vec<&str> = liste(&fsc["unites"])
            .iter()
            .filter(|l| l["role"] == "resultat")
            .map(|l| texte(&l["unite_id"]))
            .collect();
        resultats.sort_unstable();
        for uid in resultats {
            for recette in self.recettes_du_mot(uid) {
                let choix: Option<Vec<&str>> = liste(&recette["emplacements"])
                    .iter()
                    .map(|e| {
                        liste(&e["briques_acceptees"])
                            .iter()
                            .map(texte)
                            .find(|b| graines.contains_key(b))
                    })
                    .collect();
                let Some(choix) = choix else { continue };
                realisables += 1;
                for bid in choix {
                    *besoins.entry(bid).or_default() += 1;
                }
            }
        }
        let mut manquants: Vec<&str> = besoins
            .iter()
            .filter(|(bid, n)| **n > graines[**bid])
            .map(|(bid, _)| *bid)
            .collect();
        manquants.sort_unstable();
        if realisables < MIN_RECETTES_AMORCAGE || !manquants.is_empty() {
            let detail = if manquants.is_empty() {
                String::new()
            } else {
                format!(" ; exemplaires insuffisants pour {}", manquants.join(", "))
            };
            self.avert(
                "fascicule.amorcage",
                format!("{realisables} recette(s) réalisable(s) avec les graines, {MIN_RECETTES_AMORCAGE} attendues{detail}."),
                fid,
                None,
            );
        }
    }

    fn exclusions(&mut self) {
        let c = self.c;
        for (eid, exc) in &c.exclusions {
            self.reference(
                &c.fascicules,
                exc["fascicule_id"].as_str(),
                eid,
                "fascicule_id",
                "fascicule",
                None,
            );
            for bid in liste(&exc["combinaison"]) {
                self.reference(&c.briques, bid.as_str(), eid, "combinaison", "brique", None);
            }
            let suite: Vec<String> = liste(&exc["combinaison"])
                .iter()
                .map(|b| texte(b).to_owned())
                .collect();
            if let Some(recette) = self.suites_connues.get(&suite) {
                let message = format!("Cette combinaison est aussi la recette {recette}.");
                self.erreur("exclusion.recette", message, eid, None);
            }
        }
    }

    // ------------------------------------------------------------ trompe-l'œil

    /// Unités déclarées par un fascicule et tous ceux qui le précèdent (ordre de parution, puis numéro).
    fn unites_jusqua(&self, fascicule: &Value) -> HashSet<&'a str> {
        let borne = (texte(&fascicule["parution"]), entier(&fascicule["numero"]));
        self.c
            .fascicules
            .values()
            .filter(|f| (texte(&f["parution"]), entier(&f["numero"])) <= borne)
            .flat_map(|f| liste(&f["unites"]).iter().map(|u| texte(&u["unite_id"])))
            .collect()
    }

    /// Trompe-l'œil (ADR 0052) : une paire de briques qui s'écrit comme un mot sans en venir.
    /// La clé est la paire d'identifiants de briques, jamais une forme : le mauvais homographe
    /// se calcule, il ne se saisit pas.
    #[allow(clippy::too_many_lines)]
    fn trompes_l_oeil(&mut self) {
        let c = self.c;
        let autres = homographes::par_brique(&homographes::groupes(c));
        let mut deja: HashMap<Vec<String>, &str> = HashMap::new();
        let mut tries: Vec<(&String, &Value)> = c.trompes_l_oeil.iter().collect();
        tries.sort_by(|a, b| a.0.cmp(b.0));
        for (tid, trompe) in tries {
            let fascicule = self.reference(
                &c.fascicules,
                trompe["fascicule_id"].as_str(),
                tid,
                "fascicule_id",
                "fascicule",
                None,
            );
            let suite: Vec<String> = liste(&trompe["combinaison"])
                .iter()
                .map(|b| texte(b).to_owned())
                .collect();
            let briques: Vec<Option<&Value>> = suite
                .iter()
                .map(|b| self.reference(&c.briques, Some(b), tid, "combinaison", "brique", None))
                .collect();
            if briques.iter().all(Option::is_some)
                && let Some(fascicule) = fascicule
            {
                let declarees = self.unites_jusqua(fascicule);
                for brique in briques.iter().flatten() {
                    if !declarees.contains(texte(&brique["unite_id"])) {
                        self.erreur(
                            "trompe_l_oeil.paire",
                            format!(
                                "La brique {} n'est publiée ni dans ce fascicule ni dans un précédent.",
                                texte(&brique["id"])
                            ),
                            tid,
                            None,
                        );
                    }
                    if brique["rarete"] == "legendaire" {
                        self.erreur(
                            "trompe_l_oeil.legendaire",
                            format!(
                                "La brique {} est légendaire : un trompe-l'œil trahirait son existence (ADR 0015).",
                                texte(&brique["id"])
                            ),
                            tid,
                            None,
                        );
                    }
                }
            }
            if let Some(autre) = deja.insert(suite.clone(), tid) {
                self.erreur(
                    "trompe_l_oeil.doublon",
                    format!("Cette paire a déjà le trompe-l'œil {autre}."),
                    tid,
                    None,
                );
            }
            let inverse: Vec<String> = suite.iter().rev().cloned().collect();
            for (suite_testee, lien) in [(&suite, "La paire"), (&inverse, "L'inverse de la paire")]
            {
                if let Some(recette) = self.suites_connues.get(suite_testee) {
                    let message =
                        format!("{lien} est la recette {recette} : ce n'est pas une fausse piste.");
                    self.erreur("trompe_l_oeil.recette", message, tid, None);
                }
            }
            if c.exclusions.values().any(|e| {
                liste(&e["combinaison"])
                    .iter()
                    .map(texte)
                    .eq(suite.iter().map(String::as_str))
            }) {
                self.erreur(
                    "trompe_l_oeil.recette",
                    "Cette paire est une exclusion : un mot attesté, pas une fausse piste.",
                    tid,
                    None,
                );
            }
            for (rang, brique) in suite.iter().enumerate() {
                for autre in autres.get(brique).into_iter().flatten() {
                    let mut essai = suite.clone();
                    essai[rang].clone_from(autre);
                    if let Some(recette) = self.suites_connues.get(&essai) {
                        self.erreur(
                            "trompe_l_oeil.homographe_jouable",
                            format!(
                                "Remplacer la brique {rang_humain} par {autre} donne la recette {recette} : le serveur le dit déjà (« bonne forme, autre brique »).",
                                rang_humain = rang + 1
                            ),
                            tid,
                            None,
                        );
                    }
                }
            }
            if let Some(mot_id) = trompe["mot_id"].as_str() {
                self.trompe_l_oeil_du_mot(tid, trompe, mot_id, &briques);
            }
            if liste(&trompe["fait"]["sources"]).is_empty() {
                self.erreur(
                    "trompe_l_oeil.sources",
                    "Un trompe-l'œil affirme que le mot ne vient pas de ces briques : il cite au moins une source.",
                    tid,
                    None,
                );
            }
        }
    }

    /// Contrôles d'un trompe-l'œil qui vise un mot du catalogue : le mot existe, il n'est pas
    /// composé de ces deux unités, et l'explication courte ne le nomme pas.
    fn trompe_l_oeil_du_mot(
        &mut self,
        tid: &str,
        trompe: &Value,
        mot_id: &str,
        briques: &[Option<&Value>],
    ) {
        let c = self.c;
        let Some(mot) = self.reference(&c.unites, Some(mot_id), tid, "mot_id", "unité", None)
        else {
            return;
        };
        let unites: Vec<&str> = briques
            .iter()
            .flatten()
            .map(|b| texte(&b["unite_id"]))
            .collect();
        let compose_des_deux = c.compositions.values().any(|cmp| {
            cmp["mot_id"] == mot_id
                && unites.iter().all(|u| {
                    liste(&cmp["parties"])
                        .iter()
                        .any(|p| texte(&p["unite_id"]) == *u)
                })
        });
        if unites.len() == BRIQUES_PAR_FUSION && compose_des_deux {
            self.erreur(
                "trompe_l_oeil.origine",
                "La composition du mot mobilise bien ces deux unités : c'est une recette, pas une fausse piste.",
                tid,
                None,
            );
        }
        let formes: Vec<String> = liste(&mot["formes"])
            .iter()
            .flat_map(|f| mots_sans_accents(texte(&f["texte"])))
            .filter(|f| f.chars().count() >= 3)
            .collect();
        if let Some(courtes) = trompe["explication_courte"].as_object() {
            for (locale, phrase) in courtes {
                let mots = mots_sans_accents(texte(phrase));
                // Le mot lui-même ou son pluriel, jamais un simple fragment d'un autre mot.
                let nomme = mots.iter().any(|m| {
                    formes.iter().any(|f| {
                        m == f
                            || (m.starts_with(f.as_str())
                                && m.chars().count() <= f.chars().count() + 1)
                    })
                });
                if nomme {
                    self.erreur(
                        "trompe_l_oeil.divulgation",
                        format!("L'explication courte (locale {locale}) nomme le mot : elle se lit avant sa découverte."),
                        tid,
                        None,
                    );
                }
            }
        }
    }

    // ------------------------------------------------------------------ médias

    #[allow(clippy::too_many_lines)]
    fn medias(&mut self) {
        let c = self.c;
        let mut declares: HashSet<&str> = HashSet::new();
        let mut cibles: HashMap<(&str, &str, &str), &str> = HashMap::new();
        let mut tries: Vec<(&String, &Value)> = c.medias.iter().collect();
        tries.sort_by(|a, b| a.0.cmp(b.0));
        for (mid, media) in tries {
            let (nature, identifiant) = medias::cible(media);
            let (table, libelle) = match nature {
                "unite" => (&c.unites, "unité"),
                "langue" => (&c.langues, "langue"),
                _ => (&c.fascicules, "fascicule"),
            };
            self.reference(
                table,
                Some(identifiant),
                mid,
                &format!("{nature}_id"),
                libelle,
                None,
            );
            let type_media = texte(&media["type"]);
            let cle = (type_media, nature, identifiant);
            if let Some(autre) = cibles.get(&cle) {
                let mut majuscule = libelle.chars();
                let libelle = majuscule
                    .next()
                    .map(|p| p.to_uppercase().chain(majuscule).collect::<String>())
                    .unwrap_or_default();
                self.erreur(
                    "media.double",
                    format!("{libelle} {identifiant} a déjà une {type_media} : {autre}."),
                    mid,
                    None,
                );
            }
            cibles.entry(cle).or_insert(mid);

            let fichier_media = texte(&media["fichier"]);
            declares.insert(fichier_media);
            let fichier = medias::chemin(c, media);
            if !fichier.is_file() {
                self.erreur(
                    "media.fichier_absent",
                    format!("Fichier {}/{fichier_media} introuvable.", medias::DOSSIER),
                    mid,
                    None,
                );
                continue;
            }
            let problemes =
                medias::verifier_svg(&fs::read(&fichier).unwrap_or_default(), type_media);
            if !problemes.is_empty() {
                let suite = if problemes.len() > 3 {
                    format!(" (et {} autre(s))", problemes.len() - 3)
                } else {
                    String::new()
                };
                let premiers = problemes[..problemes.len().min(3)].join(" ; ");
                self.erreur(
                    "media.svg_invalide",
                    format!("{}/{fichier_media} : {premiers}{suite}.", medias::DOSSIER),
                    mid,
                    None,
                );
            }
        }

        let dossier = c.racine.join(medias::DOSSIER);
        if dossier.is_dir() {
            let mut fichiers = Vec::new();
            parcourir(&dossier, &mut Vec::new(), &mut fichiers);
            fichiers.sort();
            for parties in fichiers {
                let relatif = parties.join("/");
                if !parties.last().is_some_and(|n| n.starts_with('.'))
                    && !declares.contains(relatif.as_str())
                {
                    self.r.avertissement(
                        "media.orphelin",
                        "Aucun média ne déclare ce fichier.",
                        Some(&format!("{}/{relatif}", medias::DOSSIER)),
                        None,
                    );
                }
            }
        }

        // Chaque carte et chaque langue citée a sa silhouette, chaque fascicule sa jaquette (ADR 0016, 0023).
        let mut fascicules: Vec<(&String, &Value)> = c.fascicules.iter().collect();
        fascicules.sort_by(|a, b| a.0.cmp(b.0));
        for (fid, fsc) in fascicules {
            if vrai(&fsc["test"]) {
                continue;
            }
            let uids: Vec<String> = liste(&fsc["unites"])
                .iter()
                .map(|l| texte(&l["unite_id"]).to_owned())
                .collect();
            let cartes: Vec<&str> = uids
                .iter()
                .map(String::as_str)
                .filter(|u| !cibles.contains_key(&("silhouette", "unite", *u)))
                .collect();
            let langues: Vec<String> = medias::langues_citees(c, &uids)
                .into_iter()
                .filter(|l| !cibles.contains_key(&("silhouette", "langue", l.as_str())))
                .collect();
            let numero = entier(&fsc["numero"]);
            if !cartes.is_empty() {
                // Une carte affectée à un fascicule doit avoir son visuel, quel que soit le mode.
                let detail = format!(
                    "{} carte(s) sans silhouette : {}",
                    cartes.len(),
                    cartes.join(", ")
                );
                self.erreur(
                    "media.manquant",
                    format!("Fascicule {numero} : {detail}."),
                    fid,
                    None,
                );
            }
            if !langues.is_empty() {
                let detail = format!(
                    "{} langue(s) sans silhouette : {}",
                    langues.len(),
                    langues.join(", ")
                );
                self.signaler(
                    "media.manquant",
                    format!("Fascicule {numero} : {detail}."),
                    fid,
                );
            }
            if !cibles.contains_key(&("jaquette", "fascicule", fid.as_str())) {
                self.signaler(
                    "media.manquant",
                    format!("Fascicule {numero} : jaquette absente."),
                    fid,
                );
            }
        }

        // Carte en attente d'affectation : simple avertissement. Seule une brique ou le résultat
        // d'une recette a une carte ; un étymon n'en a pas (ADR 0013).
        let affectees: HashSet<&str> = c
            .fascicules
            .values()
            .flat_map(|f| liste(&f["unites"]))
            .map(|l| texte(&l["unite_id"]))
            .collect();
        let mut avec_carte: HashSet<&str> =
            c.briques.values().map(|b| texte(&b["unite_id"])).collect();
        avec_carte.extend(c.recettes.values().filter_map(|r| {
            c.compositions
                .get(texte(&r["composition_id"]))
                .map(|cmp| texte(&cmp["mot_id"]))
        }));
        let mut uids: Vec<&String> = c.unites.keys().collect();
        uids.sort();
        for uid in uids {
            if avec_carte.contains(uid.as_str())
                && !affectees.contains(uid.as_str())
                && !cibles.contains_key(&("silhouette", "unite", uid.as_str()))
            {
                self.avert(
                    "media.manquant",
                    "Carte en attente d’affectation sans silhouette.",
                    uid,
                    None,
                );
            }
        }
    }

    // ------------------------------------------------------ mode publication

    fn references(&self, fait: &Value) -> usize {
        liste(&fait["sources"])
            .iter()
            .filter(|s| {
                self.c
                    .sources
                    .get(texte(&s["source_id"]))
                    .is_some_and(|s| texte(&s["url"]).starts_with(SOURCE_WIKIDATA))
            })
            .count()
    }

    fn exiger_publiable(&mut self, fid: &str, fichier_de: &str, fait: &Value, fascicule: &str) {
        let etat = texte(&fait["etat_relecture"]);
        if etat != "relu" {
            self.erreur(
                "publication.relecture",
                format!("Fait publié par {fascicule} sans relecture ({etat})."),
                fid,
                Some(fichier_de),
            );
        }
        if fait["confiance"] == "non_retenue" {
            self.erreur(
                "publication.non_retenue",
                format!("Fait non retenu publié par {fascicule}."),
                fid,
                Some(fichier_de),
            );
        }
        let minimum = if fait["confiance"] == "discutee" {
            2
        } else {
            1
        };
        if self.references(fait) < minimum {
            self.erreur(
                "publication.source_reference",
                format!("Fait publié par {fascicule} : il faut au moins {minimum} source(s) Wikidata ; Wikidata est la seule source de vérité (ADR 0060)."),
                fid,
                Some(fichier_de),
            );
        }
        for lien in liste(&fait["sources"]) {
            if !vrai(&lien["localisation"]) {
                self.erreur(
                    "publication.localisation",
                    format!(
                        "Source {} sans entrée ni page consultée.",
                        texte(&lien["source_id"])
                    ),
                    fid,
                    Some(fichier_de),
                );
            }
        }
    }

    fn exclu(fait: &Value) -> bool {
        fait["etat_relecture"] == "rejete" || fait["confiance"] == "non_retenue"
    }

    fn publiable(&mut self) {
        let c = self.c;
        let publiables: Vec<(&String, &Value)> = c
            .fascicules
            .iter()
            .filter(|(_, f)| !vrai(&f["test"]))
            .collect();
        if publiables.is_empty() {
            self.r.avertissement(
                "publication.aucun_fascicule",
                "Aucun fascicule publiable : les fascicules de test ne sont jamais publiés.",
                None,
                None,
            );
        }
        for (fsc_id, fsc) in publiables {
            let numero = format!("le fascicule {}", entier(&fsc["numero"]));
            let unites: HashSet<&str> = liste(&fsc["unites"])
                .iter()
                .map(|l| texte(&l["unite_id"]))
                .collect();
            let mut essentiels: HashSet<&str> = HashSet::new();
            for uid in &unites {
                let Some(unite) = c.unites.get(*uid) else {
                    continue;
                };
                essentiels.insert(uid);
                if let Some(canonique) = Self::canonique(unite) {
                    essentiels.insert(texte(&canonique["id"]));
                }
                if let Some(sens) = self.sens_affiche(uid) {
                    essentiels.insert(texte(&sens["id"]));
                }
                for recette in self.recettes_du_mot(uid) {
                    let cmp = &c.compositions[texte(&recette["composition_id"])];
                    essentiels.insert(texte(&cmp["id"]));
                    for partie in liste(&cmp["parties"]) {
                        essentiels.insert(texte(&partie["forme_id"]));
                        essentiels.insert(texte(&partie["sens_id"]));
                    }
                }
            }
            let etymons: HashSet<&str> = c
                .relations
                .values()
                .filter(|r| unites.contains(texte(&r["unite_cible_id"])))
                .map(|r| texte(&r["unite_source_id"]))
                .collect();
            for (fid, fichier_de, fait) in self.faits() {
                let dans_perimetre = unites.contains(fichier_de)
                    || etymons.contains(fichier_de)
                    || c.compositions
                        .get(fichier_de)
                        .is_some_and(|cmp| unites.contains(texte(&cmp["mot_id"])))
                    || c.relations
                        .get(fichier_de)
                        .is_some_and(|r| unites.contains(texte(&r["unite_cible_id"])));
                if !dans_perimetre || (Self::exclu(fait) && !essentiels.contains(fid)) {
                    continue;
                }
                self.exiger_publiable(fid, fichier_de, fait, &numero);
            }
            for (tid, trompe) in &c.trompes_l_oeil {
                if trompe["fascicule_id"] == fsc_id.as_str()
                    && trompe["fait"]["etat_relecture"] != "relu"
                {
                    self.erreur(
                        "publication.trompe_l_oeil",
                        format!("Trompe-l'œil de {numero} sans relecture."),
                        tid,
                        None,
                    );
                }
            }
            for (eid, exc) in &c.exclusions {
                if exc["fascicule_id"] == fsc_id.as_str() && exc["etat_relecture"] != "relu" {
                    self.erreur(
                        "publication.exclusion",
                        format!("Exclusion de {numero} sans relecture."),
                        eid,
                        None,
                    );
                }
            }
        }
    }
}

/// Mots d'une phrase en minuscules sans accent : de quoi y retrouver un mot du catalogue.
pub(crate) fn mots_sans_accents(texte: &str) -> Vec<String> {
    let nettoye: String = texte
        .nfd()
        .filter(|c| !unicode_normalization::char::is_combining_mark(*c))
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .flat_map(char::to_lowercase)
        .collect();
    nettoye.split_whitespace().map(str::to_owned).collect()
}

/// Fichiers d'un dossier, récursivement, en chemins découpés : l'ordre de tri est celui de pathlib.
fn parcourir(dossier: &Path, prefixe: &mut Vec<String>, fichiers: &mut Vec<Vec<String>>) {
    let Ok(entrees) = fs::read_dir(dossier) else {
        return;
    };
    for entree in entrees.flatten() {
        let chemin = entree.path();
        prefixe.push(entree.file_name().to_string_lossy().into_owned());
        if chemin.is_dir() {
            parcourir(&chemin, prefixe, fichiers);
        } else if chemin.is_file() {
            fichiers.push(prefixe.clone());
        }
        prefixe.pop();
    }
}

pub fn controler(catalogue: &Catalogue, rapport: &mut Rapport, publication: bool) {
    Controleur::new(catalogue, rapport, publication).executer();
}
