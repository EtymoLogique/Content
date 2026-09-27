"""Compilation du catalogue en artefact statique (ADR 0027, ADR 0029).

L'artefact reprend l'arborescence du préfixe ``staging/`` du bucket :

- ``r/<sha256>.json`` : ressources (cartes, recettes, exclusions) ;
- ``f/<sha256>.json`` : un manifeste par fascicule ;
- ``parutions.json`` : calendrier des parutions, seul fichier modifiable. Il
  porte les référentiels et, pour chaque fascicule, sa date, son manifeste et
  la liste de ses fichiers, pour que la tâche de parution copie ces fichiers
  dans ``public/`` et écrive ``public/index.json`` à la date d'effet.

Chaque fichier haché est nommé d'après l'empreinte SHA-256 de son contenu :
un fichier ne change jamais, et un contenu identique garde le même chemin.
"""

from __future__ import annotations

import hashlib
import json
import shutil
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any

from etymo_contenu.chargement import Catalogue

FORMAT = 1

# Champs éditoriaux jamais publiés (modèle de données, carte de répartition).
CHAMPS_PRIVES = {"note_interne", "etat_relecture"}


class ErreurCompilation(Exception):
    pass


def serialiser(objet: Any) -> bytes:
    """JSON canonique : clés triées, sans espace, UTF-8. Deux contenus égaux ont la même empreinte."""
    return json.dumps(objet, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode("utf-8")


def sans_prive(objet: Any) -> Any:
    if isinstance(objet, dict):
        return {cle: sans_prive(valeur) for cle, valeur in objet.items() if cle not in CHAMPS_PRIVES}
    if isinstance(objet, list):
        return [sans_prive(valeur) for valeur in objet]
    return objet


def _retenu(objet: dict) -> bool:
    """Un fait non retenu n'est jamais publié, même en mode d'essai."""
    fait = objet.get("fait")
    return not (isinstance(fait, dict) and fait.get("confiance") == "non_retenue")


@dataclass
class Artefact:
    fichiers: dict[str, bytes] = field(default_factory=dict)

    def deposer(self, dossier: str, objet: Any) -> str:
        contenu = serialiser(objet)
        chemin = f"{dossier}/{hashlib.sha256(contenu).hexdigest()}.json"
        self.fichiers[chemin] = contenu
        return chemin


class Compilateur:
    def __init__(self, catalogue: Catalogue, essai: bool, version: str) -> None:
        self.c = catalogue
        self.essai = essai
        self.version = version
        self.artefact = Artefact()
        self.brique_par_unite = {b["unite_id"]: b for b in catalogue.briques.values()}

    # ---------------------------------------------------------------- sources
    def _sources_citees(self, objet: Any, trouvees: set[str]) -> set[str]:
        if isinstance(objet, dict):
            if isinstance(objet.get("source_id"), str):
                trouvees.add(objet["source_id"])
            for valeur in objet.values():
                self._sources_citees(valeur, trouvees)
        elif isinstance(objet, list):
            for valeur in objet:
                self._sources_citees(valeur, trouvees)
        return trouvees

    def _avec_sources(self, carte: dict) -> dict:
        ids = sorted(self._sources_citees(carte, set()))
        carte["sources"] = {sid: sans_prive(self.c.sources[sid]) for sid in ids if sid in self.c.sources}
        return carte

    # ------------------------------------------------------------------ cartes
    def _etymon(self, uid: str) -> dict:
        unite = self.c.unites[uid]
        canonique = next((f for f in unite.get("formes", []) if f.get("role") == "canonique"), None)
        return sans_prive(
            {
                "id": uid,
                "nature": unite["nature"],
                "langue_id": unite["langue_id"],
                "forme": canonique,
                "sens": [s for s in unite.get("sens", []) if _retenu(s)],
            }
        )

    def carte(self, uid: str) -> dict:
        unite = self.c.unites[uid]
        carte = {
            cle: valeur for cle, valeur in unite.items() if cle not in ("formes", "sens")
        }
        carte["formes"] = [f for f in unite.get("formes", []) if _retenu(f)]
        carte["sens"] = [s for s in unite.get("sens", []) if _retenu(s)]
        carte["relations"] = [
            {**rel, "etymon": self._etymon(rel["unite_source_id"])}
            for _, rel in sorted(self.c.relations.items())
            if rel["unite_cible_id"] == uid and _retenu(rel) and rel["unite_source_id"] in self.c.unites
        ]
        if unite["nature"] == "mot":
            carte["compositions"] = [
                cmp for _, cmp in sorted(self.c.compositions.items()) if cmp["mot_id"] == uid and _retenu(cmp)
            ]
        brique = self.brique_par_unite.get(uid)
        if brique:
            carte["brique"] = {cle: brique[cle] for cle in ("id", "forme_affichee_id", "sens_affiche_id", "rarete")}
        return self._avec_sources(sans_prive(carte))

    # ---------------------------------------------------------------- recettes
    def recette(self, rid: str) -> dict:
        recette = sans_prive(self.c.recettes[rid])
        recette["resultat_id"] = self.c.compositions[recette["composition_id"]]["mot_id"]
        return recette

    # --------------------------------------------------------------- fascicule
    def _legendaire(self, uid: str) -> bool:
        brique = self.brique_par_unite.get(uid)
        return bool(brique and brique["rarete"] == "legendaire")

    def _depend_de_legendaire(self, uid: str, recettes: list[str]) -> bool:
        for rid in recettes:
            recette = self.c.recettes[rid]
            if self.c.compositions[recette["composition_id"]]["mot_id"] != uid:
                continue
            for emplacement in recette["emplacements"]:
                if any(self._legendaire(self.c.briques[b]["unite_id"]) for b in emplacement["briques_acceptees"] if b in self.c.briques):
                    return True
        return False

    def fascicule(self, fid: str) -> tuple[dict, list[str]]:
        fsc = self.c.fascicules[fid]
        uids = [u["unite_id"] for u in fsc["unites"]]
        resultats = {u["unite_id"] for u in fsc["unites"] if u["role"] == "resultat"}
        fichiers: list[str] = []

        cartes = []
        for entree in fsc["unites"]:
            chemin = self.artefact.deposer("r", self.carte(entree["unite_id"]))
            fichiers.append(chemin)
            cartes.append({"id": entree["unite_id"], "role": entree["role"], "carte": chemin, "silhouette": None})

        ids_recettes = sorted(
            rid for rid, r in self.c.recettes.items() if self.c.compositions[r["composition_id"]]["mot_id"] in resultats
        )
        recettes = []
        for rid in ids_recettes:
            chemin = self.artefact.deposer("r", self.recette(rid))
            fichiers.append(chemin)
            recettes.append({"id": rid, "chemin": chemin})

        exclusions = [
            {"id": eid, "combinaison": exc["combinaison"], "raison": exc["raison"]}
            for eid, exc in sorted(self.c.exclusions.items())
            if exc["fascicule_id"] == fid and (self.essai or exc["etat_relecture"] == "relu")
        ]
        chemin_exclusions = self.artefact.deposer("r", {"exclusions": exclusions})
        fichiers.append(chemin_exclusions)

        # Compteurs hors légendaires et hors mots qui en dépendent (ADR 0015).
        visibles = [u for u in uids if not self._legendaire(u) and not self._depend_de_legendaire(u, ids_recettes)]
        natures = [self.c.unites[u]["nature"] for u in visibles]
        compteurs = {
            "mots": sum(1 for u in visibles if u in resultats and self.c.unites[u]["nature"] == "mot"),
            "prefixes": natures.count("prefixe"),
            "suffixes": natures.count("suffixe"),
            "langues": len({self.c.unites[u]["langue_id"] for u in visibles}),
        }

        briques = sorted(
            ({"id": b["id"], "unite_id": u, "rarete": b["rarete"]} for u in uids if (b := self.brique_par_unite.get(u))),
            key=lambda b: b["id"],
        )

        manifeste = {
            "format": FORMAT,
            "numero": fsc["numero"],
            "parution": fsc["parution"],
            "jaquette": None,
            "compteurs": compteurs,
            # Poids, base d'encre et barèmes ne sont pas encore édités : le pli
            # ne porte pour l'instant que ses briques tirables.
            "pli": {"briques": briques},
            "cartes": cartes,
            "recettes": recettes,
            "exclusions": chemin_exclusions,
            "jalons": [],
        }
        if fsc.get("test"):
            manifeste["test"] = True
        chemin_manifeste = self.artefact.deposer("f", manifeste)
        fichiers.append(chemin_manifeste)
        return {"numero": fsc["numero"], "parution": fsc["parution"], "manifeste": chemin_manifeste}, sorted(set(fichiers))

    # ----------------------------------------------------------------- global
    def referentiels(self) -> dict:
        return {
            "langues": [sans_prive(v) for _, v in sorted(self.c.langues.items())],
            "systemes_ecriture": [sans_prive(v) for _, v in sorted(self.c.ecritures.items())],
            "conventions": [sans_prive(v) for _, v in sorted(self.c.conventions.items())],
        }

    def compiler(self) -> Artefact:
        retenus = [fid for fid, f in self.c.fascicules.items() if self.essai or not f.get("test")]
        calendrier = []
        for fid in sorted(retenus, key=lambda i: (self.c.fascicules[i]["parution"], self.c.fascicules[i]["numero"])):
            entree, fichiers = self.fascicule(fid)
            calendrier.append({**entree, "fichiers": fichiers})

        parutions = {
            "format": FORMAT,
            "version": self.version,
            "essai": self.essai,
            "referentiels": self.referentiels(),
            "fascicules": calendrier,
        }
        self.artefact.fichiers["parutions.json"] = serialiser(parutions)
        return self.artefact


def compiler(catalogue: Catalogue, essai: bool, version: str) -> Artefact:
    return Compilateur(catalogue, essai, version).compiler()


def ecrire(artefact: Artefact, sortie: Path) -> None:
    if sortie.exists():
        shutil.rmtree(sortie)
    for chemin, contenu in artefact.fichiers.items():
        cible = sortie / chemin
        cible.parent.mkdir(parents=True, exist_ok=True)
        cible.write_bytes(contenu)


def verifier(sortie: Path) -> list[str]:
    """Recalcule l'empreinte de chaque fichier haché et contrôle chaque chemin cité."""
    problemes = []
    for fichier in sorted(sortie.glob("[rf]/*.json")):
        attendu = fichier.stem
        if hashlib.sha256(fichier.read_bytes()).hexdigest() != attendu:
            problemes.append(f"Empreinte fausse : {fichier.relative_to(sortie)}")
    parutions = json.loads((sortie / "parutions.json").read_text(encoding="utf-8"))
    for fsc in parutions["fascicules"]:
        for chemin in [fsc["manifeste"], *fsc["fichiers"]]:
            if not (sortie / chemin).is_file():
                problemes.append(f"Fichier absent : {chemin} (fascicule {fsc['numero']})")
    return problemes
