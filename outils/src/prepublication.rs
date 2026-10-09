//! Rapport de pré-publication d'un fascicule : tous les contrôles, puis le calendrier.
//!
//! Le rapport ne modifie rien, sauf avec `--ecrire`, qui écrit la date dans le fichier du fascicule.
//! Chaque règle montre ses cinq premiers points ; `--complet` les liste tous, `--json` donne le rapport machine.
//! Un fascicule de test est contrôlé comme s'il était publié, et signalé comme bloquant.

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::sync::LazyLock;

use chrono::{DateTime, Datelike, FixedOffset, TimeDelta, TimeZone, Utc};
use regex::Regex;
use serde_json::{Map, Value, json};

use crate::chargement::{Catalogue, charger};
use crate::controles::{FASCICULES_DE_LANCEMENT, controler};
use crate::json::{LISIBLE, ecrire, liste, texte, vrai};
use crate::medias;
use crate::rapport::{ERREUR, Rapport};

/// « environ tous les 30 jours » (ADR 0014, 0022), février compris.
const ECART_MINIMUM: TimeDelta = TimeDelta::days(28);
const ECART_MAXIMUM: TimeDelta = TimeDelta::days(45);
static MOTIF_DATE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}(:\d{2})?(Z|[+-]\d{2}:\d{2})$")
        .expect("motif valide")
});

pub const RUBRIQUES: [(&str, &str); 8] = [
    ("integrite", "Intégrité du fascicule"),
    ("relecture", "Relecture et sources"),
    ("medias", "Médias"),
    ("fascicule", "Contenu du fascicule"),
    ("exclusions", "Exclusions"),
    ("trompes_l_oeil", "Trompe-l’œil"),
    ("calendrier", "Calendrier"),
    (
        "depot",
        "Reste du dépôt (bloque aussi la compilation en prod)",
    ),
];
pub const BLOQUANT: &str = "bloquant";
pub const A_VOIR: &str = "à voir";

#[derive(Clone, Debug)]
pub struct Point {
    pub niveau: &'static str,
    pub regle: String,
    pub message: String,
    pub lieu: Option<String>,
}

impl Point {
    fn new(
        niveau: &'static str,
        regle: &str,
        message: impl Into<String>,
        lieu: Option<String>,
    ) -> Self {
        Self {
            niveau,
            regle: regle.to_owned(),
            message: message.into(),
            lieu,
        }
    }
}

#[derive(Debug)]
pub struct RapportPrePublication {
    pub fascicule_id: String,
    pub numero: Value,
    pub parution: String,
    pub date_proposee: String,
    /// Points par rubrique, dans l'ordre de `RUBRIQUES`.
    pub rubriques: Vec<Vec<Point>>,
}

impl RapportPrePublication {
    pub fn ajouter(&mut self, rubrique: &str, point: Point) {
        let i = RUBRIQUES
            .iter()
            .position(|(c, _)| *c == rubrique)
            .expect("rubrique connue");
        self.rubriques[i].push(point);
    }

    fn compter(&self, niveau: &str) -> usize {
        self.rubriques
            .iter()
            .flatten()
            .filter(|p| p.niveau == niveau)
            .count()
    }

    #[must_use]
    pub fn bloquants(&self) -> usize {
        self.compter(BLOQUANT)
    }

    #[must_use]
    pub fn a_voir(&self) -> usize {
        self.compter(A_VOIR)
    }

    #[cfg(test)]
    #[must_use]
    pub fn points(&self, rubrique: &str) -> &[Point] {
        RUBRIQUES
            .iter()
            .position(|(c, _)| *c == rubrique)
            .map_or(&[], |i| &self.rubriques[i])
    }

    #[must_use]
    pub fn en_texte(&self, complet: bool) -> String {
        const EXEMPLES: usize = 5;
        let mut lignes = vec![
            format!(
                "Fascicule {} ({}) — parution actuelle : {}",
                self.numero, self.fascicule_id, self.parution
            ),
            String::new(),
        ];
        for ((_, titre), points) in RUBRIQUES.iter().zip(&self.rubriques) {
            let etat = if points.is_empty() {
                "OK"
            } else if points.iter().any(|p| p.niveau == BLOQUANT) {
                "BLOQUANT"
            } else {
                "À VOIR"
            };
            lignes.push(format!("## {titre} : {etat}"));
            let mut tries: Vec<&Point> = points.iter().collect();
            tries.sort_by(|a, b| {
                let cle = |p: &Point| {
                    (
                        p.niveau != BLOQUANT,
                        p.regle.clone(),
                        p.lieu.clone().unwrap_or_default(),
                    )
                };
                cle(a).cmp(&cle(b))
            });
            let mut par_regle: Vec<((&str, &str), Vec<&Point>)> = Vec::new();
            for p in tries {
                match par_regle
                    .iter_mut()
                    .find(|(c, _)| *c == (p.niveau, p.regle.as_str()))
                {
                    Some((_, groupe)) => groupe.push(p),
                    None => par_regle.push(((p.niveau, &p.regle), vec![p])),
                }
            }
            for ((niveau, regle), groupe) in par_regle {
                if groupe.len() > 1 {
                    lignes.push(format!("  [{niveau}] [{regle}] × {}", groupe.len()));
                }
                let montres = if complet {
                    groupe.len()
                } else {
                    groupe.len().min(EXEMPLES)
                };
                for p in &groupe[..montres] {
                    let lieu = p
                        .lieu
                        .as_deref()
                        .filter(|l| !l.is_empty())
                        .map(|l| format!("{l} : "))
                        .unwrap_or_default();
                    let prefixe = if groupe.len() > 1 {
                        "    - ".to_owned()
                    } else {
                        format!("  [{niveau}] [{regle}] ")
                    };
                    lignes.push(format!("{prefixe}{lieu}{}", p.message));
                }
                if montres < groupe.len() {
                    lignes.push(format!(
                        "    … et {} autre(s) : relancer avec --complet.",
                        groupe.len() - montres
                    ));
                }
            }
            lignes.push(String::new());
        }
        lignes.push(format!(
            "Date proposée : {} (à faire confirmer)",
            self.date_proposee
        ));
        lignes.push(format!(
            "{} point(s) bloquant(s), {} point(s) à voir.",
            self.bloquants(),
            self.a_voir()
        ));
        lignes.join("\n")
    }

    #[must_use]
    pub fn en_json(&self) -> String {
        let rubriques: Map<String, Value> = RUBRIQUES
            .iter()
            .zip(&self.rubriques)
            .map(|((cle, _), points)| {
                let points = points.iter().map(|p| json!({"niveau": p.niveau, "regle": p.regle, "message": p.message, "lieu": p.lieu}));
                ((*cle).to_owned(), Value::Array(points.collect()))
            })
            .collect();
        ecrire(
            &json!({
                "fascicule_id": self.fascicule_id,
                "numero": self.numero,
                "parution": self.parution,
                "date_proposee": self.date_proposee,
                "bloquants": self.bloquants(),
                "a_voir": self.a_voir(),
                "rubriques": rubriques,
            }),
            LISIBLE,
        )
    }
}

// ------------------------------------------------------------------- dates

/// Analyse une date ISO 8601 avec fuseau, au format des fascicules.
pub fn lire_date(texte: &str) -> Result<DateTime<FixedOffset>, String> {
    let Some(morceaux) = MOTIF_DATE.captures(texte) else {
        return Err(format!(
            "date « {texte} » : attendu AAAA-MM-JJTHH:MM[:SS] suivi de Z ou d’un décalage ±HH:MM"
        ));
    };
    let complet = if morceaux.get(1).is_some() {
        texte.to_owned()
    } else {
        format!("{}:00{}", &texte[..16], &texte[16..])
    };
    DateTime::parse_from_rfc3339(&complet).map_err(|e| format!("date « {texte} » : {e}"))
}

#[must_use]
pub fn en_utc<Tz: TimeZone>(date: &DateTime<Tz>) -> String {
    date.with_timezone(&Utc)
        .format("%Y-%m-%dT%H:%M:%SZ")
        .to_string()
}

fn premier_du_mois(annee: i32, mois: u32) -> DateTime<Utc> {
    let (annee, mois) = if mois > 12 {
        (annee + 1, 1)
    } else {
        (annee, mois)
    };
    Utc.with_ymd_and_hms(annee, mois, 1, 0, 0, 0)
        .single()
        .expect("date valide")
}

/// Premier 1ᵉʳ du mois à 00:00 UTC qui tombe à cette date ou après.
#[must_use]
pub fn premier_du_mois_des(date: DateTime<Utc>) -> DateTime<Utc> {
    let debut = premier_du_mois(date.year(), date.month());
    if debut >= date {
        debut
    } else {
        premier_du_mois(date.year(), date.month() + 1)
    }
}

/// Le 1ᵉʳ du mois à 00:00 UTC, au moins 28 jours après la parution précédente et après maintenant.
#[must_use]
pub fn proposer_date(
    precedente: Option<DateTime<Utc>>,
    maintenant: DateTime<Utc>,
) -> DateTime<Utc> {
    let mut plancher = maintenant + TimeDelta::seconds(1);
    if let Some(p) = precedente {
        plancher = plancher.max(p + ECART_MINIMUM);
    }
    premier_du_mois_des(plancher)
}

// --------------------------------------------------------------- périmètre

pub fn trouver_fascicule(catalogue: &Catalogue, designation: &str) -> Result<String, String> {
    if catalogue.fascicules.contains_key(designation) {
        return Ok(designation.to_owned());
    }
    if !designation.is_empty()
        && designation.bytes().all(|o| o.is_ascii_digit())
        && let Ok(numero) = designation.parse::<i64>()
    {
        let du_numero = |f: &Value| f["numero"].as_i64() == Some(numero);
        let mut candidats: Vec<&String> = catalogue
            .fascicules
            .iter()
            .filter(|(_, f)| du_numero(f) && !vrai(&f["test"]))
            .map(|(i, _)| i)
            .collect();
        if candidats.is_empty() {
            candidats = catalogue
                .fascicules
                .iter()
                .filter(|(_, f)| du_numero(f))
                .map(|(i, _)| i)
                .collect();
        }
        if let [seul] = candidats[..] {
            return Ok(seul.clone());
        }
    }
    Err(format!(
        "Fascicule « {designation} » introuvable : donnez son identifiant fsc_… ou son numéro."
    ))
}

/// Identifiants des objets qu'un fascicule publie ou dont il dépend directement.
#[must_use]
pub fn perimetre(catalogue: &Catalogue, fid: &str) -> HashSet<String> {
    let c = catalogue;
    let unites: HashSet<String> = liste(&c.fascicules[fid]["unites"])
        .iter()
        .map(|l| texte(&l["unite_id"]).to_owned())
        .collect();
    let dans = |ensemble: &HashSet<String>, v: &Value| ensemble.contains(texte(v));
    let compositions: HashSet<String> = c
        .compositions
        .iter()
        .filter(|(_, x)| dans(&unites, &x["mot_id"]))
        .map(|(i, _)| i.clone())
        .collect();
    let relations: HashSet<String> = c
        .relations
        .iter()
        .filter(|(_, r)| dans(&unites, &r["unite_cible_id"]))
        .map(|(i, _)| i.clone())
        .collect();
    let etymons: HashSet<String> = relations
        .iter()
        .map(|r| texte(&c.relations[r.as_str()]["unite_source_id"]).to_owned())
        .collect();
    let briques: HashSet<String> = c
        .briques
        .iter()
        .filter(|(_, b)| dans(&unites, &b["unite_id"]))
        .map(|(i, _)| i.clone())
        .collect();
    let recettes: HashSet<String> = c
        .recettes
        .iter()
        .filter(|(_, r)| dans(&compositions, &r["composition_id"]))
        .map(|(i, _)| i.clone())
        .collect();
    let mut triees: Vec<String> = unites.iter().cloned().collect();
    triees.sort();
    let mut cibles: HashSet<String> = medias::langues_citees(c, &triees).into_iter().collect();
    cibles.extend(unites.iter().cloned());
    cibles.insert(fid.to_owned());
    let medias_ids = c
        .medias
        .iter()
        .filter(|(_, m)| cibles.contains(medias::cible(m).1))
        .map(|(i, _)| i.clone());
    let exclusions = c
        .exclusions
        .iter()
        .filter(|(_, e)| e["fascicule_id"] == fid)
        .map(|(i, _)| i.clone());
    let trompes_l_oeil = c
        .trompes_l_oeil
        .iter()
        .filter(|(_, t)| t["fascicule_id"] == fid)
        .map(|(i, _)| i.clone());
    let mut ids: HashSet<String> = [fid.to_owned()].into();
    ids.extend(unites);
    ids.extend(etymons);
    ids.extend(compositions);
    ids.extend(relations);
    ids.extend(briques);
    ids.extend(recettes);
    ids.extend(medias_ids);
    ids.extend(exclusions);
    ids.extend(trompes_l_oeil);
    ids
}

fn rubrique(regle: &str) -> &'static str {
    if regle == "publication.exclusion" || regle.starts_with("exclusion.") {
        "exclusions"
    } else if regle == "publication.trompe_l_oeil" || regle.starts_with("trompe_l_oeil.") {
        "trompes_l_oeil"
    } else if regle.starts_with("publication.") {
        "relecture"
    } else if regle.starts_with("media.") {
        "medias"
    } else if regle.starts_with("fascicule.") {
        "fascicule"
    } else {
        "integrite"
    }
}

// ----------------------------------------------------------------- rapport

pub fn preparer(
    racine: &Path,
    designation: &str,
    date: Option<&str>,
    maintenant: Option<DateTime<Utc>>,
) -> Result<RapportPrePublication, String> {
    let maintenant = maintenant.unwrap_or_else(Utc::now);
    let mut diagnostics = Rapport::default();
    let mut catalogue = charger(racine, &mut diagnostics)?;
    let fid = trouver_fascicule(&catalogue, designation)?;
    let est_test = vrai(&catalogue.fascicules[&fid]["test"]);
    // Un fascicule de test est contrôlé comme s'il était publié : c'est une répétition, rien n'est écrit.
    catalogue.fascicules[&fid]["test"] = Value::Bool(false);
    controler(&catalogue, &mut diagnostics, true);
    let fsc = &catalogue.fascicules[&fid];
    let numero = fsc["numero"].as_i64();

    let mut autres = Vec::new();
    for (i, f) in &catalogue.fascicules {
        if !vrai(&f["test"]) && *i != fid {
            autres.push((
                lire_date(texte(&f["parution"]))?.with_timezone(&Utc),
                f["numero"].as_i64(),
            ));
        }
    }
    autres.sort_by_key(|t| t.1);
    let precedent = autres.iter().rfind(|t| t.1 < numero);
    let suivant = autres.iter().find(|t| t.1 > numero);

    let proposee = proposer_date(precedent.map(|p| p.0), maintenant);
    let mut rapport = RapportPrePublication {
        fascicule_id: fid.clone(),
        numero: fsc["numero"].clone(),
        parution: texte(&fsc["parution"]).to_owned(),
        date_proposee: en_utc(&proposee),
        rubriques: vec![Vec::new(); RUBRIQUES.len()],
    };

    // Diagnostics du validateur, rangés par rubrique. La chronologie est recalculée plus bas, sur des dates analysées.
    let ids = perimetre(&catalogue, &fid);
    let fichiers: HashSet<&str> = ids
        .iter()
        .filter_map(|i| catalogue.fichier(Some(i)))
        .collect();
    for d in &diagnostics.diagnostics {
        if d.regle == "fascicule.chronologie" {
            continue;
        }
        let niveau = if d.niveau == ERREUR { BLOQUANT } else { A_VOIR };
        let objet = d.objet.as_deref().filter(|o| !o.is_empty());
        let lieu = match d.fichier.as_deref().filter(|f| !f.is_empty()) {
            Some(f) => Some(format!(
                "{f}{}",
                objet.map(|o| format!(" ({o})")).unwrap_or_default()
            )),
            None => d.objet.clone(),
        };
        let mut concerne = d.objet.as_ref().is_some_and(|o| ids.contains(o))
            || d.fichier.as_deref().is_some_and(|f| fichiers.contains(f));
        // Le message « media.manquant » d'un autre fascicule porte l'identifiant de celui-ci.
        if d.regle == "media.manquant" && d.objet.as_deref() != Some(fid.as_str()) {
            concerne = false;
        }
        if concerne {
            rapport.ajouter(
                rubrique(&d.regle),
                Point::new(niveau, &d.regle, d.message.clone(), lieu),
            );
        } else if d.niveau == ERREUR && d.regle != "publication.aucun_fascicule" {
            rapport.ajouter(
                "depot",
                Point::new(BLOQUANT, &d.regle, d.message.clone(), lieu),
            );
        }
    }

    if est_test {
        rapport.ajouter(
            "fascicule",
            Point::new(
                BLOQUANT,
                "fascicule.test",
                "Fascicule de test : il n’est jamais publié en prod. Les contrôles ont été appliqués comme s’il l’était.",
                None,
            ),
        );
    }

    // Calendrier : la date évaluée est celle passée en argument, sinon celle du fascicule.
    let texte_date = date.unwrap_or(&rapport.parution).to_owned();
    let evaluee = match lire_date(&texte_date) {
        Ok(d) => d,
        Err(e) => {
            rapport.ajouter(
                "calendrier",
                Point::new(BLOQUANT, "calendrier.format", e, None),
            );
            return Ok(rapport);
        }
    };
    if evaluee.offset().local_minus_utc() != 0 {
        rapport.ajouter(
            "calendrier",
            Point::new(
                A_VOIR,
                "calendrier.utc",
                format!("{texte_date} n’est pas en UTC : la date d’effet s’écrit en UTC, par exemple {} (ADR 0029).", en_utc(&evaluee)),
                None,
            ),
        );
    }
    let evaluee = evaluee.with_timezone(&Utc);
    if evaluee <= maintenant {
        rapport.ajouter(
            "calendrier",
            Point::new(
                BLOQUANT,
                "calendrier.passee",
                format!(
                    "{} est déjà passée : choisissez une date à venir.",
                    en_utc(&evaluee)
                ),
                None,
            ),
        );
    }
    let numero_texte = |n: Option<i64>| n.map_or_else(|| "None".to_owned(), |n| n.to_string());
    // ADR 0054 : les fascicules de lancement paraissent chaque semaine, les quatre premiers le même jour.
    let lancement = |n: Option<i64>| n.is_some_and(|n| n <= FASCICULES_DE_LANCEMENT);
    if let Some((d_prec, n_prec)) = precedent {
        if evaluee < *d_prec || (evaluee == *d_prec && !lancement(numero)) {
            rapport.ajouter(
                "calendrier",
                Point::new(
                    BLOQUANT,
                    "calendrier.chronologie",
                    format!(
                        "{} ne suit pas la parution du fascicule {} ({}).",
                        en_utc(&evaluee),
                        numero_texte(*n_prec),
                        en_utc(d_prec)
                    ),
                    None,
                ),
            );
        } else {
            let ecart = evaluee - *d_prec;
            if !lancement(numero) && !(ECART_MINIMUM..=ECART_MAXIMUM).contains(&ecart) {
                rapport.ajouter(
                    "calendrier",
                    Point::new(
                        A_VOIR,
                        "calendrier.rythme",
                        format!(
                            "{} jours après le fascicule {} : le rythme est d’un fascicule environ tous les 30 jours (ADR 0014, 0022).",
                            ecart.num_days(),
                            numero_texte(*n_prec)
                        ),
                        None,
                    ),
                );
            }
        }
    }
    if let Some((d_suiv, n_suiv)) = suivant
        && (evaluee > *d_suiv || (evaluee == *d_suiv && !lancement(*n_suiv)))
    {
        rapport.ajouter(
            "calendrier",
            Point::new(
                BLOQUANT,
                "calendrier.chronologie",
                format!(
                    "{} ne précède pas la parution du fascicule {} ({}).",
                    en_utc(&evaluee),
                    numero_texte(*n_suiv),
                    en_utc(d_suiv)
                ),
                None,
            ),
        );
    }
    Ok(rapport)
}

pub fn ecrire_date(racine: &Path, fichier: &str, date: &str) -> Result<(), String> {
    let chemin = racine.join(fichier);
    let texte = fs::read_to_string(&chemin).map_err(|e| format!("{} : {e}", chemin.display()))?;
    let mut donnees =
        crate::json::lire(&texte).map_err(|e| format!("{} : {e:?}", chemin.display()))?;
    donnees["parution"] = Value::String(date.to_owned());
    let fin = if texte.ends_with('\n') { "\n" } else { "" };
    fs::write(&chemin, ecrire(&donnees, LISIBLE) + fin)
        .map_err(|e| format!("{} : {e}", chemin.display()))
}

/// Options de la commande `pre-publication`.
pub struct Options<'a> {
    pub racine: &'a Path,
    pub fascicule: &'a str,
    pub date: Option<&'a str>,
    pub ecrire: bool,
    pub maintenant: Option<&'a str>,
}

/// Rapport de la commande ; une erreur signale un appel invalide.
pub fn executer(o: &Options) -> Result<RapportPrePublication, String> {
    let maintenant = o
        .maintenant
        .map(lire_date)
        .transpose()?
        .map(|m| m.with_timezone(&Utc));
    if let Some(date) = o.date {
        lire_date(date)?;
    }
    if o.ecrire {
        let date = o.date.ok_or("--ecrire demande --date")?;
        let catalogue = charger(o.racine, &mut Rapport::default())?;
        let fid = trouver_fascicule(&catalogue, o.fascicule)?;
        ecrire_date(
            o.racine,
            catalogue.fichier(Some(&fid)).unwrap_or_default(),
            date,
        )?;
    }
    preparer(o.racine, o.fascicule, o.date, maintenant)
}
