"""Rapport de pré-publication d'un fascicule : tous les contrôles, puis le calendrier.

Usage, depuis la racine du dépôt :

    .venv/bin/python .agents/skills/preparer-publication/pre_publication.py <numéro | fsc_…>
    .venv/bin/python .agents/skills/preparer-publication/pre_publication.py 1 --date 2026-12-01T00:00:00Z
    .venv/bin/python .agents/skills/preparer-publication/pre_publication.py 1 --date 2026-12-01T00:00:00Z --ecrire

Le script ne modifie rien, sauf avec --ecrire, qui écrit la date dans le fichier du fascicule.
Chaque règle montre ses cinq premiers points ; --complet les liste tous, --json donne le rapport machine.
Un fascicule de test est contrôlé comme s'il était publié, et signalé comme bloquant.
Il sort en 0 si rien ne bloque la publication, en 1 sinon, en 2 si l'appel est invalide.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import asdict, dataclass, field
from datetime import datetime, timedelta, timezone
from pathlib import Path

from etymo_contenu import medias
from etymo_contenu.chargement import Catalogue, charger
from etymo_contenu.controles import controler
from etymo_contenu.rapport import ERREUR, Rapport

ECART_MINIMUM = timedelta(days=28)  # « environ tous les 30 jours » (ADR 0014, 0022), février compris
ECART_MAXIMUM = timedelta(days=45)
MOTIF_DATE = re.compile(r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}(:\d{2})?(Z|[+-]\d{2}:\d{2})$")

RUBRIQUES = {
    "integrite": "Intégrité du fascicule",
    "relecture": "Relecture et sources",
    "medias": "Médias",
    "fascicule": "Contenu du fascicule",
    "exclusions": "Exclusions",
    "calendrier": "Calendrier",
    "depot": "Reste du dépôt (bloque aussi la compilation en prod)",
}
BLOQUANT, A_VOIR = "bloquant", "à voir"


@dataclass
class Point:
    niveau: str
    regle: str
    message: str
    lieu: str | None = None


@dataclass
class RapportPrePublication:
    fascicule_id: str
    numero: int
    parution: str
    date_proposee: str
    rubriques: dict[str, list[Point]] = field(default_factory=lambda: {cle: [] for cle in RUBRIQUES})

    def ajouter(self, rubrique: str, point: Point) -> None:
        self.rubriques[rubrique].append(point)

    @property
    def bloquants(self) -> int:
        return sum(p.niveau == BLOQUANT for points in self.rubriques.values() for p in points)

    @property
    def a_voir(self) -> int:
        return sum(p.niveau == A_VOIR for points in self.rubriques.values() for p in points)

    def en_texte(self, complet: bool = False, exemples: int = 5) -> str:
        lignes = [f"Fascicule {self.numero} ({self.fascicule_id}) — parution actuelle : {self.parution}", ""]
        for cle, titre in RUBRIQUES.items():
            points = self.rubriques[cle]
            etat = "OK" if not points else ("BLOQUANT" if any(p.niveau == BLOQUANT for p in points) else "À VOIR")
            lignes.append(f"## {titre} : {etat}")
            par_regle: dict[tuple[str, str], list[Point]] = {}
            for p in sorted(points, key=lambda p: (p.niveau != BLOQUANT, p.regle, p.lieu or "")):
                par_regle.setdefault((p.niveau, p.regle), []).append(p)
            for (niveau, regle), groupe in par_regle.items():
                if len(groupe) > 1:
                    lignes.append(f"  [{niveau}] [{regle}] × {len(groupe)}")
                montres = groupe if complet else groupe[:exemples]
                for p in montres:
                    lieu = f"{p.lieu} : " if p.lieu else ""
                    prefixe = "    - " if len(groupe) > 1 else f"  [{niveau}] [{regle}] "
                    lignes.append(f"{prefixe}{lieu}{p.message}")
                if len(montres) < len(groupe):
                    lignes.append(f"    … et {len(groupe) - len(montres)} autre(s) : relancer avec --complet.")
            lignes.append("")
        lignes.append(f"Date proposée : {self.date_proposee} (à faire confirmer)")
        lignes.append(f"{self.bloquants} point(s) bloquant(s), {self.a_voir} point(s) à voir.")
        return "\n".join(lignes)

    def en_json(self) -> str:
        donnees = {
            "fascicule_id": self.fascicule_id,
            "numero": self.numero,
            "parution": self.parution,
            "date_proposee": self.date_proposee,
            "bloquants": self.bloquants,
            "a_voir": self.a_voir,
            "rubriques": {cle: [asdict(p) for p in points] for cle, points in self.rubriques.items()},
        }
        return json.dumps(donnees, ensure_ascii=False, indent=2)


# ------------------------------------------------------------------- dates


def lire_date(texte: str) -> datetime:
    """Analyse une date ISO 8601 avec fuseau, au format des fascicules."""
    if not MOTIF_DATE.match(texte):
        raise ValueError(f"date « {texte} » : attendu AAAA-MM-JJTHH:MM[:SS] suivi de Z ou d’un décalage ±HH:MM")
    return datetime.fromisoformat(texte.replace("Z", "+00:00"))


def en_utc(date: datetime) -> str:
    return date.astimezone(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def _premier_du_mois_suivant(date: datetime) -> datetime:
    annee, mois = (date.year + 1, 1) if date.month == 12 else (date.year, date.month + 1)
    return datetime(annee, mois, 1, tzinfo=timezone.utc)


def premier_du_mois_des(date: datetime) -> datetime:
    """Premier 1ᵉʳ du mois à 00:00 UTC qui tombe à cette date ou après."""
    date = date.astimezone(timezone.utc)
    debut = datetime(date.year, date.month, 1, tzinfo=timezone.utc)
    return debut if debut >= date else _premier_du_mois_suivant(date)


def proposer_date(precedente: datetime | None, maintenant: datetime) -> datetime:
    """Le 1ᵉʳ du mois à 00:00 UTC, au moins 28 jours après la parution précédente et après maintenant."""
    plancher = maintenant + timedelta(seconds=1)
    if precedente is not None:
        plancher = max(plancher, precedente + ECART_MINIMUM)
    return premier_du_mois_des(plancher)


# --------------------------------------------------------------- périmètre


def trouver_fascicule(catalogue: Catalogue, designation: str) -> str:
    if designation in catalogue.fascicules:
        return designation
    if designation.isdigit():
        candidats = [fid for fid, f in catalogue.fascicules.items() if f["numero"] == int(designation) and not f["test"]]
        candidats = candidats or [fid for fid, f in catalogue.fascicules.items() if f["numero"] == int(designation)]
        if len(candidats) == 1:
            return candidats[0]
    raise LookupError(f"Fascicule « {designation} » introuvable : donnez son identifiant fsc_… ou son numéro.")


def perimetre(catalogue: Catalogue, fid: str) -> set[str]:
    """Identifiants des objets qu'un fascicule publie ou dont il dépend directement."""
    fsc = catalogue.fascicules[fid]
    unites = {l["unite_id"] for l in fsc["unites"]}
    compositions = {cid for cid, c in catalogue.compositions.items() if c["mot_id"] in unites}
    relations = {rid for rid, r in catalogue.relations.items() if r["unite_cible_id"] in unites}
    etymons = {catalogue.relations[rid]["unite_source_id"] for rid in relations}
    briques = {bid for bid, b in catalogue.briques.items() if b["unite_id"] in unites}
    recettes = {rid for rid, r in catalogue.recettes.items() if r["composition_id"] in compositions}
    langues = set(medias.langues_citees(catalogue, sorted(unites)))
    cibles = unites | langues | {fid}
    medias_ids = {mid for mid, m in catalogue.medias.items() if medias.cible(m)[1] in cibles}
    exclusions = {eid for eid, e in catalogue.exclusions.items() if e["fascicule_id"] == fid}
    return {fid} | unites | etymons | compositions | relations | briques | recettes | medias_ids | exclusions


def _rubrique(regle: str) -> str:
    if regle == "publication.exclusion" or regle.startswith("exclusion."):
        return "exclusions"
    if regle.startswith("publication."):
        return "relecture"
    if regle.startswith("media."):
        return "medias"
    if regle.startswith("fascicule."):
        return "fascicule"
    return "integrite"


# ----------------------------------------------------------------- rapport


def preparer(racine: Path, designation: str, date: str | None = None, maintenant: datetime | None = None) -> RapportPrePublication:
    maintenant = maintenant or datetime.now(timezone.utc)
    diagnostics = Rapport()
    catalogue = charger(racine, diagnostics)
    fid = trouver_fascicule(catalogue, designation)
    fsc = catalogue.fascicules[fid]
    est_test = fsc["test"]
    # Un fascicule de test est contrôlé comme s'il était publié : c'est une répétition, rien n'est écrit.
    catalogue.fascicules[fid] = fsc = {**fsc, "test": False}
    controler(catalogue, diagnostics, publication=True)
    autres = sorted(
        ((lire_date(f["parution"]), f["numero"], i) for i, f in catalogue.fascicules.items() if not f["test"] and i != fid),
        key=lambda t: t[1],
    )
    precedents = [t for t in autres if t[1] < fsc["numero"]]
    suivants = [t for t in autres if t[1] > fsc["numero"]]
    precedent = precedents[-1] if precedents else None
    suivant = suivants[0] if suivants else None

    proposee = proposer_date(precedent[0] if precedent else None, maintenant)
    rapport = RapportPrePublication(fid, fsc["numero"], fsc["parution"], en_utc(proposee))

    # Diagnostics du validateur, rangés par rubrique. La chronologie est recalculée plus bas, sur des dates analysées.
    ids = perimetre(catalogue, fid)
    fichiers = {catalogue.fichier(i) for i in ids} - {None}
    for d in diagnostics.diagnostics:
        if d.regle == "fascicule.chronologie":
            continue
        niveau = BLOQUANT if d.niveau == ERREUR else A_VOIR
        lieu = d.fichier + (f" ({d.objet})" if d.objet else "") if d.fichier else d.objet
        concerne = d.objet in ids or d.fichier in fichiers
        # Le message « media.manquant » d'un autre fascicule porte l'identifiant de celui-ci.
        if d.regle == "media.manquant" and d.objet != fid:
            concerne = False
        if concerne:
            rapport.ajouter(_rubrique(d.regle), Point(niveau, d.regle, d.message, lieu))
        elif d.niveau == ERREUR and d.regle != "publication.aucun_fascicule":
            rapport.ajouter("depot", Point(BLOQUANT, d.regle, d.message, lieu))

    if est_test:
        rapport.ajouter(
            "fascicule",
            Point(BLOQUANT, "fascicule.test", "Fascicule de test : il n’est jamais publié en prod. Les contrôles ont été appliqués comme s’il l’était."),
        )

    # Calendrier : la date évaluée est celle passée en argument, sinon celle du fascicule.
    texte = date or fsc["parution"]
    try:
        evaluee = lire_date(texte)
    except ValueError as exc:
        rapport.ajouter("calendrier", Point(BLOQUANT, "calendrier.format", str(exc)))
        return rapport
    if evaluee.utcoffset() != timedelta(0):
        rapport.ajouter(
            "calendrier",
            Point(A_VOIR, "calendrier.utc", f"{texte} n’est pas en UTC : la date d’effet s’écrit en UTC, par exemple {en_utc(evaluee)} (ADR 0029)."),
        )
    if evaluee <= maintenant:
        rapport.ajouter("calendrier", Point(BLOQUANT, "calendrier.passee", f"{en_utc(evaluee)} est déjà passée : choisissez une date à venir."))
    if precedent:
        d_prec, n_prec, _ = precedent
        if evaluee <= d_prec:
            rapport.ajouter(
                "calendrier",
                Point(BLOQUANT, "calendrier.chronologie", f"{en_utc(evaluee)} ne suit pas la parution du fascicule {n_prec} ({en_utc(d_prec)})."),
            )
        else:
            ecart = evaluee - d_prec
            if not ECART_MINIMUM <= ecart <= ECART_MAXIMUM:
                rapport.ajouter(
                    "calendrier",
                    Point(
                        A_VOIR,
                        "calendrier.rythme",
                        f"{ecart.days} jours après le fascicule {n_prec} : le rythme est d’un fascicule environ tous les 30 jours (ADR 0014, 0022).",
                    ),
                )
    if suivant:
        d_suiv, n_suiv, _ = suivant
        if evaluee >= d_suiv:
            rapport.ajouter(
                "calendrier",
                Point(BLOQUANT, "calendrier.chronologie", f"{en_utc(evaluee)} ne précède pas la parution du fascicule {n_suiv} ({en_utc(d_suiv)})."),
            )
    return rapport


def ecrire_date(racine: Path, catalogue_fichier: str, date: str) -> None:
    chemin = racine / catalogue_fichier
    texte = chemin.read_text(encoding="utf-8")
    donnees = json.loads(texte)
    donnees["parution"] = date
    sortie = json.dumps(donnees, ensure_ascii=False, indent=2)
    chemin.write_text(sortie + ("\n" if texte.endswith("\n") else ""), encoding="utf-8")


def main(argv: list[str] | None = None) -> int:
    parseur = argparse.ArgumentParser(description="Rapport de pré-publication d’un fascicule : contrôles et calendrier.")
    parseur.add_argument("fascicule", help="Numéro du fascicule ou identifiant fsc_….")
    parseur.add_argument("--racine", type=Path, default=Path.cwd(), help="Racine du dépôt (par défaut : dossier courant).")
    parseur.add_argument("--date", help="Date de parution à évaluer, en ISO 8601 UTC, par exemple 2026-12-01T00:00:00Z.")
    parseur.add_argument("--ecrire", action="store_true", help="Écrire --date dans le fichier du fascicule, puis refaire le rapport.")
    parseur.add_argument("--maintenant", help=argparse.SUPPRESS)
    parseur.add_argument("--json", action="store_true", help="Rapport au format JSON.")
    parseur.add_argument("--complet", action="store_true", help="Lister tous les points, pas seulement les premiers de chaque règle.")
    args = parseur.parse_args(argv)

    try:
        maintenant = lire_date(args.maintenant) if args.maintenant else None
        if args.date:
            lire_date(args.date)
        if args.ecrire:
            if not args.date:
                raise ValueError("--ecrire demande --date")
            catalogue = charger(args.racine, Rapport())
            fid = trouver_fascicule(catalogue, args.fascicule)
            ecrire_date(args.racine, catalogue.fichier(fid), args.date)
        rapport = preparer(args.racine, args.fascicule, args.date, maintenant)
    except (LookupError, ValueError) as exc:
        print(exc, file=sys.stderr)
        return 2
    print(rapport.en_json() if args.json else rapport.en_texte(complet=args.complet))
    return 1 if rapport.bloquants else 0


if __name__ == "__main__":
    sys.exit(main())
