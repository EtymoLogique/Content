//! Visualisation locale du catalogue : une page HTML autonome, pour l'équipe éditoriale.
//!
//! La page montre tout le dépôt, faits non relus et notes internes compris : elle
//! reste locale, dans `rapports/` que Git ignore, et n'est jamais publiée. Elle
//! n'appelle aucun service tiers. Les données sont embarquées en JSON et insérées
//! par `textContent` ; une silhouette n'est insérée que si elle passe le profil
//! SVG de `medias.rs`.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use serde_json::{Value, json};

use crate::chargement::Catalogue;
use crate::json::{COMPACT, a_cle, ecrire, liste, texte, vrai};
use crate::medias;
use crate::rapport::Rapport;

const GABARIT: &str = include_str!("visualisation.html");

fn canonique(unite: &Value) -> Value {
    let formes = liste(&unite["formes"]);
    formes
        .iter()
        .find(|f| f["role"] == "canonique")
        .or(formes.first())
        .cloned()
        .unwrap_or_else(|| json!({}))
}

fn libelle(objet: &Value, cle: &str) -> Value {
    objet["libelles"]["fr"][cle].clone()
}

fn fait(objet: &Value) -> Value {
    let fait = &objet["fait"];
    json!({"confiance": fait["confiance"], "relecture": fait["etat_relecture"], "note": fait["note_interne"]})
}

fn svg(catalogue: &Catalogue, media: Option<&&Value>) -> Value {
    let Some(media) = media else {
        return Value::Null;
    };
    let fichier = medias::chemin(catalogue, media);
    let Ok(contenu) = fs::read(&fichier)
        .map_err(drop)
        .and_then(|c| if fichier.is_file() { Ok(c) } else { Err(()) })
    else {
        return json!({"fichier": media["fichier"], "probleme": "fichier introuvable"});
    };
    match medias::verifier_svg(&contenu, texte(&media["type"])).first() {
        Some(probleme) => json!({"fichier": media["fichier"], "probleme": probleme}),
        None => json!({"fichier": media["fichier"], "svg": String::from_utf8_lossy(&contenu)}),
    }
}

fn cle(type_media: &str, nature: &str, identifiant: &str) -> medias::CleMedia {
    (
        type_media.to_owned(),
        nature.to_owned(),
        identifiant.to_owned(),
    )
}

#[must_use]
pub fn donnees(catalogue: &Catalogue, rapport: &Rapport) -> Value {
    let index_medias = medias::par_cible(catalogue);
    let brique_par_unite: HashMap<&str, &Value> = catalogue
        .briques
        .values()
        .map(|b| (texte(&b["unite_id"]), b))
        .collect();
    let mut recettes: Vec<_> = catalogue.recettes.iter().collect();
    recettes.sort_by(|a, b| a.0.cmp(b.0));
    let mut recette_par_mot: HashMap<&str, Vec<&String>> = HashMap::new();
    for (rid, recette) in recettes {
        if let Some(cmp) = catalogue
            .compositions
            .get(texte(&recette["composition_id"]))
        {
            recette_par_mot
                .entry(texte(&cmp["mot_id"]))
                .or_default()
                .push(rid);
        }
    }
    let mut fascicules_par_unite: HashMap<&str, Vec<Value>> = HashMap::new();
    for (fid, fsc) in &catalogue.fascicules {
        for ligne in liste(&fsc["unites"]) {
            fascicules_par_unite
                .entry(texte(&ligne["unite_id"]))
                .or_default()
                .push(json!({"id": fid, "numero": fsc["numero"], "role": ligne["role"]}));
        }
    }

    let mut unites_triees: Vec<_> = catalogue.unites.iter().collect();
    unites_triees.sort_by(|a, b| a.0.cmp(b.0));
    let unites: Vec<Value> = unites_triees
        .into_iter()
        .map(|(uid, unite)| {
            let forme = canonique(unite);
            let mut sens: Vec<&Value> = liste(&unite["sens"]).iter().collect();
            sens.sort_by_key(|s| s["rang"].as_i64().unwrap_or_default());
            let mut fascicules = fascicules_par_unite.get(uid.as_str()).cloned().unwrap_or_default();
            fascicules.sort_by_key(|f| f["numero"].as_i64());
            json!({
                "id": uid,
                "nature": unite["nature"],
                "langue": unite["langue_id"],
                "texte": if a_cle(&forme, "texte") { forme["texte"].clone() } else { json!(uid) },
                "translitteration": liste(&forme["translitterations"]).first().map(|t| t["texte"].clone()),
                "sens": sens.iter().map(|s| json!({"glose": libelle(s, "glose"), "definition": libelle(s, "definition")})).collect::<Vec<_>>(),
                "fait": fait(unite),
                "brique": brique_par_unite.get(uid.as_str()).map(|b| json!({"id": b["id"], "rarete": b["rarete"]})),
                "recettes": recette_par_mot.get(uid.as_str()).cloned().unwrap_or_default(),
                "fascicules": fascicules,
                "silhouette": svg(catalogue, index_medias.get(&cle("silhouette", "unite", uid))),
            })
        })
        .collect();

    let mut liens = Vec::new();
    let mut compositions: Vec<_> = catalogue.compositions.iter().collect();
    compositions.sort_by(|a, b| a.0.cmp(b.0));
    for (cid, cmp) in compositions {
        for partie in liste(&cmp["parties"]) {
            liens.push(json!({
                "id": cid, "type": "composition", "source": partie["unite_id"], "cible": cmp["mot_id"],
                "rang": partie["rang"], "preferee": if a_cle(cmp, "preferee") { cmp["preferee"].clone() } else { json!(false) },
                "sens_litteral": libelle(cmp, "sens_litteral"), "fait": fait(cmp),
            }));
        }
    }
    let mut relations: Vec<_> = catalogue.relations.iter().collect();
    relations.sort_by(|a, b| a.0.cmp(b.0));
    for (rid, rel) in relations {
        liens.push(json!({"id": rid, "type": rel["type_code"], "source": rel["unite_source_id"], "cible": rel["unite_cible_id"], "fait": fait(rel)}));
    }
    liens.retain(|l| {
        catalogue.unites.contains_key(texte(&l["source"]))
            && catalogue.unites.contains_key(texte(&l["cible"]))
    });

    let mut langues: Vec<_> = catalogue.langues.iter().collect();
    langues.sort_by(|a, b| a.0.cmp(b.0));
    let langues: Vec<Value> = langues
        .into_iter()
        .map(|(lid, l)| {
            let nom = libelle(l, "nom");
            json!({
                "id": lid, "nom": if vrai(&nom) { nom } else { l["code_interne"].clone() }, "code": l["code_interne"],
                "silhouette": svg(catalogue, index_medias.get(&cle("silhouette", "langue", lid))),
            })
        })
        .collect();

    let mut fascicules: Vec<_> = catalogue.fascicules.iter().collect();
    fascicules.sort_by_key(|(_, f)| (vrai(&f["test"]), f["numero"].as_i64()));
    let fascicules: Vec<Value> = fascicules
        .into_iter()
        .map(|(fid, f)| {
            json!({
                "id": fid, "numero": f["numero"], "parution": f["parution"], "test": f["test"],
                "unites": liste(&f["unites"]).iter().map(|l| &l["unite_id"]).collect::<Vec<_>>(),
                "jaquette": svg(catalogue, index_medias.get(&cle("jaquette", "fascicule", fid))),
            })
        })
        .collect();

    let diagnostics: Vec<Value> = rapport
        .diagnostics
        .iter()
        .map(|d| json!({"niveau": d.niveau, "regle": d.regle, "fichier": d.fichier, "message": d.message}))
        .collect();

    json!({"langues": langues, "unites": unites, "liens": liens, "fascicules": fascicules, "diagnostics": diagnostics})
}

/// JSON sûr dans un <script> : aucun « </script> » ni commentaire HTML ne peut en sortir.
fn json_pour_script(objet: &Value) -> String {
    ecrire(objet, COMPACT)
        .replace('<', "\\u003c")
        .replace('>', "\\u003e")
        .replace('&', "\\u0026")
        .replace('\u{2028}', "\\u2028")
        .replace('\u{2029}', "\\u2029")
}

#[must_use]
pub fn page(catalogue: &Catalogue, rapport: &Rapport) -> String {
    GABARIT.replace(
        "/*DONNEES*/null",
        &json_pour_script(&donnees(catalogue, rapport)),
    )
}

pub fn ecrire_page(catalogue: &Catalogue, rapport: &Rapport, sortie: &Path) -> std::io::Result<()> {
    if let Some(parent) = sortie.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::write(sortie, page(catalogue, rapport))
}
