//! Visualisation locale : données embarquées sans échappement possible, silhouettes filtrées par le profil.

use std::collections::BTreeSet;
use std::fs;

use regex::Regex;
use serde_json::{Value, json};
use tempfile::TempDir;

use super::Depot;
use super::validation::cli;
use crate::chargement::{Catalogue, charger};
use crate::json::{liste, texte};
use crate::rapport::Rapport;
use crate::visualisation::{donnees, page};

const INJECTION: &str = "</script><script>alert(1)</script><!--";

fn embarque(html: &str) -> Value {
    let motif =
        Regex::new(r#"(?s)<script type="application/json" id="donnees">(.*?)</script>"#).unwrap();
    let bloc = motif.captures(html).expect("bloc de données absent");
    serde_json::from_str(&bloc[1]).unwrap()
}

fn charge(depot: &Depot) -> (Catalogue, Rapport) {
    let mut rapport = Rapport::default();
    let catalogue = charger(&depot.racine, &mut rapport).unwrap();
    (catalogue, rapport)
}

fn unite<'a>(donnees: &'a Value, cle: &str, valeur: &str) -> &'a Value {
    liste(&donnees["unites"])
        .iter()
        .find(|u| u[cle] == valeur)
        .unwrap()
}

#[test]
fn page_embarque_tout_le_catalogue() {
    let depot = Depot::new();
    let (catalogue, rapport) = charge(&depot);
    let html = page(&catalogue, &rapport);
    let embarque = embarque(&html);
    let unites: BTreeSet<&str> = liste(&embarque["unites"])
        .iter()
        .map(|u| texte(&u["id"]))
        .collect();
    assert_eq!(
        unites,
        catalogue.unites.keys().map(String::as_str).collect()
    );
    let relations: BTreeSet<&str> = liste(&embarque["liens"])
        .iter()
        .filter(|l| l["type"] != "composition")
        .map(|l| texte(&l["id"]))
        .collect();
    assert_eq!(
        relations,
        catalogue.relations.keys().map(String::as_str).collect()
    );
    assert!(
        texte(&unite(&embarque, "id", &depot.uid("biologie"))["silhouette"]["svg"])
            .starts_with("<svg")
    );
    assert!(!html.contains("/*DONNEES*/"));
}

#[test]
fn donnees_ne_sortent_jamais_du_script() {
    let depot = Depot::new();
    depot.modifier(depot.unite("géo"), |u| {
        u["sens"][0]["libelles"]["fr"]["glose"] = json!(INJECTION);
    });
    let (catalogue, rapport) = charge(&depot);
    let html = page(&catalogue, &rapport);
    assert!(!html.contains(INJECTION));
    assert_eq!(html.matches("</script>").count(), 2);
    assert_eq!(
        unite(&embarque(&html), "texte", "géo")["sens"][0]["glose"],
        INJECTION
    );
}

#[test]
fn silhouette_hors_profil_jamais_embarquee() {
    let depot = Depot::new();
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 300 400"><script>alert(1)</script></svg>"#;
    let phile = depot.uid("phile");
    // La silhouette du catalogue cède la place à celle de l'essai.
    for chemin in depot.fichiers("ludique/medias") {
        if depot.lire(&chemin)["unite_id"] == phile.as_str() {
            fs::remove_file(chemin).unwrap();
        }
    }
    depot.media("silhouette", "unite_id", &phile, Some(svg));
    let (catalogue, rapport) = charge(&depot);
    let donnees = donnees(&catalogue, &rapport);
    let silhouette = &unite(&donnees, "id", &phile)["silhouette"];
    assert!(silhouette.get("svg").is_none());
    assert!(!texte(&silhouette["probleme"]).is_empty());
}

#[test]
fn cli_visualiser_meme_sur_un_depot_invalide() {
    let depot = Depot::new();
    depot.modifier(depot.recette("géologie"), |r| {
        r["composition_id"] = json!("cmp_zzzzzzzz");
    });
    let dossier = TempDir::new().unwrap();
    let sortie = dossier.path().join("vue/catalogue.html");
    assert_eq!(
        cli(&[
            "visualiser",
            "--racine",
            depot.racine.to_str().unwrap(),
            "--sortie",
            sortie.to_str().unwrap()
        ]),
        0
    );
    let embarque = embarque(&fs::read_to_string(sortie).unwrap());
    assert!(
        liste(&embarque["diagnostics"])
            .iter()
            .any(|d| d["niveau"] == "erreur")
    );
}

#[test]
fn jaquette_embarquee() {
    let depot = Depot::new();
    let fascicule = depot.fascicule().1["id"].clone();
    let (catalogue, rapport) = charge(&depot);
    let donnees = donnees(&catalogue, &rapport);
    let jaquette = &liste(&donnees["fascicules"])
        .iter()
        .find(|f| f["id"] == fascicule)
        .unwrap()["jaquette"];
    assert!(
        texte(&jaquette["svg"]).starts_with("<svg")
            && texte(&jaquette["fichier"]).starts_with("jaquettes/")
    );
}
