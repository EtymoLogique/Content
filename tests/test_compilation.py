"""Compilation de l'artefact : format, empreintes, champs privés, modes essai et publication."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

from etymo_contenu.chargement import charger
from etymo_contenu.cli import main
from etymo_contenu.compilation import compiler, ecrire, verifier
from etymo_contenu.rapport import Rapport


def _compiler(depot, sortie: Path, essai: bool = True, version: str = "test") -> dict:
    artefact = compiler(charger(depot.racine, Rapport()), essai=essai, version=version)
    ecrire(artefact, sortie)
    return json.loads((sortie / "parutions.json").read_text(encoding="utf-8"))


def test_essai_publie_le_fascicule_de_test(depot, tmp_path):
    parutions = _compiler(depot, tmp_path / "a")
    assert parutions["essai"] is True
    assert [f["numero"] for f in parutions["fascicules"]] == [0]
    assert verifier(tmp_path / "a") == []


def test_publication_ecarte_les_fascicules_de_test(depot, tmp_path):
    parutions = _compiler(depot, tmp_path / "a", essai=False)
    assert parutions["essai"] is False
    assert parutions["fascicules"] == []


def test_chemins_en_hash_de_contenu(depot, tmp_path):
    parutions = _compiler(depot, tmp_path / "a")
    for chemin in parutions["fascicules"][0]["fichiers"]:
        contenu = (tmp_path / "a" / chemin).read_bytes()
        assert chemin.split("/")[1] == f"{hashlib.sha256(contenu).hexdigest()}.json"
        assert not any(i in chemin for i in ("unt_", "brq_", "rct_", "fsc_")), "un chemin ne dit jamais l’identifiant"


def test_compilation_deterministe(depot, tmp_path):
    a = _compiler(depot, tmp_path / "a")
    b = _compiler(depot, tmp_path / "b")
    assert a == b
    assert sorted(p.name for p in (tmp_path / "a" / "r").iterdir()) == sorted(p.name for p in (tmp_path / "b" / "r").iterdir())


def test_aucun_champ_prive(depot, tmp_path):
    _compiler(depot, tmp_path / "a")
    for fichier in (tmp_path / "a").rglob("*.json"):
        texte = fichier.read_text(encoding="utf-8")
        assert "note_interne" not in texte and "etat_relecture" not in texte, fichier.name


def test_fait_non_retenu_jamais_publie(depot, tmp_path):
    def non_retenu(u):
        u["sens"][0]["fait"]["confiance"] = "non_retenue"

    depot.modifier(depot.unite("géologie"), non_retenu)
    sens_id = depot.unite("géologie")[1]["sens"][0]["id"]
    _compiler(depot, tmp_path / "a")
    assert not any(sens_id in f.read_text(encoding="utf-8") for f in (tmp_path / "a" / "r").glob("*.json"))


def test_manifeste_et_carte(depot, tmp_path):
    parutions = _compiler(depot, tmp_path / "a")
    manifeste = json.loads((tmp_path / "a" / parutions["fascicules"][0]["manifeste"]).read_text(encoding="utf-8"))
    uid = depot.uid("géologie")
    entree = next(c for c in manifeste["cartes"] if c["id"] == uid)
    assert entree["role"] == "resultat"
    carte = json.loads((tmp_path / "a" / entree["carte"]).read_text(encoding="utf-8"))
    assert carte["compositions"][0]["mot_id"] == uid
    assert set(carte["sources"]) >= {s["source_id"] for s in carte["fait"]["sources"]}
    assert len(manifeste["recettes"]) == len(list(depot.fichiers("ludique/recettes")))


def test_cli_refuse_la_publication_d_un_depot_invalide(depot, tmp_path, capsys):
    def reference_cassee(r):
        r["composition_id"] = "cmp_zzzzzzzz"

    depot.modifier(depot.recette("géologie"), reference_cassee)
    code = main(["compiler", "--racine", str(depot.racine), "--sortie", str(tmp_path / "a")])
    assert code == 1
    assert not (tmp_path / "a").exists()


def test_cli_essai(depot, tmp_path, capsys):
    code = main(["compiler", "--essai", "--racine", str(depot.racine), "--sortie", str(tmp_path / "a"), "--version", "v1"])
    assert code == 0
    assert "mode essai" in capsys.readouterr().out
