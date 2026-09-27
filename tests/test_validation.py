"""Un test par règle : chaque test casse une seule chose dans une copie du dépôt."""

from __future__ import annotations

import unicodedata

import pytest
from conftest import avertissements, erreurs

from etymo_contenu.cli import main
from etymo_contenu.ids import MOTIF, nouvel_id

ID_ABSENT_UNT = "unt_00000000"


# ------------------------------------------------------------------ jeu d'exemples


def test_jeu_exemples_valide(depot):
    rapport = depot.valider()
    assert rapport.erreurs == [], rapport.en_texte()
    assert rapport.avertissements == [], rapport.en_texte()


def test_publication_ignore_les_fascicules_de_test(depot):
    rapport = depot.valider(publication=True)
    assert erreurs(rapport) == set()
    assert "publication.aucun_fascicule" in avertissements(rapport)


# ------------------------------------------------------------- fichiers et schémas


def test_json_illisible(depot):
    chemin, _ = depot.brique("géo")
    chemin.write_text("{ pas du json", encoding="utf-8")
    assert "json.syntaxe" in erreurs(depot.valider())


def test_cle_dupliquee(depot):
    chemin, donnees = depot.unite("géo")
    texte = chemin.read_text(encoding="utf-8").replace('"fr": {\n', '"fr": {"glose": "x"}, "fr": {\n', 1)
    chemin.write_text(texte, encoding="utf-8")
    assert erreurs(depot.valider()) == {"json.cle_dupliquee"}


def test_schema_enumeration(depot):
    depot.modifier(depot.brique("géo"), lambda b: b.update(rarete="epique"))
    assert "schema" in erreurs(depot.valider())


def test_schema_sous_type_exclusif(depot):
    depot.modifier(depot.unite("géo"), lambda u: u.update(mot={"categorie": "nom"}))
    assert "schema" in erreurs(depot.valider())


def test_schema_locale_de_reference(depot):
    def sans_fr(u):
        u["sens"][0]["libelles"] = {"en": {"glose": "earth"}}

    depot.modifier(depot.unite("géo"), sans_fr)
    assert "schema" in erreurs(depot.valider())


def test_fichier_inattendu(depot):
    (depot.racine / "unites" / "notes.txt").write_text("brouillon", encoding="utf-8")
    assert "fichier.inattendu" in erreurs(depot.valider())


# ------------------------------------------------------------------------ identité


def test_nom_de_fichier(depot):
    chemin, _ = depot.brique("géo")
    chemin.rename(chemin.with_name("brq_zzzzzzzz.json"))
    assert "identite.nom_fichier" in erreurs(depot.valider())


def test_prefixe_du_dossier(depot):
    chemin, _ = depot.brique("géo")
    chemin.rename(depot.racine / "unites" / chemin.name)
    assert "identite.prefixe" in erreurs(depot.valider())


def test_identifiant_en_double(depot):
    autre = depot.unite("logie")[1]["formes"][0]["id"]
    depot.modifier(depot.unite("géo"), lambda u: u["formes"][0].update(id=autre))
    assert "identite.doublon" in erreurs(depot.valider())


def test_reference_cassee(depot):
    depot.modifier(depot.brique("géo"), lambda b: b.update(unite_id=ID_ABSENT_UNT))
    assert "reference.cassee" in erreurs(depot.valider())


# -------------------------------------------------------------------------- unités


def test_une_seule_forme_canonique(depot):
    depot.modifier(depot.unite("philo"), lambda u: u["formes"][1].update(role="canonique"))
    assert "unite.forme_canonique" in erreurs(depot.valider())


def test_ecriture_de_la_langue(depot):
    latin = depot.unite("géo")[1]["formes"][0]["systeme_ecriture_id"]

    def en_latin(u):
        u["formes"][0]["systeme_ecriture_id"] = latin
        u["formes"][0].pop("translitterations")

    depot.modifier(depot.unite("γῆ"), en_latin)
    assert "unite.ecriture_incompatible" in erreurs(depot.valider())


def test_forme_sans_trait_union(depot):
    depot.modifier(depot.unite("géo"), lambda u: u["formes"][0].update(texte="géo-", cle_normalisee="géo-"))
    assert "forme.trait_union" in erreurs(depot.valider())


def test_cle_normalisee_nfc(depot):
    depot.modifier(depot.unite("géo"), lambda u: u["formes"][0].update(cle_normalisee=unicodedata.normalize("NFD", "géo")))
    assert "forme.cle_normalisee" in erreurs(depot.valider())


def test_convention_de_la_bonne_ecriture(depot):
    convention = depot.unite("γῆ")[1]["formes"][0]["translitterations"][0]["convention_id"]
    depot.modifier(depot.unite("géo"), lambda u: u["formes"][0].update(translitterations=[{"convention_id": convention, "texte": "geo"}]))
    assert "translitteration.ecriture" in erreurs(depot.valider())


def test_homographes_gloses_identiques(depot):
    depot.modifier(depot.unite("a", "vers"), lambda u: u["sens"][0]["libelles"]["fr"].update(glose="sans, privé de"))
    assert "homographe.glose_identique" in erreurs(depot.valider())


def test_homographes_etymons_identiques(depot):
    grec_a = depot.uid("ἀ")
    depot.modifier(depot.relation("a", "vers"), lambda r: r.update(unite_source_id=grec_a, type_code="emprunt"))
    assert "homographe.etymon_identique" in erreurs(depot.valider())


# -------------------------------------------------------------------- compositions


def test_resultat_est_un_mot(depot):
    geo = depot.unite("géo")[1]
    depot.modifier(depot.composition("géologie"), lambda c: c.update(mot_id=geo["id"], forme_resultat_id=geo["formes"][0]["id"]))
    assert "composition.resultat_non_mot" in erreurs(depot.valider())


def test_forme_resultat_du_mot(depot):
    autre = depot.unite("biologie")[1]["formes"][0]["id"]
    depot.modifier(depot.composition("géologie"), lambda c: c.update(forme_resultat_id=autre))
    assert "composition.forme_resultat" in erreurs(depot.valider())


def test_rangs_continus(depot):
    depot.modifier(depot.composition("géologie"), lambda c: c["parties"][1].update(rang=3))
    assert "composition.rangs" in erreurs(depot.valider())


def test_sens_de_la_partie(depot):
    autre = depot.unite("bio")[1]["sens"][0]["id"]
    depot.modifier(depot.composition("géologie"), lambda c: c["parties"][0].update(sens_id=autre))
    assert "composition.sens_partie" in erreurs(depot.valider())


def test_forme_de_la_partie(depot):
    autre = depot.unite("bio")[1]["formes"][0]["id"]
    depot.modifier(depot.composition("géologie"), lambda c: c["parties"][0].update(forme_id=autre))
    assert "composition.forme_partie" in erreurs(depot.valider())


def test_position_des_affixes(depot):
    def inverser(c):
        c["parties"][0]["rang"], c["parties"][1]["rang"] = 2, 1

    depot.modifier(depot.composition("géologie"), inverser)
    assert "composition.position" in erreurs(depot.valider())


def test_segment_hors_bornes(depot):
    depot.modifier(depot.composition("géologie"), lambda c: c["parties"][1].update(segment_fin=9))
    assert "composition.segment" in erreurs(depot.valider())


def test_segments_chevauchants(depot):
    depot.modifier(depot.composition("géologie"), lambda c: c["parties"][1].update(segment_debut=2))
    assert "composition.segment" in erreurs(depot.valider())


def test_segment_incomplet(depot):
    depot.modifier(depot.composition("géologie"), lambda c: c["parties"][1].update(segment_fin=None))
    assert "composition.segment" in erreurs(depot.valider())


def test_une_seule_composition_preferee(depot):
    depot.copier_sous(depot.composition("géologie"), nouvel_id("cmp"), lambda c: c.pop("transformations", None))
    assert "composition.preferee" in erreurs(depot.valider())


def test_composition_cyclique(depot):
    biologiste = depot.unite("biologiste")[1]

    def boucler(c):
        c["parties"][0].update(unite_id=biologiste["id"], sens_id=biologiste["sens"][0]["id"], forme_id=biologiste["formes"][0]["id"])
        c["parties"][0].pop("segment_debut"), c["parties"][0].pop("segment_fin")
        c["parties"][1].pop("segment_debut"), c["parties"][1].pop("segment_fin")

    depot.modifier(depot.composition("biologie"), boucler)
    assert "composition.cycle" in erreurs(depot.valider())


def test_type_de_transformation(depot):
    depot.modifier(depot.composition("biologiste"), lambda c: c["transformations"][0].update(type_code="magie"))
    assert "transformation.type_inconnu" in erreurs(depot.valider())


def test_transformation_vise_une_partie(depot):
    depot.modifier(depot.composition("biologiste"), lambda c: c["transformations"][0].update(rang_partie=5))
    assert "transformation.rang" in erreurs(depot.valider())


# ----------------------------------------------------------------------- relations


def test_relation_boucle(depot):
    depot.modifier(depot.relation("géo"), lambda r: r.update(unite_source_id=r["unite_cible_id"]))
    assert "relation.boucle" in erreurs(depot.valider())


def test_relation_en_double(depot):
    depot.copier_sous(depot.relation("géo"), nouvel_id("rel"))
    assert "relation.doublon" in erreurs(depot.valider())


def test_emprunt_entre_deux_langues(depot):
    geo = depot.uid("géo")
    depot.modifier(depot.relation("logie"), lambda r: r.update(unite_source_id=geo))
    assert "relation.emprunt" in erreurs(depot.valider())


def test_heritage_suit_la_filiation(depot):
    depot.modifier(depot.relation("géo"), lambda r: r.update(type_code="heritage"))
    assert "relation.heritage" in erreurs(depot.valider())


def test_relation_type_inconnu(depot):
    depot.modifier(depot.relation("géo"), lambda r: r.update(type_code="magie"))
    assert "relation.type_inconnu" in erreurs(depot.valider())


def test_relations_acycliques(depot):
    ge, geo = depot.uid("γῆ"), depot.uid("géo")
    depot.copier_sous(depot.relation("géo"), nouvel_id("rel"), lambda r: r.update(unite_source_id=geo, unite_cible_id=ge))
    assert "relation.cycle" in erreurs(depot.valider())


# ------------------------------------------------------------------------- briques


def test_une_brique_par_unite(depot):
    depot.copier_sous(depot.brique("géo"), nouvel_id("brq"))
    assert "brique.unicite" in erreurs(depot.valider())


def test_brique_affiche_la_forme_canonique(depot):
    allomorphe = depot.unite("philo")[1]["formes"][1]["id"]
    depot.modifier(depot.brique("philo"), lambda b: b.update(forme_affichee_id=allomorphe))
    assert "brique.forme" in erreurs(depot.valider())


def test_brique_sens_de_son_unite(depot):
    autre = depot.unite("bio")[1]["sens"][0]["id"]
    depot.modifier(depot.brique("géo"), lambda b: b.update(sens_affiche_id=autre))
    assert "brique.sens" in erreurs(depot.valider())


def test_brique_a_une_glose(depot):
    depot.modifier(depot.unite("géo"), lambda u: u["sens"][0]["libelles"].update(fr={"definition": "Relatif à la terre."}))
    assert "brique.glose" in erreurs(depot.valider())


# ------------------------------------------------------------------------ recettes


def test_recette_ambigue(depot):
    depot.copier_sous(depot.recette("géologie"), nouvel_id("rct"))
    assert "recette.ambigue" in erreurs(depot.valider())


def test_recette_accepte_la_brique_de_la_partie(depot):
    bio = depot.brique("bio")[1]["id"]
    depot.modifier(depot.recette("géologie"), lambda r: r["emplacements"][0].update(briques_acceptees=[bio]))
    rapport = depot.valider()
    assert "recette.brique_partie" in erreurs(rapport)


def test_recette_emplacements_suivent_les_parties(depot):
    logie = depot.brique("logie")[1]["id"]
    depot.modifier(depot.recette("géologie"), lambda r: r["emplacements"].append({"rang": 3, "briques_acceptees": [logie]}))
    assert "recette.emplacements" in erreurs(depot.valider())


def test_recette_partie_sans_brique(depot):
    chemin, _ = depot.brique("iste")
    chemin.unlink()
    assert "recette.brique_absente" in erreurs(depot.valider())


def test_recette_definition_obligatoire(depot):
    depot.modifier(depot.unite("géologie"), lambda u: u["sens"][0]["libelles"].update(fr={"glose": "étude de la Terre"}))
    assert "recette.definition" in erreurs(depot.valider())


def test_recette_non_retenue(depot):
    depot.modifier(depot.composition("géologie"), lambda c: c["fait"].update(confiance="non_retenue"))
    assert "recette.non_retenue" in erreurs(depot.valider())


def test_recette_hors_fascicule(depot):
    geologie = depot.uid("géologie")
    depot.modifier(depot.fascicule(), lambda f: f.update(unites=[u for u in f["unites"] if u["unite_id"] != geologie]))
    assert "recette.hors_fascicule" in avertissements(depot.valider())


# ---------------------------------------------------------------------- fascicules


def test_autonomie(depot):
    geo = depot.uid("géo")
    depot.modifier(depot.fascicule(), lambda f: f.update(unites=[u for u in f["unites"] if u["unite_id"] != geo]))
    assert "fascicule.autonomie" in erreurs(depot.valider())


def test_ingredient_publie_plus_tard(depot):
    biologie = depot.uid("biologie")

    def biologie_en_ingredient(f):
        f["unites"] = [u for u in f["unites"] if u["unite_id"] != biologie] + [{"unite_id": biologie, "role": "ingredient"}]

    def biologie_en_resultat(f):
        ingredients = [u for u in f["unites"] if u["role"] == "ingredient" and u["unite_id"] != biologie]
        f.update(numero=1, unites=[{"unite_id": biologie, "role": "resultat"}] + ingredients)

    depot.modifier(depot.fascicule(), biologie_en_ingredient)
    depot.copier_sous(depot.fascicule(), nouvel_id("fsc"), biologie_en_resultat)
    assert "fascicule.autonomie" in erreurs(depot.valider())


def test_unicite_du_resultat(depot):
    geologie = depot.uid("géologie")

    def reprendre_geologie(f):
        f.update(numero=1, unites=[u for u in f["unites"] if u["role"] == "ingredient" or u["unite_id"] == geologie])

    depot.copier_sous(depot.fascicule(), nouvel_id("fsc"), reprendre_geologie)
    assert "fascicule.unicite" in erreurs(depot.valider())


def test_numero_unique(depot):
    depot.copier_sous(depot.fascicule(), nouvel_id("fsc"), lambda f: f.update(unites=f["unites"][:1]))
    assert "fascicule.numero" in erreurs(depot.valider())


def test_unite_declaree_deux_fois(depot):
    depot.modifier(depot.fascicule(), lambda f: f["unites"].append(dict(f["unites"][-1])))
    assert "fascicule.unite_dupliquee" in erreurs(depot.valider())


def test_resultat_est_un_mot_avec_recette(depot):
    moral = depot.uid("moral")

    def promouvoir(f):
        for u in f["unites"]:
            if u["unite_id"] == moral:
                u["role"] = "resultat"

    depot.modifier(depot.fascicule(), promouvoir)
    assert "fascicule.resultat_sans_recette" in erreurs(depot.valider())


def test_resultat_non_mot(depot):
    geo = depot.uid("géo")

    def promouvoir(f):
        for u in f["unites"]:
            if u["unite_id"] == geo:
                u["role"] = "resultat"

    depot.modifier(depot.fascicule(), promouvoir)
    assert "fascicule.resultat_non_mot" in erreurs(depot.valider())


def test_ingredient_sans_brique(depot):
    ge = depot.uid("γῆ")
    depot.modifier(depot.fascicule(), lambda f: f["unites"].append({"unite_id": ge, "role": "ingredient"}))
    assert "fascicule.ingredient_sans_brique" in erreurs(depot.valider())


def test_ingredient_inutile(depot):
    phile = depot.uid("phile")
    depot.modifier(depot.fascicule(), lambda f: f["unites"].append({"unite_id": phile, "role": "ingredient"}))
    assert "fascicule.ingredient_inutile" in avertissements(depot.valider())


def test_taille_du_fascicule(depot):
    depot.modifier(depot.fascicule(), lambda f: f.update(test=False, numero=1))
    assert "fascicule.taille" in avertissements(depot.valider())


def test_fascicule_reel_numero_zero(depot):
    depot.modifier(depot.fascicule(), lambda f: f.update(test=False))
    assert "schema" in erreurs(depot.valider())


# ---------------------------------------------------------------------- exclusions


def test_exclusion_contredit_une_recette(depot):
    fascicule = depot.fascicule()[1]["id"]
    geo, logie = depot.brique("géo")[1]["id"], depot.brique("logie")[1]["id"]
    identifiant = nouvel_id("exc")
    depot.ecrire(
        depot.racine / "ludique" / "exclusions" / f"{identifiant}.json",
        {"id": identifiant, "fascicule_id": fascicule, "combinaison": [geo, logie], "raison": "rare", "etat_relecture": "brouillon"},
    )
    assert "exclusion.recette" in erreurs(depot.valider())


def test_exclusion_valide(depot):
    fascicule = depot.fascicule()[1]["id"]
    geo, iste = depot.brique("géo")[1]["id"], depot.brique("iste")[1]["id"]
    identifiant = nouvel_id("exc")
    depot.ecrire(
        depot.racine / "ludique" / "exclusions" / f"{identifiant}.json",
        {"id": identifiant, "fascicule_id": fascicule, "combinaison": [geo, iste], "raison": "rare", "etat_relecture": "brouillon", "note_interne": "Cas de test."},
    )
    assert erreurs(depot.valider()) == set()


# --------------------------------------------------------------------- publication


def _rendre_publiable(depot):
    depot.modifier(depot.fascicule(), lambda f: f.update(test=False, numero=1))
    for dossier in ("unites", "compositions", "relations"):
        for chemin in depot.fichiers(dossier):
            donnees = depot.lire(chemin)
            faits = [donnees["fait"]]
            faits += [x["fait"] for x in donnees.get("formes", []) + donnees.get("sens", [])]
            for fait in faits:
                fait["etat_relecture"] = "relu"
                for lien in fait["sources"]:
                    lien["localisation"] = "entrée consultée"
            depot.ecrire(chemin, donnees)


def test_publication_exige_la_relecture(depot):
    depot.modifier(depot.fascicule(), lambda f: f.update(test=False, numero=1))
    regles = erreurs(depot.valider(publication=True))
    assert {"publication.relecture", "publication.localisation"} <= regles


def test_publication_complete(depot):
    _rendre_publiable(depot)
    assert erreurs(depot.valider(publication=True)) == set()


def test_publication_refuse_le_reperage_seul(depot):
    _rendre_publiable(depot)
    wiktionnaire = depot.trouver("sources", lambda s: s["role"] == "reperage")[1]["id"]
    depot.modifier(depot.unite("géo"), lambda u: u["fait"].update(sources=[{"source_id": wiktionnaire, "localisation": "géo-"}]))
    assert "publication.source_reference" in erreurs(depot.valider(publication=True))


def test_publication_discutee_deux_references(depot):
    _rendre_publiable(depot)
    depot.modifier(depot.composition("géologie"), lambda c: c["fait"].update(confiance="discutee"))
    assert "publication.source_reference" in erreurs(depot.valider(publication=True))


def test_publication_hors_perimetre_ignore(depot):
    _rendre_publiable(depot)
    depot.modifier(depot.unite("phile"), lambda u: u["fait"].update(etat_relecture="brouillon"))
    assert erreurs(depot.valider(publication=True)) == set()


# ------------------------------------------------------------ identifiants et CLI


def test_nouvel_id():
    ids = {nouvel_id("unt") for _ in range(200)}
    assert len(ids) == 200
    assert all(MOTIF.match(i) and i.startswith("unt_") for i in ids)
    with pytest.raises(ValueError):
        nouvel_id("xyz")


def test_cli_codes_de_retour(depot, capsys):
    assert main(["valider", "--racine", str(depot.racine)]) == 0
    depot.modifier(depot.brique("géo"), lambda b: b.update(unite_id=ID_ABSENT_UNT))
    assert main(["valider", "--racine", str(depot.racine), "--json"]) == 1
    assert '"reference.cassee"' in capsys.readouterr().out
