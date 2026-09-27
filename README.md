# ÉtymoLogique — Contenu

Dépôt éditorial **privé** du catalogue d’ÉtymoLogique ([ADR 0024](https://github.com/lunik/EtymoLogique/blob/master/docs/adr/0024-stockage-du-catalogue-et-des-donnees-joueur.md)). On y trouve les fichiers d’édition : langues, unités lexicales, compositions, relations, sources, briques, recettes et fascicules. Les brouillons et les notes internes y vivent aussi. Rien de ce dépôt n’est publié tel quel : seul l’artefact compilé et validé le sera.

## Place dans le pipeline

Le contenu suit le pipeline de l’[ADR 0005](https://github.com/lunik/EtymoLogique/blob/master/docs/adr/0005-pipeline-de-contenu.md). Ce dépôt couvre pour l’instant les deux premières étapes.

| Étape | État |
|---|---|
| 1. Édition dans un format structuré soumis à un schéma | **en place** : JSON, un fichier par objet, [JSON Schema](schemas/) |
| 2. Validation locale : types, références, identifiants, vocabulaires contrôlés | **en place** : `etymo-contenu valider` |
| 3. Revue linguistique, 4. validation ludique | à venir : état `etat_relecture` déjà porté par chaque fait |
| 5. Analyse globale : atteignabilité, fermeture, pools de plis | à venir |
| 6. Compilation en artefact à hash de contenu, 7. publication, 8. migrations | à venir |

Le schéma logique est décrit dans le [modèle de données](https://github.com/lunik/EtymoLogique/blob/master/docs/modele-donnees.md) et l’[ADR 0013](https://github.com/lunik/EtymoLogique/blob/master/docs/adr/0013-schema-des-briques.md).

## Arborescence

| Chemin | Contenu | Schéma |
|---|---|---|
| `referentiels/langues/` | Langues : code interne, filiation, écriture par défaut, noms par locale | `langue` |
| `referentiels/ecritures/` | Systèmes d’écriture (ISO 15924, direction) | `ecriture` |
| `referentiels/conventions/` | Conventions de translittération | `convention` |
| `referentiels/types-relation.json` | Vocabulaire contrôlé des relations étymologiques | `vocabulaire` |
| `referentiels/types-transformation.json` | Vocabulaire contrôlé des transformations | `vocabulaire` |
| `sources/` | Sources de référence et de repérage (ADR 0014) | `source` |
| `unites/` | Mots, préfixes et suffixes, avec leurs formes, translittérations et sens | `unite` |
| `compositions/` | Analyses d’un mot en parties ordonnées, transformations, sens littéral | `composition` |
| `relations/` | Relations étymologiques entre unités | `relation` |
| `ludique/briques/` | Briques : unité, forme et sens affichés, rareté | `brique` |
| `ludique/recettes/` | Recettes : composition jouée, emplacements, briques acceptées | `recette` |
| `ludique/fascicules/` | Fascicules : numéro, parution, unités résultat et ingrédient | `fascicule` |
| `ludique/exclusions/` | Combinaisons attestées volontairement écartées | `exclusion` |
| `schemas/` | JSON Schema (draft 2020-12) de chaque type d’objet | — |
| `outils/etymo_contenu/` | Validateur et générateur d’identifiants | — |
| `tests/` | Un test par règle de validation | — |
| `imports/` | Imports datés du Wiktionnaire : **ignoré par Git** | — |
| `rapports/` | Rapports de validation générés : **ignoré par Git** | — |

### Conventions d’édition

- **Un fichier par objet**, nommé d’après son identifiant : `unites/unt_7k2m9qxa.json`. Formes, sens et transformations vivent dans le fichier de leur unité ou de leur composition, avec leur propre identifiant.
- **Identifiants opaques et stables** : préfixe du type, puis 8 caractères base32 aléatoires. Ils ne contiennent jamais de graphie. Générez-les avec `etymo-contenu nouvel-id <préfixe>`.
- **Chaque affirmation est un fait** : unités, formes, sens, compositions et relations portent un bloc `fait` (confiance, état de relecture, sources, `note_interne`, `note_simplification`).
- **Libellés par locale** : `libelles.fr` est obligatoire partout. C’est la locale de référence.
- **Formes sans trait d’union** : `géo`, pas `géo-`. La position vient de la nature de l’unité.
- **`cle_normalisee`** est la forme NFC du texte. Le texte édité n’est jamais normalisé en silence (ADR 0006).
- **Segments** : comptés en points de code, sur la forme NFC du mot résultat, avec la fin exclue. La question des graphèmes reste ouverte (modèle de données).
- **Fascicule de test** (`"test": true`) : il éprouve le schéma et n’est jamais publié. Seul un fascicule de test peut porter le numéro 0.

## Commandes

Il faut Python 3.11 ou plus récent.

```sh
python3 -m venv .venv && .venv/bin/pip install -e '.[dev]'

.venv/bin/etymo-contenu valider                  # schémas et contraintes d'intégrité
.venv/bin/etymo-contenu valider --publication    # plus les exigences de publication
.venv/bin/etymo-contenu valider --json           # rapport machine
.venv/bin/etymo-contenu valider --strict         # les avertissements font échouer
.venv/bin/etymo-contenu nouvel-id unt -n 3       # trois identifiants d'unité inédits
.venv/bin/python -m pytest -q                    # tests du validateur
```

La validation échoue (code 1) dès qu’une erreur est trouvée. La CI lance les tests et `valider` à chaque push.

### Ce que vérifie `valider`

- **Fichiers** : JSON lisible, aucune clé en double, un schéma respecté, un nom de fichier égal à l’identifiant, un préfixe cohérent avec le dossier, des identifiants uniques dans tout le dépôt.
- **Références** : chaque identifiant cité existe.
- **Langues** : filiation sans cycle, codes uniques.
- **Unités** : exactement une forme canonique, dans l’écriture par défaut de la langue ; formes sans trait d’union ; `cle_normalisee` en NFC ; translittération dans une convention de la bonne écriture ; homographes distingués par la glose **et** par l’étymon.
- **Compositions** : le résultat est un mot ; les parties sont au moins deux, de rangs continus, et leurs formes et sens appartiennent à leur unité ; pas de préfixe en dernier ni de suffixe en premier ; segments dans les bornes et sans chevauchement ; une seule analyse préférée ; aucun cycle ; transformations au vocabulaire contrôlé.
- **Relations** : source et cible différentes, sans doublon ni cycle ; un emprunt relie deux langues ; un héritage suit la filiation.
- **Briques** : une par unité, qui affiche la forme canonique et un sens glosé de son unité.
- **Recettes** : les emplacements suivent les parties et acceptent la brique de chaque partie ; aucune suite de briques ne mène à deux recettes ; aucune analyse non retenue ; une définition pour le mot résultat.
- **Fascicules** : numéros uniques ; un mot n’est le résultat que d’un fascicule ; un résultat est un mot avec une recette ; un ingrédient a une brique ; autonomie (tout ingrédient est déclaré, et publié dans ce fascicule ou un précédent) ; en avertissement : taille de 20 à 30 mots (ADR 0022), ingrédients inutiles, chronologie.
- **Exclusions** : aucune ne contredit une recette.
- **Avec `--publication`**, pour chaque fascicule qui n’est pas de test : tout fait publié est `relu`, cite au moins une source de rôle `reference` (deux pour un fait `discutee`) avec l’entrée consultée, et n’est pas `non_retenue`. Le Wiktionnaire seul ne suffit jamais (ADR 0014). Chaque exclusion est relue.

## Sources et licences

- Le **Wiktionnaire** sert au repérage seulement : **aucun de ses textes n’est copié** (ADR 0014). Ses imports datés restent dans `imports/`, hors de Git.
- Les **références savantes** (TLFi, Bailly, Gaffiot…) sont citées, jamais recopiées. Les gloses, définitions et explications sont rédigées par l’équipe.
- Le jeu actuel (fascicule 0) reprend les **exemples travaillés** du modèle de données. Tout y est en `brouillon`, à vérifier avant toute publication.

| Périmètre | Licence |
|---|---|
| Outils : scripts, schémas, workflows | [GNU AGPL 3.0 ou ultérieure](LICENSE) |
| Contenu : données du catalogue (gloses, définitions, explications, relations) | [Creative Commons BY-SA 4.0](LICENSE-CONTENT) |
| Nom « ÉtymoLogique », logo et marque | Réservés, hors de ces licences |

© 2026 Guillaume MARTINEZ ([ADR 0017](https://github.com/lunik/EtymoLogique/blob/master/docs/adr/0017-licences.md)).
