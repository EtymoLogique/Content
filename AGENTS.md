# AGENTS.md — ÉtymoLogique, dépôt de contenu

Ce dépôt est le **dépôt éditorial privé** du catalogue ([ADR 0024](https://github.com/lunik/EtymoLogique/blob/master/docs/adr/0024-stockage-du-catalogue-et-des-donnees-joueur.md)). Il contient les fichiers d’édition (JSON, un fichier par objet), leurs schémas et le validateur local. Le dossier de conception vit dans le dépôt [EtymoLogique](https://github.com/lunik/EtymoLogique) : ADR, modèle de données, game design.

Tout le contenu est **en français**. Rédigez en français la documentation, les libellés et les messages du validateur. Seuls les messages de commit sont en anglais.

## Sources de vérité

1. **ADR acceptés** du dépôt de conception (`docs/adr/`), en particulier 0002, 0003, 0005, 0006, 0009, 0013, 0014, 0017, 0022 et 0024.
2. **Modèle de données** (`docs/modele-donnees.md`) : tables, cardinalités et « Contraintes d’intégrité ». Les schémas et `outils/etymo_contenu/controles.py` en sont la traduction exécutable.
3. **`docs/game-design.md`** : vocabulaire et règles de jeu.

Si le dépôt s’écarte du modèle de données, ou si deux sources se contredisent, signalez l’écart au lieu de choisir en silence.

## Carte du dépôt

| Chemin | Rôle |
|---|---|
| `referentiels/` | Langues, écritures, conventions de translittération, vocabulaires contrôlés. |
| `sources/`, `unites/`, `compositions/`, `relations/` | Couche linguistique : les faits. |
| `ludique/` | Couche ludique : briques, recettes, fascicules, exclusions, médias. Elle référence la couche linguistique, jamais l’inverse (ADR 0009). |
| `medias/` | SVG bruts des silhouettes et des jaquettes, sous des noms de travail lisibles, déclarés par `ludique/medias/` (ADR 0026). |
| `schemas/` | JSON Schema 2020-12, `communs.schema.json` pour les définitions partagées (`fait`, `libelles`, identifiants). |
| `outils/etymo_contenu/` | `chargement.py` (lecture et schémas), `controles.py` (contraintes d’intégrité), `medias.py` (profil SVG, langues citées), `compilation.py` (artefact en hash de contenu), `visualisation.py` et `visualisation.html` (page locale du catalogue, jamais publiée), `rapport.py`, `ids.py`, `cli.py`. |
| `tests/` | `conftest.py` copie le dépôt dans un dossier temporaire ; chaque test casse une seule règle. |
| `.agents/skills/` | Skills des agents (`.github/skills` est un lien symbolique vers ce dossier). |
| `imports/`, `rapports/`, `artefact/` | Ignorés par Git. |

## Règles

- **Identifiants opaques** : `etymo-contenu nouvel-id <préfixe>`. Ne réutilisez jamais un identifiant, n’y mettez jamais de graphie, ne renommez jamais un objet publié : un changement de nature ou une suppression passe par une migration (ADR 0005).
- **Aucun texte copié** du Wiktionnaire ni d’une référence savante. Rédigez les gloses, définitions et explications. N’inventez ni entrée, ni page, ni édition de source : laissez `localisation` à `null` et décrivez dans `note_interne` ce qu’il reste à vérifier.
- **Un nouveau fait naît `brouillon`.** Seule une relecture humaine le passe à `relu`.
- **Médias** : tout SVG passe le profil de `medias.py`, une liste blanche contre le XSS. Pas de texte, de titre ni de commentaire : ils trahiraient le mot. N’élargissez le profil qu’avec un test qui prouve que l’ajout est sûr.
- **`note_interne`** n’est jamais publiée. N’y mettez ni donnée personnelle ni secret.
- **Une brique par unité.** Un allomorphe est une forme de la même unité, employée par une transformation `allomorphie`.
- **La langue n’est jamais une brique**, et un étymon n’a pas de carte propre (ADR 0013).
- **Vocabulaire du jeu** : « pli » (jamais « pack »), « codex », « exemplaire », « réserve », « brique », « table de fusion », « fascicule ». Langues en toutes lettres dans les libellés (« grec ancien »).
- **Typographie française** dans les libellés : guillemets « », apostrophe ’, espace avant `: ; ? !`.

- **Branches** (ADR 0028) : `develop` publie en hors-prod, en mode `--essai` ; `master` publie en prod, avec les exigences de publication. Une branche de feature ne publie rien.
- **L’artefact n’emporte jamais** `note_interne`, `etat_relecture` ni un fait `non_retenue`. Tout nouveau champ privé s’ajoute à `CHAMPS_PRIVES` dans `compilation.py`, avec son test.

## Skills

| Skill | Quand l'utiliser |
|---|---|
| [`nouveau-fascicule`](.agents/skills/nouveau-fascicule/SKILL.md) | Ouvrir un fascicule : numéro, date provisoire, plan de 20 à 30 mots, déclaration des unités. |
| [`nouveau-mot`](.agents/skills/nouveau-mot/SKILL.md) | Repérer un mot, le vérifier dans une source de référence, puis saisir sa chaîne jusqu'au fascicule. |
| [`silhouette`](.agents/skills/silhouette/SKILL.md) | Dessiner et déclarer la silhouette d'une carte ou d'une langue. |
| [`jaquette`](.agents/skills/jaquette/SKILL.md) | Dessiner et déclarer la jaquette d'un fascicule. |
| [`preparer-publication`](.agents/skills/preparer-publication/SKILL.md) | Lancer tous les contrôles d'un fascicule, planifier sa date de parution, vérifier la compilation prod. |

## Modifier le contenu ou l’outillage

- Nouvelle règle de validation : ajoutez-la dans `controles.py` avec un code `domaine.regle` stable, un message en français, un test dans `tests/test_validation.py` et une ligne dans le README (« Ce que vérifie `valider` »).
- Changement de schéma : mettez à jour le schéma, le jeu d’exemples, les tests et le README ensemble. Si le changement modifie le modèle de données, il faut d’abord un ADR dans le dépôt de conception.
- Avant de conclure :

```sh
.venv/bin/python -m pytest -q
.venv/bin/etymo-contenu valider
```
