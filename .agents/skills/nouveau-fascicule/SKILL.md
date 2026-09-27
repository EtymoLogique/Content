---
name: nouveau-fascicule
description: Créer un nouveau fascicule du catalogue ÉtymoLogique dans ludique/fascicules/ : numéro, identifiant, date provisoire, plan éditorial de 20 à 30 mots qui respecte l'autonomie et la fermeture, déclaration des unités, puis enchaînement avec les skills nouveau-mot, silhouette et jaquette. À utiliser dès qu'on ouvre un nouveau fascicule.
---

# Skill nouveau-fascicule

Références, qui priment sur ce résumé : ADR 0014 (sources et fascicules), 0016 (jaquettes), 0022 (20 à 30 mots), 0029 (parution), `docs/modele-donnees.md` du dépôt de conception, et le README (« Ce que vérifie `valider` »).

## 1. Faire l'état des lieux

```sh
.venv/bin/etymo-contenu valider
.venv/bin/etymo-contenu visualiser     # rapports/catalogue.html, filtre par fascicule
```

- Relevez les fascicules existants (`ludique/fascicules/*.json`) : numéros, dates, `test`.
- Un fascicule de test (`"test": true`) ne compte pas : il n'est jamais publié. Seul lui peut porter le numéro 0.
- Relevez les briques déjà publiées : ce sont les ingrédients qu'on peut **reprendre**.

## 2. Créer le fichier

1. Identifiant : `.venv/bin/etymo-contenu nouvel-id fsc`. Fichier : `ludique/fascicules/<id>.json`.
2. Numéro : le plus grand numéro des fascicules **non test**, plus 1 (1 s'il n'y en a pas).
3. `parution` : date provisoire, au format UTC `AAAA-MM-JJT00:00:00Z`. Règle par défaut : le 1ᵉʳ du mois qui suit la parution précédente, au moins 28 jours après (voir le skill `preparer-publication`, qui la calcule). La date définitive se fixe au moment de préparer la publication.
4. `test: false`, et `note_interne` : ce qui reste à faire, en rappelant que la date est provisoire. Aucune donnée personnelle.

```json
{
  "id": "fsc_…",
  "numero": 1,
  "parution": "2026-11-01T00:00:00Z",
  "test": false,
  "unites": [
    { "unite_id": "unt_…", "role": "resultat" },
    { "unite_id": "unt_…", "role": "ingredient" }
  ],
  "note_interne": "Date provisoire, à confirmer. Reste : …"
}
```

`unites` demande au moins une ligne : ne créez le fichier qu'avec un premier mot déjà saisi (skill `nouveau-mot`), ou saisissez-le dans la foulée avant de valider.

## 3. Construire le plan éditorial

Avant de saisir quoi que ce soit, proposez un plan à l'utilisateur et faites-le valider :

- **Taille** : 20 à 30 mots résultat, mots croisés compris (ADR 0022). Les ingrédients ne comptent pas.
- **Affixes** : choisis selon ce qu'ils ouvrent. Un nouvel affixe crée des **mots croisés** avec toutes les briques déjà publiées : listez-les d'abord (skill `nouveau-mot`, § Repérer). Un affixe très productif (*-able*, *-ment*, *dé-*, *in-*) ne s'introduit que si peu de mots publiés peuvent le recevoir.
- **Fermeture** (ADR 0014) : pour toute suite de briques publiées, si une source de référence atteste le mot, il est publié avec sa recette, ou écarté par une **exclusion** motivée (`archaique`, `rare`, `offensant`, `analyse_non_etablie`).
- **Autonomie** : tout ingrédient est déclaré dans le fascicule, et publié par lui ou par un fascicule précédent.
- **Unicité** : un mot n'est le résultat que d'un seul fascicule. Un mot déjà publié peut revenir comme ingrédient (*biologie* + *-iste*), jamais comme résultat.
- **Langues** : elles découlent des unités et de leurs étymons, et chacune demande sa silhouette (ADR 0023). La langue n'est jamais une brique.

## 4. Remplir le fascicule

Pour chaque mot du plan, lancez le skill `nouveau-mot`. Il saisit la chaîne complète et ajoute les lignes `resultat` et `ingredient` au fascicule. Ensuite :

- une silhouette par carte et par langue citée : skill `silhouette` ;
- une jaquette : skill `jaquette` (noms attribués pour les n° 01 à 10, ADR 0016).

## 5. Contrôler

```sh
.venv/bin/python -m pytest -q
.venv/bin/etymo-contenu valider
```

- Aucune erreur n'est acceptable.
- Tant que le fascicule est incomplet, les avertissements `fascicule.taille`, `fascicule.ingredient_inutile` et `media.manquant` sont attendus : listez-les à l'utilisateur comme reste à faire.
- Pour publier, passez au skill `preparer-publication`.
