"""Visualisation locale du catalogue : une page HTML autonome, pour l'équipe éditoriale.

La page montre tout le dépôt, faits non relus et notes internes compris : elle
reste locale, dans ``rapports/`` que Git ignore, et n'est jamais publiée. Elle
n'appelle aucun service tiers. Les données sont embarquées en JSON et insérées
par ``textContent`` ; une silhouette n'est insérée que si elle passe le profil
SVG de ``medias.py``.
"""

from __future__ import annotations

import json
from pathlib import Path
from typing import Any

from etymo_contenu import medias
from etymo_contenu.chargement import Catalogue
from etymo_contenu.rapport import Rapport


def _canonique(unite: dict) -> dict:
    return next((f for f in unite.get("formes", []) if f.get("role") == "canonique"), unite["formes"][0] if unite.get("formes") else {})


def _libelle(objet: dict, cle: str) -> str | None:
    return (objet.get("libelles") or {}).get("fr", {}).get(cle)


def _fait(objet: dict) -> dict:
    fait = objet.get("fait") or {}
    return {"confiance": fait.get("confiance"), "relecture": fait.get("etat_relecture"), "note": fait.get("note_interne")}


def _svg(catalogue: Catalogue, media: dict | None) -> dict | None:
    if media is None:
        return None
    fichier = medias.chemin(catalogue, media)
    if not fichier.is_file():
        return {"fichier": media["fichier"], "probleme": "fichier introuvable"}
    contenu = fichier.read_bytes()
    problemes = medias.verifier_svg(contenu, media["type"])
    if problemes:
        return {"fichier": media["fichier"], "probleme": problemes[0]}
    return {"fichier": media["fichier"], "svg": contenu.decode("utf-8")}


def donnees(catalogue: Catalogue, rapport: Rapport) -> dict[str, Any]:
    index_medias = medias.par_cible(catalogue)
    brique_par_unite = {b["unite_id"]: b for b in catalogue.briques.values()}
    recette_par_mot: dict[str, list[str]] = {}
    for rid, recette in sorted(catalogue.recettes.items()):
        cmp = catalogue.compositions.get(recette["composition_id"])
        if cmp:
            recette_par_mot.setdefault(cmp["mot_id"], []).append(rid)
    fascicules_par_unite: dict[str, list[dict]] = {}
    for fid, fsc in catalogue.fascicules.items():
        for ligne in fsc["unites"]:
            fascicules_par_unite.setdefault(ligne["unite_id"], []).append({"id": fid, "numero": fsc["numero"], "role": ligne["role"]})

    unites = []
    for uid, unite in sorted(catalogue.unites.items()):
        forme = _canonique(unite)
        sens = sorted(unite.get("sens", []), key=lambda s: s.get("rang", 0))
        brique = brique_par_unite.get(uid)
        unites.append(
            {
                "id": uid,
                "nature": unite["nature"],
                "langue": unite["langue_id"],
                "texte": forme.get("texte", uid),
                "translitteration": next((t["texte"] for t in forme.get("translitterations") or []), None),
                "sens": [{"glose": _libelle(s, "glose"), "definition": _libelle(s, "definition")} for s in sens],
                "fait": _fait(unite),
                "brique": {"id": brique["id"], "rarete": brique["rarete"]} if brique else None,
                "recettes": recette_par_mot.get(uid, []),
                "fascicules": sorted(fascicules_par_unite.get(uid, []), key=lambda f: f["numero"]),
                "silhouette": _svg(catalogue, index_medias.get(("silhouette", "unite", uid))),
            }
        )

    liens = []
    for cid, cmp in sorted(catalogue.compositions.items()):
        for partie in cmp["parties"]:
            liens.append(
                {"id": cid, "type": "composition", "source": partie["unite_id"], "cible": cmp["mot_id"],
                 "rang": partie["rang"], "preferee": cmp.get("preferee", False),
                 "sens_litteral": _libelle(cmp, "sens_litteral"), "fait": _fait(cmp)}
            )
    for rid, rel in sorted(catalogue.relations.items()):
        liens.append({"id": rid, "type": rel["type_code"], "source": rel["unite_source_id"], "cible": rel["unite_cible_id"], "fait": _fait(rel)})

    return {
        "langues": [
            {"id": lid, "nom": _libelle(l, "nom") or l["code_interne"], "code": l["code_interne"],
             "silhouette": _svg(catalogue, index_medias.get(("silhouette", "langue", lid)))}
            for lid, l in sorted(catalogue.langues.items())
        ],
        "unites": unites,
        "liens": [l for l in liens if l["source"] in catalogue.unites and l["cible"] in catalogue.unites],
        "fascicules": [
            {"id": fid, "numero": f["numero"], "parution": f["parution"], "test": f["test"],
             "unites": [l["unite_id"] for l in f["unites"]],
             "jaquette": _svg(catalogue, index_medias.get(("jaquette", "fascicule", fid)))}
            for fid, f in sorted(catalogue.fascicules.items(), key=lambda x: (x[1]["test"], x[1]["numero"]))
        ],
        "diagnostics": [
            {"niveau": d.niveau, "regle": d.regle, "fichier": d.fichier, "message": d.message} for d in rapport.diagnostics
        ],
    }


def _json_pour_script(objet: Any) -> str:
    """JSON sûr dans un <script> : aucun « </script> » ni commentaire HTML ne peut en sortir."""
    texte = json.dumps(objet, ensure_ascii=False, separators=(",", ":"))
    return texte.replace("<", "\\u003c").replace(">", "\\u003e").replace("&", "\\u0026").replace("\u2028", "\\u2028").replace("\u2029", "\\u2029")


def page(catalogue: Catalogue, rapport: Rapport) -> str:
    gabarit = (Path(__file__).parent / "visualisation.html").read_text(encoding="utf-8")
    return gabarit.replace("/*DONNEES*/null", _json_pour_script(donnees(catalogue, rapport)))


def ecrire_page(catalogue: Catalogue, rapport: Rapport, sortie: Path) -> None:
    sortie.parent.mkdir(parents=True, exist_ok=True)
    sortie.write_text(page(catalogue, rapport), encoding="utf-8")
