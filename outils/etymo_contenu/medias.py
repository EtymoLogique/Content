"""Médias du catalogue : profil SVG admis et rattachement aux cartes (ADR 0016, 0023, 0025, 0026).

Un SVG publié est servi tel quel par le CDN, puis inséré dans la page par la
PWA : c'est un vecteur XSS. Le profil est donc une liste blanche, et il refuse
aussi tout texte, commentaire ou titre qui trahirait le mot d'une carte.
"""

from __future__ import annotations

import re
import xml.etree.ElementTree as ET
from pathlib import Path

from etymo_contenu.chargement import Catalogue

DOSSIER = "medias"
SVG_NS = "http://www.w3.org/2000/svg"

TAILLE_MAX = {"silhouette": 32 * 1024, "jaquette": 256 * 1024}
VIEWBOX_SILHOUETTE = "0 0 300 400"  # format 3:4 des cartes

ELEMENTS = {"svg", "g", "defs", "clipPath", "mask", "path", "circle", "ellipse", "rect", "line", "polyline", "polygon"}
ATTRIBUTS = {
    "viewBox", "preserveAspectRatio", "width", "height", "id",
    "d", "cx", "cy", "r", "rx", "ry", "x", "y", "x1", "y1", "x2", "y2", "points", "transform",
    "fill", "fill-rule", "fill-opacity", "stroke", "stroke-width", "stroke-linecap", "stroke-linejoin",
    "stroke-dasharray", "stroke-dashoffset", "stroke-miterlimit", "stroke-opacity", "opacity",
    "clip-path", "clip-rule", "clipPathUnits", "mask", "maskUnits", "maskContentUnits", "style",
}
PROPRIETES_STYLE = {
    "fill", "fill-rule", "fill-opacity", "stroke", "stroke-width", "stroke-linecap", "stroke-linejoin", "stroke-opacity", "opacity",
}
# Silhouette : une seule teinte, fixée par la carte ; les évidements prennent le fond (ADR 0023).
COULEURS_SILHOUETTE = {"currentColor", "none"}
EVIDEMENT = "var(--art-bg)"
OPACITES = {"opacity", "fill-opacity", "stroke-opacity"}

URL = re.compile(r"url\(")
URL_LOCALE = re.compile(r"url\(#[A-Za-z][\w-]*\)")
INTERDITS = ("javascript:", "data:", "expression(", "@import", "\\", "<", "&")


def _nom_local(balise: str) -> tuple[str | None, str]:
    if balise.startswith("{"):
        ns, _, nom = balise[1:].partition("}")
        return ns, nom
    return None, balise


def _verifier_valeur(nom: str, valeur: str) -> str | None:
    bas = valeur.lower()
    if any(motif in bas for motif in INTERDITS):
        return f"valeur interdite dans « {nom} »"
    if len(URL.findall(valeur)) != len(URL_LOCALE.findall(valeur)):
        return f"« {nom} » ne peut citer qu’une référence locale url(#…)"
    return None


def _couleur_silhouette(propriete: str, valeur: str, dans_style: bool) -> str | None:
    if propriete in OPACITES:
        return f"« {propriete} » : une silhouette n’a qu’une teinte, sans opacité propre"
    if propriete in ("fill", "stroke"):
        admises = COULEURS_SILHOUETTE | ({EVIDEMENT} if dans_style else set())
        if valeur not in admises:
            return f"« {propriete}: {valeur} » : une silhouette n’admet que {', '.join(sorted(admises))}"
    return None


def verifier_svg(contenu: bytes, type_media: str) -> list[str]:
    """Problèmes du fichier au regard du profil, dans l'ordre de lecture ; liste vide si le fichier est admis."""
    if len(contenu) > TAILLE_MAX[type_media]:
        return [f"fichier de {len(contenu)} octets, au-delà de {TAILLE_MAX[type_media]}"]
    try:
        texte = contenu.decode("utf-8")
    except UnicodeDecodeError:
        return ["le fichier n’est pas en UTF-8"]
    # Pas de DTD, donc aucune entité à développer : ElementTree ne lit jamais de ressource externe.
    for motif, message in (("<!DOCTYPE", "DOCTYPE interdit"), ("<!ENTITY", "entité interdite"), ("<!--", "commentaire interdit"), ("<![CDATA[", "CDATA interdit")):
        if motif in texte:
            return [message]
    if "<?" in texte.lstrip()[2:] or (texte.lstrip().startswith("<?") and not texte.lstrip().startswith("<?xml ")):
        return ["instruction de traitement interdite"]
    try:
        racine = ET.fromstring(texte)
    except ET.ParseError as exc:
        return [f"XML illisible : {exc}"]

    problemes: list[str] = []
    if racine.tag != f"{{{SVG_NS}}}svg":
        return ["la racine doit être <svg xmlns=\"http://www.w3.org/2000/svg\">"]
    viewbox = racine.get("viewBox")
    if not viewbox:
        problemes.append("viewBox absent sur <svg>")
    elif type_media == "silhouette" and " ".join(viewbox.split()) != VIEWBOX_SILHOUETTE:
        problemes.append(f"viewBox « {viewbox} » : une silhouette est en « {VIEWBOX_SILHOUETTE} »")

    for element in racine.iter():
        ns, nom = _nom_local(element.tag)
        if ns != SVG_NS or nom not in ELEMENTS:
            problemes.append(f"élément <{nom}> interdit")
            continue
        if (element.text or "").strip() or (element.tail or "").strip():
            problemes.append(f"texte interdit dans <{nom}>")
        for attribut, valeur in element.attrib.items():
            ns_attr, nom_attr = _nom_local(attribut)
            if ns_attr is not None or nom_attr not in ATTRIBUTS:
                problemes.append(f"attribut « {nom_attr} » interdit sur <{nom}>")
                continue
            if erreur := _verifier_valeur(nom_attr, valeur):
                problemes.append(erreur)
                continue
            if nom_attr == "style":
                for declaration in filter(None, (d.strip() for d in valeur.split(";"))):
                    propriete, deux_points, val = (x.strip() for x in declaration.partition(":"))
                    if not deux_points or propriete not in PROPRIETES_STYLE:
                        problemes.append(f"déclaration de style « {declaration} » interdite")
                    elif type_media == "silhouette" and (erreur := _couleur_silhouette(propriete, val, True)):
                        problemes.append(erreur)
            elif type_media == "silhouette" and (erreur := _couleur_silhouette(nom_attr, valeur.strip(), False)):
                problemes.append(erreur)
    return problemes


def chemin(catalogue: Catalogue, media: dict) -> Path:
    return catalogue.racine / DOSSIER / media["fichier"]


def cible(media: dict) -> tuple[str, str]:
    """(nature, identifiant) de l'objet illustré : unité, langue ou fascicule."""
    for cle, nature in (("unite_id", "unite"), ("langue_id", "langue"), ("fascicule_id", "fascicule")):
        if cle in media:
            return nature, media[cle]
    raise ValueError(f"Média sans cible : {media.get('id')}")


def par_cible(catalogue: Catalogue) -> dict[tuple[str, str, str], dict]:
    """(type, nature, identifiant) → média ; le premier par identifiant si plusieurs se disputent la cible."""
    index: dict[tuple[str, str, str], dict] = {}
    for _, media in sorted(catalogue.medias.items()):
        index.setdefault((media["type"], *cible(media)), media)
    return index


def _retenu(objet: dict) -> bool:
    fait = objet.get("fait")
    return not (isinstance(fait, dict) and fait.get("confiance") == "non_retenue")


def langues_citees(catalogue: Catalogue, uids: list[str] | set[str]) -> list[str]:
    """Langues des cartes d'un fascicule et de leurs étymons : chacune a sa silhouette (ADR 0023)."""
    langues = {catalogue.unites[u]["langue_id"] for u in uids if u in catalogue.unites}
    for relation in catalogue.relations.values():
        source = catalogue.unites.get(relation["unite_source_id"])
        if relation["unite_cible_id"] in uids and source and _retenu(relation):
            langues.add(source["langue_id"])
    return sorted(langues)
