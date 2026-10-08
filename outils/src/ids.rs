//! Identifiants opaques et stables (ADR 0005, modèle de données, principe 1).

use std::collections::HashSet;
use std::sync::LazyLock;

use regex::Regex;

/// Alphabet base32 de Crockford en minuscules : ni i, l, o, u, pour éviter les confusions.
pub const ALPHABET: &[u8; 32] = b"0123456789abcdefghjkmnpqrstvwxyz";
pub const LONGUEUR: usize = 8;

pub const PREFIXES: [(&str, &str); 17] = [
    ("lng", "langue"),
    ("ecr", "système d’écriture"),
    ("cnv", "convention de translittération"),
    ("src", "source"),
    ("unt", "unité lexicale"),
    ("frm", "forme"),
    ("sns", "sens"),
    ("cmp", "composition"),
    ("trf", "transformation"),
    ("rel", "relation étymologique"),
    ("brq", "brique"),
    ("rct", "recette"),
    ("fsc", "fascicule"),
    ("exc", "exclusion"),
    ("med", "média"),
    ("jal", "jalon"),
    ("tpl", "trompe-l’œil"),
];

// `\n?` : le `$` de Python admet un saut de ligne final.
pub static MOTIF: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([a-z]{3})_[0-9a-z]{8}\n?$").expect("motif valide"));

#[must_use]
pub fn prefixe(identifiant: &str) -> Option<&str> {
    MOTIF
        .captures(identifiant)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
}

pub fn nouvel_id(prefixe_type: &str, existants: &HashSet<String>) -> Result<String, String> {
    if !PREFIXES.iter().any(|(p, _)| *p == prefixe_type) {
        return Err(format!("Préfixe inconnu : {prefixe_type}"));
    }
    loop {
        let mut octets = [0u8; LONGUEUR];
        getrandom::fill(&mut octets).map_err(|e| e.to_string())?;
        // 256 est un multiple de 32 : le tirage reste uniforme.
        let suffixe: String = octets
            .iter()
            .map(|o| char::from(ALPHABET[usize::from(o % 32)]))
            .collect();
        let candidat = format!("{prefixe_type}_{suffixe}");
        if !existants.contains(&candidat) {
            return Ok(candidat);
        }
    }
}
