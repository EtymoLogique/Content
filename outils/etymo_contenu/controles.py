"""Contrôles d'intégrité du catalogue (modèle de données, « Contraintes d'intégrité » ; ADR 0005, 0013, 0014)."""

from __future__ import annotations

import itertools
import unicodedata
from collections import defaultdict
from typing import Iterable

from etymo_contenu import medias
from etymo_contenu.chargement import Catalogue
from etymo_contenu.rapport import Rapport

LOCALE_REFERENCE = "fr"
TRAITS_UNION = ("-", "\u2010", "\u2011", "\u2012", "\u2013", "\u2014")
TAILLE_FASCICULE = (20, 30)  # ADR 0022, paramètre d'équilibrage
# Règle d'amorçage (game-design) : trois découvertes au moins sans ouvrir de pli.
MIN_RECETTES_AMORCAGE = 3
# La table de fusion accepte exactement deux briques : jamais plus de deux éléments à la fois.
BRIQUES_PAR_FUSION = 2


class Controleur:
    def __init__(self, catalogue: Catalogue, rapport: Rapport, publication: bool = False) -> None:
        self.c = catalogue
        self.r = rapport
        self.publication = publication

    # ------------------------------------------------------------------ outils

    def _erreur(self, regle: str, message: str, objet: str | None, fichier_de: str | None = None) -> None:
        self.r.erreur(regle, message, self.c.fichier(fichier_de or objet), objet)

    def _avert(self, regle: str, message: str, objet: str | None, fichier_de: str | None = None) -> None:
        self.r.avertissement(regle, message, self.c.fichier(fichier_de or objet), objet)

    def _ref(self, table: dict, cible: str | None, regle_objet: str, champ: str, nature: str, fichier_de: str | None = None):
        """Résout une référence ; signale une référence cassée, sauf vers un fichier déjà signalé invalide."""
        if cible is None:
            return None
        if cible in table:
            return table[cible]
        if cible not in self.c.ids_invalides:
            self._erreur("reference.cassee", f"{champ} : {nature} {cible} introuvable.", regle_objet, fichier_de)
        return None

    def _forme_de(self, unite_id: str, forme_id: str) -> dict | None:
        trouve = self.c.formes.get(forme_id)
        return trouve[1] if trouve and trouve[0] == unite_id else None

    def _sens_de(self, unite_id: str, sens_id: str) -> dict | None:
        trouve = self.c.sens.get(sens_id)
        return trouve[1] if trouve and trouve[0] == unite_id else None

    def _canonique(self, unite: dict) -> dict | None:
        canoniques = [f for f in unite["formes"] if f["role"] == "canonique"]
        return canoniques[0] if len(canoniques) == 1 else None

    def _ancetres(self, langue_id: str) -> list[str]:
        chaine, courant = [], self.c.langues.get(langue_id)
        while courant and courant.get("langue_parente_id") and courant["langue_parente_id"] not in chaine:
            chaine.append(courant["langue_parente_id"])
            courant = self.c.langues.get(courant["langue_parente_id"])
        return chaine

    @staticmethod
    def _cycles(aretes: dict[str, set[str]]) -> list[list[str]]:
        """Cycles d'un graphe orienté, chacun signalé une fois (parcours en profondeur itératif)."""
        blanc, gris, noir = 0, 1, 2
        couleur: dict[str, int] = defaultdict(int)
        cycles: list[list[str]] = []
        for depart in sorted(aretes):
            if couleur[depart] != blanc:
                continue
            pile = [(depart, iter(sorted(aretes.get(depart, ()))))]
            chemin = [depart]
            couleur[depart] = gris
            while pile:
                noeud, suivants = pile[-1]
                suivant = next(suivants, None)
                if suivant is None:
                    couleur[noeud] = noir
                    pile.pop()
                    chemin.pop()
                elif couleur[suivant] == gris:
                    cycles.append(chemin[chemin.index(suivant):] + [suivant])
                elif couleur[suivant] == blanc:
                    couleur[suivant] = gris
                    chemin.append(suivant)
                    pile.append((suivant, iter(sorted(aretes.get(suivant, ())))))
        return cycles

    # --------------------------------------------------------------- contrôles

    def executer(self) -> None:
        self.langues()
        self.conventions()
        self.sources_des_faits()
        self.unites()
        self.homographes()
        self.compositions()
        self.relations()
        self.briques()
        self.recettes()
        self.fascicules()
        self.exclusions()
        self.medias()
        if self.publication:
            self.publiable()

    def langues(self) -> None:
        codes: dict[str, str] = {}
        for lid, langue in self.c.langues.items():
            self._ref(self.c.langues, langue["langue_parente_id"], lid, "langue_parente_id", "langue")
            self._ref(self.c.ecritures, langue["systeme_ecriture_defaut_id"], lid, "systeme_ecriture_defaut_id", "écriture")
            if langue["code_interne"] in codes:
                self._erreur("langue.code_duplique", f"Code interne déjà utilisé par {codes[langue['code_interne']]}.", lid)
            codes[langue["code_interne"]] = lid
            debut, fin = langue.get("periode_debut"), langue.get("periode_fin")
            if debut is not None and fin is not None and debut > fin:
                self._erreur("langue.periode", "La période commence après sa fin.", lid)
        filiation = {lid: {l["langue_parente_id"]} for lid, l in self.c.langues.items() if l["langue_parente_id"]}
        for cycle in self._cycles(filiation):
            self._erreur("langue.filiation_cyclique", "Filiation cyclique : " + " → ".join(cycle) + ".", cycle[0])
        iso: dict[str, str] = {}
        for eid, ecriture in self.c.ecritures.items():
            if ecriture["code_iso15924"] in iso:
                self._erreur("ecriture.code_duplique", f"Code ISO 15924 déjà utilisé par {iso[ecriture['code_iso15924']]}.", eid)
            iso[ecriture["code_iso15924"]] = eid

    def conventions(self) -> None:
        codes: dict[str, str] = {}
        for cid, convention in self.c.conventions.items():
            self._ref(self.c.ecritures, convention["systeme_ecriture_id"], cid, "systeme_ecriture_id", "écriture")
            if convention["code"] in codes:
                self._erreur("convention.code_duplique", f"Code déjà utilisé par {codes[convention['code']]}.", cid)
            codes[convention["code"]] = cid

    def _faits(self) -> Iterable[tuple[str, str, dict]]:
        """(identifiant du fait, identifiant du fichier, fait) pour toutes les affirmations."""
        for uid, unite in self.c.unites.items():
            yield uid, uid, unite["fait"]
            for forme in unite["formes"]:
                yield forme["id"], uid, forme["fait"]
            for sens in unite["sens"]:
                yield sens["id"], uid, sens["fait"]
        for cid, composition in self.c.compositions.items():
            yield cid, cid, composition["fait"]
        for rid, relation in self.c.relations.items():
            yield rid, rid, relation["fait"]

    def sources_des_faits(self) -> None:
        for fid, fichier_de, fait in self._faits():
            vues = set()
            for lien in fait["sources"]:
                self._ref(self.c.sources, lien["source_id"], fid, "sources", "source", fichier_de)
                if lien["source_id"] in vues:
                    self._erreur("fait.source_dupliquee", f"Source {lien['source_id']} citée deux fois.", fid, fichier_de)
                vues.add(lien["source_id"])

    def unites(self) -> None:
        for uid, unite in self.c.unites.items():
            langue = self._ref(self.c.langues, unite["langue_id"], uid, "langue_id", "langue")
            canoniques = [f for f in unite["formes"] if f["role"] == "canonique"]
            if len(canoniques) != 1:
                self._erreur("unite.forme_canonique", f"Une unité a exactement une forme canonique ({len(canoniques)} trouvée(s)).", uid)
            if langue and len(canoniques) == 1 and canoniques[0]["systeme_ecriture_id"] != langue["systeme_ecriture_defaut_id"]:
                self._erreur(
                    "unite.ecriture_incompatible",
                    "La forme canonique doit utiliser le système d’écriture par défaut de la langue.",
                    canoniques[0]["id"],
                    uid,
                )
            for forme in unite["formes"]:
                fid = forme["id"]
                texte = forme["texte"]
                if texte.startswith(TRAITS_UNION) or texte.endswith(TRAITS_UNION):
                    self._erreur("forme.trait_union", "Le texte d’une forme est stocké sans trait d’union.", fid, uid)
                if forme["cle_normalisee"] != unicodedata.normalize("NFC", texte):
                    self._erreur("forme.cle_normalisee", "cle_normalisee doit être la forme NFC du texte.", fid, uid)
                self._ref(self.c.ecritures, forme["systeme_ecriture_id"], fid, "systeme_ecriture_id", "écriture", uid)
                conventions_vues = set()
                for trl in forme.get("translitterations", []):
                    convention = self._ref(self.c.conventions, trl["convention_id"], fid, "convention_id", "convention", uid)
                    if convention and convention["systeme_ecriture_id"] != forme["systeme_ecriture_id"]:
                        self._erreur(
                            "translitteration.ecriture",
                            f"La convention {trl['convention_id']} ne s’applique pas à l’écriture de cette forme.",
                            fid,
                            uid,
                        )
                    if trl["convention_id"] in conventions_vues:
                        self._erreur("translitteration.doublon", "Deux translittérations pour une même convention.", fid, uid)
                    conventions_vues.add(trl["convention_id"])
            rangs = [s["rang"] for s in unite["sens"]]
            if len(set(rangs)) != len(rangs):
                self._erreur("sens.rang_duplique", "Deux sens ont le même rang.", uid)

    def _sens_affiche(self, uid: str) -> dict | None:
        for brique in self.c.briques.values():
            if brique["unite_id"] == uid:
                return self._sens_de(uid, brique["sens_affiche_id"])
        unite = self.c.unites[uid]
        return min(unite["sens"], key=lambda s: s["rang"])

    def _etymons(self, uid: str) -> frozenset[str]:
        return frozenset(r["unite_source_id"] for r in self.c.relations.values() if r["unite_cible_id"] == uid)

    def homographes(self) -> None:
        groupes: dict[tuple, list[str]] = defaultdict(list)
        for uid, unite in self.c.unites.items():
            canonique = self._canonique(unite)
            if canonique:
                groupes[(unite["langue_id"], unite["nature"], canonique["cle_normalisee"])].append(uid)
        for membres in groupes.values():
            for a, b in itertools.combinations(sorted(membres), 2):
                sens_a, sens_b = self._sens_affiche(a), self._sens_affiche(b)
                locales = set((sens_a or {}).get("libelles", {})) | set((sens_b or {}).get("libelles", {}))
                for locale in sorted(locales):
                    glose_a = (sens_a or {}).get("libelles", {}).get(locale, {}).get("glose")
                    glose_b = (sens_b or {}).get("libelles", {}).get(locale, {}).get("glose")
                    if glose_a == glose_b:
                        self._erreur(
                            "homographe.glose_identique",
                            f"Homographe de {b} : les gloses affichées doivent différer (locale {locale}).",
                            a,
                        )
                if self._etymons(a) == self._etymons(b):
                    self._erreur("homographe.etymon_identique", f"Homographe de {b} : les étymons doivent différer.", a)

    def compositions(self) -> None:
        preferees: dict[str, list[str]] = defaultdict(list)
        graphe: dict[str, set[str]] = defaultdict(set)
        for cid, cmp in self.c.compositions.items():
            mot = self._ref(self.c.unites, cmp["mot_id"], cid, "mot_id", "unité")
            forme_resultat = None
            if mot:
                if mot["nature"] != "mot":
                    self._erreur("composition.resultat_non_mot", "Le résultat d’une composition est un mot.", cid)
                forme_resultat = self._forme_de(cmp["mot_id"], cmp["forme_resultat_id"])
                if forme_resultat is None:
                    self._erreur("composition.forme_resultat", "forme_resultat_id n’est pas une forme du mot résultat.", cid)
            if cmp["preferee"]:
                preferees[cmp["mot_id"]].append(cid)

            parties = sorted(cmp["parties"], key=lambda p: p["rang"])
            rangs = [p["rang"] for p in parties]
            if rangs != list(range(1, len(parties) + 1)):
                self._erreur("composition.rangs", "Les rangs des parties sont uniques et continus à partir de 1.", cid)
            dernier = len(parties)
            segments = []
            for partie in parties:
                pu = partie["unite_id"]
                unite = self._ref(self.c.unites, pu, cid, f"partie {partie['rang']}", "unité")
                graphe[cmp["mot_id"]].add(pu)
                if pu == cmp["mot_id"]:
                    self._erreur("composition.partie_resultat", "Une partie ne peut pas être le résultat lui-même.", cid)
                if unite:
                    if self._sens_de(pu, partie["sens_id"]) is None:
                        self._erreur("composition.sens_partie", f"Partie {partie['rang']} : le sens n’appartient pas à l’unité.", cid)
                    if self._forme_de(pu, partie["forme_id"]) is None:
                        self._erreur("composition.forme_partie", f"Partie {partie['rang']} : la forme n’appartient pas à l’unité.", cid)
                    if unite["nature"] == "prefixe" and partie["rang"] == dernier:
                        self._erreur("composition.position", "Un préfixe n’apparaît pas au dernier rang.", cid)
                    if unite["nature"] == "suffixe" and partie["rang"] == 1:
                        self._erreur("composition.position", "Un suffixe n’apparaît pas au premier rang.", cid)
                debut, fin = partie.get("segment_debut"), partie.get("segment_fin")
                if (debut is None) != (fin is None):
                    self._erreur("composition.segment", f"Partie {partie['rang']} : segment_debut et segment_fin vont ensemble.", cid)
                elif debut is not None:
                    segments.append((debut, fin, partie["rang"]))
            if forme_resultat:
                longueur = len(unicodedata.normalize("NFC", forme_resultat["texte"]))
                for debut, fin, rang in segments:
                    if not 0 <= debut < fin <= longueur:
                        self._erreur("composition.segment", f"Partie {rang} : segment [{debut}, {fin}) hors de la forme résultat.", cid)
                for (d1, f1, r1), (d2, f2, r2) in itertools.pairwise(sorted(segments)):
                    if d2 < f1:
                        self._erreur("composition.segment", f"Les segments des parties {r1} et {r2} se chevauchent.", cid)

            ordres = set()
            for trf in cmp.get("transformations", []):
                if trf["type_code"] not in self.c.types_transformation:
                    self._erreur("transformation.type_inconnu", f"Type de transformation « {trf['type_code']} » inconnu.", cid)
                if trf["rang_partie"] is not None and trf["rang_partie"] not in rangs:
                    self._erreur("transformation.rang", f"La transformation {trf['id']} vise une partie absente.", cid)
                if trf["ordre"] in ordres:
                    self._erreur("transformation.ordre", "Deux transformations ont le même ordre.", cid)
                ordres.add(trf["ordre"])

        for mot_id, cids in preferees.items():
            if len(cids) > 1:
                self._erreur("composition.preferee", f"Plusieurs compositions préférées pour {mot_id} : {', '.join(sorted(cids))}.", cids[0])
        for cycle in self._cycles(graphe):
            self._erreur("composition.cycle", "Composition cyclique : " + " → ".join(cycle) + ".", cycle[0])

    def relations(self) -> None:
        vues: dict[tuple, str] = {}
        graphe: dict[str, set[str]] = defaultdict(set)
        for rid, rel in self.c.relations.items():
            src, cib, typ = rel["unite_source_id"], rel["unite_cible_id"], rel["type_code"]
            source = self._ref(self.c.unites, src, rid, "unite_source_id", "unité")
            cible = self._ref(self.c.unites, cib, rid, "unite_cible_id", "unité")
            type_rel = self.c.types_relation.get(typ)
            if type_rel is None:
                self._erreur("relation.type_inconnu", f"Type de relation « {typ} » inconnu.", rid)
            if src == cib:
                self._erreur("relation.boucle", "La source et la cible d’une relation sont différentes.", rid)
            cle = (src, cib, typ)
            if cle in vues:
                self._erreur("relation.doublon", f"Relation identique à {vues[cle]}.", rid)
            vues[cle] = rid
            if type_rel and type_rel.get("acyclique"):
                graphe[src].add(cib)
            if source and cible:
                if typ == "emprunt" and source["langue_id"] == cible["langue_id"]:
                    self._erreur("relation.emprunt", "Un emprunt relie deux langues différentes.", rid)
                if typ == "heritage" and source["langue_id"] not in self._ancetres(cible["langue_id"]):
                    self._erreur("relation.heritage", "Un héritage suit la filiation des langues.", rid)
        for cycle in self._cycles(graphe):
            self._erreur("relation.cycle", "Relations acycliques en cycle : " + " → ".join(cycle) + ".", cycle[0])

    def briques(self) -> None:
        par_unite: dict[str, str] = {}
        for bid, brique in self.c.briques.items():
            uid = brique["unite_id"]
            unite = self._ref(self.c.unites, uid, bid, "unite_id", "unité")
            if uid in par_unite:
                self._erreur("brique.unicite", f"L’unité {uid} a déjà la brique {par_unite[uid]}.", bid)
            par_unite[uid] = bid
            if not unite:
                continue
            forme = self._forme_de(uid, brique["forme_affichee_id"])
            if forme is None:
                self._erreur("brique.forme", "La forme affichée n’appartient pas à l’unité.", bid)
            elif forme["role"] != "canonique":
                self._erreur("brique.forme", "Une brique affiche la forme canonique de son unité.", bid)
            sens = self._sens_de(uid, brique["sens_affiche_id"])
            if sens is None:
                self._erreur("brique.sens", "Le sens affiché n’appartient pas à l’unité.", bid)
            elif not sens["libelles"].get(LOCALE_REFERENCE, {}).get("glose"):
                self._erreur("brique.glose", "Le sens affiché par une brique a une glose dans la locale de référence.", bid)

    def _brique_de(self, uid: str) -> str | None:
        for bid, brique in self.c.briques.items():
            if brique["unite_id"] == uid:
                return bid
        return None

    def _suites(self, emplacements: list[dict]) -> set[tuple[str, ...]]:
        ordonnes = sorted(emplacements, key=lambda e: e["rang"])
        return set(itertools.product(*(e["briques_acceptees"] for e in ordonnes)))

    def recettes(self) -> None:
        suites: dict[tuple[str, ...], str] = {}
        mots_en_fascicule = {u["unite_id"] for f in self.c.fascicules.values() for u in f["unites"] if u["role"] == "resultat"}
        for rid, recette in self.c.recettes.items():
            cmp = self._ref(self.c.compositions, recette["composition_id"], rid, "composition_id", "composition")
            for emp in recette["emplacements"]:
                for bid in emp["briques_acceptees"]:
                    self._ref(self.c.briques, bid, rid, f"emplacement {emp['rang']}", "brique")
            if len(recette["emplacements"]) != BRIQUES_PAR_FUSION:
                self._erreur(
                    "recette.deux_briques",
                    f"Une recette se joue avec exactement {BRIQUES_PAR_FUSION} briques ({len(recette['emplacements'])} emplacements).",
                    rid,
                )
            rangs = sorted(e["rang"] for e in recette["emplacements"])
            if rangs != list(range(1, len(rangs) + 1)):
                self._erreur("recette.rangs", "Les rangs des emplacements sont uniques et continus à partir de 1.", rid)
            for suite in self._suites(recette["emplacements"]):
                if suite in suites:
                    self._erreur("recette.ambigue", f"La même suite de briques mène aussi à {suites[suite]}.", rid)
                suites.setdefault(suite, rid)
            if not cmp:
                continue
            if cmp["fait"]["confiance"] == "non_retenue":
                self._erreur("recette.non_retenue", "Une analyse non retenue n’est jamais jouée comme solution.", rid)
            parties = {p["rang"]: p for p in cmp["parties"]}
            if sorted(parties) != rangs:
                self._erreur("recette.emplacements", "Les emplacements suivent les parties de la composition.", rid)
            for emp in recette["emplacements"]:
                partie = parties.get(emp["rang"])
                if not partie:
                    continue
                attendue = self._brique_de(partie["unite_id"])
                if attendue is None:
                    self._erreur("recette.brique_absente", f"Emplacement {emp['rang']} : l’unité {partie['unite_id']} n’a pas de brique.", rid)
                elif attendue not in emp["briques_acceptees"]:
                    self._erreur("recette.brique_partie", f"Emplacement {emp['rang']} : la brique {attendue} de la partie n’est pas acceptée.", rid)
            mot = self.c.unites.get(cmp["mot_id"])
            if mot:
                sens = min(mot["sens"], key=lambda s: s["rang"])
                if not sens["libelles"].get(LOCALE_REFERENCE, {}).get("definition"):
                    self._erreur("recette.definition", "Le mot résultat d’une recette a une définition dans la locale de référence.", rid)
            if cmp["mot_id"] not in mots_en_fascicule:
                self._avert("recette.hors_fascicule", f"Le mot {cmp['mot_id']} n’est le résultat d’aucun fascicule.", rid)
        self._suites_connues = suites

    def _recettes_du_mot(self, mot_id: str) -> list[dict]:
        return [
            r for r in self.c.recettes.values()
            if (cmp := self.c.compositions.get(r["composition_id"])) and cmp["mot_id"] == mot_id
        ]

    def fascicules(self) -> None:
        numeros: dict[tuple[bool, int], str] = {}
        jalons: dict[str, str] = {}
        resultats: dict[tuple[bool, str], str] = {}
        for fid, fsc in self.c.fascicules.items():
            cle = (fsc["test"], fsc["numero"])
            if cle in numeros:
                self._erreur("fascicule.numero", f"Numéro déjà utilisé par {numeros[cle]}.", fid)
            numeros[cle] = fid
            vus = set()
            for ligne in fsc["unites"]:
                uid = ligne["unite_id"]
                if uid in vus:
                    self._erreur("fascicule.unite_dupliquee", f"L’unité {uid} est déclarée deux fois.", fid)
                vus.add(uid)
                unite = self._ref(self.c.unites, uid, fid, "unites", "unité")
                if ligne["role"] == "resultat":
                    if (fsc["test"], uid) in resultats:
                        self._erreur("fascicule.unicite", f"Le mot {uid} est déjà le résultat de {resultats[(fsc['test'], uid)]}.", fid)
                    resultats[(fsc["test"], uid)] = fid
                    if unite and unite["nature"] != "mot":
                        self._erreur("fascicule.resultat_non_mot", f"Le résultat {uid} doit être un mot.", fid)
                    elif unite and not self._recettes_du_mot(uid):
                        self._erreur("fascicule.resultat_sans_recette", f"Le résultat {uid} n’a pas de recette.", fid)
                elif unite and self._brique_de(uid) is None:
                    self._erreur("fascicule.ingredient_sans_brique", f"L’ingrédient {uid} n’a pas de brique.", fid)
            self._graines(fid, fsc, vus)
            self._jalons(fid, fsc, vus, jalons)

        for fid, fsc in self.c.fascicules.items():
            declares = {l["unite_id"]: l["role"] for l in fsc["unites"]}
            utilises = set()
            nb_mots = 0
            for uid, role in declares.items():
                if role != "resultat":
                    continue
                nb_mots += 1
                for recette in self._recettes_du_mot(uid):
                    cmp = self.c.compositions[recette["composition_id"]]
                    for partie in cmp["parties"]:
                        pu = partie["unite_id"]
                        utilises.add(pu)
                        if pu not in declares:
                            self._erreur("fascicule.autonomie", f"L’ingrédient {pu} de {recette['id']} n’est pas déclaré.", fid)
                            continue
                        origine = resultats.get((fsc["test"], pu))
                        if origine and self.c.fascicules[origine]["numero"] > fsc["numero"]:
                            self._erreur(
                                "fascicule.autonomie",
                                f"L’ingrédient {pu} n’est publié que par un fascicule suivant ({origine}).",
                                fid,
                            )
            for uid, role in declares.items():
                if role == "ingredient" and uid not in utilises:
                    self._avert("fascicule.ingredient_inutile", f"L’ingrédient {uid} ne sert à aucune recette du fascicule.", fid)
            minimum, maximum = TAILLE_FASCICULE
            if not fsc["test"] and not minimum <= nb_mots <= maximum:
                self._avert("fascicule.taille", f"{nb_mots} mots : un fascicule en publie de {minimum} à {maximum}.", fid)
            if not fsc["test"]:
                rarete = {b["unite_id"]: b["rarete"] for b in self.c.briques.values()}
                if not any(rarete.get(uid) == "legendaire" for uid in declares):
                    signaler = self._erreur if self.publication else self._avert
                    signaler("fascicule.legendaire_absente", "Aucune brique légendaire : la chance de légendaire du pli (ADR 0043) n’aurait rien à donner.", fid)

        for test in (False, True):
            premiers = sorted((f["numero"], fid) for fid, f in self.c.fascicules.items() if f["test"] is test)
            if premiers and not self.c.fascicules[premiers[0][1]].get("graines"):
                self._avert("fascicule.amorcage", "Le premier fascicule ne déclare aucune graine : pas de réserve de départ.", premiers[0][1])

        dates = sorted((f["numero"], f["parution"], fid) for fid, f in self.c.fascicules.items() if not f["test"])
        for (n1, d1, _), (n2, d2, f2) in itertools.pairwise(dates):
            if d2 <= d1:
                self._avert("fascicule.chronologie", f"Le fascicule {n2} paraît avant ou en même temps que le fascicule {n1}.", f2)

    def _jalons(self, fid: str, fsc: dict, declares: set[str], vus: dict[str, str]) -> None:
        """Jalons (ADR 0038) : identifiants uniques, découvertes strictement croissantes,
        pli fait des préfixes et suffixes du fascicule."""
        precedent = 0
        for jalon in fsc.get("jalons", []):
            jid = jalon["id"]
            if jid in vus:
                self._erreur("fascicule.jalon_duplique", f"Le jalon {jid} est déjà déclaré par {vus[jid]}.", fid)
            vus[jid] = fid
            if jalon["decouvertes"] <= precedent:
                self._erreur("fascicule.jalon_ordre", f"Le jalon {jid} doit demander plus de {precedent} découvertes.", fid)
            precedent = jalon["decouvertes"]
            for bid in jalon["pli"]:
                brique = self._ref(self.c.briques, bid, fid, "jalons", "brique")
                if not brique:
                    continue
                if brique["unite_id"] not in declares:
                    self._erreur("fascicule.jalon_hors_fascicule", f"La brique {bid} du jalon {jid} n’est pas une brique du fascicule.", fid)
                elif self.c.unites.get(brique["unite_id"], {}).get("nature") not in ("prefixe", "suffixe"):
                    self._erreur("fascicule.jalon_mot", f"La brique {bid} du jalon {jid} est un mot : un pli ne donne que des affixes.", fid)
                if brique["rarete"] == "legendaire":
                    self._erreur("fascicule.jalon_legendaire", f"La brique {bid} du jalon {jid} est légendaire : un pli de jalon n’en donne jamais (ADR 0043).", fid)

    def _graines(self, fid: str, fsc: dict, declares: set[str]) -> None:
        """Réserve de départ (ADR 0030) : briques du fascicule, et au moins trois
        recettes réalisables avec elles seules, toutes ensemble (règle d'amorçage)."""
        graines: dict[str, int] = {}
        for graine in fsc.get("graines", []):
            bid = graine["brique_id"]
            if bid in graines:
                self._erreur("fascicule.graine_dupliquee", f"La brique {bid} est semée deux fois.", fid)
            graines[bid] = graine["exemplaires"]
            brique = self._ref(self.c.briques, bid, fid, "graines", "brique")
            if brique and brique["unite_id"] not in declares:
                self._erreur("fascicule.graine_hors_fascicule", f"La graine {bid} n’est pas une brique du fascicule.", fid)
            if brique and brique["rarete"] == "legendaire":
                self._erreur("fascicule.graine_legendaire", f"La graine {bid} est légendaire : la réserve de départ n’en donne jamais (ADR 0043).", fid)
        if not graines:
            return
        besoins: dict[str, int] = {}
        realisables = 0
        for uid in sorted(l["unite_id"] for l in fsc["unites"] if l["role"] == "resultat"):
            for recette in self._recettes_du_mot(uid):
                choix = [next((b for b in e["briques_acceptees"] if b in graines), None) for e in recette["emplacements"]]
                if None in choix:
                    continue
                realisables += 1
                for bid in choix:
                    besoins[bid] = besoins.get(bid, 0) + 1
        manquants = sorted(bid for bid, n in besoins.items() if n > graines[bid])
        if realisables < MIN_RECETTES_AMORCAGE or manquants:
            self._avert(
                "fascicule.amorcage",
                f"{realisables} recette(s) réalisable(s) avec les graines, {MIN_RECETTES_AMORCAGE} attendues"
                + (f" ; exemplaires insuffisants pour {', '.join(manquants)}" if manquants else "")
                + ".",
                fid,
            )

    def exclusions(self) -> None:
        suites = getattr(self, "_suites_connues", {})
        for eid, exc in self.c.exclusions.items():
            self._ref(self.c.fascicules, exc["fascicule_id"], eid, "fascicule_id", "fascicule")
            for bid in exc["combinaison"]:
                self._ref(self.c.briques, bid, eid, "combinaison", "brique")
            suite = tuple(exc["combinaison"])
            if suite in suites:
                self._erreur("exclusion.recette", f"Cette combinaison est aussi la recette {suites[suite]}.", eid)

    # ------------------------------------------------------------------ médias

    def medias(self) -> None:
        tables = {"unite": self.c.unites, "langue": self.c.langues, "fascicule": self.c.fascicules}
        libelles = {"unite": "unité", "langue": "langue", "fascicule": "fascicule"}
        declares: set[str] = set()
        cibles: dict[tuple[str, str, str], str] = {}
        for mid, media in sorted(self.c.medias.items()):
            nature, identifiant = medias.cible(media)
            self._ref(tables[nature], identifiant, mid, f"{nature}_id", libelles[nature])
            cle = (media["type"], nature, identifiant)
            if cle in cibles:
                self._erreur("media.double", f"{libelles[nature].capitalize()} {identifiant} a déjà une {media['type']} : {cibles[cle]}.", mid)
            cibles.setdefault(cle, mid)

            declares.add(media["fichier"])
            fichier = medias.chemin(self.c, media)
            if not fichier.is_file():
                self._erreur("media.fichier_absent", f"Fichier {medias.DOSSIER}/{media['fichier']} introuvable.", mid)
                continue
            problemes = medias.verifier_svg(fichier.read_bytes(), media["type"])
            if problemes:
                suite = f" (et {len(problemes) - 3} autre(s))" if len(problemes) > 3 else ""
                self._erreur("media.svg_invalide", f"{medias.DOSSIER}/{media['fichier']} : {' ; '.join(problemes[:3])}{suite}.", mid)

        dossier = self.c.racine / medias.DOSSIER
        if dossier.is_dir():
            for fichier in sorted(dossier.rglob("*")):
                relatif = fichier.relative_to(dossier).as_posix()
                if fichier.is_file() and not fichier.name.startswith(".") and relatif not in declares:
                    self.r.avertissement("media.orphelin", "Aucun média ne déclare ce fichier.", f"{medias.DOSSIER}/{relatif}")

        # Chaque carte et chaque langue citée a sa silhouette, chaque fascicule sa jaquette (ADR 0016, 0023).
        signaler = self._erreur if self.publication else self._avert
        for fid, fsc in sorted(self.c.fascicules.items()):
            if fsc["test"]:
                continue
            uids = [l["unite_id"] for l in fsc["unites"]]
            manquants = {
                "carte(s) sans silhouette": [u for u in uids if ("silhouette", "unite", u) not in cibles],
                "langue(s) sans silhouette": [l for l in medias.langues_citees(self.c, uids) if ("silhouette", "langue", l) not in cibles],
                "jaquette absente": [] if ("jaquette", "fascicule", fid) in cibles else [fid],
            }
            for libelle, ids in manquants.items():
                if ids:
                    detail = libelle if libelle == "jaquette absente" else f"{len(ids)} {libelle} : {', '.join(ids)}"
                    signaler("media.manquant", f"Fascicule {fsc['numero']} : {detail}.", fid)

    # ------------------------------------------------------ mode publication

    def _references(self, fait: dict) -> int:
        return sum(1 for s in fait["sources"] if self.c.sources.get(s["source_id"], {}).get("role") == "reference")

    def _exiger_publiable(self, fid: str, fichier_de: str, fait: dict, fascicule: str) -> None:
        if fait["etat_relecture"] != "relu":
            self._erreur("publication.relecture", f"Fait publié par {fascicule} sans relecture ({fait['etat_relecture']}).", fid, fichier_de)
        if fait["confiance"] == "non_retenue":
            self._erreur("publication.non_retenue", f"Fait non retenu publié par {fascicule}.", fid, fichier_de)
        minimum = 2 if fait["confiance"] == "discutee" else 1
        if self._references(fait) < minimum:
            self._erreur(
                "publication.source_reference",
                f"Fait publié par {fascicule} : il faut au moins {minimum} source(s) de référence ; le repérage ne suffit jamais.",
                fid,
                fichier_de,
            )
        for lien in fait["sources"]:
            if not lien.get("localisation"):
                self._erreur("publication.localisation", f"Source {lien['source_id']} sans entrée ni page consultée.", fid, fichier_de)

    def _exclu(self, fait: dict) -> bool:
        return fait["etat_relecture"] == "rejete" or fait["confiance"] == "non_retenue"

    def publiable(self) -> None:
        publiables = {fid: f for fid, f in self.c.fascicules.items() if not f["test"]}
        if not publiables:
            self.r.avertissement("publication.aucun_fascicule", "Aucun fascicule publiable : les fascicules de test ne sont jamais publiés.")
        for fsc_id, fsc in publiables.items():
            numero = f"le fascicule {fsc['numero']}"
            unites = {l["unite_id"] for l in fsc["unites"]}
            essentiels: set[str] = set()
            for uid in list(unites):
                unite = self.c.unites.get(uid)
                if not unite:
                    continue
                canonique = self._canonique(unite)
                essentiels |= {uid} | ({canonique["id"]} if canonique else set())
                sens = self._sens_affiche(uid)
                if sens:
                    essentiels.add(sens["id"])
                for recette in self._recettes_du_mot(uid):
                    cmp = self.c.compositions[recette["composition_id"]]
                    essentiels.add(cmp["id"])
                    essentiels |= {p["forme_id"] for p in cmp["parties"]} | {p["sens_id"] for p in cmp["parties"]}
            etymons = {r["unite_source_id"] for r in self.c.relations.values() if r["unite_cible_id"] in unites}
            perimetre_unites = unites | etymons
            for fid, fichier_de, fait in self._faits():
                dans_perimetre = (
                    fichier_de in perimetre_unites
                    or (fichier_de in self.c.compositions and self.c.compositions[fichier_de]["mot_id"] in unites)
                    or (fichier_de in self.c.relations and self.c.relations[fichier_de]["unite_cible_id"] in unites)
                )
                if not dans_perimetre:
                    continue
                if self._exclu(fait) and fid not in essentiels:
                    continue
                self._exiger_publiable(fid, fichier_de, fait, numero)
            for eid, exc in self.c.exclusions.items():
                if exc["fascicule_id"] == fsc_id and exc["etat_relecture"] != "relu":
                    self._erreur("publication.exclusion", f"Exclusion de {numero} sans relecture.", eid)


def controler(catalogue: Catalogue, rapport: Rapport, publication: bool = False) -> None:
    Controleur(catalogue, rapport, publication).executer()
