//! Lecture stricte et écriture du JSON.
//!
//! La lecture refuse une clé en double ; l'écriture reproduit `json.dumps` de Python
//! (`ensure_ascii=False`), dont dépendent les empreintes de l'artefact déjà publié.

use std::fmt::{self, Write as _};

use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};

/// Erreur de lecture : clé en double ou JSON illisible.
#[derive(Debug, PartialEq, Eq)]
pub enum ErreurLecture {
    CleDupliquee(String),
    Syntaxe(String),
}

const MARQUE: &str = "\u{0}cle_dupliquee:";

struct SansDoublon(Value);

impl<'de> Deserialize<'de> for SansDoublon {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        d.deserialize_any(Visiteur).map(SansDoublon)
    }
}

struct Visiteur;

impl<'de> Visitor<'de> for Visiteur {
    type Value = Value;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("une valeur JSON")
    }
    fn visit_bool<E>(self, v: bool) -> Result<Value, E> {
        Ok(Value::Bool(v))
    }
    fn visit_i64<E>(self, v: i64) -> Result<Value, E> {
        Ok(Value::from(v))
    }
    fn visit_u64<E>(self, v: u64) -> Result<Value, E> {
        Ok(Value::from(v))
    }
    fn visit_f64<E>(self, v: f64) -> Result<Value, E> {
        Ok(Number::from_f64(v).map_or(Value::Null, Value::Number))
    }
    fn visit_str<E>(self, v: &str) -> Result<Value, E> {
        Ok(Value::String(v.to_owned()))
    }
    fn visit_string<E>(self, v: String) -> Result<Value, E> {
        Ok(Value::String(v))
    }
    fn visit_unit<E>(self) -> Result<Value, E> {
        Ok(Value::Null)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Value, A::Error> {
        let mut liste = Vec::new();
        while let Some(SansDoublon(v)) = seq.next_element()? {
            liste.push(v);
        }
        Ok(Value::Array(liste))
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Value, A::Error> {
        let mut objet = Map::new();
        while let Some(cle) = map.next_key::<String>()? {
            if objet.contains_key(&cle) {
                return Err(de::Error::custom(format!("{MARQUE}{cle}")));
            }
            let SansDoublon(v) = map.next_value()?;
            objet.insert(cle, v);
        }
        Ok(Value::Object(objet))
    }
}

/// Lit un document JSON en refusant toute clé en double, l'ordre des clés conservé.
pub fn lire(texte: &str) -> Result<Value, ErreurLecture> {
    serde_json::from_str::<SansDoublon>(texte)
        .map(|v| v.0)
        .map_err(|e| {
            let message = e.to_string();
            match message.strip_prefix(MARQUE) {
                Some(reste) => ErreurLecture::CleDupliquee(
                    reste
                        .rsplit_once(" at line ")
                        .map_or(reste, |(cle, _)| cle)
                        .to_owned(),
                ),
                None => ErreurLecture::Syntaxe(message),
            }
        })
}

/// Valeur « vraie » au sens de Python : ni null, ni faux, ni zéro, ni vide.
#[must_use]
pub fn vrai(v: &Value) -> bool {
    match v {
        Value::Null => false,
        Value::Bool(b) => *b,
        Value::Number(n) => n.as_f64().is_some_and(|f| f != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// Texte d'une valeur, chaîne vide sinon.
#[must_use]
pub fn texte(v: &Value) -> &str {
    v.as_str().unwrap_or_default()
}

/// Éléments d'une liste, aucun sinon.
#[must_use]
pub fn liste(v: &Value) -> &[Value] {
    v.as_array().map_or(&[], Vec::as_slice)
}

/// Entier d'une valeur, zéro sinon.
#[must_use]
#[allow(clippy::cast_possible_truncation)]
pub fn entier(v: &Value) -> i64 {
    v.as_i64()
        .or_else(|| v.as_f64().map(|f| f as i64)) // int() de Python : troncature voulue
        .unwrap_or_default()
}

/// La clé est-elle présente, même à null ?
#[must_use]
pub fn a_cle(v: &Value, cle: &str) -> bool {
    v.as_object().is_some_and(|o| o.contains_key(cle))
}

// ------------------------------------------------------------------ écriture

/// Options d'écriture, celles de `json.dumps`.
#[derive(Clone, Copy, Default)]
pub struct Format {
    /// `sort_keys=True`.
    pub trie: bool,
    /// `indent=2` ; sinon séparateurs compacts `(",", ":")`.
    pub indente: bool,
}

/// JSON canonique de l'artefact : clés triées, sans espace.
pub const CANONIQUE: Format = Format {
    trie: true,
    indente: false,
};
/// JSON compact, clés dans l'ordre d'insertion.
pub const COMPACT: Format = Format {
    trie: false,
    indente: false,
};
/// JSON lisible, indenté de deux espaces.
pub const LISIBLE: Format = Format {
    trie: false,
    indente: true,
};

#[must_use]
pub fn ecrire(v: &Value, format: Format) -> String {
    let mut sortie = String::new();
    valeur(&mut sortie, v, format, 0);
    sortie
}

fn retour(sortie: &mut String, format: Format, niveau: usize) {
    if format.indente {
        sortie.push('\n');
        sortie.extend(std::iter::repeat_n(' ', 2 * niveau));
    }
}

fn valeur(sortie: &mut String, v: &Value, format: Format, niveau: usize) {
    match v {
        Value::Null => sortie.push_str("null"),
        Value::Bool(b) => sortie.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => match n.as_f64() {
            Some(f) if n.is_f64() => sortie.push_str(&flottant(f)),
            _ => sortie.push_str(&n.to_string()),
        },
        Value::String(s) => chaine(sortie, s),
        Value::Array(a) if a.is_empty() => sortie.push_str("[]"),
        Value::Array(a) => {
            sortie.push('[');
            for (i, x) in a.iter().enumerate() {
                if i > 0 {
                    sortie.push(',');
                }
                retour(sortie, format, niveau + 1);
                valeur(sortie, x, format, niveau + 1);
            }
            retour(sortie, format, niveau);
            sortie.push(']');
        }
        Value::Object(o) if o.is_empty() => sortie.push_str("{}"),
        Value::Object(o) => {
            let mut paires: Vec<(&String, &Value)> = o.iter().collect();
            if format.trie {
                paires.sort_by(|a, b| a.0.cmp(b.0));
            }
            sortie.push('{');
            for (i, (cle, x)) in paires.into_iter().enumerate() {
                if i > 0 {
                    sortie.push(',');
                }
                retour(sortie, format, niveau + 1);
                chaine(sortie, cle);
                sortie.push_str(if format.indente { ": " } else { ":" });
                valeur(sortie, x, format, niveau + 1);
            }
            retour(sortie, format, niveau);
            sortie.push('}');
        }
    }
}

fn chaine(sortie: &mut String, s: &str) {
    sortie.push('"');
    for c in s.chars() {
        match c {
            '"' => sortie.push_str("\\\""),
            '\\' => sortie.push_str("\\\\"),
            '\n' => sortie.push_str("\\n"),
            '\r' => sortie.push_str("\\r"),
            '\t' => sortie.push_str("\\t"),
            '\u{8}' => sortie.push_str("\\b"),
            '\u{c}' => sortie.push_str("\\f"),
            c if c < ' ' => {
                let _ = write!(sortie, "\\u{:04x}", c as u32);
            }
            c => sortie.push(c),
        }
    }
    sortie.push('"');
}

/// `repr(float)` de Python : notation fixe de 1e-4 à 1e16, scientifique au-delà.
fn flottant(f: f64) -> String {
    if f.is_nan() {
        return "NaN".into();
    }
    if f.is_infinite() {
        return if f > 0.0 { "Infinity" } else { "-Infinity" }.into();
    }
    let sci = format!("{f:e}");
    let (mantisse, exposant) = sci.split_once('e').unwrap_or((&sci, "0"));
    let exposant: i32 = exposant.parse().unwrap_or_default();
    let (signe, mantisse) = mantisse
        .strip_prefix('-')
        .map_or(("", mantisse), |m| ("-", m));
    if (-4..16).contains(&exposant) {
        let chiffres: String = mantisse.chars().filter(char::is_ascii_digit).collect();
        let (entiere, fraction) = if exposant >= 0 {
            let n = usize::try_from(exposant).unwrap_or_default() + 1;
            if chiffres.len() <= n {
                (format!("{chiffres:0<n$}"), String::new())
            } else {
                (chiffres[..n].to_owned(), chiffres[n..].to_owned())
            }
        } else {
            let zeros = usize::try_from(-exposant - 1).unwrap_or_default();
            ("0".to_owned(), format!("{}{chiffres}", "0".repeat(zeros)))
        };
        let fraction = if fraction.is_empty() {
            "0".to_owned()
        } else {
            fraction
        };
        format!("{signe}{entiere}.{fraction}")
    } else {
        let signe_exposant = if exposant < 0 { '-' } else { '+' };
        format!("{signe}{mantisse}e{signe_exposant}{:02}", exposant.abs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flottants_comme_python() {
        for (f, attendu) in [
            (1.0, "1.0"),
            (0.1, "0.1"),
            (100.0, "100.0"),
            (1e15, "1000000000000000.0"),
            (1e16, "1e+16"),
            (1.5e-7, "1.5e-07"),
            (0.0001, "0.0001"),
            (-2.5, "-2.5"),
            (123.456, "123.456"),
            (-0.0, "-0.0"),
        ] {
            assert_eq!(flottant(f), attendu);
        }
    }

    #[test]
    fn cle_en_double() {
        assert_eq!(
            lire(r#"{"a": {"b": 1, "b": 2}}"#),
            Err(ErreurLecture::CleDupliquee("b".into()))
        );
        assert!(matches!(
            lire("{ pas du json"),
            Err(ErreurLecture::Syntaxe(_))
        ));
    }

    #[test]
    fn ecriture_comme_python() {
        let v = lire(r#"{"b": [1, {"z": "é\u0001\"", "a": []}], "a": {}}"#).unwrap();
        assert_eq!(
            ecrire(&v, CANONIQUE),
            r#"{"a":{},"b":[1,{"a":[],"z":"é\u0001\""}]}"#
        );
        assert_eq!(
            ecrire(&v, LISIBLE),
            "{\n  \"b\": [\n    1,\n    {\n      \"z\": \"é\\u0001\\\"\",\n      \"a\": []\n    }\n  ],\n  \"a\": {}\n}"
        );
    }
}
