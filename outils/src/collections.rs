//! Quêtes de collection par fascicule (App#90) : tous les préfixes, tous les
//! suffixes, tous les mots d'une rareté. Le fascicule déclare la quête et sa
//! récompense ; ses membres se calculent ici, une seule fois, pour le
//! validateur et pour la compilation, qui les publie dans le manifeste.
//!
//! Jamais une légendaire, ni un mot qui en exige une (ADR 0043) : une quête
//! ne compte que ce que le joueur peut voir.

use std::collections::BTreeSet;

use serde_json::Value;

use crate::chargement::Catalogue;
use crate::json::{liste, texte};

/// Une quête de moins de cartes n'est pas déclarée.
pub const MEMBRES_MIN: usize = 3;

const RARETES: [&str; 4] = ["commune", "peu_commune", "rare", "legendaire"];

fn rang(rarete: &str) -> usize {
    RARETES.iter().position(|r| *r == rarete).unwrap_or(0)
}

fn brique_de<'a>(c: &'a Catalogue, unite: &str) -> Option<(&'a String, &'a Value)> {
    c.briques
        .iter()
        .find(|(_, b)| texte(&b["unite_id"]) == unite)
}

fn legendaire(c: &Catalogue, unite: &str) -> bool {
    brique_de(c, unite).is_some_and(|(_, b)| b["rarete"] == "legendaire")
}

/// Recettes qui donnent ce mot.
fn recettes_de<'a>(c: &'a Catalogue, mot: &str) -> Vec<&'a Value> {
    c.recettes
        .values()
        .filter(|r| {
            c.compositions
                .get(texte(&r["composition_id"]))
                .is_some_and(|comp| texte(&comp["mot_id"]) == mot)
        })
        .collect()
}

/// Rareté d'une brique au pli ; un mot posé tel quel compte comme commun.
fn rarete_brique(c: &Catalogue, brique: &str) -> &'static str {
    let Some(b) = c.briques.get(brique) else {
        return "commune";
    };
    let mot = c
        .unites
        .get(texte(&b["unite_id"]))
        .is_some_and(|u| u["nature"] == "mot");
    if mot {
        return "commune";
    }
    RARETES[rang(texte(&b["rarete"]))]
}

/// Rareté d'un mot (ADR 0040), miroir de `rareteMot` (App) : la plus haute
/// rareté des briques qu'exige sa recette la plus accessible ; pour un
/// emplacement à variantes, la moins rare compte.
pub fn rarete_mot(c: &Catalogue, mot: &str) -> &'static str {
    recettes_de(c, mot)
        .iter()
        .map(|r| {
            liste(&r["emplacements"])
                .iter()
                .map(|e| {
                    liste(&e["briques_acceptees"])
                        .iter()
                        .map(|b| rang(rarete_brique(c, texte(b))))
                        .min()
                        .unwrap_or(0)
                })
                .max()
                .unwrap_or(0)
        })
        .min()
        .map_or("commune", |r| RARETES[r])
}

/// Un mot qui exige une légendaire reste invisible jusqu'à sa découverte.
fn depend_de_legendaire(c: &Catalogue, mot: &str) -> bool {
    recettes_de(c, mot).iter().any(|r| {
        liste(&r["emplacements"]).iter().any(|e| {
            liste(&e["briques_acceptees"])
                .iter()
                .filter_map(|b| c.briques.get(texte(b)))
                .any(|b| b["rarete"] == "legendaire")
        })
    })
}

/// Membres d'une quête de collection, triés : des identifiants de brique pour
/// les préfixes et les suffixes (les briques du pli du fascicule, reprises
/// comprises), des identifiants d'unité pour les mots. Vide pour un critère inconnu.
pub fn membres(c: &Catalogue, fid: &str, critere: &str) -> Vec<String> {
    let Some(fsc) = c.fascicules.get(fid) else {
        return Vec::new();
    };
    let unites = liste(&fsc["unites"]);
    let nature = |u: &str| c.unites.get(u).map_or("", |u| texte(&u["nature"]));
    let mut ids: BTreeSet<String> = BTreeSet::new();
    match critere {
        "prefixes" | "suffixes" => {
            let voulue = &critere[..critere.len() - 1];
            for u in unites.iter().map(|u| texte(&u["unite_id"])) {
                if nature(u) == voulue
                    && !legendaire(c, u)
                    && let Some((bid, _)) = brique_de(c, u)
                {
                    ids.insert(bid.clone());
                }
            }
        }
        "mots_communs" | "mots_peu_communs" | "mots_rares" => {
            let voulue = match critere {
                "mots_communs" => "commune",
                "mots_peu_communs" => "peu_commune",
                _ => "rare",
            };
            for u in unites
                .iter()
                .filter(|u| u["role"] == "resultat")
                .map(|u| texte(&u["unite_id"]))
            {
                if nature(u) == "mot"
                    && !legendaire(c, u)
                    && !depend_de_legendaire(c, u)
                    && rarete_mot(c, u) == voulue
                {
                    ids.insert(u.to_owned());
                }
            }
        }
        _ => {}
    }
    ids.into_iter().collect()
}
