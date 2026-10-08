//! Pièges probables (ADR 0052) : les paires de briques dont la concaténation est la forme d'un mot du
//! catalogue, sans recette, sans exclusion et sans trompe-l'œil. Un rapport d'aide à l'édition : il ne
//! publie rien et ne bloque rien. Un mot absent du catalogue (*semoule*) reste à repérer à la main.

use std::collections::HashSet;

use crate::chargement::Catalogue;
use crate::controles::{Controleur, mots_sans_accents};
use crate::homographes;
use crate::json::{liste, texte};

pub struct Candidat {
    pub premiere: String,
    pub seconde: String,
    pub mot_id: String,
    pub forme: String,
}

fn lettres(texte: &str) -> String {
    mots_sans_accents(texte).concat()
}

fn forme_canonique(c: &Catalogue, brique: &serde_json::Value) -> Option<String> {
    let unite = c.unites.get(texte(&brique["unite_id"]))?;
    liste(&unite["formes"])
        .iter()
        .find(|f| f["role"] == "canonique")
        .map(|f| lettres(texte(&f["texte"])))
}

/// Paires candidates, triées par identifiants de briques.
#[must_use]
pub fn chercher(c: &Catalogue) -> Vec<Candidat> {
    let mut connues: HashSet<Vec<String>> = HashSet::new();
    for recette in c.recettes.values() {
        connues.extend(Controleur::suites(liste(&recette["emplacements"])));
    }
    for fausse in c.trompes_l_oeil.values().chain(c.exclusions.values()) {
        connues.insert(
            liste(&fausse["combinaison"])
                .iter()
                .map(|b| texte(b).to_owned())
                .collect(),
        );
    }
    let autres = homographes::par_brique(&homographes::groupes(c));
    let mots: Vec<(&str, String)> = c
        .unites
        .iter()
        .filter(|(_, u)| u["nature"] == "mot")
        .flat_map(|(uid, u)| {
            liste(&u["formes"])
                .iter()
                .map(move |f| (uid.as_str(), lettres(texte(&f["texte"]))))
        })
        .filter(|(_, f)| !f.is_empty())
        .collect();
    let mut briques: Vec<(&String, String)> = c
        .briques
        .iter()
        .filter(|(_, b)| b["rarete"] != "legendaire")
        .filter_map(|(bid, b)| Some((bid, forme_canonique(c, b)?)))
        .filter(|(_, f)| !f.is_empty())
        .collect();
    briques.sort_by(|a, b| a.0.cmp(b.0));
    let mut trouves = Vec::new();
    for (a, forme_a) in &briques {
        for (b, forme_b) in &briques {
            let suite = vec![(*a).clone(), (*b).clone()];
            if a == b || connues.contains(&suite) {
                continue;
            }
            let ensemble = format!("{forme_a}{forme_b}");
            // Une paire déjà couverte par un mauvais homographe n'a rien à saisir.
            let homographe_jouable = [0, 1].iter().any(|&rang| {
                autres.get(&suite[rang]).into_iter().flatten().any(|autre| {
                    let mut essai = suite.clone();
                    essai[rang].clone_from(autre);
                    connues.contains(&essai)
                })
            });
            if homographe_jouable {
                continue;
            }
            for (mot_id, forme) in &mots {
                if *forme == ensemble {
                    trouves.push(Candidat {
                        premiere: (*a).clone(),
                        seconde: (*b).clone(),
                        mot_id: (*mot_id).to_owned(),
                        forme: ensemble.clone(),
                    });
                }
            }
        }
    }
    trouves
}
