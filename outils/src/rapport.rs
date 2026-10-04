//! Diagnostics et rapport de validation.

#[cfg(test)]
use std::collections::BTreeSet;

use serde_json::{Value, json};

use crate::json::{LISIBLE, ecrire};

pub const ERREUR: &str = "erreur";
pub const AVERTISSEMENT: &str = "avertissement";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub niveau: &'static str,
    pub regle: String,
    pub fichier: Option<String>,
    pub objet: Option<String>,
    pub message: String,
}

#[derive(Debug, Default)]
pub struct Rapport {
    pub diagnostics: Vec<Diagnostic>,
    pub fichiers_lus: usize,
}

fn pluriel(n: usize, mot: &str) -> String {
    format!("{n} {mot}{}", if n > 1 { "s" } else { "" })
}

impl Rapport {
    pub fn erreur(
        &mut self,
        regle: &str,
        message: impl Into<String>,
        fichier: Option<&str>,
        objet: Option<&str>,
    ) {
        self.ajouter(ERREUR, regle, message.into(), fichier, objet);
    }

    pub fn avertissement(
        &mut self,
        regle: &str,
        message: impl Into<String>,
        fichier: Option<&str>,
        objet: Option<&str>,
    ) {
        self.ajouter(AVERTISSEMENT, regle, message.into(), fichier, objet);
    }

    fn ajouter(
        &mut self,
        niveau: &'static str,
        regle: &str,
        message: String,
        fichier: Option<&str>,
        objet: Option<&str>,
    ) {
        self.diagnostics.push(Diagnostic {
            niveau,
            regle: regle.to_owned(),
            fichier: fichier.map(str::to_owned),
            objet: objet.map(str::to_owned),
            message,
        });
    }

    #[must_use]
    pub fn erreurs(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.niveau == ERREUR)
            .collect()
    }

    #[must_use]
    pub fn avertissements(&self) -> Vec<&Diagnostic> {
        self.diagnostics
            .iter()
            .filter(|d| d.niveau == AVERTISSEMENT)
            .collect()
    }

    /// Règles signalées, à un niveau donné ou à tous.
    #[cfg(test)]
    #[must_use]
    pub fn regles(&self, niveau: Option<&str>) -> BTreeSet<String> {
        self.diagnostics
            .iter()
            .filter(|d| niveau.is_none_or(|n| d.niveau == n))
            .map(|d| d.regle.clone())
            .collect()
    }

    #[must_use]
    pub fn trier(&self) -> Vec<&Diagnostic> {
        let mut tries: Vec<&Diagnostic> = self.diagnostics.iter().collect();
        tries.sort_by(|a, b| {
            let cle = |d: &Diagnostic| {
                (
                    d.niveau != ERREUR,
                    d.fichier.clone().unwrap_or_default(),
                    d.objet.clone().unwrap_or_default(),
                    d.regle.clone(),
                    d.message.clone(),
                )
            };
            cle(a).cmp(&cle(b))
        });
        tries
    }

    #[must_use]
    pub fn en_texte(&self) -> String {
        let mut lignes = Vec::new();
        for d in self.trier() {
            let mut lieu = d
                .fichier
                .clone()
                .filter(|f| !f.is_empty())
                .unwrap_or_else(|| "—".into());
            if let Some(objet) = d.objet.as_deref().filter(|o| !o.is_empty()) {
                lieu = format!("{lieu} ({objet})");
            }
            lignes.push(format!(
                "{:<13} {lieu}  [{}]  {}",
                d.niveau, d.regle, d.message
            ));
        }
        let n = self.fichiers_lus;
        lignes.push(format!(
            "{} lu{}, {}, {}.",
            pluriel(n, "fichier"),
            if n > 1 { "s" } else { "" },
            pluriel(self.erreurs().len(), "erreur"),
            pluriel(self.avertissements().len(), "avertissement"),
        ));
        lignes.join("\n")
    }

    #[must_use]
    pub fn en_json(&self) -> String {
        let diagnostics: Vec<Value> = self
            .trier()
            .into_iter()
            .map(|d| json!({"niveau": d.niveau, "regle": d.regle, "fichier": d.fichier, "objet": d.objet, "message": d.message}))
            .collect();
        ecrire(
            &json!({
                "fichiers_lus": self.fichiers_lus,
                "erreurs": self.erreurs().len(),
                "avertissements": self.avertissements().len(),
                "diagnostics": diagnostics,
            }),
            LISIBLE,
        )
    }
}
