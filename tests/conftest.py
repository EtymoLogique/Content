"""Fixtures : une copie jetable du dépôt, que chaque test modifie pour déclencher une règle."""

from __future__ import annotations

import json
import shutil
from pathlib import Path
from typing import Callable

import pytest

from etymo_contenu.cli import valider
from etymo_contenu.ids import nouvel_id
from etymo_contenu.rapport import Rapport

RACINE = Path(__file__).resolve().parents[1]
A_COPIER = ["schemas", "referentiels", "sources", "unites", "compositions", "relations", "ludique", "medias"]
SVG_SILHOUETTE = (
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 300 400">'
    '<circle cx="200" cy="280" r="90" fill="currentColor"/><path d="M150 280L250 280" style="stroke:var(--art-bg)"/></svg>\n'
)
SVG_JAQUETTE = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 600 800"><rect width="600" height="800" fill="#1f5c4a"/></svg>\n'


class Depot:
    def __init__(self, racine: Path) -> None:
        self.racine = racine

    def lire(self, chemin: Path) -> dict:
        return json.loads(chemin.read_text(encoding="utf-8"))

    def ecrire(self, chemin: Path, donnees: dict) -> None:
        chemin.parent.mkdir(parents=True, exist_ok=True)
        chemin.write_text(json.dumps(donnees, ensure_ascii=False, indent=2), encoding="utf-8")

    def fichiers(self, dossier: str) -> list[Path]:
        return sorted((self.racine / dossier).glob("*.json"))

    def trouver(self, dossier: str, critere: Callable[[dict], bool]) -> tuple[Path, dict]:
        trouves = [(p, d) for p in self.fichiers(dossier) if critere(d := self.lire(p))]
        assert len(trouves) == 1, f"{len(trouves)} objets trouvés dans {dossier}"
        return trouves[0]

    def unite(self, texte: str, glose: str | None = None) -> tuple[Path, dict]:
        def critere(u: dict) -> bool:
            canonique = [f for f in u["formes"] if f["role"] == "canonique"][0]
            return canonique["texte"] == texte and (glose is None or u["sens"][0]["libelles"]["fr"].get("glose") == glose)

        return self.trouver("unites", critere)

    def uid(self, texte: str, glose: str | None = None) -> str:
        return self.unite(texte, glose)[1]["id"]

    def brique(self, texte: str, glose: str | None = None) -> tuple[Path, dict]:
        uid = self.uid(texte, glose)
        return self.trouver("ludique/briques", lambda b: b["unite_id"] == uid)

    def composition(self, texte_mot: str) -> tuple[Path, dict]:
        uid = self.uid(texte_mot)
        return self.trouver("compositions", lambda c: c["mot_id"] == uid)

    def recette(self, texte_mot: str) -> tuple[Path, dict]:
        cid = self.composition(texte_mot)[1]["id"]
        return self.trouver("ludique/recettes", lambda r: r["composition_id"] == cid)

    def relation(self, texte_cible: str, glose: str | None = None) -> tuple[Path, dict]:
        uid = self.uid(texte_cible, glose)
        return self.trouver("relations", lambda r: r["unite_cible_id"] == uid)

    def fascicule(self) -> tuple[Path, dict]:
        return self.trouver("ludique/fascicules", lambda f: True)

    def media(self, type_media: str, cle: str, cible: str, svg: str | None = None) -> dict:
        """Déclare un média et écrit son fichier : par défaut, un SVG minimal conforme au profil."""
        identifiant = nouvel_id("med")
        dossier = "silhouettes" if type_media == "silhouette" else "jaquettes"
        fichier = f"{dossier}/essai-{identifiant.removeprefix('med_')}.svg"
        if svg is None:
            svg = SVG_SILHOUETTE if type_media == "silhouette" else SVG_JAQUETTE
        cible_fichier = self.racine / "medias" / fichier
        cible_fichier.parent.mkdir(parents=True, exist_ok=True)
        cible_fichier.write_text(svg, encoding="utf-8")
        donnees = {"id": identifiant, "type": type_media, "fichier": fichier, cle: cible}
        self.ecrire(self.racine / "ludique" / "medias" / f"{identifiant}.json", donnees)
        return donnees

    def media_de(self, cle: str, cible: str) -> tuple[Path, dict]:
        return self.trouver("ludique/medias", lambda m: m.get(cle) == cible)

    def modifier(self, trouve: tuple[Path, dict], fonction: Callable[[dict], None]) -> dict:
        chemin, donnees = trouve
        fonction(donnees)
        self.ecrire(chemin, donnees)
        return donnees

    def copier_sous(self, trouve: tuple[Path, dict], nouvel_id: str, fonction: Callable[[dict], None] | None = None) -> dict:
        chemin, donnees = trouve
        donnees = json.loads(json.dumps(donnees))
        donnees["id"] = nouvel_id
        if fonction:
            fonction(donnees)
        self.ecrire(chemin.with_name(f"{nouvel_id}.json"), donnees)
        return donnees

    def valider(self, publication: bool = False) -> Rapport:
        return valider(self.racine, publication=publication)


@pytest.fixture
def depot(tmp_path: Path) -> Depot:
    for dossier in A_COPIER:
        shutil.copytree(RACINE / dossier, tmp_path / dossier)
    return Depot(tmp_path)


def erreurs(rapport: Rapport) -> set[str]:
    return rapport.regles("erreur")


def avertissements(rapport: Rapport) -> set[str]:
    return rapport.regles("avertissement")
