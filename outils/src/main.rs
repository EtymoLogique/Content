//! Outillage éditorial du catalogue ÉtymoLogique : « valider », « compiler », « visualiser »,
//! « nouvel-id » et « pre-publication ».

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use clap::builder::PossibleValuesParser;
use clap::{Parser, Subcommand};

mod candidats;
mod chargement;
mod collections;
mod compilation;
mod controles;
mod homographes;
mod ids;
mod json;
mod medias;
mod prepublication;
mod rapport;
#[cfg(test)]
mod tests;
mod visualisation;

use chargement::{Catalogue, charger};
use rapport::Rapport;

#[derive(Parser)]
#[command(
    name = "etymo-contenu",
    about = "Outillage éditorial du catalogue ÉtymoLogique.",
    disable_version_flag = true
)]
struct Cli {
    #[command(subcommand)]
    commande: Commande,
}

#[derive(Subcommand)]
enum Commande {
    /// Valider le dépôt : schémas et contraintes d’intégrité.
    Valider {
        /// Racine du dépôt (par défaut : dossier courant).
        #[arg(long, default_value = ".")]
        racine: PathBuf,
        /// Appliquer aussi les exigences de publication (ADR 0005, ADR 0014).
        #[arg(long)]
        publication: bool,
        /// Rapport au format JSON.
        #[arg(long)]
        json: bool,
        /// Traiter les avertissements comme des erreurs.
        #[arg(long)]
        strict: bool,
    },
    /// Compiler l’artefact publié : ressources et manifestes en hash de contenu, calendrier des parutions.
    Compiler {
        /// Racine du dépôt (par défaut : dossier courant).
        #[arg(long, default_value = ".")]
        racine: PathBuf,
        /// Dossier de sortie, vidé avant compilation (par défaut : artefact).
        #[arg(long, default_value = "artefact")]
        sortie: PathBuf,
        /// Identifiant de la publication, par exemple le commit.
        #[arg(long = "version", id = "version_publication", default_value = "locale")]
        version: String,
        /// Hors-prod seulement : inclure les fascicules de test et les faits non relus. L’artefact est marqué « essai ».
        #[arg(long)]
        essai: bool,
    },
    /// Écrire une page HTML locale qui montre tout le catalogue : graphe, cartes, silhouettes.
    Visualiser {
        /// Racine du dépôt (par défaut : dossier courant).
        #[arg(long, default_value = ".")]
        racine: PathBuf,
        /// Fichier écrit (par défaut : rapports/catalogue.html, ignoré par Git).
        #[arg(long, default_value = "rapports/catalogue.html")]
        sortie: PathBuf,
    },
    /// Lister les paires de briques qui s’écrivent comme un mot du catalogue sans en venir : pièges probables (ADR 0052).
    TrompeLOeilCandidats {
        /// Racine du dépôt (par défaut : dossier courant).
        #[arg(long, default_value = ".")]
        racine: PathBuf,
    },
    /// Générer des identifiants opaques inédits.
    NouvelId {
        /// Type d’objet.
        #[arg(value_parser = PossibleValuesParser::new(prefixes()))]
        prefixe: String,
        /// Nombre d’identifiants.
        #[arg(short, default_value_t = 1)]
        n: usize,
        /// Racine du dépôt (par défaut : dossier courant).
        #[arg(long, default_value = ".")]
        racine: PathBuf,
    },
    /// Rapport de pré-publication d’un fascicule : contrôles et calendrier.
    PrePublication {
        /// Numéro du fascicule ou identifiant fsc_….
        fascicule: String,
        /// Racine du dépôt (par défaut : dossier courant).
        #[arg(long, default_value = ".")]
        racine: PathBuf,
        /// Date de parution à évaluer, en ISO 8601 UTC, par exemple 2026-12-01T00:00:00Z.
        #[arg(long)]
        date: Option<String>,
        /// Écrire --date dans le fichier du fascicule, puis refaire le rapport.
        #[arg(long)]
        ecrire: bool,
        #[arg(long, hide = true)]
        maintenant: Option<String>,
        /// Rapport au format JSON.
        #[arg(long)]
        json: bool,
        /// Lister tous les points, pas seulement les premiers de chaque règle.
        #[arg(long)]
        complet: bool,
    },
}

fn prefixes() -> Vec<&'static str> {
    let mut prefixes: Vec<&str> = ids::PREFIXES.iter().map(|(p, _)| *p).collect();
    prefixes.sort_unstable();
    prefixes
}

fn valider(racine: &Path, publication: bool) -> Result<(Catalogue, Rapport), String> {
    let mut rapport = Rapport::default();
    let catalogue = charger(racine, &mut rapport)?;
    controles::controler(&catalogue, &mut rapport, publication);
    Ok((catalogue, rapport))
}

fn executer(commande: Commande) -> Result<u8, String> {
    match commande {
        Commande::Valider {
            racine,
            publication,
            json,
            strict,
        } => {
            let (_, rapport) = valider(&racine, publication)?;
            println!(
                "{}",
                if json {
                    rapport.en_json()
                } else {
                    rapport.en_texte()
                }
            );
            let echec =
                !rapport.erreurs().is_empty() || (strict && !rapport.avertissements().is_empty());
            Ok(u8::from(echec))
        }
        Commande::Compiler {
            racine,
            sortie,
            version,
            essai,
        } => {
            // Hors essai, l'artefact ne part que si les exigences de publication passent.
            let (catalogue, rapport) = valider(&racine, !essai)?;
            if !rapport.erreurs().is_empty() {
                println!("{}", rapport.en_texte());
                eprintln!("Compilation refusée : le dépôt ne passe pas la validation.");
                return Ok(1);
            }
            let artefact = compilation::compiler(&catalogue, essai, &version)?;
            compilation::ecrire_artefact(&artefact, &sortie)
                .map_err(|e| format!("{} : {e}", sortie.display()))?;
            let problemes = compilation::verifier(&sortie);
            for probleme in &problemes {
                eprintln!("{probleme}");
            }
            if !problemes.is_empty() {
                return Ok(1);
            }
            let parutions = json::lire(
                &std::fs::read_to_string(sortie.join("parutions.json"))
                    .map_err(|e| e.to_string())?,
            )
            .map_err(|e| format!("parutions.json : {e:?}"))?;
            println!(
                "{} fichiers, {} fascicule(s), version {}, mode {} → {}",
                artefact.fichiers.len(),
                json::liste(&parutions["fascicules"]).len(),
                json::texte(&parutions["version"]),
                if essai { "essai" } else { "publication" },
                sortie.display()
            );
            Ok(0)
        }
        Commande::TrompeLOeilCandidats { racine } => {
            let mut rapport = Rapport::default();
            let catalogue = charger(&racine, &mut rapport)?;
            let candidats = candidats::chercher(&catalogue);
            for c in &candidats {
                println!(
                    "{} + {} s’écrit « {} » : unité {}",
                    c.premiere, c.seconde, c.forme, c.mot_id
                );
            }
            eprintln!(
                "{} paire(s) à examiner (aucune recette, exclusion ni trompe-l’œil ne la couvre).",
                candidats.len()
            );
            Ok(0)
        }
        Commande::Visualiser { racine, sortie } => {
            // La page s'écrit même sur un dépôt invalide : elle montre ses diagnostics.
            let (catalogue, rapport) = valider(&racine, false)?;
            visualisation::ecrire_page(&catalogue, &rapport, &sortie)
                .map_err(|e| format!("{} : {e}", sortie.display()))?;
            println!(
                "{} unités, {} erreur(s), {} avertissement(s) → {}",
                catalogue.unites.len(),
                rapport.erreurs().len(),
                rapport.avertissements().len(),
                sortie.display()
            );
            Ok(0)
        }
        Commande::NouvelId { prefixe, n, racine } => {
            let mut existants: HashSet<String> = charger(&racine, &mut Rapport::default())?
                .fichiers
                .into_keys()
                .collect();
            for _ in 0..n {
                let identifiant = ids::nouvel_id(&prefixe, &existants)?;
                println!("{identifiant}");
                existants.insert(identifiant);
            }
            Ok(0)
        }
        Commande::PrePublication {
            fascicule,
            racine,
            date,
            ecrire,
            maintenant,
            json,
            complet,
        } => {
            // 0 si rien ne bloque la publication, 1 sinon, 2 si l'appel est invalide.
            let options = prepublication::Options {
                racine: &racine,
                fascicule: &fascicule,
                date: date.as_deref(),
                ecrire,
                maintenant: maintenant.as_deref(),
            };
            match prepublication::executer(&options) {
                Err(e) => {
                    eprintln!("{e}");
                    Ok(2)
                }
                Ok(r) => {
                    println!(
                        "{}",
                        if json {
                            r.en_json()
                        } else {
                            r.en_texte(complet)
                        }
                    );
                    Ok(u8::from(r.bloquants() > 0))
                }
            }
        }
    }
}

fn main() -> ExitCode {
    match executer(Cli::parse().commande) {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("{e}");
            ExitCode::FAILURE
        }
    }
}
