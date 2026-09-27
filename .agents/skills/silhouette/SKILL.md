---
name: silhouette
description: Dessiner la silhouette SVG d'une carte (mot, préfixe, suffixe) ou d'une langue d'ÉtymoLogique, selon l'ADR 0023 et la grammaire de dessin de la charte, la déclarer dans ludique/medias/ et vérifier qu'elle passe le profil SVG (liste blanche anti-XSS, aucun texte). À utiliser dès qu'une carte ou une langue citée par un fascicule n'a pas de silhouette.
---

# Skill silhouette

Références, qui priment sur ce résumé : ADR 0023 (textures des cartes), 0025 et 0026 (médias), la **grammaire de dessin** de la charte (`docs/identite.html#textures-grammaire` du dépôt de conception) et le profil de `outils/etymo_contenu/medias.py`.

## 1. Trouver ce qui manque

```sh
.venv/bin/etymo-contenu valider      # « media.manquant » : cartes et langues sans silhouette, par fascicule
```

L'onglet « Cartes » de `rapports/catalogue.html` (`etymo-contenu visualiser`) montre en pointillé les cartes sans silhouette.

## 2. Choisir le sujet

- **Carte d'unité** : ce que dit son **sens** (la glose de la brique ou la définition du mot) : une double hélice pour *biologie*, une pousse pour *bio-*, un livre ouvert pour *-logie*, une chouette pour *-sophie*.
- **Carte de langue** : un objet de sa culture (rosace pour le français, aqueduc pour le latin, amphore pour le grec ancien). **Jamais** de drapeau, de parchemin ni de plume.
- **Un sujet par carte** : deux cartes proches par le sens (*géo-*, *géographie*, *géologie*) prennent des sujets distincts. Parcourez `medias/silhouettes/` pour ne pas répéter un sujet.
- **Seule exception** : le même mot dans plusieurs langues (unités reliées par `emprunt` ou `heritage`, avec le même sens, comme *philosophie*, *philosophia* et *φιλοσοφία*). Elles peuvent partager un fichier : un média par unité, qui cite le même `fichier`.
- La silhouette est décorative : elle ne dit ni le type, ni la rareté, ni le fascicule, ni la langue d'une carte de mot.

Proposez le sujet à l'utilisateur avant de dessiner s'il y a un doute.

## 3. Dessiner

Grammaire de la charte, en unités de la grille de 300 × 400 :

| Élément | Règle |
|---|---|
| Sujet | Un seul objet, être vivant ou paysage. Un groupe seulement s'il forme un tout. |
| Échelle | Hauteur de 60 à 90 %, largeur de 55 à 90 %. |
| Cadrage | Centre de masse dans la moitié basse, décalé à droite. Rien de massif dans la zone du mot (x 0–190, y 0–90). Aucun détail indispensable à moins de 15 unités des bords. |
| Débord | Au moins un bord franchi, à droite ou en bas. Jamais par la gauche, ni par le haut (sauf un fil ou une tige). |
| Inclinaison | 0° pour ce qui a une verticale naturelle, 3 à 12° pour un objet posé, 28 à 32° pour un objet long. |
| Point de vue | De face ou de profil strict. Ni perspective, ni volume, ni ombre portée. |
| Plein | Aplats pleins, sans contour. Trait plein de 9 à 24, fil de 6 au moins. |
| Évidements | Détails creusés dans la couleur du fond. Deux épaisseurs : structure 6–7, détail 4–5. Au moins 18 entre deux évidements parallèles. Trou : disque de rayon 4 au moins. |
| Finitions | `stroke-linecap` et `stroke-linejoin` `round`. Angles d'objet arrondis (6, 8–10, 14). Arêtes vives pour l'architecture et le relief seulement. Pointes seulement si le sujet l'impose. Béziers souples, symétrie exacte pour un objet tourné. |
| Détails | De 2 à 5 familles d'évidement. Un visage n'a qu'un trait pour l'œil, ni bouche ni pupille. |
| Poids | 3 ko au plus visés (le profil refuse au-delà de 32 Kio). |

Format exigé par le profil (vérifié par `valider`) :

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 300 400" preserveAspectRatio="xMidYMid slice"><path d="…" fill="currentColor"/><path d="…" fill="none" stroke-width="5" stroke-linecap="round" stroke-linejoin="round" style="stroke:var(--art-bg)"/></svg>
```

- Pleins en `fill="currentColor"`, évidements en `style="fill:var(--art-bg)"` ou `style="stroke:var(--art-bg)"`, sinon `none`. Aucune autre couleur, **aucune opacité** : la carte porte l'opacité (10 %, 8,5 % en grand).
- Éléments admis : `svg g defs clipPath mask path circle ellipse rect line polyline polygon`. Tout le reste est refusé : `text`, `title`, `desc`, `style`, `use`, `image`, `script`, `foreignObject`, `aria-*`, `on*`, liens externes, DTD, entités, **commentaires**.
- Aucun texte qui trahirait le mot : pas de `<title>`, pas de commentaire, pas d'`id` parlant (`id="c1"` plutôt que `id="helice"`).

Inspirez-vous des fichiers de `medias/silhouettes/` (par exemple `geo.svg`), exportés des maquettes du codex (`cardArt` dans `docs/assets/etymo.js` du dépôt de conception).

## 4. Déclarer

1. Fichier : `medias/silhouettes/<nom-de-travail>.svg`, un nom lisible en minuscules (`geologie.svg`, `langue-latin.svg`). Il n'est jamais publié : l'artefact renomme le fichier d'après l'empreinte de ses octets (ADR 0025).
2. Objet : `.venv/bin/etymo-contenu nouvel-id med`, puis `ludique/medias/<id>.json` :

```json
{
  "id": "med_…",
  "type": "silhouette",
  "fichier": "silhouettes/<nom-de-travail>.svg",
  "unite_id": "unt_…",
  "note_interne": "Sujet : … (ADR 0023)."
}
```

Pour une langue, remplacez `unite_id` par `langue_id`. Une seule silhouette par unité et par langue. La couche linguistique ne cite jamais un média.

## 5. Contrôler

```sh
.venv/bin/etymo-contenu valider      # profil SVG (media.svg_invalide), doublon (media.double), fichier absent ou orphelin
.venv/bin/etymo-contenu visualiser   # onglet Cartes : la silhouette sur sa carte
```

- Pour tester un fichier seul :
  `.venv/bin/python -c "import sys; from etymo_contenu.medias import verifier_svg; print(verifier_svg(open(sys.argv[1],'rb').read(), 'silhouette') or 'conforme')" medias/silhouettes/<nom>.svg`
- Revue visuelle : ouvrez `rapports/catalogue.html` (si des outils de navigateur sont disponibles, faites une capture de l'onglet Cartes) et relisez la grammaire ci-dessus, en grille comme en grand.
- N'élargissez jamais le profil de `medias.py` pour faire passer un dessin. Un ajout au profil demande un test qui prouve qu'il est sûr.
