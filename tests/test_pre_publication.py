"""Script de pré-publication du skill « preparer-publication »."""

from __future__ import annotations

import importlib.util
import sys
from datetime import datetime, timezone
from pathlib import Path

import pytest

SCRIPT = Path(__file__).resolve().parents[1] / ".agents" / "skills" / "preparer-publication" / "pre_publication.py"
_spec = importlib.util.spec_from_file_location("pre_publication", SCRIPT)
pp = importlib.util.module_from_spec(_spec)
sys.modules["pre_publication"] = pp
_spec.loader.exec_module(pp)

MAINTENANT = datetime(2026, 9, 27, 12, 0, tzinfo=timezone.utc)


def regles(rapport, rubrique: str) -> set[str]:
    return {p.regle for p in rapport.rubriques[rubrique]}


def publier(depot, numero: int = 1, parution: str = "2026-11-01T00:00:00Z") -> dict:
    return depot.modifier(depot.fascicule(), lambda f: f.update(test=False, numero=numero, parution=parution))


def second_fascicule(depot, numero: int, parution: str) -> None:
    """Un fascicule de plus, qui reprend un ingrédient du premier pour rester valide."""
    _, premier = depot.fascicule()
    ingredient = next(l for l in premier["unites"] if l["role"] == "ingredient")
    depot.ecrire(
        depot.racine / "ludique" / "fascicules" / "fsc_zzzzzzzz.json",
        {"id": "fsc_zzzzzzzz", "numero": numero, "nom": "Essai", "parution": parution, "test": False, "unites": [ingredient]},
    )


# ------------------------------------------------------------------ dates


@pytest.mark.parametrize(
    ("precedente", "attendue"),
    [
        ("2026-11-01T00:00:00Z", "2026-12-01T00:00:00Z"),
        ("2027-02-01T00:00:00Z", "2027-03-01T00:00:00Z"),
        ("2026-12-01T00:00:00Z", "2027-01-01T00:00:00Z"),
        ("2027-01-15T08:00:00+02:00", "2027-03-01T00:00:00Z"),
    ],
)
def test_date_proposee_le_premier_du_mois(precedente, attendue):
    assert pp.en_utc(pp.proposer_date(pp.lire_date(precedente), MAINTENANT)) == attendue


def test_date_proposee_toujours_a_venir():
    assert pp.en_utc(pp.proposer_date(pp.lire_date("2026-01-01T00:00:00Z"), MAINTENANT)) == "2026-10-01T00:00:00Z"
    assert pp.en_utc(pp.proposer_date(None, MAINTENANT)) == "2026-10-01T00:00:00Z"


def test_date_mal_formee():
    with pytest.raises(ValueError):
        pp.lire_date("2026-12-01")


# ---------------------------------------------------------------- rapport


def test_fascicule_de_test_controle_comme_publie(depot):
    rapport = pp.preparer(depot.racine, "0", maintenant=MAINTENANT)
    assert "fascicule.test" in regles(rapport, "fascicule")
    assert {"publication.relecture", "publication.localisation"} <= regles(rapport, "relecture")
    assert rapport.bloquants


def test_fascicule_publiable_signale_relecture_et_medias(depot):
    publier(depot)
    _, med = depot.media_de("fascicule_id", depot.fascicule()[1]["id"])
    (depot.racine / "ludique" / "medias" / f"{med['id']}.json").unlink()
    rapport = pp.preparer(depot.racine, "1", maintenant=MAINTENANT)
    assert "publication.relecture" in regles(rapport, "relecture")
    assert "media.manquant" in regles(rapport, "medias")
    assert "fascicule.test" not in regles(rapport, "fascicule")


def test_date_passee_bloquante(depot):
    publier(depot, parution="2026-09-01T00:00:00Z")
    assert "calendrier.passee" in regles(pp.preparer(depot.racine, "1", maintenant=MAINTENANT), "calendrier")


def test_date_hors_utc_a_voir(depot):
    publier(depot, parution="2026-11-01T08:00:00+02:00")
    points = pp.preparer(depot.racine, "1", maintenant=MAINTENANT).rubriques["calendrier"]
    assert [(p.regle, p.niveau) for p in points] == [("calendrier.utc", pp.A_VOIR)]


def test_chronologie_sur_dates_analysees(depot):
    # 01:00 à Paris le 1ᵉʳ décembre, c'est 23:00 UTC le 30 novembre : le fascicule 2 paraît bien après.
    # Une comparaison de chaînes conclurait l'inverse.
    publier(depot, parution="2026-12-01T01:00:00+02:00")
    second_fascicule(depot, 2, "2026-12-01T00:00:00Z")
    rapport = pp.preparer(depot.racine, "2", maintenant=MAINTENANT)
    assert "calendrier.chronologie" not in regles(rapport, "calendrier")
    assert "calendrier.rythme" in regles(rapport, "calendrier")


def test_chronologie_avant_le_precedent(depot):
    publier(depot, parution="2026-12-01T00:00:00Z")
    second_fascicule(depot, 2, "2026-11-01T00:00:00Z")
    rapport = pp.preparer(depot.racine, "2", maintenant=MAINTENANT)
    assert "calendrier.chronologie" in regles(rapport, "calendrier")
    assert rapport.date_proposee == "2027-01-01T00:00:00Z"


def test_chronologie_apres_le_suivant(depot):
    publier(depot, parution="2027-01-01T00:00:00Z")
    second_fascicule(depot, 2, "2026-12-01T00:00:00Z")
    assert "calendrier.chronologie" in regles(pp.preparer(depot.racine, "1", maintenant=MAINTENANT), "calendrier")


def test_erreur_hors_fascicule_bloque_aussi(depot):
    publier(depot)
    second_fascicule(depot, 2, "2026-12-01T00:00:00Z")
    rapport = pp.preparer(depot.racine, "2", maintenant=MAINTENANT)
    assert "publication.relecture" in regles(rapport, "depot")


def test_ecrire_date(depot, capsys):
    publier(depot)
    code = pp.main(["1", "--racine", str(depot.racine), "--date", "2026-12-01T00:00:00Z", "--ecrire", "--maintenant", "2026-09-27T12:00:00Z"])
    assert code == 1  # les faits du jeu d'essai restent en brouillon
    assert depot.fascicule()[1]["parution"] == "2026-12-01T00:00:00Z"
    assert "Date proposée" in capsys.readouterr().out


def test_ecrire_sans_date_refuse(depot, capsys):
    assert pp.main(["0", "--racine", str(depot.racine), "--ecrire"]) == 2


def test_fascicule_introuvable(depot, capsys):
    assert pp.main(["42", "--racine", str(depot.racine)]) == 2
