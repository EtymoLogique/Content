---
name: nouveau-mot
description: Repérer un mot candidat pour un fascicule d'ÉtymoLogique (fermeture, mots croisés, import du Wiktionnaire), le vérifier dans une source de référence, puis saisir toute sa chaîne dans le dépôt de contenu, dans l'ordre unités → composition → relations → briques → recette → fascicule. À utiliser pour ajouter un mot, un préfixe ou un suffixe au catalogue.
---

# Skill nouveau-mot

Références, qui priment sur ce résumé : ADR 0002, 0003 (recettes), 0006 (multilingue, NFC), 0009 (couches linguistique et ludique), 0013 (briques), 0014 (sources, fermeture), `docs/modele-donnees.md` du dépôt de conception, `AGENTS.md` et le README (« Conventions d'édition », « Ce que vérifie `valider` »). Les schémas de `schemas/` font foi pour la structure.

## 1. Repérer un candidat

Partez du plan du fascicule (skill `nouveau-fascicule`) ou d'une demande précise. Sources de candidats :

- **Mots croisés** : pour chaque nouvel affixe, parcourez les briques déjà publiées (`ludique/briques/`, fascicules précédents) et cherchez les mots attestés qu'il forme avec elles, dans les deux sens et avec les mots déjà publiés comme ingrédients (*biologie* + *-iste*).
- **Fermeture** : toute suite de briques publiées qui forme un mot attesté doit être publiée avec sa recette, ou écartée par une exclusion.
- **Import du Wiktionnaire** : s'il existe un import daté dans `imports/` (ignoré par Git), servez-vous-en pour **repérer** seulement. N'en copiez aucun texte, ni glose, ni définition, ni étymologie.

Une simple ressemblance de chaînes ne prouve jamais une composition (ADR 0003). Présentez les candidats à l'utilisateur, avec pour chacun l'analyse proposée et ce qu'il reste à vérifier, et laissez-le choisir.

Avant de saisir, vérifiez que le mot n'existe pas déjà : `grep -rl '"texte": "<forme>"' unites/`, ou la recherche de `rapports/catalogue.html` (`etymo-contenu visualiser`).

## 2. Vérifier dans une source de référence

| Langue | Sources de référence (ADR 0014) |
|---|---|
| Français | TLFi, Dictionnaire de l'Académie (9ᵉ édition), Littré |
| Latin | Gaffiot, Ernout-Meillet |
| Grec ancien | Bailly, LSJ, Chantraine |

- Réutilisez une source de `sources/` si elle existe. Sinon, créez-la (`nouvel-id src`), avec `role: reference` ou `reperage`, l'édition et la licence (« Consultation seulement : cité, jamais copié »).
- **Le Wiktionnaire seul ne suffit jamais.** Chaque fait doit citer au moins une source `reference`, deux si la confiance est `discutee`.
- `localisation` : l'entrée ou la page **réellement consultée**. Si vous ne l'avez pas consultée vous-même, laissez `null` et écrivez dans `note_interne` ce qu'il reste à vérifier. N'inventez jamais d'entrée, de page ni d'édition.
- `confiance` : `etablie` (une référence l'affirme), `probable` (hypothèse d'une référence), `discutee` (deux références en désaccord : enregistrez chaque analyse), `non_retenue` (jamais publiée comme solution).

## 3. Saisir la chaîne

Générez les identifiants d'un coup : `cargo run -q --manifest-path outils/Cargo.toml -- nouvel-id unt -n 3`, puis `frm`, `sns`, `cmp`, `trf`, `rel`, `brq`, `rct` selon le besoin. Ne mettez jamais de graphie dans un identifiant, n'en réutilisez jamais un. Chaque affirmation porte un bloc `fait` à l'état **`brouillon`** : seule une relecture humaine le passe à `relu`.

Bloc `fait` type :

```json
{
  "confiance": "etablie",
  "etat_relecture": "brouillon",
  "sources": [{ "source_id": "src_…", "localisation": null }],
  "note_interne": "À vérifier : entrée du TLFi à préciser."
}
```

### 3.1 Unités (`unites/unt_….json`)

D'abord les parties et les étymons qui manquent, puis le mot.

- `nature` : `mot` (avec `mot.categorie`), `prefixe` ou `suffixe` (avec `affixe.statut` : `affixe` ou `element_de_composition`). `langue_id` pris dans `referentiels/langues/`.
- `formes` : **exactement une** `canonique`, dans l'écriture par défaut de la langue. Pas de trait d'union (`géo`, pas `géo-`). `cle_normalisee` = forme NFC du texte (`unicodedata.normalize("NFC", texte)`), sans autre transformation. Translittération dans une convention de `referentiels/conventions/` de la bonne écriture pour le grec. Un allomorphe (`role: allomorphe`, avec `contexte`) est une forme de la même unité, jamais une autre unité.
- `sens` : `rang` à partir de 1. `libelles.fr.glose` courte, affichée sur la brique (« terre »). Pour un mot résultat d'une recette, `libelles.fr.definition` en une phrase est obligatoire (`recette.definition`). **Rédigez** gloses et définitions vous-même.
- Homographes : deux unités de même forme doivent différer par la glose **et** par l'étymon.

### 3.2 Composition (`compositions/cmp_….json`)

- `mot_id`, `forme_resultat_id` (forme canonique du mot), `type_analyse` (`synchronique` ou `historique`), `preferee` (une seule analyse préférée par mot).
- `parties` : au moins deux, `rang` continus à partir de 1, avec `unite_id`, `forme_id` et `sens_id` **de cette unité**. Pas de préfixe en dernier, pas de suffixe en premier.
- Segments facultatifs : `segment_debut` / `segment_fin`, en points de code sur la forme NFC du mot, fin exclue, sans chevauchement.
- `transformations` : `type_code` dans `referentiels/types-transformation.json` (`elision`, `voyelle_de_liaison`, `allomorphie`, `adaptation_graphie`, `adaptation_phonetique`, `emprunt_intermediaire`), `rang_partie` (ou `null`), `ordre`, et `libelles.fr.texte` rédigé.
- `libelles.fr` : `sens_litteral` (« étude de la vie ») et `explication_courte`, rédigés. Le schéma ne les exige pas, mais une carte sans eux n'explique rien au joueur.

### 3.3 Relations (`relations/rel_….json`)

- `unite_source_id` (l'étymon), `unite_cible_id`, `type_code` dans `referentiels/types-relation.json` : `heritage` (suit la filiation des langues), `emprunt` (entre deux langues), `derivation`, `forme_de_composition`, `adaptation`, `apparentement`.
- Pas de doublon, pas de cycle, source et cible différentes. Un étymon n'a pas de carte propre (ADR 0013).

### 3.4 Couche ludique

Elle référence la couche linguistique, jamais l'inverse.

- **Brique** (`ludique/briques/brq_….json`) : **une par unité** qui sert d'ingrédient, avec la forme canonique (`forme_affichee_id`) et un sens glosé (`sens_affiche_id`) de cette unité. `rarete` : `commune`, `peu_commune`, `rare` ou `legendaire`, provisoire, à signaler dans `note_interne` (paramètre d'équilibrage, ADR 0009). La langue n'est jamais une brique.
- **Recette** (`ludique/recettes/rct_….json`) : `composition_id`, puis un emplacement par partie (même `rang`), dont `briques_acceptees` contient la brique de la partie. Aucune suite de briques ne doit mener à deux recettes. Jamais sur une analyse `non_retenue`.
- **Exclusion** (`ludique/exclusions/exc_….json`) : pour une combinaison attestée écartée, avec `raison` et `etat_relecture: brouillon`. Elle ne doit contredire aucune recette.

### 3.5 Fascicule

Ajoutez au fascicule le mot en `resultat`, et chaque partie en `ingredient` si elle n'y est pas déjà (autonomie). Un mot n'est le résultat que d'un seul fascicule.

Chaque `resultat` porte une `difficulte` (ADR 0047), jamais un `ingredient` : `facile` (sens actuel proche du sens littéral, recette sans transformation), `moyen` (un seul écart : une transformation, ou un sens qui a dérivé), `difficile` (les deux). Visez un tiers de chaque niveau par fascicule, à un mot près, légendaires exclus (avertissement `fascicule.tiers`).

## 4. Contrôler

```sh
cargo run -q --manifest-path outils/Cargo.toml -- valider
cargo run -q --manifest-path outils/Cargo.toml -- visualiser        # relire la fiche du mot dans rapports/catalogue.html
cargo test --manifest-path outils/Cargo.toml
```

- Aucune erreur n'est acceptable. Corrigez sans élargir une règle.
- Typographie française dans tous les libellés : guillemets « », apostrophe ’, espace avant `: ; ? !`. Langues en toutes lettres (« grec ancien »). Vocabulaire du jeu : brique, table de fusion, fascicule, pli.
- Signalez à l'utilisateur les `note_interne` laissées (localisations à renseigner, raretés provisoires) et les silhouettes à dessiner (skill `silhouette`).
