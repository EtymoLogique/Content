"""Interface en ligne de commande : « valider » et « nouvel-id »."""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

from etymo_contenu.chargement import charger
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

    rapport = Rapport()
    existants = set(charger(args.racine, rapport).fichiers)
    for _ in range(args.n):
        identifiant = nouvel_id(args.prefixe, existants)
        existants.add(identifiant)
        print(identifiant)
    return 0


if __name__ == "__main__":
    sys.exit(main())
