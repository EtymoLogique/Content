---
name: silhouette
description: Dessiner, corriger ou revoir la silhouette SVG d'une carte (mot, préfixe, suffixe) ou d'une langue d'ÉtymoLogique, selon l'ADR 0023 et la grammaire de dessin de la charte, la déclarer dans ludique/medias/ et la vérifier sur planche rendue (rognage réel de la carte, profil SVG anti-XSS, aucun texte). À utiliser dès qu'une carte ou une langue citée par un fascicule n'a pas de silhouette, ou qu'une liste de retouches de silhouettes arrive.
---

# Skill silhouette

Références, qui priment sur ce résumé : ADR 0023 (textures des cartes), 0025 et 0026 (médias), la **grammaire de dessin** de la charte (`docs/identite.html#textures-grammaire` du dépôt de conception), le profil de `outils/src/medias.rs` et le rendu réel des cartes (`.card-art` dans `app/web/src/codex.css`).

## 1. Trouver ce qui manque

```sh
cargo run -q --manifest-path outils/Cargo.toml -- valider      # « media.manquant » : cartes et langues sans silhouette, par fascicule
```

L'onglet « Cartes » de `rapports/catalogue.html` (`etymo-contenu visualiser`) montre en pointillé les cartes sans silhouette.

## 2. Choisir le sujet

Le test : **un joueur reconnaît l'objet en une seconde, et l'objet évoque le sens**. Un dessin juste mais illisible est raté.

- **Carte d'unité** : ce que dit son **sens** (la glose de la brique ou la définition du mot) : une double hélice pour *biologie*, une pousse pour *bio-*, un livre ouvert pour *-logie*, une chouette pour *-sophie*.
- **Carte de langue** : un objet de sa culture (rosace pour le français, aqueduc pour le latin, amphore pour le grec ancien). **Jamais** de drapeau, de parchemin ni de plume.
- **Préférez une scène concrète et explicite à un symbole abstrait.** Exemples retenus : vampire pour *photophobie*, cabine UV pour *photothérapie*, présentateur devant un écran pour *démo-*, soleil rayonnant pour *photo-*, batterie pour *autonomie*, frise pour *chronologie*, patient sur le divan pour *psychologue*, signature et stylo pour *autographe*, ver de terre sous un sol simple pour *géophage*, bûche creusée d'un termite pour *xylophage*, neurone dont les dendrites sont des pousses pour *neurobiologie*.
- **Un suffixe, un rôle**, pour distinguer les familles (*calligraphe*, *calligraphie*, *calligraphiste* ; *cardiogramme*, *cardiographe*, *cardiographie*) :
  - *-graphe* : l'outil ou la main qui écrit. *-graphie* : la page, le résultat. *-gramme* : la trace (bande, courbe).
  - *-logue*, *-logiste*, *-iste* : la personne ou ses outils. *-logie* : la discipline, son objet.
  - *-mètre* : l'instrument de mesure. *-scope* : l'instrument d'observation. *-phobie* : une personne face à ce qu'elle craint, ou la chose crainte. *-thérapie* : le soin.
- **Un sujet par carte** : deux cartes proches par le sens (*géo-*, *géographie*, *géologie*) prennent des sujets distincts. Parcourez `medias/silhouettes/` (ou faites une planche de la famille) pour ne pas répéter un sujet ni un motif (deux micros, deux roues, deux anneaux).
- **Seule exception** : le même mot dans plusieurs langues (unités reliées par `emprunt` ou `heritage`, avec le même sens, comme *philosophie*, *philosophia* et *φιλοσοφία*). Elles peuvent partager un fichier : un média par unité, qui cite le même `fichier`.
- La silhouette est décorative : elle ne dit ni le type, ni la rareté, ni le fascicule, ni la langue d'une carte de mot.

**Formes à éviter**, relevées en revue :
- Un disque à évidements concentriques se lit comme une **cible**. Un cercle barré se lit comme un panneau **interdit**, et le *a-* privatif garde sa cage vide. Un disque entamé se lit comme un **Pac-Man**.
- Des traits courts et parallèles sur un objet se lisent comme des **lettres** (« VIT », un « M ») ou comme un **bandeau**. Un reflet ou un trait isolé sous un objet passe pour un défaut.
- Un visage de face rogné par le bord, un visage grimaçant ou à l'air bête. Préférez le profil, un trait pour l'œil.
- Un animal approximatif (éléphant qui tient du cochon, cheval informe) : redessinez sa silhouette caractéristique (trompe, oreilles, crinière) ou changez d'animal.

Proposez le sujet à l'utilisateur avant de dessiner s'il y a un doute.

## 3. Dessiner

Dessinez directement dans la grille de 300 × 400, à l'échelle de la carte. N'agrandissez jamais un petit pictogramme : le rendu fait « dessin d'enfant ». Les dessins d'origine (`geo`, `bio`, `biologie`, `logie`, `sophie`, `mener`, `a-privatif`) sont l'étalon : grands, francs, courbes de Bézier organiques, vrais débords.

Grammaire de la charte, en unités de la grille :

| Élément | Règle |
|---|---|
| Sujet | Un seul objet, être vivant ou paysage. Un groupe seulement s'il forme un tout (une scène simple). |
| Échelle | Hauteur de 60 à 90 %, largeur de 55 à 90 %. |
| Cadrage | Centre de masse dans la moitié basse, décalé à droite. Rien de massif dans la zone du mot (x 0–190, y 0–90). |
| **Zone sûre** | **Tout détail lisible entre y 45 et y 355, et entre x 15 et x 285.** Voir « Rognage réel » ci-dessous. |
| Débord | Au moins un bord franchi, à droite ou en bas, par la masse seulement (sol, corps, manche). Jamais par la gauche, ni par le haut (sauf un fil ou une tige). |
| Inclinaison | 0° pour ce qui a une verticale naturelle, 3 à 12° pour un objet posé, 28 à 32° pour un objet long. Tournez autour du centre de l'objet, puis revérifiez le cadrage. |
| Point de vue | De face ou de profil strict. Ni perspective, ni volume, ni ombre portée. |
| Plein | Aplats pleins, sans contour. Trait plein de 9 à 24, fil de 6 au moins. |
| Évidements | Détails creusés dans la couleur du fond, **toujours sur un plein** : sur le fond, ils sont invisibles. Deux épaisseurs : structure 6–7, détail 4–5. Au moins 18 entre deux évidements parallèles. Trou : disque de rayon 4 au moins. Un liseré évidé sépare deux pleins qui se touchent (combiné et socle, pinceau et palette). |
| Finitions | `stroke-linecap` et `stroke-linejoin` `round`. Angles d'objet arrondis (6, 8–10, 14). Arêtes vives pour l'architecture et le relief seulement. Pointes seulement si le sujet l'impose. Béziers souples, symétrie exacte pour un objet tourné. |
| Détails | De 2 à 5 familles d'évidement. Un visage n'a qu'un trait pour l'œil, ni bouche ni pupille. Pas de reflet décoratif. |
| Poids | 3 ko au plus visés (le profil refuse au-delà de 32 Kio). |

**Rognage réel.** Toutes les silhouettes sont en `preserveAspectRatio="xMidYMid slice"`, et la carte rogne plus que les 15 unités de marge de la charte :
- **Mini-carte** (3:4, illustration à `inset: 25px 0 0`) : elle perd environ 3750 / largeur unités en haut et en bas. Pour une grille de 132 px, c'est 28 unités. Pour la grille mobile de 92 px, c'est 41 unités, donc la zone visible va de y ≈ 41 à y ≈ 359.
- **Grande carte** (5:7, `inset: 0`) : elle perd environ 7 unités à gauche et à droite.

Format exigé par le profil (vérifié par `valider`) :

```xml
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 300 400" preserveAspectRatio="xMidYMid slice"><path d="…" fill="currentColor"/><path d="…" fill="none" stroke-width="5" stroke-linecap="round" stroke-linejoin="round" style="stroke:var(--art-bg)"/></svg>
```

- Pleins en `fill="currentColor"`, évidements en `style="fill:var(--art-bg)"` ou `style="stroke:var(--art-bg)"`, sinon `none`. Aucune autre couleur, **aucune opacité** : la carte porte l'opacité (10 %, 8,5 % en grand).
- Éléments admis : `svg g defs clipPath mask path circle ellipse rect line polyline polygon`. Tout le reste est refusé : `text`, `title`, `desc`, `style`, `use`, `image`, `script`, `foreignObject`, `aria-*`, `on*`, liens externes, DTD, entités, **commentaires**.
- Aucun texte qui trahirait le mot : pas de `<title>`, pas de commentaire, pas d'`id` parlant (`id="c1"` plutôt que `id="helice"`).
- Pour un lot, écrivez les SVG avec un petit script Python dans le dossier temporaire (constantes `F`, `bg(w)`, `st(w)` pour les attributs). Un recadrage se fait en enveloppant le contenu dans `<g transform="translate(…) scale(…)">`, mais **revalidez le XML** après toute retouche scriptée par `sed` ou par expression régulière : un `<` ou un `</g>` en trop casse le fichier sans bruit.

## 4. Déclarer

1. Fichier : `medias/silhouettes/<nom-de-travail>.svg`, un nom lisible en minuscules (`geologie.svg`, `langue-latin.svg`, `graphe-suffixe.svg`, `re-prefixe.svg`). Il n'est jamais publié : l'artefact renomme le fichier d'après l'empreinte de ses octets (ADR 0025).
2. Objet : `cargo run -q --manifest-path outils/Cargo.toml -- nouvel-id med`, puis `ludique/medias/<id>.json` :

```json
{
  "id": "med_…",
  "type": "silhouette",
  "fichier": "silhouettes/<nom-de-travail>.svg",
  "unite_id": "unt_…",
  "note_interne": "Sujet : … (ADR 0023)."
}
```

Pour une langue, remplacez `unite_id` par `langue_id`. Une seule silhouette par unité et par langue. La couche linguistique ne cite jamais un média. **Quand le sujet change, mettez à jour la `note_interne`** du média (`grep -l <nom-de-travail> ludique/medias/*.json`).

## 5. Contrôler

Ne déclarez jamais un dessin bon sans l'avoir **vu rendu**. Lire le code SVG ne suffit pas.

1. **Planche avant.** Rendez l'existant, avec une ou deux silhouettes de référence (`geo`, ou une voisine réussie de la même famille) :

   ```sh
   .agents/skills/silhouette/rendre.sh $TMPDIR/avant.png geo cardiopathie democratie
   ```

   Le script valide le XML (`xmllint`), rend chaque silhouette en 300 × 400 avec Inkscape, couvre de rouge les bandes rognées sur mobile (y < 41 et y > 359), encadre la zone du mot et assemble une rangée. Lisez la planche (outil de lecture d'image) avant de dessiner.

2. **Dessiner, rendre, relire, en boucle**, par lots de 4 à 12 cartes. À chaque planche, notez pour chaque carte ce qui ne va pas, puis corrigez et refaites la planche. Grille de relecture :
   - **Lisible** : l'objet se nomme d'un coup d'œil, sans connaître le mot. Aucune forme du § 2 « Formes à éviter ».
   - **Sens** : l'objet évoque la glose, et se distingue des cartes de la même famille (rendez la famille côte à côte au besoin).
   - **Cadrage** : aucun détail utile dans une bande rouge ni coupé à droite, rien de massif dans le cadre du mot, rien ne sort par la gauche ou le haut.
   - **Évidements** : chacun est visible, donc posé sur un plein, et assez épais. Aucun ne forme une lettre.
   - **Échelle** : de 60 à 90 % de la hauteur, avec un vrai débord. Pas de petit objet perdu au milieu.

3. **Validation** du profil et du catalogue, une fois les planches propres :

   ```sh
   cargo run -q --manifest-path outils/Cargo.toml -- valider      # profil SVG (media.svg_invalide), doublon (media.double), fichier absent ou orphelin
   ```

4. **Rendu en contexte**, facultatif : `cargo run -q --manifest-path outils/Cargo.toml -- visualiser`, onglet Cartes de `rapports/catalogue.html`, ou l'app en grille mobile.

5. **Compte rendu** : pour chaque carte, le sujet en une ligne et ce qui a changé. Dites franchement ce qui reste faible et ce qui n'a pas été vérifié (planche non refaite après la dernière retouche, `valider` non lancé).

N'élargissez jamais le profil de `outils/src/medias.rs` pour faire passer un dessin. Un ajout au profil demande un test qui prouve qu'il est sûr.
