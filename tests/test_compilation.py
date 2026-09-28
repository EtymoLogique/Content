"""Compilation de l'artefact : format, empreintes, champs privés, modes essai et publication."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

from etymo_contenu.chargement import charger
from etymo_contenu.cli import main
from etymo_contenu.compilation import compiler, ecrire, ressources, verifier
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


def _manifeste(sortie: Path, fascicule: dict) -> dict:
    return json.loads((sortie / fascicule["manifeste"]).read_text(encoding="utf-8"))


def test_calendrier_ne_cite_que_les_manifestes(depot, tmp_path):
    parutions = _compiler(depot, tmp_path / "a")
    for fascicule in parutions["fascicules"]:
        assert set(fascicule) == {"numero", "parution", "manifeste"}
        assert fascicule["manifeste"].startswith("f/")


def test_chaque_ressource_est_citee_par_un_manifeste(depot, tmp_path):
    parutions = _compiler(depot, tmp_path / "a")
    citees = {c for f in parutions["fascicules"] for c in ressources(_manifeste(tmp_path / "a", f))}
    ecrites = {f"{d}/{p.name}" for d in ("r", "a") for p in (tmp_path / "a" / d).iterdir()}
    assert citees == ecrites


def test_verifier_signale_une_ressource_absente(depot, tmp_path):
    parutions = _compiler(depot, tmp_path / "a")
    chemin = ressources(_manifeste(tmp_path / "a", parutions["fascicules"][0]))[0]
    (tmp_path / "a" / chemin).unlink()
    assert verifier(tmp_path / "a") == [f"Ressource absente : {chemin} (fascicule 0)"]


def test_chemins_en_hash_de_contenu(depot, tmp_path):
    parutions = _compiler(depot, tmp_path / "a")
    fascicule = parutions["fascicules"][0]
    for chemin in [fascicule["manifeste"], *ressources(_manifeste(tmp_path / "a", fascicule))]:
        contenu = (tmp_path / "a" / chemin).read_bytes()
        dossier, nom = chemin.split("/")
        assert nom == f"{hashlib.sha256(contenu).hexdigest()}{'.svg' if dossier == 'a' else '.json'}"
        assert not any(i in chemin for i in ("unt_", "brq_", "rct_", "fsc_", "med_", "lng_")), "un chemin ne dit jamais l’identifiant"


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
    # Seules les recettes des mots résultats du fascicule y figurent, pas celles des mots orphelins.
    resultats = {u["unite_id"] for u in depot.fascicule()[1]["unites"] if u["role"] == "resultat"}
    mots = {c["id"]: c["mot_id"] for c in map(depot.lire, depot.fichiers("compositions"))}
    recettes = {r["id"]: mots[r["composition_id"]] for r in map(depot.lire, depot.fichiers("ludique/recettes"))}
    assert {r["id"] for r in manifeste["recettes"]} == {rid for rid, mot in recettes.items() if mot in resultats}
    assert any(mot not in resultats for mot in recettes.values()), "le jeu d’exemples doit porter un mot orphelin"


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


def test_silhouettes_des_cartes_et_des_langues(depot, tmp_path):
    chemin, sans = depot.media_de("unite_id", depot.uid("mener"))
    chemin.unlink()
    (depot.racine / "medias" / sans["fichier"]).unlink()
    parutions = _compiler(depot, tmp_path / "a")
    manifeste = _manifeste(tmp_path / "a", parutions["fascicules"][0])
    _, media = depot.media_de("unite_id", depot.uid("biologie"))
    entree = next(c for c in manifeste["cartes"] if c["id"] == depot.uid("biologie"))
    assert entree["silhouette"].startswith("a/")
    assert (tmp_path / "a" / entree["silhouette"]).read_bytes() == (depot.racine / "medias" / media["fichier"]).read_bytes()
    assert next(c for c in manifeste["cartes"] if c["id"] == depot.uid("mener"))["silhouette"] is None
    assert [l["id"] for l in manifeste["langues"]] == sorted(l["id"] for l in manifeste["langues"])
    assert {"lng_wa90y54n", "lng_kmf44d76"} <= {l["id"] for l in manifeste["langues"] if l["silhouette"]}
    assert manifeste["jaquette"].startswith("a/")


def test_jaquette_et_media_partage(depot, tmp_path):
    # Deux cartes au même dessin partagent un seul fichier publié.
    _, geo = depot.media_de("unite_id", depot.uid("géo"))
    _, iste = depot.media_de("unite_id", depot.uid("iste"))
    (depot.racine / "medias" / iste["fichier"]).write_bytes((depot.racine / "medias" / geo["fichier"]).read_bytes())
    parutions = _compiler(depot, tmp_path / "a")
    manifeste = _manifeste(tmp_path / "a", parutions["fascicules"][0])
    assert manifeste["jaquette"].startswith("a/")
    silhouettes = {c["id"]: c["silhouette"] for c in manifeste["cartes"]}
    assert silhouettes[depot.uid("iste")] == silhouettes[depot.uid("géo")]
    assert verifier(tmp_path / "a") == []


def test_media_ne_fuit_ni_nom_ni_note(depot, tmp_path):
    _compiler(depot, tmp_path / "a")
    textes = [f.read_text(encoding="utf-8") for f in (tmp_path / "a").rglob("*") if f.is_file()]
    for media in (depot.lire(p) for p in depot.fichiers("ludique/medias")):
        assert not any(media["id"] in t or media["fichier"] in t for t in textes)
    assert not any("cardArt" in t for t in textes)


def test_verifier_signale_un_media_altere(depot, tmp_path):
    _compiler(depot, tmp_path / "a")
    fichier = next((tmp_path / "a" / "a").iterdir())
    fichier.write_bytes(fichier.read_bytes() + b" ")
    assert verifier(tmp_path / "a") == [f"Empreinte fausse : a/{fichier.name}"]


def test_manifeste_porte_les_graines(depot, tmp_path):
    parutions = _compiler(depot, tmp_path / "a")
    manifeste = json.loads((tmp_path / "a" / parutions["fascicules"][0]["manifeste"]).read_text())
    attendues = sorted(depot.fascicule()[1]["graines"], key=lambda g: g["brique_id"])
    assert manifeste["graines"] == attendues
    assert manifeste["id"] == depot.fascicule()[1]["id"]
