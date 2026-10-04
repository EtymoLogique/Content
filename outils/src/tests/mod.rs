//! Tests : une copie jetable du dépôt, que chaque test modifie pour déclencher une règle.

mod compilation;
mod prepublication;
mod validation;
mod visualisation;

use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use serde_json::{Value, json};
use tempfile::TempDir;

use crate::json::{LISIBLE, ecrire, liste, texte};
use crate::rapport::{AVERTISSEMENT, ERREUR, Rapport};
use crate::valider as cli_valider;

const A_COPIER: [&str; 8] = [
    "schemas",
    "referentiels",
    "sources",
    "unites",
    "compositions",
    "relations",
    "ludique",
    "medias",
];
pub const SVG_SILHOUETTE: &str = concat!(
    r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 300 400">"#,
    r#"<circle cx="200" cy="280" r="90" fill="currentColor"/><path d="M150 280L250 280" style="stroke:var(--art-bg)"/></svg>"#,
    "\n"
);
pub const SVG_JAQUETTE: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 600 800\"><rect width=\"600\" height=\"800\" fill=\"#1f5c4a\"/></svg>\n";

/// Un mot saisi avant d’être affecté à un fascicule reste un avertissement du validateur,
/// jamais un échec des tests : le dépôt peut porter des cartes orphelines.
const TOLERES: [&str; 2] = ["recette.hors_fascicule", "media.manquant"];

pub type Trouve = (PathBuf, Value);

fn copier(source: &Path, cible: &Path) {
    fs::create_dir_all(cible).unwrap();
    for entree in fs::read_dir(source).unwrap() {
        let entree = entree.unwrap();
        let chemin = cible.join(entree.file_name());
        if entree.file_type().unwrap().is_dir() {
            copier(&entree.path(), &chemin);
        } else {
            fs::write(chemin, fs::read(entree.path()).unwrap()).unwrap();
        }
    }
}

pub fn id(prefixe: &str) -> String {
    crate::ids::nouvel_id(prefixe, &HashSet::new()).unwrap()
}

pub fn retirer(objet: &mut Value, cle: &str) -> Value {
    objet
        .as_object_mut()
        .and_then(|o| o.shift_remove(cle))
        .unwrap_or(Value::Null)
}

pub fn erreurs(rapport: &Rapport) -> BTreeSet<String> {
    rapport.regles(Some(ERREUR))
}

pub fn avertissements(rapport: &Rapport) -> BTreeSet<String> {
    rapport.regles(Some(AVERTISSEMENT))
}

pub fn avertissements_notables(rapport: &Rapport) -> BTreeSet<String> {
    avertissements(rapport)
        .into_iter()
        .filter(|r| !TOLERES.contains(&r.as_str()))
        .collect()
}

pub fn ensemble(regles: &[&str]) -> BTreeSet<String> {
    regles.iter().map(|r| (*r).to_owned()).collect()
}

/// Catalogue du dépôt, plus le fascicule d'essai, sans les fascicules publiés.
/// Le dossier temporaire n'est jamais supprimé (statique) : un par exécution, comme `tmp_path` de pytest.
fn gabarit() -> &'static Path {
    static GABARIT: OnceLock<TempDir> = OnceLock::new();
    GABARIT
        .get_or_init(|| {
            let dossier = TempDir::new().unwrap();
            let racine = dossier.path();
            let source = Path::new(env!("CARGO_MANIFEST_DIR"));
            for d in A_COPIER {
                copier(&source.parent().unwrap().join(d), &racine.join(d));
            }
            // Le fascicule d'essai Bêta n'est plus dans le catalogue : sa copie de test s'ajoute au dépôt jetable.
            copier(&source.join("tests/fixtures"), racine);
            let depot = Depot {
                racine: racine.to_owned(),
                _dossier: TempDir::new().unwrap(),
            };
            depot.retirer_fascicules_publies();
            // Le fascicule d'essai porte une légendaire, comme l'exige le validateur (ADR 0043).
            depot.modifier(depot.brique("étymo"), |b| {
                b["rarete"] = json!("legendaire");
            });
            dossier
        })
        .path()
}

pub struct Depot {
    pub racine: PathBuf,
    _dossier: TempDir,
}

#[allow(clippy::unused_self)] // lire et ecrire restent des méthodes du dépôt, comme dans conftest.py
impl Depot {
    /// Copie du gabarit : le catalogue du dépôt, préparé une fois par exécution.
    pub fn new() -> Self {
        let dossier = TempDir::new().unwrap();
        let racine = dossier.path().join("depot");
        copier(gabarit(), &racine);
        Self {
            racine,
            _dossier: dossier,
        }
    }

    /// Les tests partent du seul fascicule d'essai : les fascicules publiés, leurs jaquettes et leurs exclusions partent de la copie.
    fn retirer_fascicules_publies(&self) {
        let mut publies = HashSet::new();
        for chemin in self.fichiers("ludique/fascicules") {
            let f = self.lire(&chemin);
            if f["test"] == false {
                publies.insert(texte(&f["id"]).to_owned());
                fs::remove_file(chemin).unwrap();
            }
        }
        for chemin in self.fichiers("ludique/medias") {
            let m = self.lire(&chemin);
            if publies.contains(texte(&m["fascicule_id"])) {
                fs::remove_file(self.racine.join("medias").join(texte(&m["fichier"]))).unwrap();
                fs::remove_file(chemin).unwrap();
            }
        }
        for chemin in self.fichiers("ludique/exclusions") {
            if publies.contains(texte(&self.lire(&chemin)["fascicule_id"])) {
                fs::remove_file(chemin).unwrap();
            }
        }
    }

    pub fn lire(&self, chemin: &Path) -> Value {
        crate::json::lire(&fs::read_to_string(chemin).unwrap()).unwrap()
    }

    pub fn ecrire(&self, chemin: &Path, donnees: &Value) {
        fs::create_dir_all(chemin.parent().unwrap()).unwrap();
        fs::write(chemin, ecrire(donnees, LISIBLE)).unwrap();
    }

    pub fn fichiers(&self, dossier: &str) -> Vec<PathBuf> {
        let mut fichiers: Vec<PathBuf> = fs::read_dir(self.racine.join(dossier))
            .into_iter()
            .flatten()
            .map(|e| e.unwrap().path())
            .filter(|p| p.extension().is_some_and(|e| e == "json"))
            .collect();
        fichiers.sort();
        fichiers
    }

    pub fn trouver(&self, dossier: &str, critere: impl Fn(&Value) -> bool) -> Trouve {
        let mut trouves: Vec<Trouve> = self
            .fichiers(dossier)
            .into_iter()
            .map(|p| (p.clone(), self.lire(&p)))
            .filter(|(_, d)| critere(d))
            .collect();
        assert_eq!(
            trouves.len(),
            1,
            "{} objets trouvés dans {dossier}",
            trouves.len()
        );
        trouves.pop().unwrap()
    }

    pub fn unite_glose(&self, texte_forme: &str, glose: Option<&str>) -> Trouve {
        self.trouver("unites", |u| {
            let canonique = liste(&u["formes"])
                .iter()
                .find(|f| f["role"] == "canonique")
                .unwrap();
            canonique["texte"] == texte_forme
                && glose.is_none_or(|g| u["sens"][0]["libelles"]["fr"]["glose"] == g)
        })
    }

    pub fn unite(&self, texte_forme: &str) -> Trouve {
        self.unite_glose(texte_forme, None)
    }

    pub fn uid(&self, texte_forme: &str) -> String {
        texte(&self.unite(texte_forme).1["id"]).to_owned()
    }

    pub fn brique(&self, texte_forme: &str) -> Trouve {
        let uid = self.uid(texte_forme);
        self.trouver("ludique/briques", |b| b["unite_id"] == uid.as_str())
    }

    pub fn id_brique(&self, texte_forme: &str) -> String {
        texte(&self.brique(texte_forme).1["id"]).to_owned()
    }

    pub fn composition(&self, texte_mot: &str) -> Trouve {
        let uid = self.uid(texte_mot);
        self.trouver("compositions", |c| c["mot_id"] == uid.as_str())
    }

    pub fn recette(&self, texte_mot: &str) -> Trouve {
        let cid = self.composition(texte_mot).1["id"].clone();
        self.trouver("ludique/recettes", |r| r["composition_id"] == cid)
    }

    pub fn relation_glose(&self, texte_cible: &str, glose: Option<&str>) -> Trouve {
        let uid = self.unite_glose(texte_cible, glose).1["id"].clone();
        self.trouver("relations", |r| r["unite_cible_id"] == uid)
    }

    pub fn relation(&self, texte_cible: &str) -> Trouve {
        self.relation_glose(texte_cible, None)
    }

    /// Le fascicule Bêta n° 0, base de tous les tests.
    pub fn fascicule(&self) -> Trouve {
        self.trouver("ludique/fascicules", |f| f["id"] == "fsc_qcnkaf5m")
    }

    /// Déclare un média et écrit son fichier : par défaut, un SVG minimal conforme au profil.
    pub fn media(&self, type_media: &str, cle: &str, cible: &str, svg: Option<&str>) -> Value {
        let identifiant = id("med");
        let dossier = if type_media == "silhouette" {
            "silhouettes"
        } else {
            "jaquettes"
        };
        let fichier = format!("{dossier}/essai-{}.svg", &identifiant[4..]);
        let svg = svg.unwrap_or(if type_media == "silhouette" {
            SVG_SILHOUETTE
        } else {
            SVG_JAQUETTE
        });
        let cible_fichier = self.racine.join("medias").join(&fichier);
        fs::create_dir_all(cible_fichier.parent().unwrap()).unwrap();
        fs::write(cible_fichier, svg).unwrap();
        let mut donnees = json!({"id": identifiant, "type": type_media, "fichier": fichier});
        donnees[cle] = json!(cible);
        self.ecrire(
            &self
                .racine
                .join("ludique/medias")
                .join(format!("{identifiant}.json")),
            &donnees,
        );
        donnees
    }

    pub fn media_de(&self, cle: &str, cible: &str) -> Trouve {
        self.trouver("ludique/medias", |m| m[cle] == cible)
    }

    pub fn modifier(&self, trouve: Trouve, fonction: impl FnOnce(&mut Value)) -> Value {
        let (chemin, mut donnees) = trouve;
        fonction(&mut donnees);
        self.ecrire(&chemin, &donnees);
        donnees
    }

    pub fn copier_sous(
        &self,
        trouve: Trouve,
        nouvel_id: &str,
        fonction: impl FnOnce(&mut Value),
    ) -> Value {
        let (chemin, mut donnees) = trouve;
        donnees["id"] = json!(nouvel_id);
        fonction(&mut donnees);
        self.ecrire(
            &chemin.with_file_name(format!("{nouvel_id}.json")),
            &donnees,
        );
        donnees
    }

    pub fn valider(&self) -> Rapport {
        cli_valider(&self.racine, false).unwrap().1
    }

    pub fn valider_publication(&self) -> Rapport {
        cli_valider(&self.racine, true).unwrap().1
    }
}
