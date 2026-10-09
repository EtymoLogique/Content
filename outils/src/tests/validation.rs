//! Un test par règle : chaque test casse une seule chose dans une copie du dépôt.

use std::fs;

use serde_json::{Value, json};
use unicode_normalization::UnicodeNormalization;

use super::{Depot, avertissements, avertissements_notables, ensemble, erreurs, id, retirer};
use crate::chargement::charger;
use crate::json::{liste, texte};
use crate::medias::{cible, langues_citees, verifier_svg};
use crate::rapport::{AVERTISSEMENT, Rapport};
use crate::{Cli, executer};

const ID_ABSENT_UNT: &str = "unt_00000000";

fn a_erreur(depot: &Depot, regle: &str) -> bool {
    erreurs(&depot.valider()).contains(regle)
}

fn a_avertissement(depot: &Depot, regle: &str) -> bool {
    avertissements(&depot.valider()).contains(regle)
}

fn sans_unite(uid: String) -> impl FnOnce(&mut Value) {
    move |f| {
        let unites: Vec<Value> = liste(&f["unites"])
            .iter()
            .filter(|u| u["unite_id"] != uid.as_str())
            .cloned()
            .collect();
        f["unites"] = json!(unites);
    }
}

fn publier_numero_1(f: &mut Value) {
    f["test"] = json!(false);
    f["numero"] = json!(1);
}

// ------------------------------------------------------------------ jeu d'exemples

#[test]
fn jeu_exemples_valide() {
    let rapport = Depot::new().valider();
    assert!(rapport.erreurs().is_empty(), "{}", rapport.en_texte());
    assert!(
        avertissements_notables(&rapport).is_empty(),
        "{}",
        rapport.en_texte()
    );
}

#[test]
fn publication_ignore_les_fascicules_de_test() {
    let rapport = Depot::new().valider_publication();
    assert!(erreurs(&rapport).is_empty());
    assert!(avertissements(&rapport).contains("publication.aucun_fascicule"));
}

// ------------------------------------------------------------- fichiers et schémas

#[test]
fn json_illisible() {
    let depot = Depot::new();
    fs::write(depot.brique("géo").0, "{ pas du json").unwrap();
    assert!(a_erreur(&depot, "json.syntaxe"));
}

#[test]
fn cle_dupliquee() {
    let depot = Depot::new();
    let (chemin, _) = depot.unite("géo");
    let texte = fs::read_to_string(&chemin).unwrap().replacen(
        "\"fr\": {\n",
        "\"fr\": {\"glose\": \"x\"}, \"fr\": {\n",
        1,
    );
    fs::write(chemin, texte).unwrap();
    assert_eq!(erreurs(&depot.valider()), ensemble(&["json.cle_dupliquee"]));
}

#[test]
fn schema_enumeration() {
    let depot = Depot::new();
    depot.modifier(depot.brique("géo"), |b| b["rarete"] = json!("epique"));
    assert!(a_erreur(&depot, "schema"));
}

#[test]
fn schema_sous_type_exclusif() {
    let depot = Depot::new();
    depot.modifier(depot.unite("géo"), |u| {
        u["mot"] = json!({"categorie": "nom"});
    });
    assert!(a_erreur(&depot, "schema"));
}

#[test]
fn schema_locale_de_reference() {
    let depot = Depot::new();
    depot.modifier(depot.unite("géo"), |u| {
        u["sens"][0]["libelles"] = json!({"en": {"glose": "earth"}});
    });
    assert!(a_erreur(&depot, "schema"));
}

#[test]
fn fichier_inattendu() {
    let depot = Depot::new();
    fs::write(depot.racine.join("unites/notes.txt"), "brouillon").unwrap();
    assert!(a_erreur(&depot, "fichier.inattendu"));
}

// ------------------------------------------------------------------------ identité

#[test]
fn nom_de_fichier() {
    let depot = Depot::new();
    let (chemin, _) = depot.brique("géo");
    fs::rename(&chemin, chemin.with_file_name("brq_zzzzzzzz.json")).unwrap();
    assert!(a_erreur(&depot, "identite.nom_fichier"));
}

#[test]
fn prefixe_du_dossier() {
    let depot = Depot::new();
    let (chemin, _) = depot.brique("géo");
    fs::rename(
        &chemin,
        depot
            .racine
            .join("unites")
            .join(chemin.file_name().unwrap()),
    )
    .unwrap();
    assert!(a_erreur(&depot, "identite.prefixe"));
}

#[test]
fn identifiant_en_double() {
    let depot = Depot::new();
    let autre = depot.unite("logie").1["formes"][0]["id"].clone();
    depot.modifier(depot.unite("géo"), |u| u["formes"][0]["id"] = autre);
    assert!(a_erreur(&depot, "identite.doublon"));
}

#[test]
fn reference_cassee() {
    let depot = Depot::new();
    depot.modifier(depot.brique("géo"), |b| {
        b["unite_id"] = json!(ID_ABSENT_UNT);
    });
    assert!(a_erreur(&depot, "reference.cassee"));
}

// -------------------------------------------------------------------------- unités

#[test]
fn une_seule_forme_canonique() {
    let depot = Depot::new();
    depot.modifier(depot.unite("philo"), |u| {
        u["formes"][1]["role"] = json!("canonique");
    });
    assert!(a_erreur(&depot, "unite.forme_canonique"));
}

#[test]
fn ecriture_de_la_langue() {
    let depot = Depot::new();
    let latin = depot.unite("géo").1["formes"][0]["systeme_ecriture_id"].clone();
    depot.modifier(depot.unite("γῆ"), |u| {
        u["formes"][0]["systeme_ecriture_id"] = latin;
        retirer(&mut u["formes"][0], "translitterations");
    });
    assert!(a_erreur(&depot, "unite.ecriture_incompatible"));
}

#[test]
fn forme_sans_trait_union() {
    let depot = Depot::new();
    depot.modifier(depot.unite("géo"), |u| {
        u["formes"][0]["texte"] = json!("géo-");
        u["formes"][0]["cle_normalisee"] = json!("géo-");
    });
    assert!(a_erreur(&depot, "forme.trait_union"));
}

#[test]
fn cle_normalisee_nfc() {
    let depot = Depot::new();
    depot.modifier(depot.unite("géo"), |u| {
        u["formes"][0]["cle_normalisee"] = json!("géo".nfd().collect::<String>());
    });
    assert!(a_erreur(&depot, "forme.cle_normalisee"));
}

#[test]
fn convention_de_la_bonne_ecriture() {
    let depot = Depot::new();
    let convention =
        depot.unite("γῆ").1["formes"][0]["translitterations"][0]["convention_id"].clone();
    depot.modifier(depot.unite("géo"), |u| {
        u["formes"][0]["translitterations"] =
            json!([{"convention_id": convention, "texte": "geo"}]);
    });
    assert!(a_erreur(&depot, "translitteration.ecriture"));
}

#[test]
fn homographes_gloses_identiques() {
    let depot = Depot::new();
    depot.modifier(depot.unite_glose("a", Some("vers")), |u| {
        u["sens"][0]["libelles"]["fr"]["glose"] = json!("sans, privé de");
    });
    assert!(a_erreur(&depot, "homographe.glose_identique"));
}

#[test]
fn homographes_etymons_identiques() {
    let depot = Depot::new();
    let grec_a = depot.uid("ἀ");
    depot.modifier(depot.relation_glose("a", Some("vers")), |r| {
        r["unite_source_id"] = json!(grec_a);
        r["type_code"] = json!("emprunt");
    });
    assert!(a_erreur(&depot, "homographe.etymon_identique"));
}

// -------------------------------------------------------------------- compositions

#[test]
fn resultat_est_un_mot() {
    let depot = Depot::new();
    let geo = depot.unite("géo").1;
    depot.modifier(depot.composition("géologie"), |c| {
        c["mot_id"] = geo["id"].clone();
        c["forme_resultat_id"] = geo["formes"][0]["id"].clone();
    });
    assert!(a_erreur(&depot, "composition.resultat_non_mot"));
}

#[test]
fn forme_resultat_du_mot() {
    let depot = Depot::new();
    let autre = depot.unite("biologie").1["formes"][0]["id"].clone();
    depot.modifier(depot.composition("géologie"), |c| {
        c["forme_resultat_id"] = autre;
    });
    assert!(a_erreur(&depot, "composition.forme_resultat"));
}

#[test]
fn rangs_continus() {
    let depot = Depot::new();
    depot.modifier(depot.composition("géologie"), |c| {
        c["parties"][1]["rang"] = json!(3);
    });
    assert!(a_erreur(&depot, "composition.rangs"));
}

#[test]
fn sens_de_la_partie() {
    let depot = Depot::new();
    let autre = depot.unite("bio").1["sens"][0]["id"].clone();
    depot.modifier(depot.composition("géologie"), |c| {
        c["parties"][0]["sens_id"] = autre;
    });
    assert!(a_erreur(&depot, "composition.sens_partie"));
}

#[test]
fn forme_de_la_partie() {
    let depot = Depot::new();
    let autre = depot.unite("bio").1["formes"][0]["id"].clone();
    depot.modifier(depot.composition("géologie"), |c| {
        c["parties"][0]["forme_id"] = autre;
    });
    assert!(a_erreur(&depot, "composition.forme_partie"));
}

#[test]
fn position_des_affixes() {
    let depot = Depot::new();
    depot.modifier(depot.composition("géologie"), |c| {
        c["parties"][0]["rang"] = json!(2);
        c["parties"][1]["rang"] = json!(1);
    });
    assert!(a_erreur(&depot, "composition.position"));
}

#[test]
fn segment_hors_bornes() {
    let depot = Depot::new();
    depot.modifier(depot.composition("géologie"), |c| {
        c["parties"][1]["segment_fin"] = json!(9);
    });
    assert!(a_erreur(&depot, "composition.segment"));
}

#[test]
fn segments_chevauchants() {
    let depot = Depot::new();
    depot.modifier(depot.composition("géologie"), |c| {
        c["parties"][1]["segment_debut"] = json!(2);
    });
    assert!(a_erreur(&depot, "composition.segment"));
}

#[test]
fn segment_incomplet() {
    let depot = Depot::new();
    depot.modifier(depot.composition("géologie"), |c| {
        c["parties"][1]["segment_fin"] = Value::Null;
    });
    assert!(a_erreur(&depot, "composition.segment"));
}

#[test]
fn segments_manquants() {
    let depot = Depot::new();
    depot.modifier(depot.composition("géologie"), |c| {
        for i in 0..2 {
            c["parties"][i]["segment_debut"] = Value::Null;
            c["parties"][i]["segment_fin"] = Value::Null;
        }
    });
    assert!(a_erreur(&depot, "composition.segments_manquants"));
}

#[test]
fn segments_absents() {
    let depot = Depot::new();
    depot.modifier(depot.composition("géologie"), |c| {
        retirer(&mut c["parties"][1], "segment_debut");
        retirer(&mut c["parties"][1], "segment_fin");
    });
    assert!(a_erreur(&depot, "composition.segments_manquants"));
}

#[test]
fn segments_avec_un_trou() {
    let depot = Depot::new();
    depot.modifier(depot.composition("géologie"), |c| {
        c["parties"][1]["segment_debut"] = json!(4);
    });
    assert!(a_erreur(&depot, "composition.pavage"));
}

#[test]
fn segments_qui_ne_vont_pas_au_bout() {
    let depot = Depot::new();
    depot.modifier(depot.composition("géologie"), |c| {
        c["parties"][1]["segment_fin"] = json!(7);
    });
    assert!(a_erreur(&depot, "composition.pavage"));
}

#[test]
fn une_seule_composition_preferee() {
    let depot = Depot::new();
    depot.copier_sous(depot.composition("géologie"), &id("cmp"), |c| {
        retirer(c, "transformations");
    });
    assert!(a_erreur(&depot, "composition.preferee"));
}

#[test]
fn composition_cyclique() {
    let depot = Depot::new();
    let biologiste = depot.unite("biologiste").1;
    depot.modifier(depot.composition("biologie"), |c| {
        c["parties"][0]["unite_id"] = biologiste["id"].clone();
        c["parties"][0]["sens_id"] = biologiste["sens"][0]["id"].clone();
        c["parties"][0]["forme_id"] = biologiste["formes"][0]["id"].clone();
        for i in 0..2 {
            retirer(&mut c["parties"][i], "segment_debut");
            retirer(&mut c["parties"][i], "segment_fin");
        }
    });
    assert!(a_erreur(&depot, "composition.cycle"));
}

#[test]
fn type_de_transformation() {
    let depot = Depot::new();
    depot.modifier(depot.composition("biologiste"), |c| {
        c["transformations"][0]["type_code"] = json!("magie");
    });
    assert!(a_erreur(&depot, "transformation.type_inconnu"));
}

#[test]
fn transformation_vise_une_partie() {
    let depot = Depot::new();
    depot.modifier(depot.composition("biologiste"), |c| {
        c["transformations"][0]["rang_partie"] = json!(5);
    });
    assert!(a_erreur(&depot, "transformation.rang"));
}

// ----------------------------------------------------------------------- relations

#[test]
fn relation_boucle() {
    let depot = Depot::new();
    depot.modifier(depot.relation("géo"), |r| {
        r["unite_source_id"] = r["unite_cible_id"].clone();
    });
    assert!(a_erreur(&depot, "relation.boucle"));
}

#[test]
fn relation_en_double() {
    let depot = Depot::new();
    depot.copier_sous(depot.relation("géo"), &id("rel"), |_| {});
    assert!(a_erreur(&depot, "relation.doublon"));
}

#[test]
fn emprunt_entre_deux_langues() {
    let depot = Depot::new();
    let geo = depot.uid("géo");
    depot.modifier(depot.relation("logie"), |r| {
        r["unite_source_id"] = json!(geo);
    });
    assert!(a_erreur(&depot, "relation.emprunt"));
}

#[test]
fn heritage_suit_la_filiation() {
    let depot = Depot::new();
    depot.modifier(depot.relation("géo"), |r| {
        r["type_code"] = json!("heritage");
    });
    assert!(a_erreur(&depot, "relation.heritage"));
}

#[test]
fn relation_type_inconnu() {
    let depot = Depot::new();
    depot.modifier(depot.relation("géo"), |r| r["type_code"] = json!("magie"));
    assert!(a_erreur(&depot, "relation.type_inconnu"));
}

#[test]
fn relations_acycliques() {
    let depot = Depot::new();
    let (ge, geo) = (depot.uid("γῆ"), depot.uid("géo"));
    depot.copier_sous(depot.relation("géo"), &id("rel"), |r| {
        r["unite_source_id"] = json!(geo);
        r["unite_cible_id"] = json!(ge);
    });
    assert!(a_erreur(&depot, "relation.cycle"));
}

// ------------------------------------------------------------------------- briques

#[test]
fn une_brique_par_unite() {
    let depot = Depot::new();
    depot.copier_sous(depot.brique("géo"), &id("brq"), |_| {});
    assert!(a_erreur(&depot, "brique.unicite"));
}

#[test]
fn brique_affiche_la_forme_canonique() {
    let depot = Depot::new();
    let allomorphe = depot.unite("philo").1["formes"][1]["id"].clone();
    depot.modifier(depot.brique("philo"), |b| {
        b["forme_affichee_id"] = allomorphe;
    });
    assert!(a_erreur(&depot, "brique.forme"));
}

#[test]
fn brique_sens_de_son_unite() {
    let depot = Depot::new();
    let autre = depot.unite("bio").1["sens"][0]["id"].clone();
    depot.modifier(depot.brique("géo"), |b| b["sens_affiche_id"] = autre);
    assert!(a_erreur(&depot, "brique.sens"));
}

#[test]
fn brique_a_une_glose() {
    let depot = Depot::new();
    depot.modifier(depot.unite("géo"), |u| {
        u["sens"][0]["libelles"]["fr"] = json!({"definition": "Relatif à la terre."});
    });
    assert!(a_erreur(&depot, "brique.glose"));
}

// ------------------------------------------------------------------------ recettes

#[test]
fn recette_ambigue() {
    let depot = Depot::new();
    depot.copier_sous(depot.recette("géologie"), &id("rct"), |_| {});
    assert!(a_erreur(&depot, "recette.ambigue"));
}

#[test]
fn recette_accepte_la_brique_de_la_partie() {
    let depot = Depot::new();
    let bio = depot.id_brique("bio");
    depot.modifier(depot.recette("géologie"), |r| {
        r["emplacements"][0]["briques_acceptees"] = json!([bio]);
    });
    assert!(a_erreur(&depot, "recette.brique_partie"));
}

#[test]
fn recette_emplacements_suivent_les_parties() {
    let depot = Depot::new();
    let logie = depot.id_brique("logie");
    depot.modifier(depot.recette("géologie"), |r| {
        r["emplacements"]
            .as_array_mut()
            .unwrap()
            .push(json!({"rang": 3, "briques_acceptees": [logie]}));
    });
    assert!(a_erreur(&depot, "recette.emplacements"));
}

#[test]
fn recette_partie_sans_brique() {
    let depot = Depot::new();
    fs::remove_file(depot.brique("iste").0).unwrap();
    assert!(a_erreur(&depot, "recette.brique_absente"));
}

#[test]
fn recette_definition_obligatoire() {
    let depot = Depot::new();
    depot.modifier(depot.unite("géologie"), |u| {
        u["sens"][0]["libelles"]["fr"] = json!({"glose": "étude de la Terre"});
    });
    assert!(a_erreur(&depot, "recette.definition"));
}

#[test]
fn recette_non_retenue() {
    let depot = Depot::new();
    depot.modifier(depot.composition("géologie"), |c| {
        c["fait"]["confiance"] = json!("non_retenue");
    });
    assert!(a_erreur(&depot, "recette.non_retenue"));
}

#[test]
fn recette_hors_fascicule() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), sans_unite(depot.uid("géologie")));
    assert!(a_avertissement(&depot, "recette.hors_fascicule"));
}

#[test]
fn recette_de_trois_briques() {
    let depot = Depot::new();
    let iste = depot.id_brique("iste");
    depot.modifier(depot.recette("géologie"), |r| {
        r["emplacements"]
            .as_array_mut()
            .unwrap()
            .push(json!({"rang": 3, "briques_acceptees": [iste]}));
    });
    assert!(a_erreur(&depot, "recette.deux_briques"));
}

// ---------------------------------------------------------------------- fascicules

#[test]
fn autonomie() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), sans_unite(depot.uid("géo")));
    assert!(a_erreur(&depot, "fascicule.autonomie"));
}

#[test]
fn ingredient_publie_plus_tard() {
    let depot = Depot::new();
    let biologie = depot.uid("biologie");
    depot.modifier(depot.fascicule(), |f| {
        sans_unite(biologie.clone())(f);
        f["unites"]
            .as_array_mut()
            .unwrap()
            .push(json!({"unite_id": biologie, "role": "ingredient"}));
    });
    depot.copier_sous(depot.fascicule(), &id("fsc"), |f| {
        let mut unites =
            vec![json!({"unite_id": biologie, "role": "resultat", "difficulte": "facile"})];
        unites.extend(
            liste(&f["unites"])
                .iter()
                .filter(|u| u["role"] == "ingredient" && u["unite_id"] != biologie.as_str())
                .cloned(),
        );
        f["numero"] = json!(1);
        f["unites"] = json!(unites);
    });
    assert!(a_erreur(&depot, "fascicule.autonomie"));
}

#[test]
fn unicite_du_resultat() {
    let depot = Depot::new();
    let geologie = depot.uid("géologie");
    depot.copier_sous(depot.fascicule(), &id("fsc"), |f| {
        let unites: Vec<Value> = liste(&f["unites"])
            .iter()
            .filter(|u| u["role"] == "ingredient" || u["unite_id"] == geologie.as_str())
            .cloned()
            .collect();
        f["numero"] = json!(1);
        f["unites"] = json!(unites);
    });
    assert!(a_erreur(&depot, "fascicule.unicite"));
}

#[test]
fn numero_unique() {
    let depot = Depot::new();
    depot.copier_sous(depot.fascicule(), &id("fsc"), |f| {
        f["unites"] = json!([f["unites"][0]]);
    });
    assert!(a_erreur(&depot, "fascicule.numero"));
}

#[test]
fn unite_declaree_deux_fois() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), |f| {
        let unites = f["unites"].as_array_mut().unwrap();
        unites.push(unites.last().unwrap().clone());
    });
    assert!(a_erreur(&depot, "fascicule.unite_dupliquee"));
}

fn promouvoir(uid: String) -> impl FnOnce(&mut Value) {
    move |f| {
        for u in f["unites"].as_array_mut().unwrap() {
            if u["unite_id"] == uid.as_str() {
                u["role"] = json!("resultat");
                u["difficulte"] = json!("facile");
            }
        }
    }
}

#[test]
fn resultat_est_un_mot_avec_recette() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), promouvoir(depot.uid("moral")));
    assert!(a_erreur(&depot, "fascicule.resultat_sans_recette"));
}

#[test]
fn resultat_non_mot() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), promouvoir(depot.uid("géo")));
    assert!(a_erreur(&depot, "fascicule.resultat_non_mot"));
}

#[test]
fn ingredient_sans_brique() {
    let depot = Depot::new();
    let ge = depot.uid("γῆ");
    depot.modifier(depot.fascicule(), |f| {
        f["unites"]
            .as_array_mut()
            .unwrap()
            .push(json!({"unite_id": ge, "role": "ingredient"}));
    });
    assert!(a_erreur(&depot, "fascicule.ingredient_sans_brique"));
}

#[test]
fn ingredient_inutile() {
    let depot = Depot::new();
    let phile = depot.uid("phile");
    depot.modifier(depot.fascicule(), |f| {
        f["unites"]
            .as_array_mut()
            .unwrap()
            .push(json!({"unite_id": phile, "role": "ingredient"}));
    });
    assert!(a_avertissement(&depot, "fascicule.ingredient_inutile"));
}

#[test]
fn taille_du_fascicule() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), |f| {
        f["test"] = json!(false);
        f["numero"] = json!(11);
    });
    assert!(a_avertissement(&depot, "fascicule.taille"));
}

#[test]
fn taille_libre_pour_un_fascicule_de_lancement() {
    // ADR 0054 : les dix premiers fascicules publient plus de 30 mots.
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), publier_numero_1);
    assert!(!a_avertissement(&depot, "fascicule.taille"));
}

#[test]
fn difficulte_obligatoire_pour_un_resultat() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), |f| {
        retirer(&mut f["unites"][0], "difficulte");
    });
    assert!(a_erreur(&depot, "schema"));
}

#[test]
fn difficulte_interdite_pour_un_ingredient() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), |f| {
        let ingredient = f["unites"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|u| u["role"] == "ingredient")
            .unwrap();
        ingredient["difficulte"] = json!("facile");
    });
    assert!(a_erreur(&depot, "schema"));
}

#[test]
fn tiers_de_difficulte() {
    // Le fascicule d'essai ne compte que des mots faciles : écart aux tiers.
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), |f| {
        f["test"] = json!(false);
        f["numero"] = json!(11);
    });
    assert!(a_avertissement(&depot, "fascicule.tiers"));
}

#[test]
fn tiers_libres_pour_un_fascicule_de_lancement() {
    // ADR 0059 : les dix premiers fascicules ne sont pas tenus aux tiers.
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), publier_numero_1);
    assert!(!a_avertissement(&depot, "fascicule.tiers"));
}

#[test]
fn fascicule_sans_legendaire() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), publier_numero_1);
    for chemin in depot.fichiers("ludique/briques") {
        let mut brique = depot.lire(&chemin);
        if brique["rarete"] == "legendaire" {
            brique["rarete"] = json!("rare");
            depot.ecrire(&chemin, &brique);
        }
    }
    assert!(a_avertissement(&depot, "fascicule.legendaire_absente"));
    assert!(erreurs(&depot.valider_publication()).contains("fascicule.legendaire_absente"));
}

#[test]
fn fascicule_avec_legendaire() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), publier_numero_1);
    assert!(!a_avertissement(&depot, "fascicule.legendaire_absente"));
}

#[test]
fn graine_legendaire() {
    let depot = Depot::new();
    let brique = depot.id_brique("étymo"); // légendaire dans le dépôt d'essai
    depot.modifier(depot.fascicule(), |f| {
        f["graines"]
            .as_array_mut()
            .unwrap()
            .push(json!({"brique_id": brique, "exemplaires": 1}));
    });
    assert!(a_erreur(&depot, "fascicule.graine_legendaire"));
}

#[test]
fn jalon_legendaire() {
    let depot = Depot::new();
    let brique = depot.id_brique("étymo");
    depot.modifier(depot.fascicule(), |f| {
        f["jalons"][1]["pli"] = json!([brique]);
    });
    assert!(a_erreur(&depot, "fascicule.jalon_legendaire"));
}

#[test]
fn fascicule_reel_numero_zero() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), |f| f["test"] = json!(false));
    assert!(a_erreur(&depot, "schema"));
}

#[test]
fn graine_inconnue() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), |f| {
        f["graines"]
            .as_array_mut()
            .unwrap()
            .push(json!({"brique_id": "brq_00000000", "exemplaires": 1}));
    });
    assert!(a_erreur(&depot, "reference.cassee"));
}

#[test]
fn graine_dupliquee() {
    let depot = Depot::new();
    let geo = depot.id_brique("géo");
    depot.modifier(depot.fascicule(), |f| {
        f["graines"]
            .as_array_mut()
            .unwrap()
            .push(json!({"brique_id": geo, "exemplaires": 1}));
    });
    assert!(a_erreur(&depot, "fascicule.graine_dupliquee"));
}

#[test]
fn graine_hors_fascicule() {
    let depot = Depot::new();
    let phile = depot.id_brique("phile");
    depot.modifier(depot.fascicule(), |f| {
        sans_unite(depot.uid("phile"))(f);
        f["graines"]
            .as_array_mut()
            .unwrap()
            .push(json!({"brique_id": phile, "exemplaires": 1}));
    });
    assert!(a_erreur(&depot, "fascicule.graine_hors_fascicule"));
}

#[test]
fn graine_exemplaires_bornes() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), |f| {
        f["graines"][0]["exemplaires"] = json!(6);
    });
    assert!(a_erreur(&depot, "schema"));
}

#[test]
fn amorcage_insuffisant() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), |f| {
        f["graines"].as_array_mut().unwrap().truncate(2);
    });
    assert!(a_avertissement(&depot, "fascicule.amorcage"));
}

#[test]
fn amorcage_exemplaires_insuffisants() {
    let depot = Depot::new();
    let logie = depot.id_brique("logie");
    depot.modifier(depot.fascicule(), |f| {
        for g in f["graines"].as_array_mut().unwrap() {
            if g["brique_id"] == logie.as_str() {
                g["exemplaires"] = json!(1);
            }
        }
    });
    assert!(a_avertissement(&depot, "fascicule.amorcage"));
}

#[test]
fn premier_fascicule_sans_graines() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), |f| {
        retirer(f, "graines");
    });
    assert!(a_avertissement(&depot, "fascicule.amorcage"));
}

#[test]
fn jalon_ordre() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), |f| {
        f["jalons"][1]["decouvertes"] = json!(3);
    });
    assert!(a_erreur(&depot, "fascicule.jalon_ordre"));
}

#[test]
fn jalon_duplique() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), |f| {
        f["jalons"][1]["id"] = f["jalons"][0]["id"].clone();
    });
    assert!(a_erreur(&depot, "fascicule.jalon_duplique"));
}

#[test]
fn jalon_sans_recompense() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), |f| f["jalons"][1]["sabliers"] = json!(0));
    assert!(a_erreur(&depot, "schema"));
}

#[test]
fn jalon_brique_mot() {
    let depot = Depot::new();
    let declares: Vec<Value> = liste(&depot.fascicule().1["unites"])
        .iter()
        .map(|l| l["unite_id"].clone())
        .collect();
    let mot = depot
        .fichiers("ludique/briques")
        .into_iter()
        .map(|p| depot.lire(&p))
        .find(|b| {
            let unite = depot.lire(
                &depot
                    .racine
                    .join("unites")
                    .join(format!("{}.json", texte(&b["unite_id"]))),
            );
            declares.contains(&b["unite_id"]) && unite["nature"] == "mot"
        })
        .unwrap()["id"]
        .clone();
    depot.modifier(depot.fascicule(), |f| {
        f["jalons"][0]["pli"].as_array_mut().unwrap().push(mot);
    });
    assert!(a_erreur(&depot, "fascicule.jalon_mot"));
}

#[test]
fn jalon_hors_fascicule() {
    let depot = Depot::new();
    let phile = depot.id_brique("phile");
    depot.modifier(depot.fascicule(), |f| {
        sans_unite(depot.uid("phile"))(f);
        f["jalons"][0]["pli"]
            .as_array_mut()
            .unwrap()
            .push(json!(phile));
    });
    assert!(a_erreur(&depot, "fascicule.jalon_hors_fascicule"));
}

// ---------------------------------------------------------------------- exclusions

fn exclusion(depot: &Depot, combinaison: [&str; 2], note: Option<&str>) {
    let identifiant = id("exc");
    let mut donnees = json!({
        "id": identifiant, "fascicule_id": depot.fascicule().1["id"], "combinaison": combinaison,
        "raison": "archaique", "etat_relecture": "brouillon",
    });
    if let Some(note) = note {
        donnees["note_interne"] = json!(note);
    }
    depot.ecrire(
        &depot
            .racine
            .join("ludique/exclusions")
            .join(format!("{identifiant}.json")),
        &donnees,
    );
}

#[test]
fn exclusion_contredit_une_recette() {
    let depot = Depot::new();
    exclusion(
        &depot,
        [&depot.id_brique("géo"), &depot.id_brique("logie")],
        None,
    );
    assert!(a_erreur(&depot, "exclusion.recette"));
}

#[test]
fn exclusion_valide() {
    let depot = Depot::new();
    exclusion(
        &depot,
        [&depot.id_brique("géo"), &depot.id_brique("iste")],
        Some("Cas de test."),
    );
    assert!(erreurs(&depot.valider()).is_empty());
}

// ----------------------------------------------------------------------- trompe-l'œil

pub fn source_quelconque(depot: &Depot) -> String {
    texte(&depot.lire(&depot.fichiers("sources")[0])["id"]).to_owned()
}

/// Écrit un trompe-l'œil valide sur la paire donnée, après l'avoir laissé se modifier.
pub fn trompe_l_oeil(
    depot: &Depot,
    combinaison: [&str; 2],
    modifier: impl FnOnce(&mut Value),
) -> String {
    let identifiant = id("tpl");
    let mut donnees = json!({
        "id": identifiant, "fascicule_id": depot.fascicule().1["id"], "combinaison": combinaison,
        "motif": "autre_origine",
        "explication_courte": {"fr": "Les lettres se suivent, mais le mot n’en vient pas."},
        "explication": {"fr": "Le mot a une autre histoire que ces deux briques."},
        "fait": {
            "confiance": "etablie", "etat_relecture": "brouillon",
            "sources": [{"source_id": source_quelconque(depot), "localisation": null}],
            "note_interne": "Cas de test."
        },
    });
    modifier(&mut donnees);
    depot.ecrire(
        &depot
            .racine
            .join("ludique/trompe-l-oeil")
            .join(format!("{identifiant}.json")),
        &donnees,
    );
    identifiant
}

fn paire_libre(depot: &Depot) -> [String; 2] {
    [depot.id_brique("géo"), depot.id_brique("iste")]
}

#[test]
fn trompe_l_oeil_valide() {
    let depot = Depot::new();
    let [a, b] = paire_libre(&depot);
    trompe_l_oeil(&depot, [&a, &b], |_| {});
    assert!(erreurs(&depot.valider()).is_empty());
}

#[test]
fn trompe_l_oeil_exige_deux_briques() {
    let depot = Depot::new();
    let [a, b] = paire_libre(&depot);
    trompe_l_oeil(&depot, [&a, &b], |t| {
        t["combinaison"] = json!([a, b, a]);
    });
    assert!(!erreurs(&depot.valider()).is_empty());
}

#[test]
fn trompe_l_oeil_doublon() {
    let depot = Depot::new();
    let [a, b] = paire_libre(&depot);
    trompe_l_oeil(&depot, [&a, &b], |_| {});
    trompe_l_oeil(&depot, [&a, &b], |_| {});
    assert!(a_erreur(&depot, "trompe_l_oeil.doublon"));
}

#[test]
fn trompe_l_oeil_ne_contredit_pas_une_recette_ni_son_inverse() {
    let depot = Depot::new();
    let (geo, logie) = (depot.id_brique("géo"), depot.id_brique("logie"));
    trompe_l_oeil(&depot, [&geo, &logie], |_| {});
    assert!(a_erreur(&depot, "trompe_l_oeil.recette"));
    let depot = Depot::new();
    trompe_l_oeil(&depot, [&logie, &geo], |_| {});
    assert!(a_erreur(&depot, "trompe_l_oeil.recette"));
}

#[test]
fn trompe_l_oeil_n_est_pas_une_exclusion() {
    let depot = Depot::new();
    let [a, b] = paire_libre(&depot);
    exclusion(&depot, [&a, &b], None);
    trompe_l_oeil(&depot, [&a, &b], |_| {});
    assert!(a_erreur(&depot, "trompe_l_oeil.recette"));
}

#[test]
fn trompe_l_oeil_brique_inconnue() {
    let depot = Depot::new();
    let [a, _] = paire_libre(&depot);
    trompe_l_oeil(&depot, [&a, "brq_zzzzzzzz"], |_| {});
    assert!(a_erreur(&depot, "reference.cassee"));
}

#[test]
fn trompe_l_oeil_brique_non_publiee_dans_le_fascicule() {
    let depot = Depot::new();
    let declarees: Vec<String> = liste(&depot.fascicule().1["unites"])
        .iter()
        .map(|u| texte(&u["unite_id"]).to_owned())
        .collect();
    let hors = depot
        .fichiers("ludique/briques")
        .iter()
        .map(|p| depot.lire(p))
        .find(|b| !declarees.contains(&texte(&b["unite_id"]).to_owned()))
        .expect("une brique d'une unité non déclarée par le fascicule d'essai");
    let [a, _] = paire_libre(&depot);
    trompe_l_oeil(&depot, [&a, texte(&hors["id"])], |_| {});
    assert!(a_erreur(&depot, "trompe_l_oeil.paire"));
}

#[test]
fn trompe_l_oeil_ne_vise_pas_une_legendaire() {
    let depot = Depot::new();
    let [a, _] = paire_libre(&depot);
    trompe_l_oeil(&depot, [&a, &depot.id_brique("étymo")], |_| {});
    assert!(a_erreur(&depot, "trompe_l_oeil.legendaire"));
}

#[test]
fn trompe_l_oeil_deja_couvert_par_un_homographe() {
    // Une seconde brique « géo » : poser celle-là avec « logie » est un mauvais homographe, que le
    // serveur calcule déjà ; le saisir à la main ferait deux mécanismes pour un seul cas.
    let depot = Depot::new();
    let logie = depot.id_brique("logie");
    let (unite, brique) = (depot.unite("géo"), depot.brique("géo"));
    let nouvelle_unite = id("unt");
    depot.copier_sous(unite, &nouvelle_unite, |u| {
        u["formes"][0]["id"] = json!(id("frm"));
        u["sens"][0]["id"] = json!(id("sns"));
    });
    let nouvelle_brique = id("brq");
    depot.copier_sous(brique, &nouvelle_brique, |b| {
        b["unite_id"] = json!(nouvelle_unite);
    });
    trompe_l_oeil(&depot, [&nouvelle_brique, &logie], |_| {});
    assert!(a_erreur(&depot, "trompe_l_oeil.homographe_jouable"));
}

#[test]
fn trompe_l_oeil_du_mot_composé_de_ces_deux_unites_est_une_recette() {
    let depot = Depot::new();
    let (geo, logie) = (depot.id_brique("géo"), depot.id_brique("logie"));
    let mot = depot.uid("géologie");
    trompe_l_oeil(&depot, [&geo, &logie], |t| {
        t["mot_id"] = json!(mot);
    });
    assert!(a_erreur(&depot, "trompe_l_oeil.origine"));
}

#[test]
fn trompe_l_oeil_mot_inconnu() {
    let depot = Depot::new();
    let [a, b] = paire_libre(&depot);
    trompe_l_oeil(&depot, [&a, &b], |t| {
        t["mot_id"] = json!("unt_zzzzzzzz");
    });
    assert!(a_erreur(&depot, "reference.cassee"));
}

#[test]
fn trompe_l_oeil_ne_nomme_pas_le_mot_dans_l_explication_courte() {
    let depot = Depot::new();
    let [a, b] = paire_libre(&depot);
    let mot = depot.uid("géologie");
    trompe_l_oeil(&depot, [&a, &b], |t| {
        t["mot_id"] = json!(mot);
        t["explication_courte"] = json!({"fr": "La Géologie n’en vient pas."});
    });
    assert!(a_erreur(&depot, "trompe_l_oeil.divulgation"));
    // Un fragment d'un autre mot n'est pas une divulgation.
    let depot = Depot::new();
    let [a, b] = paire_libre(&depot);
    let mot = depot.uid("géologie");
    trompe_l_oeil(&depot, [&a, &b], |t| {
        t["mot_id"] = json!(mot);
        t["explication_courte"] = json!({"fr": "Les lettres se suivent, mais l’histoire diffère."});
    });
    assert!(!a_erreur(&depot, "trompe_l_oeil.divulgation"));
}

#[test]
fn trompe_l_oeil_cite_une_source() {
    let depot = Depot::new();
    let [a, b] = paire_libre(&depot);
    trompe_l_oeil(&depot, [&a, &b], |t| {
        t["fait"]["sources"] = json!([]);
    });
    assert!(a_erreur(&depot, "trompe_l_oeil.sources"));
}

#[test]
fn trompe_l_oeil_non_relu_bloque_la_publication() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), publier_numero_1);
    let [a, b] = paire_libre(&depot);
    trompe_l_oeil(&depot, [&a, &b], |_| {});
    assert!(erreurs(&depot.valider_publication()).contains("publication.trompe_l_oeil"));
    // Relu, il ne bloque plus.
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), publier_numero_1);
    trompe_l_oeil(&depot, [&a, &b], |t| {
        t["fait"]["etat_relecture"] = json!("relu");
    });
    assert!(!erreurs(&depot.valider_publication()).contains("publication.trompe_l_oeil"));
}

#[test]
fn candidats_listent_les_paires_dont_la_concatenation_est_un_mot() {
    use crate::candidats::chercher;
    use crate::chargement::charger;
    use crate::rapport::Rapport;

    let depot = Depot::new();
    let (geo, iste, logie) = (
        depot.id_brique("géo"),
        depot.id_brique("iste"),
        depot.id_brique("logie"),
    );
    // Un mot « géoiste » : géo + iste s'écrivent comme lui, sans recette.
    let nouveau = id("unt");
    depot.copier_sous(depot.unite("géologie"), &nouveau, |u| {
        u["formes"][0]["id"] = json!(id("frm"));
        u["formes"][0]["texte"] = json!("géoiste");
        u["formes"][0]["cle_normalisee"] = json!("geoiste");
        u["sens"][0]["id"] = json!(id("sns"));
    });
    let catalogue = charger(&depot.racine, &mut Rapport::default()).unwrap();
    let paires: Vec<(String, String)> = chercher(&catalogue)
        .into_iter()
        .map(|c| (c.premiere, c.seconde))
        .collect();
    assert!(paires.contains(&(geo.clone(), iste.clone())));
    // Une recette n'est jamais un candidat.
    assert!(!paires.contains(&(geo.clone(), logie.clone())));
    // Une fois déclarée, la fausse piste sort de la liste.
    trompe_l_oeil(&depot, [&geo, &iste], |_| {});
    let catalogue = charger(&depot.racine, &mut Rapport::default()).unwrap();
    assert!(
        !chercher(&catalogue)
            .into_iter()
            .any(|c| c.premiere == geo && c.seconde == iste)
    );
}

// -------------------------------------------------------------------------- médias

#[test]
fn media_cible_inconnue() {
    let depot = Depot::new();
    depot.modifier(depot.media_de("unite_id", &depot.uid("géo")), |m| {
        m["unite_id"] = json!(ID_ABSENT_UNT);
    });
    assert_eq!(erreurs(&depot.valider()), ensemble(&["reference.cassee"]));
}

#[test]
fn media_deux_cibles() {
    let depot = Depot::new();
    depot.modifier(depot.media_de("unite_id", &depot.uid("géo")), |m| {
        m["langue_id"] = json!("lng_wa90y54n");
    });
    assert!(a_erreur(&depot, "schema"));
}

#[test]
fn media_jaquette_sans_fascicule() {
    let depot = Depot::new();
    depot.modifier(depot.media_de("unite_id", &depot.uid("géo")), |m| {
        m["type"] = json!("jaquette");
        m["fichier"] = json!("jaquettes/geo.svg");
    });
    assert!(a_erreur(&depot, "schema"));
}

#[test]
fn media_chemin_hors_du_dossier() {
    let depot = Depot::new();
    depot.modifier(depot.media_de("unite_id", &depot.uid("géo")), |m| {
        m["fichier"] = json!("silhouettes/../../unites/x.svg");
    });
    assert!(a_erreur(&depot, "schema"));
}

#[test]
fn media_fichier_absent() {
    let depot = Depot::new();
    let (_, media) = depot.media_de("unite_id", &depot.uid("géo"));
    fs::remove_file(depot.racine.join("medias").join(texte(&media["fichier"]))).unwrap();
    assert_eq!(
        erreurs(&depot.valider()),
        ensemble(&["media.fichier_absent"])
    );
}

#[test]
fn media_double() {
    let depot = Depot::new();
    depot.media("silhouette", "unite_id", &depot.uid("géo"), None);
    assert_eq!(erreurs(&depot.valider()), ensemble(&["media.double"]));
}

#[test]
fn media_orphelin() {
    let depot = Depot::new();
    fs::write(
        depot.racine.join("medias/silhouettes/brouillon.svg"),
        "<svg/>",
    )
    .unwrap();
    let rapport = depot.valider();
    assert!(erreurs(&rapport).is_empty());
    assert_eq!(
        avertissements_notables(&rapport),
        ensemble(&["media.orphelin"])
    );
}

#[test]
fn media_svg_invalide() {
    let depot = Depot::new();
    fs::remove_file(depot.media_de("unite_id", &depot.uid("phile")).0).unwrap(); // une seule silhouette par unité
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 300 400"><script>alert(1)</script></svg>"#;
    depot.media("silhouette", "unite_id", &depot.uid("phile"), Some(svg));
    assert_eq!(erreurs(&depot.valider()), ensemble(&["media.svg_invalide"]));
}

fn svg(corps: &str) -> Vec<u8> {
    format!(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 300 400">{corps}</svg>"#)
        .into_bytes()
}

#[test]
fn profil_svg_refuse() {
    for corps in [
        "<script>alert(1)</script>",
        r#"<foreignObject><div xmlns="http://www.w3.org/1999/xhtml">x</div></foreignObject>"#,
        r#"<image href="https://exemple.invalid/x.png"/>"#,
        r##"<use href="#a"/>"##,
        "<style>circle{fill:red}</style>",
        r#"<circle r="4" onload="alert(1)"/>"#,
        r#"<circle r="4" fill="url(https://exemple.invalid/x.svg#a)"/>"#,
        r#"<circle r="4" style="fill:url(javascript:alert(1))"/>"#,
        r#"<circle r="4" style="behavior:url(#a)"/>"#,
        r#"<a xmlns:xlink="http://www.w3.org/1999/xlink" xlink:href="javascript:alert(1)"><circle r="4"/></a>"#,
        "<text>biologie</text>",
        "<title>biologie</title>",
        "<g>biologie</g>",
        "<!-- biologie -->",
        r##"<circle r="4" fill="#c0392b"/>"##,
        r#"<circle r="4" fill="currentColor" opacity="0.5"/>"#,
    ] {
        assert!(
            !verifier_svg(&svg(corps), "silhouette").is_empty(),
            "{corps}"
        );
    }
}

#[test]
fn profil_svg_refuse_la_dtd() {
    let svg = r#"<?xml version="1.0"?><!DOCTYPE svg [<!ENTITY x "biologie">]><svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 300 400"/>"#;
    assert!(!verifier_svg(svg.as_bytes(), "silhouette").is_empty());
}

#[test]
fn profil_svg_silhouette_au_format_carte() {
    let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 600 800"><circle r="4" fill="currentColor"/></svg>"#;
    assert!(!verifier_svg(svg.as_bytes(), "silhouette").is_empty());
    assert!(verifier_svg(svg.as_bytes(), "jaquette").is_empty());
}

#[test]
fn profil_svg_jaquette_en_couleur() {
    let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 600 800"><rect width="600" height="800" fill="#1f5c4a" opacity="0.9"/></svg>"##;
    assert!(verifier_svg(svg.as_bytes(), "jaquette").is_empty());
}

#[test]
fn profil_svg_admet_evidement_et_decoupe() {
    let corps = concat!(
        r#"<defs><clipPath id="c1"><circle cx="1" cy="1" r="1"/></clipPath></defs>"#,
        r##"<g clip-path="url(#c1)" fill="currentColor"><path d="M0 0L1 1"/></g>"##,
        r#"<path d="M0 0L1 1" fill="none" stroke="currentColor" stroke-width="4" style="stroke:var(--art-bg)"/>"#,
    );
    let mut contenu = b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n".to_vec();
    contenu.extend(svg(corps));
    assert_eq!(verifier_svg(&contenu, "silhouette"), Vec::<String>::new());
}

#[test]
fn media_manquant_avertit_hors_publication() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), publier_numero_1);
    fs::remove_file(
        depot
            .media_de("fascicule_id", texte(&depot.fascicule().1["id"]))
            .0,
    )
    .unwrap();
    let rapport = depot.valider();
    assert!(avertissements(&rapport).contains("media.manquant"));
    assert!(!erreurs(&rapport).contains("media.manquant"));
}

#[test]
fn media_manquant_bloque_la_publication() {
    let depot = Depot::new();
    rendre_publiable(&depot);
    fs::remove_file(depot.media_de("langue_id", "lng_kmf44d76").0).unwrap();
    assert_eq!(
        erreurs(&depot.valider_publication()),
        ensemble(&["media.manquant"])
    );
}

#[test]
fn media_manquant_ignore_les_fascicules_de_test() {
    let depot = Depot::new();
    fs::remove_file(depot.media_de("unite_id", &depot.uid("géo")).0).unwrap();
    let rapport = depot.valider_publication();
    assert!(!erreurs(&rapport).contains("media.manquant"));
    assert!(
        !rapport
            .diagnostics
            .iter()
            .any(|d| d.regle == "media.manquant" && d.message.starts_with("Fascicule"))
    );
}

#[test]
fn carte_sans_visuel_affectee_est_une_erreur_meme_hors_publication() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), publier_numero_1);
    fs::remove_file(depot.media_de("unite_id", &depot.uid("géo")).0).unwrap();
    assert!(a_erreur(&depot, "media.manquant"));
}

#[test]
fn carte_sans_visuel_en_attente_avertit() {
    let depot = Depot::new();
    let affectees: Vec<Value> = liste(&depot.fascicule().1["unites"])
        .iter()
        .map(|l| l["unite_id"].clone())
        .collect();
    let catalogue = charger(&depot.racine, &mut Rapport::default()).unwrap();
    let libre = catalogue
        .unites
        .keys()
        .find(|u| !affectees.contains(&json!(u)))
        .unwrap();
    let rapport = depot.valider();
    assert!(rapport.diagnostics.iter().any(|d| d.niveau == AVERTISSEMENT
        && d.regle == "media.manquant"
        && d.objet.as_deref() == Some(libre.as_str())));
    assert!(!erreurs(&rapport).contains("media.manquant"));
}

// --------------------------------------------------------------------- publication

/// Une silhouette pour chaque carte et chaque langue citée, et la jaquette du fascicule.
fn completer_medias(depot: &Depot) {
    let fascicule = depot.fascicule().1;
    let catalogue = charger(&depot.racine, &mut Rapport::default()).unwrap();
    let illustres: Vec<String> = catalogue
        .medias
        .values()
        .map(|m| cible(m).1.to_owned())
        .collect();
    let uids: Vec<String> = liste(&fascicule["unites"])
        .iter()
        .map(|l| texte(&l["unite_id"]).to_owned())
        .collect();
    for uid in &uids {
        if !illustres.contains(uid) {
            depot.media("silhouette", "unite_id", uid, None);
        }
    }
    for lid in langues_citees(&catalogue, &uids) {
        if !illustres.contains(&lid) {
            depot.media("silhouette", "langue_id", &lid, None);
        }
    }
    let fid = texte(&fascicule["id"]);
    if !illustres.iter().any(|i| i == fid) {
        depot.media("jaquette", "fascicule_id", fid, None);
    }
}

pub(super) fn rendre_publiable(depot: &Depot) {
    depot.modifier(depot.fascicule(), publier_numero_1);
    completer_medias(depot);
    let relire = |fait: &mut Value| {
        fait["etat_relecture"] = json!("relu");
        for lien in fait["sources"].as_array_mut().unwrap() {
            lien["localisation"] = json!("entrée consultée");
        }
    };
    for dossier in ["unites", "compositions", "relations"] {
        for chemin in depot.fichiers(dossier) {
            let mut donnees = depot.lire(&chemin);
            relire(&mut donnees["fait"]);
            for cle in ["formes", "sens"] {
                for x in donnees
                    .get_mut(cle)
                    .and_then(Value::as_array_mut)
                    .into_iter()
                    .flatten()
                {
                    relire(&mut x["fait"]);
                }
            }
            depot.ecrire(&chemin, &donnees);
        }
    }
}

#[test]
fn publication_exige_la_relecture() {
    let depot = Depot::new();
    depot.modifier(depot.fascicule(), publier_numero_1);
    let regles = erreurs(&depot.valider_publication());
    assert!(ensemble(&["publication.relecture", "publication.localisation"]).is_subset(&regles));
}

#[test]
fn publication_complete() {
    let depot = Depot::new();
    rendre_publiable(&depot);
    let rapport = depot.valider_publication();
    assert!(erreurs(&rapport).is_empty(), "{}", rapport.en_texte());
}

#[test]
fn publication_refuse_le_reperage_seul() {
    let depot = Depot::new();
    rendre_publiable(&depot);
    let wiktionnaire = depot.trouver("sources", |s| s["role"] == "reperage").1["id"].clone();
    depot.modifier(depot.unite("géo"), |u| {
        u["fait"]["sources"] = json!([{"source_id": wiktionnaire, "localisation": "géo-"}]);
    });
    assert!(erreurs(&depot.valider_publication()).contains("publication.source_reference"));
}

#[test]
fn publication_discutee_deux_references() {
    let depot = Depot::new();
    rendre_publiable(&depot);
    depot.modifier(depot.composition("géologie"), |c| {
        c["fait"]["confiance"] = json!("discutee");
    });
    assert!(erreurs(&depot.valider_publication()).contains("publication.source_reference"));
}

#[test]
fn publication_hors_perimetre_ignore() {
    let depot = Depot::new();
    rendre_publiable(&depot);
    depot.modifier(depot.unite("phile"), |u| {
        u["fait"]["etat_relecture"] = json!("brouillon");
    });
    assert!(erreurs(&depot.valider_publication()).is_empty());
}

// ------------------------------------------------------------ identifiants et CLI

#[test]
fn nouvel_id() {
    let ids: std::collections::HashSet<String> = (0..200).map(|_| id("unt")).collect();
    assert_eq!(ids.len(), 200);
    assert!(
        ids.iter()
            .all(|i| crate::ids::MOTIF.is_match(i) && i.starts_with("unt_"))
    );
    assert!(crate::ids::nouvel_id("xyz", &std::collections::HashSet::new()).is_err());
}

pub(super) fn cli(arguments: &[&str]) -> u8 {
    use clap::Parser;
    executer(
        Cli::try_parse_from(std::iter::once("etymo-contenu").chain(arguments.iter().copied()))
            .unwrap()
            .commande,
    )
    .unwrap()
}

#[test]
fn cli_codes_de_retour() {
    let depot = Depot::new();
    let racine = depot.racine.to_str().unwrap();
    assert_eq!(cli(&["valider", "--racine", racine]), 0);
    depot.modifier(depot.brique("géo"), |b| {
        b["unite_id"] = json!(ID_ABSENT_UNT);
    });
    assert_eq!(cli(&["valider", "--racine", racine, "--json"]), 1);
    assert!(depot.valider().en_json().contains("\"reference.cassee\""));
}
