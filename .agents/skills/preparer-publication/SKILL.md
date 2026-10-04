---
name: preparer-publication
description: Préparer la publication d'un fascicule d'ÉtymoLogique : lancer tous les contrôles (validation, exigences de publication, relecture, sources, médias, taille, exclusions, reste du dépôt), proposer puis faire confirmer une date de parution en UTC, l'écrire dans le fascicule, vérifier la compilation prod, et proposer la PR develop → master sans la créer. À utiliser quand un fascicule est jugé complet.
---

# Skill preparer-publication

Références, qui priment sur ce résumé : ADR 0005 (pipeline), 0014 (sources, rythme), 0022 (20 à 30 mots), 0025 et 0029 (parution programmée, dates en UTC), 0028 (gitflow), et le README (« Ce que vérifie `valider` », « Compilation et publication »).

Ce skill **ne publie rien**. La CI dépose l'artefact dans `staging/` au merge sur `master`, et la tâche de parution le promeut dans `public/` à la date du calendrier.

## 1. Lancer le rapport

Depuis la racine du dépôt, sur `develop` à jour :

```sh
cargo run -q --manifest-path outils/Cargo.toml -- pre-publication <numéro | fsc_…>
cargo run -q --manifest-path outils/Cargo.toml -- pre-publication <numéro> --complet   # tous les points
cargo run -q --manifest-path outils/Cargo.toml -- pre-publication <numéro> --json      # rapport machine
```

Le script lance `valider --publication`, range les diagnostics par rubrique et contrôle le calendrier. Il sort en 1 s'il reste un point bloquant, en 2 si l'appel est invalide.

| Rubrique | Contenu | Bloquant si |
|---|---|---|
| Intégrité du fascicule | Schémas, références, unités, compositions, recettes, briques de son périmètre | erreur |
| Relecture et sources | Faits non `relu`, sans source `reference` (deux pour `discutee`), sans `localisation`, `non_retenue` | toujours |
| Médias | Silhouette de chaque carte et de chaque langue citée, jaquette, profil SVG | erreur |
| Contenu du fascicule | Autonomie, unicité, fascicule de test ; en « à voir » : taille hors 20–30, ingrédient inutile | erreur |
| Exclusions | Exclusions non relues | toujours |
| Calendrier | Date passée, avant le fascicule précédent ou après le suivant ; en « à voir » : hors UTC, écart hors 28–45 jours | voir colonne |
| Reste du dépôt | Erreurs hors du fascicule : elles font aussi échouer `compiler` en prod | toujours |

Un fascicule de test est contrôlé comme s'il était publié, pour répéter, puis signalé comme bloquant.

La chronologie est recalculée sur des dates analysées. La règle `fascicule.chronologie` du validateur compare les dates comme du texte, ce qui se trompe quand les fuseaux diffèrent : le script l'écarte. Signalez l'écart à l'utilisateur s'il apparaît dans `valider`.

## 2. Traiter les points bloquants

- **Relecture** : ne passez **jamais** vous-même un fait à `relu`. C'est une relecture humaine. Listez à l'utilisateur les faits à relire, regroupés par fichier.
- **Sources** : ne remplissez une `localisation` qu'avec une entrée ou une page réellement consultée. Sinon, laissez `null` et listez ce qu'il reste à vérifier. Le Wiktionnaire seul ne suffit jamais.
- **Médias** : skills `silhouette` et `jaquette`.
- **Intégrité et contenu** : corrigez (skill `nouveau-mot`) ou demandez à l'utilisateur. N'affaiblissez jamais une règle de `outils/src/controles.rs` pour passer.
- **Taille hors 20–30** : avertissement seulement, paramètre d'équilibrage (ADR 0022). Faites-le trancher par l'utilisateur.
- **Reste du dépôt** : un autre fascicule non test qui ne passe pas bloque aussi la compilation prod. Signalez-le.

Relancez le script après chaque correction.

## 3. Planifier la date

Règle par défaut, calculée par le script (« Date proposée ») : **le 1ᵉʳ du mois à 00:00 UTC**, au moins 28 jours après la parution du fascicule précédent, et dans le futur. Le rythme visé est d'un fascicule environ tous les 30 jours (ADR 0014, 0022). Les 28 jours laissent passer février.

1. Évaluez la date proposée ou celle voulue :
   `cargo run -q --manifest-path outils/Cargo.toml -- pre-publication <numéro> --date 2026-12-01T00:00:00Z`
2. **Faites confirmer la date par l'utilisateur** avec `ask_user`, en proposant d'abord la date calculée, puis les options utiles (1ᵉʳ du mois suivant, date libre). N'écrivez rien sans confirmation.
3. Écrivez-la, au format UTC `AAAA-MM-JJTHH:MM:SSZ` :
   `cargo run -q --manifest-path outils/Cargo.toml -- pre-publication <numéro> --date <date confirmée> --ecrire`
   Le script écrit `parution` dans le fichier du fascicule, puis refait le rapport.
4. Mettez à jour la `note_interne` du fascicule si elle parlait d'une date provisoire.

Reporter un fascicule, c'est changer sa date avant le merge. Après le merge, c'est une nouvelle PR (ADR 0029).

## 4. Vérifier la compilation prod

```sh
cargo test --manifest-path outils/Cargo.toml
cargo run -q --manifest-path outils/Cargo.toml -- valider --publication
cargo run -q --manifest-path outils/Cargo.toml -- compiler          # sans --essai : refuse si valider --publication échoue
```

`compiler` écrit dans `artefact/`, ignoré par Git. Vérifiez dans `artefact/parutions.json` que le fascicule figure avec sa date et son manifeste. Aucun fascicule de test ne doit y figurer.

## 5. Conclure

Résumez à l'utilisateur :

- l'état de chaque rubrique et les points encore ouverts ;
- la date écrite, en UTC et en heure de Paris ;
- les fichiers modifiés.

Si plus rien ne bloque, **proposez** d'ouvrir la PR `develop → master` (ADR 0028 : `master` publie en prod), avec un message de commit en anglais, **sans la créer** : l'utilisateur la lance, par exemple avec le skill `create-pr`. S'il reste des points bloquants, ne proposez pas la PR : listez ce qu'il reste à faire.
