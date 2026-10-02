"""Visualisation locale : données embarquées sans échappement possible, silhouettes filtrées par le profil."""

from __future__ import annotations

import json
import re

from etymo_contenu.chargement import charger
from etymo_contenu.cli import main
from etymo_contenu.rapport import Rapport
from etymo_contenu.visualisation import donnees, page

INJECTION = "</script><script>alert(1)</script><!--"


def _embarque(html: str) -> dict:
    bloc = re.search(r'<script type="application/json" id="donnees">(.*?)</script>', html, re.S)
    assert bloc, "bloc de données absent"
    return json.loads(bloc.group(1))


def test_page_embarque_tout_le_catalogue(depot):
    rapport = Rapport()
    catalogue = charger(depot.racine, rapport)
    html = page(catalogue, rapport)
    embarque = _embarque(html)
    assert {u["id"] for u in embarque["unites"]} == set(catalogue.unites)
    assert {l["id"] for l in embarque["liens"] if l["type"] != "composition"} == set(catalogue.relations)
    biologie = next(u for u in embarque["unites"] if u["id"] == depot.uid("biologie"))
    assert biologie["silhouette"]["svg"].startswith("<svg")
    assert "/*DONNEES*/" not in html


def test_donnees_ne_sortent_jamais_du_script(depot):
    def piegee(u):
        u["sens"][0]["libelles"]["fr"]["glose"] = INJECTION

    depot.modifier(depot.unite("géo"), piegee)
    rapport = Rapport()
    html = page(charger(depot.racine, rapport), rapport)
    assert INJECTION not in html
    assert html.count("</script>") == 2
    geo = next(u for u in _embarque(html)["unites"] if u["texte"] == "géo")
    assert geo["sens"][0]["glose"] == INJECTION


def test_silhouette_hors_profil_jamais_embarquee(depot):
    svg = '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 300 400"><script>alert(1)</script></svg>'
    for chemin in depot.fichiers("ludique/medias"):  # la silhouette du catalogue cède la place à celle de l'essai
        if depot.lire(chemin).get("unite_id") == depot.uid("phile"):
            chemin.unlink()
    depot.media("silhouette", "unite_id", depot.uid("phile"), svg=svg)
    rapport = Rapport()
    iste = next(u for u in donnees(charger(depot.racine, rapport), rapport)["unites"] if u["id"] == depot.uid("phile"))
    assert "svg" not in iste["silhouette"]
    assert iste["silhouette"]["probleme"]


def test_cli_visualiser_meme_sur_un_depot_invalide(depot, tmp_path, capsys):
    depot.modifier(depot.recette("géologie"), lambda r: r.update(composition_id="cmp_zzzzzzzz"))
    sortie = tmp_path / "vue" / "catalogue.html"
    assert main(["visualiser", "--racine", str(depot.racine), "--sortie", str(sortie)]) == 0
    embarque = _embarque(sortie.read_text(encoding="utf-8"))
    assert any(d["niveau"] == "erreur" for d in embarque["diagnostics"])
    assert "erreur(s)" in capsys.readouterr().out


def test_jaquette_embarquee(depot):
    fascicule = depot.fascicule()[1]["id"]
    rapport = Rapport()
    embarque = donnees(charger(depot.racine, rapport), rapport)
    jaquette = next(f for f in embarque["fascicules"] if f["id"] == fascicule)["jaquette"]
    assert jaquette["svg"].startswith("<svg") and jaquette["fichier"].startswith("jaquettes/")
