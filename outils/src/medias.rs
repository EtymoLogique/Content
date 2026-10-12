//! Médias du catalogue : profil SVG admis et rattachement aux cartes (ADR 0016, 0023, 0025, 0026).
//!
//! Un SVG publié est servi tel quel par le CDN, puis inséré dans la page par la
//! PWA : c'est un vecteur XSS. Le profil est donc une liste blanche, et il refuse
//! aussi tout texte, commentaire ou titre qui trahirait le mot d'une carte.

use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::sync::LazyLock;

use regex::Regex;
use roxmltree::{Document, Node};
use serde_json::Value;

use crate::chargement::Catalogue;
use crate::json::{a_cle, texte};

pub const DOSSIER: &str = "medias";
pub const SVG_NS: &str = "http://www.w3.org/2000/svg";

pub const VIEWBOX_SILHOUETTE: &str = "0 0 300 400"; // format 3:4 des cartes

const ELEMENTS: [&str; 12] = [
    "svg", "g", "defs", "clipPath", "mask", "path", "circle", "ellipse", "rect", "line",
    "polyline", "polygon",
];
const ATTRIBUTS: [&str; 38] = [
    "viewBox",
    "preserveAspectRatio",
    "width",
    "height",
    "id",
    "d",
    "cx",
    "cy",
    "r",
    "rx",
    "ry",
    "x",
    "y",
    "x1",
    "y1",
    "x2",
    "y2",
    "points",
    "transform",
    "fill",
    "fill-rule",
    "fill-opacity",
    "stroke",
    "stroke-width",
    "stroke-linecap",
    "stroke-linejoin",
    "stroke-dasharray",
    "stroke-dashoffset",
    "stroke-miterlimit",
    "stroke-opacity",
    "opacity",
    "clip-path",
    "clip-rule",
    "clipPathUnits",
    "mask",
    "maskUnits",
    "maskContentUnits",
    "style",
];
const PROPRIETES_STYLE: [&str; 9] = [
    "fill",
    "fill-rule",
    "fill-opacity",
    "stroke",
    "stroke-width",
    "stroke-linecap",
    "stroke-linejoin",
    "stroke-opacity",
    "opacity",
];
/// Silhouette : une seule teinte, fixée par la carte ; les évidements prennent le fond (ADR 0023).
const COULEURS_SILHOUETTE: [&str; 2] = ["currentColor", "none"];
const EVIDEMENT: &str = "var(--art-bg)";
const OPACITES: [&str; 3] = ["opacity", "fill-opacity", "stroke-opacity"];
const INTERDITS: [&str; 7] = [
    "javascript:",
    "data:",
    "expression(",
    "@import",
    "\\",
    "<",
    "&",
];

static URL: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"url\(").expect("motif valide"));
static URL_LOCALE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"url\(#[A-Za-z][\w-]*\)").expect("motif valide"));

#[must_use]
pub fn taille_max(type_media: &str) -> usize {
    if type_media == "silhouette" {
        32 * 1024
    } else {
        256 * 1024
    }
}

fn verifier_valeur(nom: &str, valeur: &str) -> Option<String> {
    let bas = valeur.to_lowercase();
    if INTERDITS.iter().any(|m| bas.contains(m)) {
        return Some(format!("valeur interdite dans « {nom} »"));
    }
    if URL.find_iter(valeur).count() != URL_LOCALE.find_iter(valeur).count() {
        return Some(format!(
            "« {nom} » ne peut citer qu’une référence locale url(#…)"
        ));
    }
    None
}

fn couleur_silhouette(propriete: &str, valeur: &str, dans_style: bool) -> Option<String> {
    if OPACITES.contains(&propriete) {
        return Some(format!(
            "« {propriete} » : une silhouette n’a qu’une teinte, sans opacité propre"
        ));
    }
    if propriete == "fill" || propriete == "stroke" {
        let mut admises: BTreeSet<&str> = COULEURS_SILHOUETTE.into_iter().collect();
        if dans_style {
            admises.insert(EVIDEMENT);
        }
        if !admises.contains(valeur) {
            let liste: Vec<&str> = admises.into_iter().collect();
            return Some(format!(
                "« {propriete}: {valeur} » : une silhouette n’admet que {}",
                liste.join(", ")
            ));
        }
    }
    None
}

/// Texte d'une suite de nœuds texte, jusqu'au prochain élément.
fn texte_jusqu_a_element<'a>(mut noeud: Option<Node<'a, 'a>>) -> String {
    let mut texte = String::new();
    while let Some(n) = noeud {
        if n.is_element() {
            break;
        }
        texte.push_str(n.text().unwrap_or_default());
        noeud = n.next_sibling();
    }
    texte
}

/// Problèmes du fichier au regard du profil, dans l'ordre de lecture ; liste vide si le fichier est admis.
#[must_use]
pub fn verifier_svg(contenu: &[u8], type_media: &str) -> Vec<String> {
    let maximum = taille_max(type_media);
    if contenu.len() > maximum {
        return vec![format!(
            "fichier de {} octets, au-delà de {maximum}",
            contenu.len()
        )];
    }
    let Ok(texte) = std::str::from_utf8(contenu) else {
        return vec!["le fichier n’est pas en UTF-8".into()];
    };
    // Pas de DTD, donc aucune entité à développer : le lecteur ne lit jamais de ressource externe.
    for (motif, message) in [
        ("<!DOCTYPE", "DOCTYPE interdit"),
        ("<!ENTITY", "entité interdite"),
        ("<!--", "commentaire interdit"),
        ("<![CDATA[", "CDATA interdit"),
    ] {
        if texte.contains(motif) {
            return vec![message.into()];
        }
    }
    let debut = texte.trim_start();
    let apres: String = debut.chars().skip(2).collect();
    if apres.contains("<?") || (debut.starts_with("<?") && !debut.starts_with("<?xml ")) {
        return vec!["instruction de traitement interdite".into()];
    }
    let document = match Document::parse(texte) {
        Ok(d) => d,
        Err(e) => return vec![format!("XML illisible : {e}")],
    };

    let mut problemes = Vec::new();
    let racine = document.root_element();
    if racine.tag_name().namespace() != Some(SVG_NS) || racine.tag_name().name() != "svg" {
        return vec!["la racine doit être <svg xmlns=\"http://www.w3.org/2000/svg\">".into()];
    }
    match racine.attribute("viewBox").filter(|v| !v.is_empty()) {
        None => problemes.push("viewBox absent sur <svg>".into()),
        Some(v)
            if type_media == "silhouette"
                && v.split_whitespace().collect::<Vec<_>>().join(" ") != VIEWBOX_SILHOUETTE =>
        {
            problemes.push(format!(
                "viewBox « {v} » : une silhouette est en « {VIEWBOX_SILHOUETTE} »"
            ));
        }
        Some(_) => {}
    }

    for element in racine.descendants().filter(Node::is_element) {
        let nom = element.tag_name().name();
        if element.tag_name().namespace() != Some(SVG_NS) || !ELEMENTS.contains(&nom) {
            problemes.push(format!("élément <{nom}> interdit"));
            continue;
        }
        let interieur = texte_jusqu_a_element(element.first_child());
        let suite = texte_jusqu_a_element(element.next_sibling());
        if !interieur.trim().is_empty() || !suite.trim().is_empty() {
            problemes.push(format!("texte interdit dans <{nom}>"));
        }
        for attribut in element.attributes() {
            let (nom_attr, valeur) = (attribut.name(), attribut.value());
            if attribut.namespace().is_some() || !ATTRIBUTS.contains(&nom_attr) {
                problemes.push(format!("attribut « {nom_attr} » interdit sur <{nom}>"));
                continue;
            }
            if let Some(erreur) = verifier_valeur(nom_attr, valeur) {
                problemes.push(erreur);
                continue;
            }
            if nom_attr == "style" {
                for declaration in valeur.split(';').map(str::trim).filter(|d| !d.is_empty()) {
                    match declaration
                        .split_once(':')
                        .map(|(p, v)| (p.trim(), v.trim()))
                    {
                        Some((propriete, val)) if PROPRIETES_STYLE.contains(&propriete) => {
                            if type_media == "silhouette"
                                && let Some(erreur) = couleur_silhouette(propriete, val, true)
                            {
                                problemes.push(erreur);
                            }
                        }
                        _ => problemes
                            .push(format!("déclaration de style « {declaration} » interdite")),
                    }
                }
            } else if type_media == "silhouette"
                && let Some(erreur) = couleur_silhouette(nom_attr, valeur.trim(), false)
            {
                problemes.push(erreur);
            }
        }
    }
    problemes
}

#[must_use]
pub fn chemin(catalogue: &Catalogue, media: &Value) -> PathBuf {
    catalogue
        .racine
        .join(DOSSIER)
        .join(texte(&media["fichier"]))
}

/// (nature, identifiant) de l'objet illustré : unité, langue ou fascicule.
#[must_use]
pub fn cible(media: &Value) -> (&'static str, &str) {
    for (cle, nature) in [
        ("unite_id", "unite"),
        ("langue_id", "langue"),
        ("fascicule_id", "fascicule"),
    ] {
        if a_cle(media, cle) {
            return (nature, texte(&media[cle]));
        }
    }
    ("", "")
}

/// Clé d'un média : (type, nature, identifiant).
pub type CleMedia = (String, String, String);

/// (type, nature, identifiant) → média ; le premier par identifiant si plusieurs se disputent la cible.
#[must_use]
pub fn par_cible(catalogue: &Catalogue) -> HashMap<CleMedia, &Value> {
    let mut tries: Vec<(&String, &Value)> = catalogue.medias.iter().collect();
    tries.sort_by(|a, b| a.0.cmp(b.0));
    let mut index = HashMap::new();
    for (_, media) in tries {
        let (nature, identifiant) = cible(media);
        index
            .entry((
                texte(&media["type"]).to_owned(),
                nature.to_owned(),
                identifiant.to_owned(),
            ))
            .or_insert(media);
    }
    index
}

#[must_use]
pub fn retenu(objet: &Value) -> bool {
    objet["fait"]["confiance"] != "non_retenue"
}

/// Langues des cartes d'un fascicule et de tous les étymons de leur chaîne : chacune a sa silhouette (ADR 0023).
#[must_use]
pub fn langues_citees(catalogue: &Catalogue, uids: &[String]) -> Vec<String> {
    let mut vus: BTreeSet<&str> = uids.iter().map(String::as_str).collect();
    let mut pile: Vec<&str> = vus.iter().copied().collect();
    while let Some(cible) = pile.pop() {
        for relation in catalogue.relations.values().filter(|r| retenu(r)) {
            let source = texte(&relation["unite_source_id"]);
            if texte(&relation["unite_cible_id"]) == cible
                && catalogue.unites.contains_key(source)
                && vus.insert(source)
            {
                pile.push(source);
            }
        }
    }
    vus.into_iter()
        .filter_map(|u| catalogue.unites.get(u))
        .map(|u| texte(&u["langue_id"]).to_owned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
