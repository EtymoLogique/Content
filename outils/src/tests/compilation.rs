//! Compilation de l'artefact : format, empreintes, champs privés, modes essai et publication.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use tempfile::TempDir;

use super::Depot;
use super::validation::cli;
use crate::chargement::charger;
use crate::compilation::{compiler, ecrire_artefact, empreinte, ressources, verifier};
use crate::json::{liste, texte};
use crate::rapport::Rapport;

fn compiler_vers(depot: &Depot, sortie: &Path, essai: bool) -> Value {
    let catalogue = charger(&depot.racine, &mut Rapport::default()).unwrap();
    ecrire_artefact(&compiler(&catalogue, essai, "test").unwrap(), sortie).unwrap();
    lire(&sortie.join("parutions.json"))
}

fn lire(chemin: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(chemin).unwrap()).unwrap()
}

fn manifeste(sortie: &Path, fascicule: &Value) -> Value {
    lire(&sortie.join(texte(&fascicule["manifeste"])))
}

fn noms(dossier: &Path) -> BTreeSet<String> {
    fs::read_dir(dossier)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect()
}

fn tous_les_fichiers(dossier: &Path) -> Vec<PathBuf> {
    let mut fichiers = Vec::new();
    for entree in fs::read_dir(dossier).unwrap() {
        let chemin = entree.unwrap().path();
        if chemin.is_dir() {
            fichiers.extend(tous_les_fichiers(&chemin));
        } else {
            fichiers.push(chemin);
        }
    }
    fichiers
}

/// Dépôt d'essai, dossier de sortie et calendrier compilé en mode essai.
fn essai() -> (Depot, TempDir, Value) {
    let depot = Depot::new();
    let sortie = TempDir::new().unwrap();
    let parutions = compiler_vers(&depot, sortie.path(), true);
    (depot, sortie, parutions)
}

#[test]
fn essai_publie_le_fascicule_de_test() {
    let (_, sortie, parutions) = essai();
    assert_eq!(parutions["essai"], true);
    assert_eq!(
        liste(&parutions["fascicules"])
            .iter()
            .map(|f| f["numero"].clone())
            .collect::<Vec<_>>(),
        [json!(0)]
    );
    assert!(verifier(sortie.path()).is_empty());
}

#[test]
fn publication_ecarte_les_fascicules_de_test() {
    let depot = Depot::new();
    let sortie = TempDir::new().unwrap();
    let parutions = compiler_vers(&depot, sortie.path(), false);
    assert_eq!(parutions["essai"], false);
    assert_eq!(parutions["fascicules"], json!([]));
}

#[test]
fn referentiels_nomment_relations_et_transformations() {
    let (_, _, parutions) = essai();
    let nom = |liste_types: &Value, code: &str| {
        liste(liste_types)
            .iter()
            .find(|t| t["code"] == code)
            .unwrap()["libelles"]["fr"]["nom"]
            .clone()
    };
    assert_eq!(
        nom(
            &parutions["referentiels"]["types_relation"],
            "forme_de_composition"
        ),
        "forme de composition"
    );
    assert_eq!(
        nom(
            &parutions["referentiels"]["types_transformation"],
            "elision"
        ),
        "élision"
    );
}

#[test]
fn calendrier_ne_cite_que_les_manifestes() {
    let (_, _, parutions) = essai();
    for fascicule in liste(&parutions["fascicules"]) {
        let cles: BTreeSet<&str> = fascicule
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(cles, BTreeSet::from(["numero", "parution", "manifeste"]));
        assert!(texte(&fascicule["manifeste"]).starts_with("f/"));
    }
}

#[test]
fn chaque_ressource_est_citee_par_un_manifeste() {
    let (_, sortie, parutions) = essai();
    let citees: BTreeSet<String> = liste(&parutions["fascicules"])
        .iter()
        .flat_map(|f| ressources(&manifeste(sortie.path(), f)))
        .collect();
    let ecrites: BTreeSet<String> = ["r", "a"]
        .into_iter()
        .flat_map(|d| {
            noms(&sortie.path().join(d))
                .into_iter()
                .map(move |n| format!("{d}/{n}"))
        })
        .collect();
    assert_eq!(citees, ecrites);
}

#[test]
fn verifier_signale_une_ressource_absente() {
    let (_, sortie, parutions) = essai();
    let chemin = ressources(&manifeste(sortie.path(), &parutions["fascicules"][0])).remove(0);
    fs::remove_file(sortie.path().join(&chemin)).unwrap();
    // Une ressource partagée (silhouette d'un ingrédient) manque dans chaque fascicule qui la porte.
    let erreurs = verifier(sortie.path());
    assert!(erreurs.contains(&format!("Ressource absente : {chemin} (fascicule 0)")));
    assert!(erreurs.iter().all(|e| e.contains(&chemin)));
}

#[test]
fn chemins_en_hash_de_contenu() {
    let (_, sortie, parutions) = essai();
    let fascicule = &parutions["fascicules"][0];
    let mut chemins = vec![texte(&fascicule["manifeste"]).to_owned()];
    chemins.extend(ressources(&manifeste(sortie.path(), fascicule)));
    for chemin in chemins {
        let contenu = fs::read(sortie.path().join(&chemin)).unwrap();
        let (dossier, nom) = chemin.split_once('/').unwrap();
        assert_eq!(
            nom,
            format!(
                "{}{}",
                empreinte(&contenu),
                if dossier == "a" { ".svg" } else { ".json" }
            )
        );
        assert!(
            !["unt_", "brq_", "rct_", "fsc_", "med_", "lng_"]
                .iter()
                .any(|i| chemin.contains(i)),
            "un chemin ne dit jamais l’identifiant"
        );
    }
}

#[test]
fn compilation_deterministe() {
    let (depot, a, parutions_a) = essai();
    let b = TempDir::new().unwrap();
    assert_eq!(parutions_a, compiler_vers(&depot, b.path(), true));
    assert_eq!(noms(&a.path().join("r")), noms(&b.path().join("r")));
}

#[test]
fn aucun_champ_prive() {
    let (_, sortie, _) = essai();
    for fichier in tous_les_fichiers(sortie.path())
        .into_iter()
        .filter(|f| f.extension().is_some_and(|e| e == "json"))
    {
        let texte = fs::read_to_string(&fichier).unwrap();
        assert!(
            !texte.contains("note_interne") && !texte.contains("etat_relecture"),
            "{}",
            fichier.display()
        );
    }
}

#[test]
fn fait_non_retenu_jamais_publie() {
    let depot = Depot::new();
    depot.modifier(depot.unite("géologue"), |u| {
        u["sens"][0]["fait"]["confiance"] = json!("non_retenue");
    });
    let sens_id = texte(&depot.unite("géologue").1["sens"][0]["id"]).to_owned();
    let sortie = TempDir::new().unwrap();
    compiler_vers(&depot, sortie.path(), true);
    assert!(
        !tous_les_fichiers(&sortie.path().join("r"))
            .iter()
            .any(|f| fs::read_to_string(f).unwrap().contains(&sens_id))
    );
}

#[test]
fn manifeste_et_carte() {
    let (depot, sortie, parutions) = essai();
    let manifeste = manifeste(sortie.path(), &parutions["fascicules"][0]);
    let uid = depot.uid("géologie");
    let entree = liste(&manifeste["cartes"])
        .iter()
        .find(|c| c["id"] == uid.as_str())
        .unwrap();
    assert_eq!(entree["role"], "resultat");
    let carte = lire(&sortie.path().join(texte(&entree["carte"])));
    assert_eq!(carte["compositions"][0]["mot_id"], uid.as_str());
    let sources: BTreeSet<&String> = carte["sources"].as_object().unwrap().keys().collect();
    for lien in liste(&carte["fait"]["sources"]) {
        assert!(sources.contains(&texte(&lien["source_id"]).to_owned()));
    }
    // Seules les recettes des mots résultats du fascicule y figurent, pas celles des mots orphelins.
    let resultats: BTreeSet<String> = liste(&depot.fascicule().1["unites"])
        .iter()
        .filter(|u| u["role"] == "resultat")
        .map(|u| texte(&u["unite_id"]).to_owned())
        .collect();
    let mot = |cid: &Value| {
        texte(
            &depot.lire(
                &depot
                    .racine
                    .join("compositions")
                    .join(format!("{}.json", texte(cid))),
            )["mot_id"],
        )
        .to_owned()
    };
    let recettes: Vec<(String, String)> = depot
        .fichiers("ludique/recettes")
        .iter()
        .map(|p| depot.lire(p))
        .map(|r| (texte(&r["id"]).to_owned(), mot(&r["composition_id"])))
        .collect();
    let publiees: BTreeSet<String> = liste(&manifeste["recettes"])
        .iter()
        .map(|r| texte(&r["id"]).to_owned())
        .collect();
    let attendues: BTreeSet<String> = recettes
        .iter()
        .filter(|(_, m)| resultats.contains(m))
        .map(|(r, _)| r.clone())
        .collect();
    assert_eq!(publiees, attendues);
    assert!(
        recettes.iter().any(|(_, m)| !resultats.contains(m)),
        "le jeu d’exemples doit porter un mot orphelin"
    );
}

#[test]
fn cli_refuse_la_publication_d_un_depot_invalide() {
    let depot = Depot::new();
    depot.modifier(depot.recette("géologie"), |r| {
        r["composition_id"] = json!("cmp_zzzzzzzz");
    });
    let sortie = TempDir::new().unwrap();
    let a = sortie.path().join("a");
    assert_eq!(
        cli(&[
            "compiler",
            "--racine",
            depot.racine.to_str().unwrap(),
            "--sortie",
            a.to_str().unwrap()
        ]),
        1
    );
    assert!(!a.exists());
}

#[test]
fn cli_essai() {
    let depot = Depot::new();
    let sortie = TempDir::new().unwrap();
    let a = sortie.path().join("a");
    let racine = depot.racine.to_str().unwrap();
    assert_eq!(
        cli(&[
            "compiler",
            "--essai",
            "--racine",
            racine,
            "--sortie",
            a.to_str().unwrap(),
            "--version",
            "v1"
        ]),
        0
    );
    let parutions = lire(&a.join("parutions.json"));
    assert_eq!(
        (parutions["essai"].clone(), parutions["version"].clone()),
        (json!(true), json!("v1"))
    );
}

#[test]
fn silhouettes_des_cartes_et_des_langues() {
    let depot = Depot::new();
    let (chemin, sans) = depot.media_de("unite_id", &depot.uid("mener"));
    fs::remove_file(chemin).unwrap();
    fs::remove_file(depot.racine.join("medias").join(texte(&sans["fichier"]))).unwrap();
    let sortie = TempDir::new().unwrap();
    let parutions = compiler_vers(&depot, sortie.path(), true);
    let manifeste = manifeste(sortie.path(), &parutions["fascicules"][0]);
    let carte = |uid: String| {
        liste(&manifeste["cartes"])
            .iter()
            .find(|c| c["id"] == uid.as_str())
            .unwrap()
            .clone()
    };
    let media = depot.media_de("unite_id", &depot.uid("biologie")).1;
    let entree = carte(depot.uid("biologie"));
    assert!(texte(&entree["silhouette"]).starts_with("a/"));
    assert_eq!(
        fs::read(sortie.path().join(texte(&entree["silhouette"]))).unwrap(),
        fs::read(depot.racine.join("medias").join(texte(&media["fichier"]))).unwrap()
    );
    assert_eq!(carte(depot.uid("mener"))["silhouette"], Value::Null);
    let langues: Vec<&str> = liste(&manifeste["langues"])
        .iter()
        .map(|l| texte(&l["id"]))
        .collect();
    assert!(langues.is_sorted());
    let illustrees: BTreeSet<&str> = liste(&manifeste["langues"])
        .iter()
        .filter(|l| !l["silhouette"].is_null())
        .map(|l| texte(&l["id"]))
        .collect();
    assert!(BTreeSet::from(["lng_wa90y54n", "lng_kmf44d76"]).is_subset(&illustrees));
    assert!(texte(&manifeste["jaquette"]).starts_with("a/"));
}

#[test]
fn jaquette_et_media_partage() {
    // Deux cartes au même dessin partagent un seul fichier publié.
    let depot = Depot::new();
    let geo = depot.media_de("unite_id", &depot.uid("géo")).1;
    let iste = depot.media_de("unite_id", &depot.uid("iste")).1;
    let medias = depot.racine.join("medias");
    fs::copy(
        medias.join(texte(&geo["fichier"])),
        medias.join(texte(&iste["fichier"])),
    )
    .unwrap();
    let sortie = TempDir::new().unwrap();
    let parutions = compiler_vers(&depot, sortie.path(), true);
    let manifeste = manifeste(sortie.path(), &parutions["fascicules"][0]);
    assert!(texte(&manifeste["jaquette"]).starts_with("a/"));
    let silhouette = |uid: String| {
        liste(&manifeste["cartes"])
            .iter()
            .find(|c| c["id"] == uid.as_str())
            .unwrap()["silhouette"]
            .clone()
    };
    assert_eq!(silhouette(depot.uid("iste")), silhouette(depot.uid("géo")));
    assert!(verifier(sortie.path()).is_empty());
}

#[test]
fn media_ne_fuit_ni_nom_ni_note() {
    let (depot, sortie, _) = essai();
    let textes: Vec<String> = tous_les_fichiers(sortie.path())
        .iter()
        .map(|f| String::from_utf8_lossy(&fs::read(f).unwrap()).into_owned())
        .collect();
    for media in depot
        .fichiers("ludique/medias")
        .iter()
        .map(|p| depot.lire(p))
    {
        assert!(
            !textes
                .iter()
                .any(|t| t.contains(texte(&media["id"])) || t.contains(texte(&media["fichier"])))
        );
    }
    assert!(!textes.iter().any(|t| t.contains("cardArt")));
}

#[test]
fn verifier_signale_un_media_altere() {
    let (_, sortie, _) = essai();
    let nom = noms(&sortie.path().join("a")).pop_first().unwrap();
    let fichier = sortie.path().join("a").join(&nom);
    let mut contenu = fs::read(&fichier).unwrap();
    contenu.push(b' ');
    fs::write(&fichier, contenu).unwrap();
    assert_eq!(
        verifier(sortie.path()),
        [format!("Empreinte fausse : a/{nom}")]
    );
}

#[test]
fn manifeste_porte_le_nom_du_fascicule() {
    let (_, sortie, parutions) = essai();
    assert_eq!(
        manifeste(sortie.path(), &parutions["fascicules"][0])["nom"],
        "Bêta"
    );
}

#[test]
fn manifeste_porte_l_equilibrage_du_pli() {
    let (_, sortie, parutions) = essai();
    let pli = &manifeste(sortie.path(), &parutions["fascicules"][0])["pli"];
    assert_eq!(pli["equilibrage"]["poids_raretes"]["commune"], 62);
    assert_eq!(pli["equilibrage"]["garantie_nouveaute"], 6);
    assert!(pli["equilibrage"].get("poids_types").is_none());
    // Un mot n'est jamais tiré d'un pli (ADR 0035).
    let natures: BTreeSet<&str> = liste(&pli["briques"])
        .iter()
        .map(|b| texte(&b["nature"]))
        .collect();
    assert_eq!(natures, BTreeSet::from(["prefixe", "suffixe"]));
}

#[test]
fn manifeste_porte_les_graines() {
    let (depot, sortie, parutions) = essai();
    let manifeste = manifeste(sortie.path(), &parutions["fascicules"][0]);
    let fascicule = depot.fascicule().1;
    let mut attendues = liste(&fascicule["graines"]).to_vec();
    attendues.sort_by(|a, b| texte(&a["brique_id"]).cmp(texte(&b["brique_id"])));
    assert_eq!(manifeste["graines"], json!(attendues));
    assert_eq!(manifeste["id"], fascicule["id"]);
}

#[test]
fn manifeste_publie_les_jalons() {
    let (_, sortie, parutions) = essai();
    let jalons = manifeste(sortie.path(), &parutions["fascicules"][0])["jalons"].clone();
    assert_eq!(
        liste(&jalons)
            .iter()
            .map(|j| j["decouvertes"].clone())
            .collect::<Vec<_>>(),
        [json!(3), json!(6)]
    );
    let cles: BTreeSet<&str> = jalons[0]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    assert_eq!(
        cles,
        BTreeSet::from(["id", "decouvertes", "pli", "sabliers"])
    );
    assert_eq!(jalons[1]["sabliers"], 2);
}
