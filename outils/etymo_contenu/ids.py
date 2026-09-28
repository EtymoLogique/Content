"""Identifiants opaques et stables (ADR 0005, modèle de données, principe 1)."""

from __future__ import annotations

import re
import secrets

# Alphabet base32 de Crockford en minuscules : ni i, l, o, u, pour éviter les confusions.
ALPHABET = "0123456789abcdefghjkmnpqrstvwxyz"
LONGUEUR = 8

PREFIXES = {
    "lng": "langue",
    "ecr": "système d’écriture",
    "cnv": "convention de translittération",
    "src": "source",
    "unt": "unité lexicale",
    "frm": "forme",
    "sns": "sens",
    "cmp": "composition",
    "trf": "transformation",
    "rel": "relation étymologique",
    "brq": "brique",
    "rct": "recette",
    "fsc": "fascicule",
    "exc": "exclusion",
    "med": "média",
    "jal": "jalon",
}

MOTIF = re.compile(r"^(?P<prefixe>[a-z]{3})_[0-9a-z]{8}$")


def prefixe(identifiant: str) -> str | None:
    correspondance = MOTIF.match(identifiant) if isinstance(identifiant, str) else None
    return correspondance.group("prefixe") if correspondance else None


def nouvel_id(prefixe_type: str, existants: set[str] | None = None) -> str:
    if prefixe_type not in PREFIXES:
        raise ValueError(f"Préfixe inconnu : {prefixe_type}")
    existants = existants or set()
    while True:
        candidat = f"{prefixe_type}_" + "".join(secrets.choice(ALPHABET) for _ in range(LONGUEUR))
        if candidat not in existants:
            return candidat
