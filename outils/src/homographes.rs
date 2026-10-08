//! Briques de même forme (ADR 0052).
//!
//! Deux briques sont homographes quand leurs unités ont la même langue, la même nature et la même
//! forme canonique normalisée. Le validateur s'en sert pour refuser un trompe-l'œil déjà couvert
//! par le calcul du serveur, le compilateur pour publier les groupes dans le manifeste.

use std::collections::{BTreeMap, HashMap};

use serde_json::Value;

use crate::chargement::Catalogue;
use crate::json::{liste, texte};

/// Forme canonique d'une unité, si elle en a exactement une.
fn canonique(unite: &Value) -> Option<&Value> {
    let mut canoniques = liste(&unite["formes"])
        .iter()
        .filter(|f| f["role"] == "canonique");
    let premiere = canoniques.next()?;
    canoniques.next().is_none().then_some(premiere)
}

/// Groupes d'au moins deux briques homographes, triés : identifiants de briques, jamais de texte.
#[must_use]
pub fn groupes(c: &Catalogue) -> Vec<Vec<String>> {
    let mut par_cle: BTreeMap<(&str, &str, &str), Vec<String>> = BTreeMap::new();
    for (bid, brique) in &c.briques {
        let Some(unite) = c.unites.get(texte(&brique["unite_id"])) else {
            continue;
        };
        let Some(forme) = canonique(unite) else {
            continue;
        };
        par_cle
            .entry((
                texte(&unite["langue_id"]),
                texte(&unite["nature"]),
                texte(&forme["cle_normalisee"]),
            ))
            .or_default()
            .push(bid.clone());
    }
    par_cle
        .into_values()
        .filter(|g| g.len() > 1)
        .map(|mut g| {
            g.sort_unstable();
            g
        })
        .collect()
}

/// Pour chaque brique, les autres briques de son groupe.
#[must_use]
pub fn par_brique(groupes: &[Vec<String>]) -> HashMap<String, Vec<String>> {
    let mut autres: HashMap<String, Vec<String>> = HashMap::new();
    for groupe in groupes {
        for brique in groupe {
            autres.insert(
                brique.clone(),
                groupe.iter().filter(|b| *b != brique).cloned().collect(),
            );
        }
    }
    autres
}
