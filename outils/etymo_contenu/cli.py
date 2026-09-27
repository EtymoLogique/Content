"""Interface en ligne de commande : « valider », « compiler » et « nouvel-id »."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

from etymo_contenu.chargement import charger
from etymo_contenu.compilation import compiler, ecrire, verifier
from etymo_contenu.controles import controler
from etymo_contenu.ids import PREFIXES, nouvel_id
from etymo_contenu.rapport import Rapport


def valider(racine: Path, publication: bool = False) -> Rapport:
    rapport = Rapport()
    catalogue = charger(racine, rapport)
    controler(catalogue, rapport, publication=publication)
    return rapport


def main(argv: list[str] | None = None) -> int:
    parseur = argparse.ArgumentParser(prog="etymo-contenu", description="Outillage éditorial du catalogue ÉtymoLogique.")
    sous = parseur.add_subparsers(dest="commande", required=True)

    p_valider = sous.add_parser("valider", help="Valider le dépôt : schémas et contraintes d’intégrité.")
    p_valider.add_argument("--racine", type=Path, default=Path.cwd(), help="Racine du dépôt (par défaut : dossier courant).")
    p_valider.add_argument("--publication", action="store_true", help="Appliquer aussi les exigences de publication (ADR 0005, ADR 0014).")
    p_valider.add_argument("--json", action="store_true", help="Rapport au format JSON.")
    p_valider.add_argument("--strict", action="store_true", help="Traiter les avertissements comme des erreurs.")

    p_compiler = sous.add_parser("compiler", help="Compiler l’artefact publié : ressources et manifestes en hash de contenu, calendrier des parutions.")
    p_compiler.add_argument("--racine", type=Path, default=Path.cwd(), help="Racine du dépôt (par défaut : dossier courant).")
    p_compiler.add_argument("--sortie", type=Path, default=Path("artefact"), help="Dossier de sortie, vidé avant compilation (par défaut : artefact).")
    p_compiler.add_argument("--version", default="locale", help="Identifiant de la publication, par exemple le commit.")
    p_compiler.add_argument(
        "--essai",
        action="store_true",
        help="Hors-prod seulement : inclure les fascicules de test et les faits non relus. L’artefact est marqué « essai ».",
    )

    p_id = sous.add_parser("nouvel-id", help="Générer des identifiants opaques inédits.")
    p_id.add_argument("prefixe", choices=sorted(PREFIXES), help="Type d’objet.")
    p_id.add_argument("-n", type=int, default=1, help="Nombre d’identifiants.")
    p_id.add_argument("--racine", type=Path, default=Path.cwd())

    args = parseur.parse_args(argv)

    if args.commande == "valider":
        rapport = valider(args.racine, publication=args.publication)
        print(rapport.en_json() if args.json else rapport.en_texte())
        echec = rapport.erreurs or (args.strict and rapport.avertissements)
        return 1 if echec else 0

    if args.commande == "compiler":
        # Hors essai, l'artefact ne part que si les exigences de publication passent.
        rapport = valider(args.racine, publication=not args.essai)
        if rapport.erreurs:
            print(rapport.en_texte())
            print("Compilation refusée : le dépôt ne passe pas la validation.", file=sys.stderr)
            return 1
        catalogue = charger(args.racine, Rapport())
        artefact = compiler(catalogue, essai=args.essai, version=args.version)
        ecrire(artefact, args.sortie)
        problemes = verifier(args.sortie)
        for probleme in problemes:
            print(probleme, file=sys.stderr)
        if problemes:
            return 1
        parutions = json.loads((args.sortie / "parutions.json").read_text(encoding="utf-8"))
        mode = "essai" if args.essai else "publication"
        print(
            f"{len(artefact.fichiers)} fichiers, {len(parutions['fascicules'])} fascicule(s), "
            f"version {parutions['version']}, mode {mode} → {args.sortie}"
        )
        return 0

    rapport = Rapport()
    existants = set(charger(args.racine, rapport).fichiers)
    for _ in range(args.n):
        identifiant = nouvel_id(args.prefixe, existants)
        existants.add(identifiant)
        print(identifiant)
    return 0


if __name__ == "__main__":
    sys.exit(main())
