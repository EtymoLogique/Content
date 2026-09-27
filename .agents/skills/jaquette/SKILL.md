---
name: jaquette
description: Dessiner la jaquette SVG d'un fascicule d'ÉtymoLogique selon l'ADR 0016 (neutres et Ambre seulement, rien qui trahisse le contenu), la déclarer dans ludique/medias/ et vérifier qu'elle passe le profil SVG. À utiliser pour chaque nouveau fascicule, qui a besoin de sa jaquette.
---

# Skill jaquette

Références, qui priment sur ce résumé : ADR 0016 (plis et jaquettes par fascicule), 0025 et 0026 (médias), la section « Jaquettes » de `docs/plis.html`, les jetons de `docs/assets/etymo.css` et les jaquettes des maquettes (`JACKETS` dans `docs/assets/etymo.js`), tous dans le dépôt de conception. Le profil SVG de `outils/etymo_contenu/medias.py` fait foi.

## 1. Choisir le motif

- **Une jaquette par fascicule**, portée par tous ses plis.
- Fascicules n° 01 à 10 : les jaquettes sont déjà attribuées, dans l'ordre : Rosette, Argile, Aleph, Boustrophédon, Routes des mots, Arbre des langues, Lapidaire, Casse, Lettrine, Palimpseste. Partez de leur dessin dans `JACKETS` (`docs/assets/etymo.js`).
- Au-delà : un motif nouveau, tiré de l'écriture et de l'histoire des langues, en version graphique et plate. Jamais de parchemin ni de plume. Proposez le thème à l'utilisateur avant de dessiner.
- **Rien ne se devine** : ni le motif ni la couleur ne trahissent une brique, une rareté ou une langue du fascicule. La jaquette ne dit que le fascicule.

## 2. Dessiner

Palette **neutres et Ambre seulement** : les couleurs de préfixe, de suffixe, de langue ou de famille annonceraient le contenu.

| Jeton | Valeur | Usage |
|---|---|---|
| Encre `--ink` | `#0f1f19` | Traits, cadre |
| Crème `--cream` | `#fffbf3` | Surface intérieure |
| Sable `--paper-2` | `#e8dfcc` | Fond |
| Ambre `--c-pli` | `#f4b740` | Accent, cercle du sceau |
| Ambre teinte `--c-pli-tint` | `#fcecc6` | Fond, aplats |
| Ambre profond `--c-pli-deep` | `#8f6200` | Accent fin sur fond clair |

Pas de blanc pur, pas de dégradé, pas d'ombre.

Composition, sur le modèle de `medias/jaquettes/beta.svg` et des maquettes :

- `viewBox="0 0 182 254"` et `preserveAspectRatio="xMidYMid slice"`, fond plein sur toute la surface ;
- **le sceau reste au centre** : laissez libre la zone centrale (cercle d'environ 30 de rayon autour de 91, 130), où la PWA pose le sceau Ambre ;
- motifs discrets en haut et en bas, pour qu'on lise le numéro du pli et le nom de la jaquette.

Profil SVG (vérifié par `valider`) : les couleurs sont admises, le reste est identique aux silhouettes.

- Éléments admis : `svg g defs clipPath mask path circle ellipse rect line polyline polygon`.
- **Aucun `<text>`** : les maquettes de `etymo.js` en emploient parfois (lettres grecques de Rosette) ; convertissez chaque glyphe en `path`, dessiné à la main ou vectorisé.
- Ni `title`, ni `desc`, ni commentaire, ni `style` en élément, ni `use`, ni `image`, ni `aria-hidden` ou `focusable` (retirez ceux des maquettes), ni lien externe. Un `id` éventuel reste neutre.
- 256 Kio au plus.

## 3. Déclarer

1. Fichier : `medias/jaquettes/<nom>.svg`, du nom de la jaquette en minuscules sans accent (`rosette.svg`, `routes-des-mots.svg`).
2. Objet : `.venv/bin/etymo-contenu nouvel-id med`, puis `ludique/medias/<id>.json` :

```json
{
  "id": "med_…",
  "type": "jaquette",
  "fichier": "jaquettes/<nom>.svg",
  "fascicule_id": "fsc_…",
  "note_interne": "Jaquette « … » (ADR 0016)."
}
```

Une seule jaquette par fascicule.

## 4. Contrôler

```sh
.venv/bin/etymo-contenu valider      # media.svg_invalide, media.double, media.manquant
.venv/bin/etymo-contenu visualiser   # onglet Cartes : la jaquette en tête de son fascicule
```

- Fichier seul :
  `.venv/bin/python -c "import sys; from etymo_contenu.medias import verifier_svg; print(verifier_svg(open(sys.argv[1],'rb').read(), 'jaquette') or 'conforme')" medias/jaquettes/<nom>.svg`
- Palette : vérifiez que chaque couleur du fichier figure dans le tableau ci-dessus (`grep -o '#[0-9a-fA-F]\{6\}' medias/jaquettes/<nom>.svg | sort -u`). Pour une revue complète de l'identité, le skill `visual-identity-check` du dépôt de conception s'applique aux couleurs.
- Revue visuelle dans `rapports/catalogue.html` : lisible en petit, sceau dégagé, rien qui évoque une brique ou une langue.
