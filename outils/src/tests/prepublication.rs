//! Rapport de pré-publication du skill « preparer-publication ».

use std::collections::BTreeSet;

use chrono::{DateTime, TimeZone, Utc};
use serde_json::{Value, json};

use super::Depot;
use crate::json::liste;
use crate::prepublication::{
    A_VOIR, Options, RapportPrePublication, en_utc, executer, lire_date, preparer, proposer_date,
};

fn maintenant() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 9, 27, 12, 0, 0).unwrap()
}

fn regles(rapport: &RapportPrePublication, rubrique: &str) -> BTreeSet<String> {
    rapport
        .points(rubrique)
        .iter()
        .map(|p| p.regle.clone())
        .collect()
}

fn rapport(depot: &Depot, designation: &str) -> RapportPrePublication {
    preparer(&depot.racine, designation, None, Some(maintenant())).unwrap()
}

fn publier(depot: &Depot, parution: &str) -> Value {
    publier_numero(depot, 1, parution)
}

fn publier_numero(depot: &Depot, numero: i64, parution: &str) -> Value {
    depot.modifier(depot.fascicule(), |f| {
        f["test"] = json!(false);
        f["numero"] = json!(numero);
        f["parution"] = json!(parution);
    })
}

/// Un fascicule de plus, qui reprend un ingrédient du premier pour rester valide.
fn second_fascicule(depot: &Depot, numero: i64, parution: &str) {
    let premier = depot.fascicule().1;
    let ingredient = liste(&premier["unites"])
        .iter()
        .find(|l| l["role"] == "ingredient")
        .unwrap()
        .clone();
    depot.ecrire(
        &depot.racine.join("ludique/fascicules/fsc_zzzzzzzz.json"),
        &json!({"id": "fsc_zzzzzzzz", "numero": numero, "nom": "Essai", "parution": parution, "test": false, "unites": [ingredient]}),
    );
}

// ------------------------------------------------------------------ dates

#[test]
fn date_proposee_le_premier_du_mois() {
    for (precedente, attendue) in [
        ("2026-11-01T00:00:00Z", "2026-12-01T00:00:00Z"),
        ("2027-02-01T00:00:00Z", "2027-03-01T00:00:00Z"),
        ("2026-12-01T00:00:00Z", "2027-01-01T00:00:00Z"),
        ("2027-01-15T08:00:00+02:00", "2027-03-01T00:00:00Z"),
    ] {
        let precedente = lire_date(precedente).unwrap().with_timezone(&Utc);
        assert_eq!(
            en_utc(&proposer_date(Some(precedente), maintenant())),
            attendue
        );
    }
}

#[test]
fn date_proposee_toujours_a_venir() {
    let ancienne = lire_date("2026-01-01T00:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    assert_eq!(
        en_utc(&proposer_date(Some(ancienne), maintenant())),
        "2026-10-01T00:00:00Z"
    );
    assert_eq!(
        en_utc(&proposer_date(None, maintenant())),
        "2026-10-01T00:00:00Z"
    );
}

#[test]
fn date_mal_formee() {
    assert!(lire_date("2026-12-01").is_err());
    assert!(lire_date("2026-12-01T08:00+02:00").is_ok());
}

// ---------------------------------------------------------------- rapport

#[test]
fn fascicule_de_test_controle_comme_publie() {
    let depot = Depot::new();
    let rapport = rapport(&depot, "0");
    assert!(regles(&rapport, "fascicule").contains("fascicule.test"));
    let relecture = regles(&rapport, "relecture");
    assert!(
        relecture.contains("publication.relecture")
            && relecture.contains("publication.localisation")
    );
    assert!(rapport.bloquants() > 0);
}

#[test]
fn fascicule_publiable_signale_relecture_et_medias() {
    let depot = Depot::new();
    let fascicule = publier(&depot, "2026-11-01T00:00:00Z");
    std::fs::remove_file(
        depot
            .media_de("fascicule_id", fascicule["id"].as_str().unwrap())
            .0,
    )
    .unwrap();
    let rapport = rapport(&depot, "1");
    assert!(regles(&rapport, "relecture").contains("publication.relecture"));
    assert!(regles(&rapport, "medias").contains("media.manquant"));
    assert!(!regles(&rapport, "fascicule").contains("fascicule.test"));
}

#[test]
fn date_passee_bloquante() {
    let depot = Depot::new();
    publier(&depot, "2026-09-01T00:00:00Z");
    assert!(regles(&rapport(&depot, "1"), "calendrier").contains("calendrier.passee"));
}

#[test]
fn date_hors_utc_a_voir() {
    let depot = Depot::new();
    publier(&depot, "2026-11-01T08:00:00+02:00");
    let rapport = rapport(&depot, "1");
    let points: Vec<(&str, &str)> = rapport
        .points("calendrier")
        .iter()
        .map(|p| (p.regle.as_str(), p.niveau))
        .collect();
    assert_eq!(points, [("calendrier.utc", A_VOIR)]);
}

#[test]
fn chronologie_sur_dates_analysees() {
    // 01:00 à Paris le 1ᵉʳ décembre, c'est 23:00 UTC le 30 novembre : le fascicule 2 paraît bien après.
    // Une comparaison de chaînes conclurait l'inverse.
    let depot = Depot::new();
    publier_numero(&depot, 11, "2026-12-01T01:00:00+02:00");
    second_fascicule(&depot, 12, "2026-12-01T00:00:00Z");
    let regles = regles(&rapport(&depot, "12"), "calendrier");
    assert!(!regles.contains("calendrier.chronologie"));
    assert!(regles.contains("calendrier.rythme"));
}

#[test]
fn chronologie_avant_le_precedent() {
    let depot = Depot::new();
    publier_numero(&depot, 11, "2026-12-01T00:00:00Z");
    second_fascicule(&depot, 12, "2026-11-01T00:00:00Z");
    let rapport = rapport(&depot, "12");
    assert!(regles(&rapport, "calendrier").contains("calendrier.chronologie"));
    assert_eq!(rapport.date_proposee, "2027-01-01T00:00:00Z");
}

#[test]
fn chronologie_apres_le_suivant() {
    let depot = Depot::new();
    publier_numero(&depot, 11, "2027-01-01T00:00:00Z");
    second_fascicule(&depot, 12, "2026-12-01T00:00:00Z");
    assert!(regles(&rapport(&depot, "11"), "calendrier").contains("calendrier.chronologie"));
}

#[test]
fn fascicules_de_lancement_le_meme_jour() {
    // ADR 0054 : les quatre premiers fascicules paraissent ensemble, les suivants chaque semaine.
    let depot = Depot::new();
    publier(&depot, "2026-12-01T00:00:00Z");
    second_fascicule(&depot, 2, "2026-12-01T00:00:00Z");
    for numero in ["1", "2"] {
        let regles = regles(&rapport(&depot, numero), "calendrier");
        assert!(!regles.contains("calendrier.chronologie"), "{numero}");
        assert!(!regles.contains("calendrier.rythme"), "{numero}");
    }
}

#[test]
fn fascicule_ordinaire_pas_le_meme_jour_que_le_dernier_de_lancement() {
    let depot = Depot::new();
    publier_numero(&depot, 10, "2026-12-01T00:00:00Z");
    second_fascicule(&depot, 11, "2026-12-01T00:00:00Z");
    assert!(regles(&rapport(&depot, "11"), "calendrier").contains("calendrier.chronologie"));
    assert!(regles(&rapport(&depot, "10"), "calendrier").contains("calendrier.chronologie"));
}

#[test]
fn erreur_hors_fascicule_bloque_aussi() {
    let depot = Depot::new();
    publier(&depot, "2026-11-01T00:00:00Z");
    second_fascicule(&depot, 2, "2026-12-01T00:00:00Z");
    assert!(regles(&rapport(&depot, "2"), "depot").contains("publication.relecture"));
}

fn options<'a>(
    depot: &'a Depot,
    fascicule: &'a str,
    date: Option<&'a str>,
    ecrire: bool,
) -> Options<'a> {
    Options {
        racine: &depot.racine,
        fascicule,
        date,
        ecrire,
        maintenant: Some("2026-09-27T12:00:00Z"),
    }
}

#[test]
fn ecrire_date() {
    let depot = Depot::new();
    publier(&depot, "2026-11-01T00:00:00Z");
    let rapport = executer(&options(&depot, "1", Some("2026-12-01T00:00:00Z"), true)).unwrap();
    assert!(rapport.bloquants() > 0); // les faits du jeu d'essai restent en brouillon
    assert_eq!(depot.fascicule().1["parution"], "2026-12-01T00:00:00Z");
    assert!(rapport.en_texte(false).contains("Date proposée"));
}

#[test]
fn ecrire_sans_date_refuse() {
    let depot = Depot::new();
    assert!(executer(&options(&depot, "0", None, true)).is_err());
}

#[test]
fn fascicule_introuvable() {
    let depot = Depot::new();
    assert!(executer(&options(&depot, "42", None, false)).is_err());
}
