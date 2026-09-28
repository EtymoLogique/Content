"""Lecture du dépôt éditorial : fichiers JSON, validation par schéma, index des objets."""

from __future__ import annotations

import json
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from jsonschema import Draft202012Validator
from referencing import Registry, Resource

from etymo_contenu.ids import prefixe
from etymo_contenu.rapport import Rapport

# Dossier → (nom du schéma, préfixe attendu, attribut du catalogue)
DOSSIERS = {
    "referentiels/langues": ("langue", "lng", "langues"),
    "referentiels/ecritures": ("ecriture", "ecr", "ecritures"),
    "referentiels/conventions": ("convention", "cnv", "conventions"),
    "sources": ("source", "src", "sources"),
    "unites": ("unite", "unt", "unites"),
    "compositions": ("composition", "cmp", "compositions"),
    "relations": ("relation", "rel", "relations"),
    "ludique/briques": ("brique", "brq", "briques"),
    "ludique/recettes": ("recette", "rct", "recettes"),
    "ludique/fascicules": ("fascicule", "fsc", "fascicules"),
    "ludique/exclusions": ("exclusion", "exc", "exclusions"),
    "ludique/medias": ("media", "med", "medias"),
}

EQUILIBRAGE = "ludique/equilibrage.json"

VOCABULAIRES = {
    "referentiels/types-relation.json": "types_relation",
    "referentiels/types-transformation.json": "types_transformation",
}


class CleDupliquee(ValueError):
    pass


def _sans_doublon(paires: list[tuple[str, Any]]) -> dict[str, Any]:
    objet: dict[str, Any] = {}
    for cle, valeur in paires:
        if cle in objet:
            raise CleDupliquee(cle)
        objet[cle] = valeur
    return objet


@dataclass
class Catalogue:
    racine: Path
    langues: dict[str, dict] = field(default_factory=dict)
    ecritures: dict[str, dict] = field(default_factory=dict)
    conventions: dict[str, dict] = field(default_factory=dict)
    sources: dict[str, dict] = field(default_factory=dict)
    unites: dict[str, dict] = field(default_factory=dict)
    compositions: dict[str, dict] = field(default_factory=dict)
    relations: dict[str, dict] = field(default_factory=dict)
    briques: dict[str, dict] = field(default_factory=dict)
    recettes: dict[str, dict] = field(default_factory=dict)
    fascicules: dict[str, dict] = field(default_factory=dict)
    exclusions: dict[str, dict] = field(default_factory=dict)
    medias: dict[str, dict] = field(default_factory=dict)
    types_relation: dict[str, dict] = field(default_factory=dict)
    types_transformation: dict[str, dict] = field(default_factory=dict)
    # Paramètres des plis (ludique/equilibrage.json), recopiés dans chaque manifeste.
    equilibrage: dict | None = None
    # Objets imbriqués : identifiant → (unité, objet)
    formes: dict[str, tuple[str, dict]] = field(default_factory=dict)
    sens: dict[str, tuple[str, dict]] = field(default_factory=dict)
    # Identifiant → fichier relatif, pour les diagnostics
    fichiers: dict[str, str] = field(default_factory=dict)
    # Identifiants présents dans un fichier invalide : leurs références ne sont pas signalées
    ids_invalides: set[str] = field(default_factory=set)

    def fichier(self, identifiant: str | None) -> str | None:
        return self.fichiers.get(identifiant) if identifiant else None


def _registre(dossier_schemas: Path) -> tuple[Registry, dict[str, Draft202012Validator]]:
    ressources = []
    schemas = {}
    for chemin in sorted(dossier_schemas.glob("*.schema.json")):
        contenu = json.loads(chemin.read_text(encoding="utf-8"))
        ressources.append((contenu["$id"], Resource.from_contents(contenu)))
        schemas[chemin.name.removesuffix(".schema.json")] = contenu
    registre = Registry().with_resources(ressources)
    validateurs = {nom: Draft202012Validator(s, registry=registre) for nom, s in schemas.items()}
    return registre, validateurs


def _lire_json(chemin: Path, relatif: str, rapport: Rapport) -> Any | None:
    rapport.fichiers_lus += 1
    try:
        return json.loads(chemin.read_text(encoding="utf-8"), object_pairs_hook=_sans_doublon)
    except CleDupliquee as exc:
        rapport.erreur("json.cle_dupliquee", f"Clé « {exc} » en double : une seule ligne par objet et par locale.", relatif)
    except (json.JSONDecodeError, UnicodeDecodeError) as exc:
        rapport.erreur("json.syntaxe", f"JSON illisible : {exc}", relatif)
    return None


def _valider_schema(validateur: Draft202012Validator, donnees: Any, relatif: str, rapport: Rapport) -> bool:
    erreurs = sorted(validateur.iter_errors(donnees), key=lambda e: list(e.absolute_path))
    for err in erreurs:
        chemin = "/".join(str(p) for p in err.absolute_path) or "(racine)"
        rapport.erreur("schema", f"{chemin} : {err.message}", relatif)
    return not erreurs


def _ids_imbriques(donnees: dict) -> list[str]:
    ids = []
    for forme in donnees.get("formes", []) or []:
        if isinstance(forme, dict) and isinstance(forme.get("id"), str):
            ids.append(forme["id"])
    for sens in donnees.get("sens", []) or []:
        if isinstance(sens, dict) and isinstance(sens.get("id"), str):
            ids.append(sens["id"])
    for trf in donnees.get("transformations", []) or []:
        if isinstance(trf, dict) and isinstance(trf.get("id"), str):
            ids.append(trf["id"])
    return ids


def charger(racine: Path, rapport: Rapport) -> Catalogue:
    racine = racine.resolve()
    catalogue = Catalogue(racine=racine)
    _, validateurs = _registre(racine / "schemas")

    for relatif, attribut in VOCABULAIRES.items():
        chemin = racine / relatif
        if not chemin.exists():
            rapport.erreur("fichier.absent", "Vocabulaire contrôlé absent.", relatif)
            continue
        donnees = _lire_json(chemin, relatif, rapport)
        if donnees is None or not _valider_schema(validateurs["vocabulaire"], donnees, relatif, rapport):
            continue
        cible = getattr(catalogue, attribut)
        for entree in donnees["types"]:
            if entree["code"] in cible:
                rapport.erreur("vocabulaire.doublon", f"Code « {entree['code']} » déclaré deux fois.", relatif)
            cible[entree["code"]] = entree

    relatif = EQUILIBRAGE
    if not (racine / relatif).exists():
        rapport.erreur("fichier.absent", "Paramètres d’équilibrage des plis absents.", relatif)
    else:
        donnees = _lire_json(racine / relatif, relatif, rapport)
        if donnees is not None and _valider_schema(validateurs["equilibrage"], donnees, relatif, rapport):
            catalogue.equilibrage = donnees

    vus: dict[str, str] = {}
    for dossier, (nom_schema, prefixe_attendu, attribut) in DOSSIERS.items():
        repertoire = racine / dossier
        if not repertoire.is_dir():
            continue
        for chemin in sorted(repertoire.iterdir()):
            relatif = chemin.relative_to(racine).as_posix()
            if chemin.name.startswith(".") or chemin.is_dir():
                continue
            if chemin.suffix != ".json":
                rapport.erreur("fichier.inattendu", "Seuls des fichiers .json sont admis dans ce dossier.", relatif)
                continue
            donnees = _lire_json(chemin, relatif, rapport)
            if donnees is None:
                catalogue.ids_invalides.add(chemin.stem)
                continue
            identifiant = donnees.get("id") if isinstance(donnees, dict) else None
            valide = _valider_schema(validateurs[nom_schema], donnees, relatif, rapport)

            if isinstance(identifiant, str):
                if chemin.stem != identifiant:
                    rapport.erreur("identite.nom_fichier", f"Le fichier doit s’appeler {identifiant}.json.", relatif, identifiant)
                    valide = False
                if prefixe(identifiant) != prefixe_attendu:
                    rapport.erreur(
                        "identite.prefixe",
                        f"Un objet de « {dossier} » a un identifiant en {prefixe_attendu}_.",
                        relatif,
                        identifiant,
                    )
                    valide = False

            tous_ids = ([identifiant] if isinstance(identifiant, str) else []) + (
                _ids_imbriques(donnees) if isinstance(donnees, dict) else []
            )
            for i in tous_ids:
                if i in vus:
                    rapport.erreur("identite.doublon", f"Identifiant déjà utilisé dans {vus[i]}.", relatif, i)
                    valide = False
                else:
                    vus[i] = relatif
                    catalogue.fichiers[i] = relatif

            if not valide:
                catalogue.ids_invalides.update(tous_ids)
                continue

            getattr(catalogue, attribut)[identifiant] = donnees
            if nom_schema == "unite":
                for forme in donnees["formes"]:
                    catalogue.formes[forme["id"]] = (identifiant, forme)
                for sens in donnees["sens"]:
                    catalogue.sens[sens["id"]] = (identifiant, sens)

    return catalogue
