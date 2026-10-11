//! Lecture du dépôt éditorial : fichiers JSON, validation par schéma, index des objets.

use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use indexmap::IndexMap;
use jsonschema::paths::LocationSegment;
use jsonschema::{Registry, Validator};
use serde_json::Value;

use crate::ids::prefixe;
use crate::json::{self, ErreurLecture, liste, texte};
use crate::rapport::Rapport;

/// Objets d'un type, par identifiant, dans l'ordre de lecture.
pub type Table = IndexMap<String, Value>;

/// Dossier → (nom du schéma, préfixe attendu).
pub const DOSSIERS: [(&str, &str, &str); 13] = [
    ("referentiels/langues", "langue", "lng"),
    ("referentiels/ecritures", "ecriture", "ecr"),
    ("referentiels/conventions", "convention", "cnv"),
    ("sources", "source", "src"),
    ("unites", "unite", "unt"),
    ("compositions", "composition", "cmp"),
    ("relations", "relation", "rel"),
    ("ludique/briques", "brique", "brq"),
    ("ludique/recettes", "recette", "rct"),
    ("ludique/fascicules", "fascicule", "fsc"),
    ("ludique/exclusions", "exclusion", "exc"),
    ("ludique/trompe-l-oeil", "trompe-l-oeil", "tpl"),
    ("ludique/medias", "media", "med"),
];

pub const EQUILIBRAGE: &str = "ludique/equilibrage.json";
/// Quêtes au long cours et cadeau de bienvenue, communs à tous les fascicules (ADR 0045, 0046).
pub const QUETES: &str = "ludique/quetes.json";

const VOCABULAIRES: [&str; 2] = [
    "referentiels/types-relation.json",
    "referentiels/types-transformation.json",
];

#[derive(Debug, Default)]
pub struct Catalogue {
    pub racine: PathBuf,
    pub langues: Table,
    pub ecritures: Table,
    pub conventions: Table,
    pub sources: Table,
    pub unites: Table,
    pub compositions: Table,
    pub relations: Table,
    pub briques: Table,
    pub recettes: Table,
    pub fascicules: Table,
    pub exclusions: Table,
    pub trompes_l_oeil: Table,
    pub medias: Table,
    pub types_relation: Table,
    pub types_transformation: Table,
    /// Paramètres des plis (ludique/equilibrage.json), recopiés dans chaque manifeste.
    pub equilibrage: Option<Value>,
    /// Quêtes au long cours et cadeau de bienvenue (ludique/quetes.json), recopiés dans l'index global.
    pub quetes: Option<Value>,
    /// Objets imbriqués : identifiant → (unité, objet).
    pub formes: HashMap<String, (String, Value)>,
    pub sens: HashMap<String, (String, Value)>,
    /// Identifiant → fichier relatif, pour les diagnostics.
    pub fichiers: HashMap<String, String>,
    /// Identifiants présents dans un fichier invalide : leurs références ne sont pas signalées.
    pub ids_invalides: HashSet<String>,
}

impl Catalogue {
    #[must_use]
    pub fn fichier(&self, identifiant: Option<&str>) -> Option<&str> {
        identifiant
            .filter(|i| !i.is_empty())
            .and_then(|i| self.fichiers.get(i))
            .map(String::as_str)
    }

    fn table_mut(&mut self, schema: &str) -> &mut Table {
        match schema {
            "langue" => &mut self.langues,
            "ecriture" => &mut self.ecritures,
            "convention" => &mut self.conventions,
            "source" => &mut self.sources,
            "unite" => &mut self.unites,
            "composition" => &mut self.compositions,
            "relation" => &mut self.relations,
            "brique" => &mut self.briques,
            "recette" => &mut self.recettes,
            "fascicule" => &mut self.fascicules,
            "exclusion" => &mut self.exclusions,
            "trompe-l-oeil" => &mut self.trompes_l_oeil,
            "media" => &mut self.medias,
            "types-relation" => &mut self.types_relation,
            _ => &mut self.types_transformation,
        }
    }
}

fn validateurs(dossier: &Path) -> Result<HashMap<String, Validator>, String> {
    let mut schemas = Vec::new();
    let mut entrees: Vec<PathBuf> = fs::read_dir(dossier)
        .map_err(|e| format!("{} : {e}", dossier.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.ends_with(".schema.json"))
        })
        .collect();
    entrees.sort();
    for chemin in entrees {
        let contenu: Value = fs::read_to_string(&chemin)
            .map_err(|e| e.to_string())
            .and_then(|t| serde_json::from_str(&t).map_err(|e| e.to_string()))
            .map_err(|e| format!("{} : {e}", chemin.display()))?;
        let nom = chemin
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .trim_end_matches(".schema.json")
            .to_owned();
        schemas.push((nom, contenu));
    }
    let registre = Registry::new()
        .extend(
            schemas
                .iter()
                .map(|(_, s)| (texte(&s["$id"]).to_owned(), s.clone())),
        )
        .and_then(jsonschema::RegistryBuilder::prepare)
        .map_err(|e| format!("Schémas : {e}"))?;
    schemas
        .iter()
        .map(|(nom, schema)| {
            jsonschema::options()
                .with_registry(&registre)
                .should_validate_formats(false)
                .build(schema)
                .map(|v| (nom.clone(), v))
                .map_err(|e| format!("Schéma {nom} : {e}"))
        })
        .collect()
}

fn lire_json(chemin: &Path, relatif: &str, rapport: &mut Rapport) -> Option<Value> {
    rapport.fichiers_lus += 1;
    let lu = fs::read(chemin)
        .map_err(|e| e.to_string())
        .and_then(|o| String::from_utf8(o).map_err(|e| e.to_string()));
    match lu
        .map_err(ErreurLecture::Syntaxe)
        .and_then(|t| json::lire(&t))
    {
        Ok(v) => Some(v),
        Err(ErreurLecture::CleDupliquee(cle)) => {
            rapport.erreur(
                "json.cle_dupliquee",
                format!("Clé « {cle} » en double : une seule ligne par objet et par locale."),
                Some(relatif),
                None,
            );
            None
        }
        Err(ErreurLecture::Syntaxe(e)) => {
            rapport.erreur(
                "json.syntaxe",
                format!("JSON illisible : {e}"),
                Some(relatif),
                None,
            );
            None
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum Segment {
    Indice(usize),
    Cle(String),
}

fn valider_schema(
    validateur: &Validator,
    donnees: &Value,
    relatif: &str,
    rapport: &mut Rapport,
) -> bool {
    let mut erreurs: Vec<(Vec<Segment>, String)> = validateur
        .iter_errors(donnees)
        .map(|e| {
            let chemin = e
                .instance_path()
                .iter()
                .map(|s| match s {
                    LocationSegment::Property(p) => Segment::Cle(p.into_owned()),
                    LocationSegment::Index(i) => Segment::Indice(i),
                })
                .collect();
            (chemin, e.to_string())
        })
        .collect();
    erreurs.sort_by(|a, b| a.0.cmp(&b.0));
    for (chemin, message) in &erreurs {
        let texte: Vec<String> = chemin
            .iter()
            .map(|s| match s {
                Segment::Indice(i) => i.to_string(),
                Segment::Cle(c) => c.clone(),
            })
            .collect();
        let texte = if texte.is_empty() {
            "(racine)".to_owned()
        } else {
            texte.join("/")
        };
        rapport.erreur(
            "schema",
            format!("{texte} : {message}"),
            Some(relatif),
            None,
        );
    }
    erreurs.is_empty()
}

fn ids_imbriques(donnees: &Value) -> Vec<String> {
    ["formes", "sens", "transformations"]
        .iter()
        .flat_map(|cle| liste(&donnees[*cle]))
        .chain(donnees.get("attestation"))
        .filter_map(|o| o["id"].as_str().map(str::to_owned))
        .collect()
}

fn nom(chemin: &Path) -> &str {
    chemin
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
}

fn tronc(chemin: &Path) -> &str {
    chemin
        .file_stem()
        .and_then(|n| n.to_str())
        .unwrap_or_default()
}

/// Lit tout le dépôt. Une erreur ne concerne que les schémas, illisibles ou absents.
pub fn charger(racine: &Path, rapport: &mut Rapport) -> Result<Catalogue, String> {
    let racine = fs::canonicalize(racine).map_err(|e| format!("{} : {e}", racine.display()))?;
    let mut catalogue = Catalogue {
        racine: racine.clone(),
        ..Catalogue::default()
    };
    let validateurs = validateurs(&racine.join("schemas"))?;
    let validateur = |nom: &str| {
        validateurs
            .get(nom)
            .ok_or_else(|| format!("Schéma {nom} absent."))
    };

    for relatif in VOCABULAIRES {
        let chemin = racine.join(relatif);
        if !chemin.exists() {
            rapport.erreur(
                "fichier.absent",
                "Vocabulaire contrôlé absent.",
                Some(relatif),
                None,
            );
            continue;
        }
        let Some(donnees) = lire_json(&chemin, relatif, rapport) else {
            continue;
        };
        if !valider_schema(validateur("vocabulaire")?, &donnees, relatif, rapport) {
            continue;
        }
        let cible = catalogue.table_mut(
            relatif
                .trim_start_matches("referentiels/")
                .trim_end_matches(".json"),
        );
        let mut doublons = Vec::new();
        for entree in liste(&donnees["types"]) {
            let code = texte(&entree["code"]).to_owned();
            if cible.contains_key(&code) {
                doublons.push(code.clone());
            }
            cible.insert(code, entree.clone());
        }
        for code in doublons {
            rapport.erreur(
                "vocabulaire.doublon",
                format!("Code « {code} » déclaré deux fois."),
                Some(relatif),
                None,
            );
        }
    }

    let relatif = EQUILIBRAGE;
    if racine.join(relatif).exists() {
        if let Some(donnees) = lire_json(&racine.join(relatif), relatif, rapport)
            && valider_schema(validateur("equilibrage")?, &donnees, relatif, rapport)
        {
            catalogue.equilibrage = Some(donnees);
        }
    } else {
        rapport.erreur(
            "fichier.absent",
            "Paramètres d’équilibrage des plis absents.",
            Some(relatif),
            None,
        );
    }

    // Facultatif : sans ce fichier, l'index ne déclare ni quête ni cadeau.
    let relatif = QUETES;
    if racine.join(relatif).exists()
        && let Some(donnees) = lire_json(&racine.join(relatif), relatif, rapport)
        && valider_schema(validateur("quetes")?, &donnees, relatif, rapport)
    {
        catalogue.quetes = Some(donnees);
    }

    let mut vus: HashMap<String, String> = HashMap::new();
    for (dossier, nom_schema, prefixe_attendu) in DOSSIERS {
        let repertoire = racine.join(dossier);
        if !repertoire.is_dir() {
            continue;
        }
        let mut chemins: Vec<PathBuf> = fs::read_dir(&repertoire)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .collect();
        chemins.sort();
        for chemin in chemins {
            let relatif = format!("{dossier}/{}", nom(&chemin));
            if nom(&chemin).starts_with('.') || chemin.is_dir() {
                continue;
            }
            if chemin.extension().is_none_or(|e| e != "json") {
                rapport.erreur(
                    "fichier.inattendu",
                    "Seuls des fichiers .json sont admis dans ce dossier.",
                    Some(&relatif),
                    None,
                );
                continue;
            }
            let Some(donnees) = lire_json(&chemin, &relatif, rapport) else {
                catalogue.ids_invalides.insert(tronc(&chemin).to_owned());
                continue;
            };
            let identifiant = donnees["id"].as_str().map(str::to_owned);
            let mut valide = valider_schema(validateur(nom_schema)?, &donnees, &relatif, rapport);

            if let Some(identifiant) = &identifiant {
                if tronc(&chemin) != identifiant {
                    rapport.erreur(
                        "identite.nom_fichier",
                        format!("Le fichier doit s’appeler {identifiant}.json."),
                        Some(&relatif),
                        Some(identifiant),
                    );
                    valide = false;
                }
                if prefixe(identifiant) != Some(prefixe_attendu) {
                    rapport.erreur(
                        "identite.prefixe",
                        format!(
                            "Un objet de « {dossier} » a un identifiant en {prefixe_attendu}_."
                        ),
                        Some(&relatif),
                        Some(identifiant),
                    );
                    valide = false;
                }
            }

            let tous_ids: Vec<String> = identifiant
                .iter()
                .cloned()
                .chain(ids_imbriques(&donnees))
                .collect();
            for i in &tous_ids {
                if let Some(deja) = vus.get(i) {
                    rapport.erreur(
                        "identite.doublon",
                        format!("Identifiant déjà utilisé dans {deja}."),
                        Some(&relatif),
                        Some(i),
                    );
                    valide = false;
                } else {
                    vus.insert(i.clone(), relatif.clone());
                    catalogue.fichiers.insert(i.clone(), relatif.clone());
                }
            }

            let (true, Some(identifiant)) = (valide, identifiant) else {
                catalogue.ids_invalides.extend(tous_ids);
                continue;
            };
            if nom_schema == "unite" {
                for forme in liste(&donnees["formes"]) {
                    catalogue.formes.insert(
                        texte(&forme["id"]).to_owned(),
                        (identifiant.clone(), forme.clone()),
                    );
                }
                for sens in liste(&donnees["sens"]) {
                    catalogue.sens.insert(
                        texte(&sens["id"]).to_owned(),
                        (identifiant.clone(), sens.clone()),
                    );
                }
            }
            catalogue.table_mut(nom_schema).insert(identifiant, donnees);
        }
    }
    Ok(catalogue)
}
