---
name: nouveau-mot
description: Repérer un mot candidat pour un fascicule d'ÉtymoLogique (fermeture, mots croisés), le vérifier dans Wikidata (seule source de vérité), puis saisir toute sa chaîne dans le dépôt de contenu, dans l'ordre unités → composition → relations → briques → recette → fascicule. À utiliser pour ajouter un mot, un préfixe ou un suffixe au catalogue.
---

# Skill nouveau-mot

Références, qui priment sur ce résumé : ADR 0002, 0003 (recettes), 0006 (multilingue, NFC), 0009 (couches linguistique et ludique), 0013 (briques), 0014 (fermeture) et 0060 (sources), `docs/modele-donnees.md` du dépôt de conception, `AGENTS.md` et le README (« Conventions d'édition », « Ce que vérifie `valider` »). Les schémas de `schemas/` font foi pour la structure.

## 1. Repérer un candidat

Partez du plan du fascicule (skill `nouveau-fascicule`) ou d'une demande précise. Sources de candidats :

- **Mots croisés** : pour chaque nouvel affixe, parcourez les briques déjà publiées (`ludique/briques/`, fascicules précédents) et cherchez les mots attestés qu'il forme avec elles, dans les deux sens et avec les mots déjà publiés comme ingrédients (*biologie* + *-iste*).
- **Fermeture** : toute suite de briques publiées qui forme un mot attesté doit être publiée avec sa recette, ou écartée par une exclusion.
- **Wikidata** : repérez les lexèmes qui combinent des briques publiées par SPARQL (§ 2). Il n'y a plus d'import du Wiktionnaire.

Une simple ressemblance de chaînes ne prouve jamais une composition (ADR 0003). Présentez les candidats à l'utilisateur, avec pour chacun l'analyse proposée et ce qu'il reste à vérifier, et laissez-le choisir.

Avant de saisir, vérifiez que le mot n'existe pas déjà : `grep -rl '"texte": "<forme>"' unites/`, ou la recherche de `rapports/catalogue.html` (`etymo-contenu visualiser`).

## 2. Vérifier dans Wikidata (MCP `wikidata`)

**Wikidata est la seule source de vérité** (ADR 0060, qui remplace en partie l'ADR 0014). Le TLFi, l'Académie, Littré, le Gaffiot, Ernout-Meillet, le Bailly, le LSJ, Chantraine et le Wiktionnaire ne sont plus des sources : ne les consultez pas pour établir un fait et ne les citez pas. Il n'y a plus de rôle `reference` ou `reperage`.

Le serveur MCP `wikidata` est déclaré dans `.mcp.json` (`https://wd-mcp.wmcloud.org/mcp/`). Lui seul est utilisé, et seulement par l'agent qui génère de nouveaux mots. Ordre imposé par le serveur : découvrir, vérifier la structure, puis interroger.

1. **Découvrir** : `search_items` (`lang: "fr"`) et `search_properties`. Ne tapez jamais un QID, un PID ou un LID de mémoire : le serveur les a brouillés. La recherche d'items est bruyante (un mot courant renvoie surtout des articles et des catégories) ; pour un mot, passez directement au SPARQL sur les lexèmes.
2. **Vérifier** : `get_statements` sur l'entité retenue (`include_external_ids: true` pour les codes ISO, Glottolog), `get_instance_and_subclass_hierarchy` pour la classe. Ne supposez aucun modèle.
3. **Interroger** : `execute_sparql` (`K` = nombre de lignes max, 10 par défaut). Les préfixes `wikibase:`, `ontolex:`, `dct:` sont déjà déclarés. Si le serveur répond « currently unavailable », réessayez plus tard.

| Il faut établir | Comment |
|---|---|
| Une langue (`referentiels/langues/`) : filiation, code, écriture | `search_items` puis `get_statements` : instance de, sous-classe de (filiation), codes ISO 639 et Glottolog, système d'écriture. Comparez à `langue_parente_id` et à `systeme_ecriture_defaut_id`. |
| La forme d'un mot ou d'un étymon | SPARQL sur les lexèmes : `?l a ontolex:LexicalEntry; wikibase:lemma ?lemma; dct:language wd:<QID de la langue>. FILTER(STR(?lemma)="γῆ")`. Comparez le lemme à la forme canonique de l'unité (NFC, écriture par défaut). |
| La chaîne étymologique | Sur le lexème, `get_statements` : « dérivé du lexème » et « combine les lexèmes » (PID à trouver avec `search_properties`). Comparez à la composition et aux relations saisies. |
| La première attestation d'un mot français | Déclaration « first attested from » du lexème (PID à trouver avec `search_properties`). Gardez la précision de Wikidata, jamais plus fine. |
| Les homographes | Propriété « lexème homographe ». |

Règles :

- **Pas de déclaration Wikidata, pas de fait publié.** Un lexème ou une déclaration absente ne prouve rien contre le fait, mais le fait reste `brouillon`, avec dans `note_interne` ce qu'il manque dans Wikidata. N'inventez jamais une entrée, un QID ou un LID.
- **Ne contribuez jamais à Wikidata** (ADR 0061) : ne créez ni ne modifiez aucune entité, déclaration ni sens, et n'y signalez rien. Une erreur probable de Wikidata se note dans `note_interne`, et le fait reste `brouillon`.
- **Wikidata fait référence** (ADR 0061) : si Wikidata déclare une autre analyse que celle du catalogue, saisissez la sienne, et passez celle du catalogue en `non_retenue` avec la déclaration qui la contredit. Un sens Wikidata plus étroit que la glose n'est pas une contradiction : ne réécrivez pas la glose.
- **Pas de résultat vide pris pour un « non ».** Résultat vide ou incohérent : reprenez les étapes 1 et 2, affinez, relancez ; n'en déduisez rien.
- **Rien n'est copié.** Rédigez gloses, définitions et explications vous-même, même si Wikidata en donne.
- `confiance` : `etablie` (déclaration Wikidata qui porte elle-même une référence), `probable` (déclaration sans référence), `discutee` (Wikidata porte deux analyses en désaccord : enregistrez chacune), `non_retenue` (jamais publiée comme solution).
- **Source** : réutilisez la source `src_…` de l'entité si elle existe dans `sources/`, sinon créez-la (`nouvel-id src`) avec `type: base`, `url` de l'entité, `licence` « CC0 », `edition` = date de la consultation. `localisation` = l'identifiant consulté (QID, LID ou énoncé), jamais un identifiant non consulté.

## 3. Saisir la chaîne

Générez les identifiants d'un coup : `cargo run -q --manifest-path outils/Cargo.toml -- nouvel-id unt -n 3`, puis `frm`, `sns`, `cmp`, `trf`, `rel`, `brq`, `rct` selon le besoin. Ne mettez jamais de graphie dans un identifiant, n'en réutilisez jamais un. Chaque affirmation porte un bloc `fait` à l'état **`brouillon`** : seule une relecture humaine le passe à `relu`.

Bloc `fait` type :

```json
{
  "confiance": "etablie",
  "etat_relecture": "brouillon",
  "sources": [{ "source_id": "src_…", "localisation": null }],
  "note_interne": "À vérifier : déclaration Wikidata à confirmer."
}
```

### 3.1 Unités (`unites/unt_….json`)

D'abord les parties et les étymons qui manquent, puis le mot.

- `nature` : `mot` (avec `mot.categorie`), `prefixe` ou `suffixe` (avec `affixe.statut` : `affixe` ou `element_de_composition`). `langue_id` pris dans `referentiels/langues/`.
- `formes` : **exactement une** `canonique`, dans l'écriture par défaut de la langue. Pas de trait d'union (`géo`, pas `géo-`). `cle_normalisee` = forme NFC du texte (`unicodedata.normalize("NFC", texte)`), sans autre transformation. Translittération dans une convention de `referentiels/conventions/` de la bonne écriture pour le grec. Un allomorphe (`role: allomorphe`, avec `contexte`) est une forme de la même unité, jamais une autre unité.
- `sens` : `rang` à partir de 1. `libelles.fr.glose` courte, affichée sur la brique (« terre »). Pour un mot résultat d'une recette, `libelles.fr.definition` en une phrase est obligatoire (`recette.definition`). **Rédigez** gloses et définitions vous-même.
- Homographes : deux unités de même forme doivent différer par la glose **et** par l'étymon.
- `attestation` (mot français seulement, ADR 0056) : première attestation du mot, quel que soit son sens, lue dans la déclaration « first attested from » du lexème Wikidata. `id` en `att_…`, `annee`, `precision` (`annee` « 1557 », `vers` « ca 1175 », `avant` « av. 1350 », `intervalle` « 1225-50 » avec `annee_fin`, `siecle` « xiiie s. » avec une année du siècle), et son propre bloc `fait`. Gardez la précision de la source, jamais plus fine. Pas de date sûre, pas d'attestation.

### 3.2 Composition (`compositions/cmp_….json`)

- `mot_id`, `forme_resultat_id` (forme canonique du mot), `type_analyse` (`synchronique` ou `historique`), `preferee` (une seule analyse préférée par mot).
- `parties` : au moins deux, `rang` continus à partir de 1, avec `unite_id`, `forme_id` et `sens_id` **de cette unité**. Pas de préfixe en dernier, pas de suffixe en premier.
- Segments obligatoires : `segment_debut` / `segment_fin` sur chaque partie, en points de code sur la forme NFC du mot, fin exclue, sans chevauchement ni trou, de la première à la dernière lettre. La partie finale garde sa forme entière et la partie précédente perd la lettre élidée (`biolog · iste`, `top · onyme`). Sans eux, `valider` échoue et le ricochet « Découper » n’a pas de coupe.
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
