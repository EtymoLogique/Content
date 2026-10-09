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
| 6. Compilation en artefact à hash de contenu | **en place** : `etymo-contenu compiler` |
| 7. Publication | **en place** : la CI dépose l’artefact dans `staging/`, puis lance la tâche de parution, qui le promeut dans `public/` (ADR 0029) |
| 8. Migrations | à venir |

Le schéma logique est décrit dans le [modèle de données](https://github.com/lunik/EtymoLogique/blob/master/docs/modele-donnees.md) et l’[ADR 0013](https://github.com/lunik/EtymoLogique/blob/master/docs/adr/0013-schema-des-briques.md).

## Arborescence

| Chemin | Contenu | Schéma |
|---|---|---|
| `referentiels/langues/` | Langues : code interne, filiation, écriture par défaut, glyphe de la carte du codex, noms par locale | `langue` |
| `referentiels/ecritures/` | Systèmes d’écriture (ISO 15924, direction) | `ecriture` |
| `referentiels/conventions/` | Conventions de translittération | `convention` |
| `referentiels/types-relation.json` | Vocabulaire contrôlé des relations étymologiques | `vocabulaire` |
| `referentiels/types-transformation.json` | Vocabulaire contrôlé des transformations | `vocabulaire` |
| `sources/` | Sources : Wikidata, seule source de vérité (ADR 0060) ; les sources héritées attendent leur recoupement | `source` |
| `unites/` | Mots, préfixes et suffixes, avec leurs formes, translittérations, sens et première attestation (ADR 0056) | `unite` |
| `compositions/` | Analyses d’un mot en parties ordonnées, transformations, sens littéral | `composition` |
| `relations/` | Relations étymologiques entre unités | `relation` |
| `ludique/briques/` | Briques : unité, forme et sens affichés, rareté | `brique` |
| `ludique/recettes/` | Recettes : composition jouée, emplacements, briques acceptées | `recette` |
| `ludique/fascicules/` | Fascicules : numéro, parution, unités résultat et ingrédient, graines (réserve de départ, [ADR 0030](https://github.com/lunik/EtymoLogique/blob/master/docs/adr/0030-reserve-de-depart-par-fascicule.md)), jalons ([ADR 0038](https://github.com/lunik/EtymoLogique/blob/master/docs/adr/0038-indices-a-prix-croissant-et-format-des-jalons.md)) | `fascicule` |
| `ludique/exclusions/` | Combinaisons attestées volontairement écartées | `exclusion` |
| `ludique/trompe-l-oeil/` | Fausses fusions expliquées : paire ordonnée de briques qui s’écrit comme un mot sans en venir, deux explications rédigées et un fait sourcé ([ADR 0052](https://github.com/lunik/EtymoLogique/blob/master/docs/adr/0052-fusions-trompeuses-expliquees.md)) | `trompe-l-oeil` |
| `ludique/medias/` | Médias : type (silhouette ou jaquette), fichier, unité, langue ou fascicule illustré | `media` |
| `medias/silhouettes/`, `medias/jaquettes/` | Fichiers SVG bruts, sous des noms de travail lisibles (ADR 0026) | profil SVG |
| `schemas/` | JSON Schema (draft 2020-12) de chaque type d’objet | — |
| `outils/` | Outil Rust `etymo-contenu` : validateur, compilateur de l’artefact, visualisation locale, pré-publication et générateur d’identifiants | — |
| `artefact/` | Artefact compilé : **ignoré par Git** | — |
| `outils/src/tests/` | Un test par règle de validation | — |
| `.agents/skills/` | Skills des agents : nouveau fascicule, nouveau mot, silhouette, jaquette, préparation de la publication (`.github/skills` y renvoie) | — |
| `rapports/` | Rapports de validation et visualisation générés : **ignoré par Git** | — |

### Conventions d’édition

- **Un fichier par objet**, nommé d’après son identifiant : `unites/unt_7k2m9qxa.json`. Formes, sens et transformations vivent dans le fichier de leur unité ou de leur composition, avec leur propre identifiant.
- **Identifiants opaques et stables** : préfixe du type, puis 8 caractères base32 aléatoires. Ils ne contiennent jamais de graphie. Générez-les avec `etymo-contenu nouvel-id <préfixe>`.
- **Chaque affirmation est un fait** : unités, formes, sens, compositions et relations portent un bloc `fait` (confiance, état de relecture, sources, `note_interne`, `note_simplification`).
- **Libellés par locale** : `libelles.fr` est obligatoire partout. C’est la locale de référence.
- **Formes sans trait d’union** : `géo`, pas `géo-`. La position vient de la nature de l’unité.
- **`cle_normalisee`** est la forme NFC du texte. Le texte édité n’est jamais normalisé en silence (ADR 0006).
- **Segments** : comptés en points de code, sur la forme NFC du mot résultat, avec la fin exclue. La question des graphèmes reste ouverte (modèle de données).
- **Fascicule de test** (`"test": true`) : il éprouve le schéma et n’est jamais publié. Seul un fascicule de test peut porter le numéro 0.
- **Médias** : une silhouette par carte et par langue (ADR 0023), une jaquette par fascicule (ADR 0016). Le fichier SVG vit sous `medias/`, et un objet `ludique/medias/med_….json` le rattache à son unité, sa langue ou son fascicule. La couche linguistique ne cite jamais un média. Le nom de travail n’est jamais publié.
- **Profil SVG** : liste blanche, parce que la PWA insère le SVG dans la page. Sont admis `svg g defs clipPath mask path circle ellipse rect line polyline polygon`, avec leurs attributs de géométrie et de peinture, et `url(#…)` pour les seules références locales. Sont refusés tout autre élément (`script`, `style`, `image`, `use`, `foreignObject`, `text`, `title`…), les attributs `on*`, les liens externes, les DTD, les entités, les commentaires et tout texte, qui trahirait le mot. Une silhouette est en `viewBox="0 0 300 400"`, en `currentColor` seulement, avec les évidements en `style="fill|stroke:var(--art-bg)"` et sans opacité propre (ADR 0023). Une jaquette admet les couleurs.

## Commandes

Il faut Rust 1.90 ou plus récent ; la première commande compile l’outil, les suivantes le réutilisent.

```sh
cargo run -q --manifest-path outils/Cargo.toml -- valider                  # schémas et contraintes d'intégrité
cargo run -q --manifest-path outils/Cargo.toml -- valider --publication    # plus les exigences de publication
cargo run -q --manifest-path outils/Cargo.toml -- valider --json           # rapport machine
cargo run -q --manifest-path outils/Cargo.toml -- valider --strict         # les avertissements font échouer
cargo run -q --manifest-path outils/Cargo.toml -- compiler                 # artefact publiable dans artefact/
cargo run -q --manifest-path outils/Cargo.toml -- compiler --essai         # hors-prod : fascicules de test et faits non relus compris
cargo run -q --manifest-path outils/Cargo.toml -- visualiser               # page locale rapports/catalogue.html : graphe, cartes, silhouettes
cargo run -q --manifest-path outils/Cargo.toml -- nouvel-id unt -n 3       # trois identifiants d'unité inédits
cargo run -q --manifest-path outils/Cargo.toml -- trompe-l-oeil-candidats  # paires de briques qui s'écrivent comme un mot du catalogue sans en venir
cargo test --manifest-path outils/Cargo.toml                    # tests du validateur et du compilateur
cargo run -q --manifest-path outils/Cargo.toml -- pre-publication 1   # rapport de pré-publication du fascicule 1
```

La validation échoue (code 1) dès qu’une erreur est trouvée. La CI lance les tests et `valider` à chaque push.

### Visualiser le catalogue

`etymo-contenu visualiser` écrit une page HTML autonome, par défaut `rapports/catalogue.html`, à ouvrir dans un navigateur. Elle ne charge rien d’extérieur.

- **Graphe** : chaque unité est un nœud. Sa forme donne la nature (rond pour un mot, carré pour une brique), son contour donne la langue. Les flèches pleines vont des parties au mot composé, les flèches pointillées suivent les relations étymologiques. On peut filtrer par fascicule (ou « sans fascicule »), nature, langue et type de lien, ne garder que les brouillons, et chercher une forme ou une glose. Un clic ouvre la fiche : sens, faits et relecture, brique, recettes, fascicules, silhouette, compositions et relations. Échap ou un clic sur le fond la referme.
- **Cartes** : pour chaque fascicule, sa jaquette (un clic ouvre la fiche du fascicule), puis ses cartes avec leur silhouette. Viennent ensuite les langues, puis les unités hors fascicule. Une carte sans silhouette a un bord en pointillé.
- **Diagnostics** : le rapport de `valider` est affiché dans la page, qui se génère même sur un dépôt invalide.

La page montre les notes internes et les faits non relus. Elle reste locale : `rapports/` est ignoré par Git, et la page ne se publie jamais. Une silhouette n’y est insérée que si elle passe le profil SVG.

### Ce que vérifie `valider`

- **Fichiers** : JSON lisible, aucune clé en double, un schéma respecté, un nom de fichier égal à l’identifiant, un préfixe cohérent avec le dossier, des identifiants uniques dans tout le dépôt.
- **Références** : chaque identifiant cité existe.
- **Langues** : filiation sans cycle, codes uniques.
- **Unités** : exactement une forme canonique, dans l’écriture par défaut de la langue ; formes sans trait d’union ; `cle_normalisee` en NFC ; translittération dans une convention de la bonne écriture ; homographes distingués par la glose **et** par l’étymon. **Attestation** (ADR 0056) : seul un mot français porte une première attestation (`attestation.mot_francais`), jamais dans le futur (`attestation.future`), un intervalle finit après son début (`attestation.intervalle`) ; `annee_fin` n’existe que pour un intervalle (schéma).
- **Compositions** : le résultat est un mot ; les parties sont au moins deux, de rangs continus, et leurs formes et sens appartiennent à leur unité ; pas de préfixe en dernier ni de suffixe en premier ; segments dans les bornes, sans chevauchement, et qui pavent le mot de la première à la dernière lettre : chaque partie en porte un (`composition.segments_manquants`, `composition.pavage`), ce sont eux qui donnent ses coupes au ricochet « Découper » (ADR 0044) ; une seule analyse préférée ; aucun cycle ; transformations au vocabulaire contrôlé.
- **Relations** : source et cible différentes, sans doublon ni cycle ; un emprunt relie deux langues ; un héritage suit la filiation.
- **Briques** : une par unité, qui affiche la forme canonique et un sens glosé de son unité.
- **Recettes** : exactement deux emplacements (on ne fusionne jamais plus de deux briques) ; les emplacements suivent les parties et acceptent la brique de chaque partie ; aucune suite de briques ne mène à deux recettes ; aucune analyse non retenue ; une définition pour le mot résultat.
- **Fascicules** : numéros uniques ; un mot n’est le résultat que d’un fascicule ; un résultat est un mot avec une recette ; un ingrédient a une brique ; autonomie (tout ingrédient est déclaré, et publié dans ce fascicule ou un précédent) ; graines (ADR 0030) : briques existantes, déclarées par le fascicule, sans doublon, de 1 à 5 exemplaires ; jalons (ADR 0038) : identifiants uniques, découvertes strictement croissantes, pli fait de préfixes et de suffixes déclarés par le fascicule, un pli ou des sabliers ; chaque résultat porte une `difficulte` (`facile`, `moyen`, `difficile`), qu’un ingrédient n’a jamais (ADR 0047) ; en avertissement : taille de 20 à 30 mots (ADR 0022), sauf pour les dix fascicules de lancement (ADR 0054), un tiers de mots de chaque difficulté à un mot près, légendaires exclus (`fascicule.tiers`, ADR 0047), à partir du onzième fascicule (ADR 0059), ingrédients inutiles, chronologie (les fascicules de lancement peuvent paraître le même jour, ADR 0054), règle d’amorçage (au moins trois recettes réalisables avec les seules graines, toutes ensemble ; un premier fascicule sans graines) ; ni jalon ni graine légendaires (`fascicule.jalon_legendaire`, `fascicule.graine_legendaire`) ; au moins une brique légendaire par fascicule (`fascicule.legendaire_absente`, ADR 0043), avertissement, erreur en mode publication.
- **Exclusions** : aucune ne contredit une recette.
- **Trompe-l’œil** ([ADR 0052](https://github.com/lunik/EtymoLogique/blob/master/docs/adr/0052-fusions-trompeuses-expliquees.md)) : les deux briques existent et sont publiées dans ce fascicule ou un précédent, aucune n’est légendaire (`trompe_l_oeil.paire`, `trompe_l_oeil.legendaire`) ; au plus un par paire ordonnée (`trompe_l_oeil.doublon`) ; ni la paire ni son inverse n’est une recette, et la paire n’est pas une exclusion (`trompe_l_oeil.recette`) ; aucun remplacement d’une brique par un homographe ne donne une recette, car le serveur le calcule déjà (`trompe_l_oeil.homographe_jouable`) ; si `mot_id` est renseigné, sa composition ne mobilise pas à la fois les deux unités (`trompe_l_oeil.origine`) et l’explication courte ne le nomme pas (`trompe_l_oeil.divulgation`) ; au moins une source (`trompe_l_oeil.sources`). Avec `--publication`, chaque trompe-l’œil du fascicule est relu (`publication.trompe_l_oeil`).
- **Médias** : cible existante ; une seule silhouette par unité ou langue, une seule jaquette par fascicule ; fichier présent et conforme au profil SVG ; en avertissement : fichier de `medias/` que nul média ne déclare. Pour chaque fascicule qui n’est pas de test : chaque carte et chaque langue citée (celle de l’unité ou d’un étymon) a sa silhouette, et le fascicule sa jaquette. C’est un avertissement, et une erreur avec `--publication`. Une carte hors fascicule (brique ou résultat d’une recette) sans silhouette est signalée en avertissement ; un étymon n’a pas de carte, donc rien à signaler (ADR 0013).
- **Avec `--publication`**, pour chaque fascicule qui n’est pas de test : tout fait publié est `relu`, cite au moins une source Wikidata (deux pour un fait `discutee`) avec l’identifiant consulté, et n’est pas `non_retenue`. Wikidata est la seule source de vérité (ADR 0060) ; une source héritée (TLFi, Bailly, Gaffiot) ne suffit plus. Chaque exclusion est relue.

## Compilation et publication

`etymo-contenu compiler` produit l’artefact de l’[ADR 0027](https://github.com/lunik/EtymoLogique/blob/master/docs/adr/0027-catalogue-statique-et-donnees-joueur.md), dans l’arborescence du préfixe `staging/` du bucket ([ADR 0029](https://github.com/lunik/EtymoLogique/blob/master/docs/adr/0029-parution-par-promotion-de-prefixe.md)) :

| Fichier | Contenu |
|---|---|
| `r/<sha256>.json` | Ressources : carte de chaque unité déclarée, recette, exclusions et trompe-l’œil d’un fascicule. Un trompe-l’œil publié n’emporte ni son fait, ni sa relecture, ni sa note : seulement la paire, le motif et les deux explications. |
| `f/<sha256>.json` | Manifeste de fascicule : identifiant, compteurs hors légendaires, briques tirables, graines, jalons, chemins des cartes et de leurs silhouettes, silhouettes des langues citées (`langues`), jaquette, recettes, exclusions et `ricochets` (sens affiché et classe de sens des briques, classe du sens littéral et coupes des mots, sans aucun texte ; ni légendaire ni mot qui en dépend, [ADR 0044](https://github.com/lunik/EtymoLogique/blob/master/docs/adr/0044-ricochets-reviser-et-maitriser-les-cartes.md)), `trompes_l_oeil` (chemin de la ressource) et `homographes` (groupes d’identifiants de briques de même forme, hors légendaires, [ADR 0052](https://github.com/lunik/EtymoLogique/blob/master/docs/adr/0052-fusions-trompeuses-expliquees.md)). Ces deux clés n’existent que si elles ne sont pas vides : un fascicule sans fausse piste ni homographe garde son empreinte. |
| `a/<sha256>.svg` | Médias : silhouettes et jaquettes, nommées d’après l’empreinte de leurs octets, jamais d’après le mot ni l’identifiant (ADR 0025). |
| `parutions.json` | Calendrier des parutions, seul fichier modifiable : version, référentiels, et pour chaque fascicule sa date et son manifeste. La tâche de parution en tire `public/index.json`. |

Le calendrier, comme l’index, ne cite que des manifestes ; seul le manifeste cite ses ressources. `ressources()`, dans `outils/src/compilation.rs`, fixe la liste des champs qui en portent un chemin.

- **Hash de contenu** : chaque fichier est rangé sous l’empreinte SHA-256 de son JSON canonique (clés triées, UTF-8). Un chemin ne contient ni identifiant ni graphie, et un contenu identique garde le même chemin.
- **Jamais publiés** : `note_interne`, `etat_relecture` et tout fait `non_retenue`.
- **Sans `--essai`** (prod) : la compilation exige d’abord `valider --publication`, et écarte les fascicules de test.
- **Avec `--essai`** (hors-prod seulement) : fascicules de test et faits non relus compris. L’artefact est marqué `"essai": true`, comme chaque manifeste de test.
- **Médias** : une carte, une langue ou un fascicule sans média garde `null` à sa place dans le manifeste. Deux cartes au même dessin partagent un seul fichier.
- **Jalons** : repris tels quels, triés par nombre de découvertes. Leur place dans le point fixe d’atteignabilité (ADR 0004) viendra avec l’étape 5 du pipeline.

La CI (`.github/workflows/publication.yml`) suit l’[ADR 0028](https://github.com/lunik/EtymoLogique/blob/master/docs/adr/0028-gitflow-des-depots.md) :

| Branche | Environnement | Mode |
|---|---|---|
| `develop` | horsprod | `--essai` |
| `master` | prod | publication |
| autre | rien | — |

Elle teste, compile, garde l’artefact 30 jours dans GitHub, puis le dépose dans `staging/` : fichiers hachés d’abord (JSON en `application/json`, médias en `image/svg+xml`), calendrier en dernier. Enfin, elle vérifie la présence de chaque manifeste du calendrier et de chaque ressource que ces manifestes citent. Une fois le calendrier vérifié, elle retire de `staging/` les fichiers hachés que l’artefact ne contient plus (`aws s3 sync --delete`) : anciens manifestes, cartes ou médias dont le contenu a changé. Le bucket étant versionné, un fichier retiré reste récupérable. Elle n’écrit jamais dans `public/` : la politique du bucket le lui interdit. Un fascicule ne devient lisible qu’une fois promu par la tâche de parution, à sa date.

Pour finir, elle lance la tâche Scaleway `parution` de son environnement (`develop` : hors-prod, `master` : prod) et attend sa fin. Un échec du dépôt ne la lance pas ; un échec de la tâche fait échouer le workflow. La CI retrouve la tâche par son nom (`…-parution`) dans le projet de l’environnement. Le droit de la lancer vient de la clé `content-ci`, dont les droits se posent dans `organisation/` du dépôt Infra.

## Sources et licences

- **Wikidata** est la seule source de vérité (ADR 0060), interrogée par le MCP `wikidata` par l’agent qui génère de nouveaux mots. Le Wiktionnaire et les références savantes ne sont plus des sources. **Aucun texte n’est copié** : les gloses, définitions et explications sont rédigées par l’équipe.
- Le jeu actuel (fascicule 0) reprend les **exemples travaillés** du modèle de données. Tout y est en `brouillon`, à vérifier avant toute publication.
- Ses silhouettes (`medias/silhouettes/`) sont exportées des maquettes du codex (`cardArt`, dossier de conception) ou dessinées selon la même grammaire, et sa jaquette « BETA » (`medias/jaquettes/beta.svg`) suit la palette des jaquettes. Elles illustrent le format, sans préfigurer un fascicule réel (ADR 0026).

| Périmètre | Licence |
|---|---|
| Outils : scripts, schémas, workflows | [GNU AGPL 3.0 ou ultérieure](LICENSE) |
| Contenu : données du catalogue (gloses, définitions, explications, relations) | [Creative Commons BY-SA 4.0](LICENSE-CONTENT) |
| Nom « ÉtymoLogique », logo et marque | Réservés, hors de ces licences |

© 2026 Guillaume MARTINEZ ([ADR 0017](https://github.com/lunik/EtymoLogique/blob/master/docs/adr/0017-licences.md)).
