# Handoff — ressources non vérifiées dans Wikidata (ADR 0060)

État au 10 octobre 2026, branche `feature/recoupement-wikidata` (PR #31), après traitement du handoff précédent sur les 10 fascicules. Requêtes faites par le MCP `wikidata` seulement. Tout fait listé ici est `brouillon`, avec `sources: []` (ou un désaccord noté) et une `note_interne` qui dit ce qui manque. Rien de cette liste n’est publiable en prod tant que Wikidata ne le porte pas. Il ne liste que ce qui reste après ce traitement.

Méthode de recherche, rejouée le 10 octobre 2026 avant de conclure à une absence : lemme exact par langue ; affixes français avec et sans trait d’union, y compris le trait d’union insécable U+2011 (*bio‑*, L1569090, était manqué ainsi) et en vérifiant la catégorie lexicale (préfixe, suffixe) ; grec ancien sans tenir compte des accents ni des esprits (expression régulière sur le squelette consonantique) ; latin avec ou sans macrons ; forme de citation de Wikidata pour les verbes ; homographes départagés par catégorie, formes et sens. **Un « absent » veut dire « non trouvé par cette méthode »**, pas « inexistant ».

## Ce qui a été traité

- **Désaccords** : *tri-* et *nano-* reçoivent leur étymon latin déclaré par Wikidata (tres, nanus) ; *télescope* reçoit l’emprunt direct à *tēlescopium* ; *toponyme* reçoit la dérivation de *toponymie*. Les relations grecques ou italiennes que Wikidata ne cite pas sont gardées en confiance `discutee` (§ 1).
- **Homographes** : un seul lexème retenu pour caedere (L261461), inde (L276077), mille (L278705), τάφος (L1102328), avec la raison dans la note ; *minare* est confirmé (forme du lexème *mino*, L278739).
- **Allomorphes** : *ré* et *r* sont des formes déclarées du lexème L10464 (F6, F4).
- **Sens** : les 490 gloses rattachées à un lexème ont été comparées au sens Wikidata : 324 concordantes, 12 partielles, 8 discordantes (source Wikidata retirée, § 4), 154 sur des lexèmes sans sens, 1 sur un sens sans glose. La localisation cite désormais le sens retenu, et non plus le texte de la définition (défaut de 47 localisations du type `L313914-S1=terrain ou aménagé…`).
- **Chaînes des fascicules 5 à 10** : Wikidata ne déclare « dérivé du lexème » (P5191) que sur 16 des 308 lexèmes français du catalogue. Toutes les déclarations utiles sont saisies : 8 étymons (tres, nanus, antonymum, synonymum, συνώνυμος, τεχνικός, Autismus, Monismus) et 10 relations (antonyme, synonyme ← synonymum ← συνώνυμος, technique, autisme, monisme, toponyme, tri-, nano-, télescope), ainsi que βίος → bio-. Aucune autre chaîne n’est saisie : les mots restants n’ont pas de P5191.
- **Explications détaillées** : 493 explications rédigées pour les fascicules 5 à 10, sans date ni personne ; dans les fascicules 1 à 4, les attributions et datations que Wikidata ne porte pas sont retirées, celles qu’il contredit sont corrigées (télescope, photophone, écologie), dans 45 compositions (§ 8).
- **Langues** : le latin reçoit son début de période (−800, P580 de Q397).

## Résumé de ce qui reste

| Ressource | Non vérifiée |
|---|---|
| Unités sans lexème trouvé | 584 (français 543, grec ancien 37, latin 4) |
| Allomorphes non vérifiés | 5 |
| Sens discordants ou partiels | 8 + 12 |
| Sens sur un lexème sans sens | 154 |
| Relations sans déclaration P5191 | 228 (dont 3 discutées) |
| Compositions sans déclaration P5238 | 707 (dont 6 désaccords) |
| Attestations retirées (aucun P6684) | 466 |
| Langues sans période | 5 |

## 1. Désaccords avec Wikidata (à trancher en premier)

- Relation `forme_de_composition` τρεῖς → tri- (`relations/rel_5wg55tka.json`, confiance `discutee`) : Wikidata dérive « tri- » (L620229, déclaration référencée) du latin tres, relation saisie à part ; il ne cite pas le grec τρεῖς : à trancher en relecture.
- Relation `forme_de_composition` νᾶνος → nano- (`relations/rel_bcpepxgt.json`, confiance `discutee`) : Wikidata dérive « nano- » (L20367, déclaration référencée) du latin nanus, relation saisie à part ; il ne cite pas le grec νᾶνος : à trancher en relecture.
- Relation `emprunt` telescopio → télescope (`relations/rel_f4zjf60q.json`, confiance `discutee`) : Wikidata dérive « télescope » (L1460339, déclaration référencée) du latin tēlescopium, relation saisie à part ; il ne cite pas l’italien telescopio : à trancher en relecture.
- Composition mégamètre = méga- + -mètre (`compositions/cmp_350t2c9a.json`) : Wikidata compose « mégamètre » de L18790 + méga-, le catalogue de méga + mètre : à trancher en relecture.
- Composition millimètre = milli- + -mètre (`compositions/cmp_aww7r1kk.json`) : Wikidata compose « millimètre » de milli- + L18790, le catalogue de milli + mètre : à trancher en relecture.
- Composition décimètre = déci- + -mètre (`compositions/cmp_c6kngvyk.json`) : Wikidata compose « décimètre » de déci- + L18790, le catalogue de déci + mètre : à trancher en relecture.
- Composition kilomètre = kilo- + -mètre (`compositions/cmp_dsnpzz1h.json`) : Wikidata compose « kilomètre » de L18790 + kilo-, le catalogue de kilo + mètre : à trancher en relecture.
- Composition nanomètre = nano- + -mètre (`compositions/cmp_km6m6pcq.json`) : Wikidata compose « nanomètre » de nano- + L18790, le catalogue de nano + mètre : à trancher en relecture.
- Composition centimètre = centi- + -mètre (`compositions/cmp_q4jcnyh5.json`) : Wikidata compose « centimètre » de centi- + L18790, le catalogue de centi + mètre : à trancher en relecture.
- Les six mesures en *-mètre* ne sont pas réalignées : Wikidata les compose avec le nom *mètre* (L18790), le catalogue avec le suffixe *-mètre*, qui porte une brique. Changer la partie changerait les recettes : décision de game design.
- Composition synchronique toponyme = topo- + -onyme : Wikidata (L1469114) dérive le mot de *toponymie* ; la dérivation est désormais saisie, l’analyse en briques reste à confirmer.
- Composition historique Ökologie = οἶκος + λόγος : Wikidata (L499774) la compose de *öko-* + *-logie* (allemand), absents du catalogue.
- Unité φαγεῖν : rattachée à ἐσθίω (L1109170), dont c’est l’aoriste supplétif ; Wikidata ne liste pas la forme.
- Sens de caedere : le lexème retenu (L261461) n’a qu’un sens « keikogi », sans rapport avec le verbe : erreur probable dans Wikidata, à signaler sur Wikidata.

## 2. Unités sans lexème Wikidata

Fascicule entre parenthèses (— : étymon ou hors fascicule). Pour les mots français, beaucoup ont un élément Q (concept) mais pas de lexème : un lexème est ce qui déclare la forme du mot.

### français (543)

-gramme (1), -graphe (1), -graphie (1), -logie (1), -logue (1), -morphe (1), -nomie (1), -pathie (1), -phage (1), -phile (1), -phobie (1), -sophie (1), -sphère (1), -thérapie (1), anthropo- (1), anthropobiologie (1), anthropographie (1), anthropologiste (1), anthropomorphe (1), anthropophage (1), anthropophobie (1), biographiste (1), biogène (1), biologue (1), biomorphe (1), bionomie (1), biothérapie (1), neuro- (1), neurologiste (1), neuropathie (1), philo- (1), psycho- (1), psychographie (1), psychogène (1), psychologiste (1), sociogramme (1), sociologiste (1), sociopathie (1), zoo- (1), zoomorphe (1), zoonomie (1), zoophile (1), zoophobie (1), -mètre (2), -phone (2), -scope (2), agro- (2), agrologie (2), archéo- (2), astrophobie (2), baro- (2), barogramme (2), géologiste (2), géophage (2), géophone (2), géosphère (2), hydrogramme (2), hydrologiste (2), hydromorphe (2), hydrophile (2), hydrophobie (2), hydrophone (2), hydrosphère (2), thermo- (2), thermobiologie (2), thermogramme (2), thermographe (2), thermogène (2), thermophile (2), thermophobie (2), thermoscope (2), thermosphère (2), thermothérapie (2), xylo- (2), xylophage (2), xylophone (2), biblio- (3), calli- (3), calligraphiste (3), chrono- (3), chronogramme (3), chronophobie (3), chronoscope (3), dia- (3), micrographie (3), paléo- (3), paléographe (3), photo- (3), photogramme (3), photogène (3), photomètre (3), photophile (3), photophobie (3), photophone (3), photothérapie (3), péri- (3), télé- (3), télégramme (3), télégraphiste (3), télémètre (3), télépathie (3), étymo- (3), -chrome (4), -cratie (4), -gamie (4), a- (4), a- (4), amoral (4), aphone (4), autochrome (4), cardio- (4), cardiogramme (4), cardiographe (4), cardiographie (4), démo- (4), démographe (4), en- (4), mono- (4), monochrome (4), monocratie (4), monogamie (4), monogramme (4), monomorphe (4), monophobie (4), poly- (4), polychrome (4), polygamie (4), polygraphie (4), polymorphe (4), polyphone (4), techno- (4), technocratie (4), technophile (4), technophobie (4), théo- (4), théocratie (4), théologue (4), éco- (4), -onyme (5), -onymie (5), achrome (5), agamie (5), agraphie (5), alogie (5), amorphe (5), anthroponyme (5), antigène (5), antilogie (5), antonymie (5), autonyme (5), autonymie (5), barographe (5), baroscope (5), biomètre (5), biophile (5), bioscope (5), cardiomètre (5), cardiophone (5), cardioscope (5), chronographie (5), diaphone (5), diascope (5), démonyme (5), géophile (5), géoscope (5), homo- (5), homochrome (5), homogamie (5), homographe (5), homogène (5), homologie (5), homologue (5), homomorphe (5), homophile (5), homophone (5), homosphère (5), hydrique (5), hydromètre (5), hydronyme (5), hydronymie (5), hydroscope (5), hétéro- (5), hétérochrome (5), hétérogamie (5), hétérographie (5), hétérogène (5), hétérologue (5), hétéromorphe (5), hétéronomie (5), hétéronyme (5), microbiologique (5), monomètre (5), monophone (5), neurologique (5), photique (5), photochrome (5), polyscope (5), pseudo- (5), pseudologie (5), pseudomorphe (5), pseudonymie (5), remener (5), syn- (5), syngamie (5), synonymie (5), thermologie (5), thermophone (5), théonyme (5), topique (5), topo- (5), télégraphique (5), xylophile (5), zoographe (5), zoographie (5), zoologique (5), zoomètre (5), zooscope (5), -crate (6), -gonie (6), -phonie (6), anthropogonie (6), antiphonie (6), aphonie (6), autocrate (6), autophonie (6), cosmique (6), cosmo- (6), cosmobiologie (6), cosmologique (6), cosmologue (6), cosmosophie (6), cryptogamie (6), cryptogramme (6), cryptonyme (6), diaphonie (6), ethno- (6), ethnologique (6), ethnonyme (6), grapho- (6), graphomètre (6), graphothérapie (6), géogonie (6), homophonie (6), hétérophonie (6), idéo- (6), idéogramme (6), idéographie (6), idéologique (6), idéologue (6), idéophone (6), lexico- (6), microphonie (6), monocrate (6), monophonie (6), mytho- (6), mythographie (6), mythologique (6), ortho- (6), orthochrome (6), orthographie (6), orthonyme (6), orthophonie (6), orthoscope (6), phono- (6), phonogramme (6), phonographe (6), phonographie (6), phonologique (6), phonologue (6), phonomètre (6), phonophobie (6), photophonie (6), technocrate (6), théocrate (6), théogonie (6), -archie (7), -arque (7), -glyphe (7), -lithe (7), -mancie (7), -taphe (7), -thèque (7), -tomie (7), ana- (7), anaglyphe (7), anarchique (7), anatomique (7), anthropomancie (7), astromancie (7), atmo- (7), atmomètre (7), autarchie (7), autotomie (7), bibliomancie (7), cardiotomie (7), carto- (7), cartogramme (7), cristallo- (7), cristallomancie (7), ethnarchie (7), ethnarque (7), géoglyphe (7), hiérarchique (7), hiérarque (7), hiéro- (7), hiérocrate (7), hiéroglyphe (7), hiérogramme (7), hydromancie (7), hétérarchie (7), lithique (7), litho- (7), lithographique (7), lithographiste (7), lithomancie (7), lithophage (7), lithophone (7), lithosphère (7), lithothèque (7), lithothérapie (7), lithotomie (7), microlithe (7), microtomie (7), monarchique (7), monarchiste (7), monolithe (7), mégaphone (7), mégascope (7), neurotomie (7), oligarchie (7), oligo- (7), phonolithe (7), phonothèque (7), photothèque (7), polyarchie (7), pétro- (7), pétroglyphe (7), pétrographe (7), strato- (7), stratosphère (7), théarchie (7), théomancie (7), xylothèque (7), zoolithe (7), zootomie (7), épi- (7), épigraphe (7), épigène (7), épilogue (7), épiscope (7), éponyme (7), -cardie (8), -some (8), -type (8), -typie (8), autosome (8), biotype (8), bradycardie (8), centilitre (8), centimétrique (8), chromique (8), chromo- (8), chromogène (8), chromosome (8), chromosphère (8), chromothérapie (8), chromotypie (8), diarchie (8), digamie (8), digramme (8), dimorphe (8), diphone (8), décalogue (8), décamètre (8), hectolitre (8), hectomètre (8), homotype (8), kilolitre (8), lithotypie (8), microlitre (8), microsome (8), millilitre (8), millimétrique (8), monotype (8), monotypie (8), nanolitre (8), phototype (8), phototypie (8), polysome (8), polytype (8), proto- (8), sténo- (8), sténogramme (8), sténographique (8), sténotype (8), sténotypie (8), stéréo- (8), stéréogramme (8), stéréographie (8), stéréomètre (8), stéréophonie (8), stéréophonique (8), stéréoscope (8), stéréotypie (8), tachy- (8), tachycardie (8), tachygraphe (8), tachygraphie (8), tachymètre (8), triarchie (8), trigamie (8), triglyphe (8), trilogie (8), trimorphe (8), trimètre (8), typo- (8), typomètre (8), télétype (8), tétra- (8), tétragramme (8), tétralogie (8), tétramètre (8), tétrarchie (8), écotype (8), -algie (9), -iatre (9), -iatrie (9), -scopie (9), cardialgie (9), dermato- (9), dermatoglyphe (9), dermatologique (9), dermatologiste (9), endo- (9), endogamie (9), endogène (9), endoscope (9), exo- (9), exogamie (9), exogène (9), exosphère (9), gastralgie (9), gastrique (9), gastro- (9), gastropathie (9), gastroscope (9), gastroscopie (9), gastrotomie (9), gynéco- (9), gynécocratie (9), gynécologiste (9), hématique (9), hémato- (9), hématologique (9), hématologiste (9), hématophage (9), hématophobie (9), myalgie (9), myo- (9), myologie (9), myopathie (9), nost- (9), névralgie (9), odontalgie (9), odonto- (9), ophtalmique (9), ophtalmo- (9), ophtalmologique (9), ophtalmologiste (9), ophtalmoscope (9), ophtalmoscopie (9), ostéo- (9), ostéotomie (9), otalgie (9), oto- (9), otoscope (9), otoscopie (9), patho- (9), phoniatre (9), -cide (10), -drome (10), -mane (10), -manie (10), -naute (10), -phagie (10), -thermie (10), anomie (10), anthropophagie (10), aphagie (10), autodrome (10), aéro- (10), aérogramme (10), aéronaute (10), aérophagie (10), aérophobie (10), bibliomane (10), bibliomanie (10), biocide (10), cosmodrome (10), diathermie (10), ethnocide (10), graphomane (10), graphomanie (10), géno- (10), génotype (10), géophagie (10), hippiatrie (10), hippo- (10), hippodrome (10), hippophage (10), hippophagie (10), hippothérapie (10), hypergamie (10), hyperphagie (10), hyperthermie (10), hypo- (10), hypogamie (10), hyponyme (10), hématophagie (10), monarchisme (10), monomane (10), monomanie (10), monothéisme (10), moralisme (10), mythomane (10), mythomanie (10), néo- (10), néologie (10), néophobie (10), palin- (10), pan- (10), pangramme (10), panthéisme (10), polyphagie (10), polythéisme (10), prodrome (10), psychisme (10), psychologisme (10), théisme (10), théiste (10), xylophagie (10), écocide (10), économisme (10)

### grec ancien (37)

-λογία (—), αὐτοκράτεια (—), αὐτόγραφος (—), αὐτός (—), βιογραφία (—), γεωγράφος (—), γεωμέτρης (—), δίς (—), διά (—), διαγράφειν (—), καλλιγράφος (—), καλλιγραφία (—), μονόχρωμος (—), ξυλοφάγος (—), πάλιν (—), περίμετρος (—), πολυγραφία (—), πολύγαμος (—), πολύγραφος (—), πολύμορφος (—), πολύφωνος (—), πολύχρωμος (—), πᾶς (—), σύν (—), τέτταρες (—), τῆλε (—), χίλιοι (—), ἀνά (—), ἀνθρωπολόγος (—), ἀνθρωπόμορφος (—), ἀντί (—), ἀρχαιολογία (—), ἑκατόν (—), ἔνδον (—), ἰατρεία (—), ὑδροφοβία (—), ὑπό (—)

### latin (4)

ismus (—), palaeographia (—), psychologia (—), xylophagus (—)

## 3. Allomorphes

- a- : forme « an » — allomorphe « an » non vérifié parmi les formes du lexème (aucun lexème).
- en- : forme « em » — allomorphe « em » non vérifié parmi les formes du lexème (aucun lexème).
- neuro- : forme « névr » — allomorphe « névr » non vérifié parmi les formes du lexème (aucun lexème).
- philo- : forme « phil » — allomorphe « phil » non vérifié parmi les formes du lexème (aucun lexème).
- syn- : forme « sym » — allomorphe « sym » non vérifié parmi les formes du lexème (aucun lexème).

## 4. Sens

### Discordants (retirés)

- ramener (4) : glose comparée au sens Wikidata (L16496-S1) : le seul sens Wikidata est « donner une courbure convexe à une lentille », pas « amener de nouveau ».
- mener (4) : glose comparée au sens Wikidata (L9287-S1) : le seul sens Wikidata est « être en tête (d’une course) », pas « conduire, guider ».
- amener (4) : glose comparée au sens Wikidata (L9162-S1) : le seul sens Wikidata est « abaisser (une voile, un pavillon) », pas « faire venir ».
- archéographie (2) : glose comparée au sens Wikidata (L1583073-S1) : le sens Wikidata est l’édition de sources historiques, pas la description des monuments antiques.
- micromètre (3) : glose comparée au sens Wikidata (L1506060-S1) : le seul sens Wikidata est l’unité de longueur (micromètre), pas l’instrument de mesure.
- polygraphe (4) : glose comparée au sens Wikidata (L25537-S1 ou L25537-S2) : Wikidata ne donne que « auteur qui écrit sur des sujets variés » et un métier de l’imprimerie, pas le détecteur de mensonge.
- caedere (—) : glose comparée au sens Wikidata (L261461-S1) : le lexème retenu (L261461) n’a qu’un sens « keikogi », sans rapport avec « abattre, tuer » : erreur probable dans Wikidata.
- prologue (4) : glose comparée au sens Wikidata (L28728-S1) : le seul sens Wikidata est l’étape de cyclisme, pas le texte qui ouvre une œuvre.

### Concordance partielle (gardés, à relire)

- -ique (L709304-S2) : le sens Wikidata le plus proche est « qui a, qui comprend », la glose « qui se rapporte à » est plus large.
- photographiste (L1577227-S1) : le sens Wikidata est l’artiste qui pratique le « photographisme », pas le professionnel qui réalise ou retouche des images.
- ἀ- (L36457-S1) : le sens Wikidata est « contraire de », la glose « sans » donne la privation.
- géothermie (L20727-S1) : le sens Wikidata ne couvre que l’électricité tirée de la géothermie, pas la chaleur interne de la Terre.
- analogue (L17169-S1) : le sens Wikidata est chimique (composé de structure voisine), la glose est générale.
- gastronomie (L19567-S1) : le sens Wikidata vise la restauration (métier), la glose l’art de bien manger.
- cosmographe (L1577047-S1) : le sens Wikidata est un métier médiéval, ancêtre du cartographe, pas le savant qui décrit l’univers.
- calligraphique (L12366-S1) : le sens Wikidata est une classe de caractères (Vox-ATypI), la glose est générale.
- théosophie (L1583380-S1) : le sens Wikidata vise le mouvement fondé par Helena Blavatsky, la glose est plus générale.
- analogie (L20729-S1) : le sens Wikidata est le raisonnement par analogie, plus étroit que le rapport de ressemblance.
- oligarque (L1582122-S1) : le sens Wikidata ne couvre que l’homme d’affaires influent, pas le membre d’une oligarchie.
- méga- (L20359-S1) : le sens Wikidata est le préfixe d’unité (×10⁶), la glose « grand » donne le sens grec.

### Lexèmes sans sens

Ces gloses citent le lexème, qui ne déclare aucun sens : elles restent à vérifier. Par langue : grec ancien 124, latin 28, allemand 1, français 1.

## 5. Relations sans appui

| Type | Source → cible | Ce qui manque | Fichier |
|---|---|---|---|
| forme_de_composition | μέτρον → -mètre | aucune déclaration « dérivé du lexème » entre « mètre » (sans lexème) et « μέτρον » (L656868). | `rel_02awsdna.json` |
| emprunt | hydrophobia → hydrophobie | aucune déclaration « dérivé du lexème » entre « hydrophobie » (sans lexème) et « hydrophobia » (L1053347). | `rel_07hfy4zm.json` |
| emprunt | βιογραφία → biographie | aucune déclaration « dérivé du lexème » entre « biographie » (L19853) et « βιογραφία » (sans lexème). | `rel_091730da.json` |
| emprunt | πολύγραφος → polygraphe | aucune déclaration « dérivé du lexème » entre « polygraphe » (L25537) et « πολύγραφος » (sans lexème). | `rel_0dngbna1.json` |
| forme_de_composition | λίθος → -lithe | aucune déclaration « dérivé du lexème » entre « lithe » (sans lexème) et « λίθος » (L1102032). | `rel_0fxcm9yp.json` |
| forme_de_composition | πάθος → -pathie | aucune déclaration « dérivé du lexème » entre « pathie » (sans lexème) et « πάθος » (L1102083). | `rel_0wq2h5a9.json` |
| forme_de_composition | φόβος → -phobie | aucune déclaration « dérivé du lexème » entre « phobie » (sans lexème) et « φόβος » (L1101868). | `rel_0y8crpt2.json` |
| forme_de_composition | νεῦρον → neuro- | aucune déclaration « dérivé du lexème » entre « neuro » (sans lexème) et « νεῦρον » (L1102681). | `rel_1149zaxz.json` |
| forme_de_composition | γράφειν → -graphe | aucune déclaration « dérivé du lexème » entre « graphe » (sans lexème) et « γράφειν » (L1109137). | `rel_11x3jwy4.json` |
| emprunt | icus → -ique | aucune déclaration « dérivé du lexème » entre « ique » (L709304) et « icus » (L640177). | `rel_13qhmyxn.json` |
| emprunt | prologus → prologue | aucune déclaration « dérivé du lexème » entre « prologue » (L28728) et « prologus » (L282387). | `rel_14a1xerm.json` |
| forme_de_composition | κράτος → -crate | aucune déclaration « dérivé du lexème » entre « crate » (sans lexème) et « κράτος » (L1102056). | `rel_1cmry18x.json` |
| emprunt | διάλογος → dialogus | aucune déclaration « dérivé du lexème » entre « dialogus » (L271800) et « διάλογος » (L1104015). | `rel_1e7mqxyv.json` |
| forme_de_composition | τομή → -tomie | aucune déclaration « dérivé du lexème » entre « tomie » (sans lexème) et « τομή » (L1103207). | `rel_1ga7swn7.json` |
| forme_de_composition | decimus → déci- | aucune déclaration « dérivé du lexème » entre « déci » (L20363) et « decimus » (L1049002). | `rel_1h9yw0dg.json` |
| emprunt | telescopium → telescopio | aucune déclaration « dérivé du lexème » entre « telescopio » (L1179565) et « telescopium » (L579312). | `rel_1hy16v5j.json` |
| forme_de_composition | φωνή → phono- | aucune déclaration « dérivé du lexème » entre « phono » (sans lexème) et « φωνή » (L1101957). | `rel_1tdgeqhv.json` |
| forme_de_composition | γράφειν → grapho- | aucune déclaration « dérivé du lexème » entre « grapho » (sans lexème) et « γράφειν » (L1109137). | `rel_1wh9mv59.json` |
| emprunt | oeconomia → économie | aucune déclaration « dérivé du lexème » entre « économie » (L13696) et « oeconomia » (L279937). | `rel_24gf1m9g.json` |
| forme_de_composition | ταχύς → tachy- | aucune déclaration « dérivé du lexème » entre « tachy » (sans lexème) et « ταχύς » (L1110846). | `rel_2b9nym0c.json` |
| emprunt | polygamia → polygamie | aucune déclaration « dérivé du lexème » entre « polygamie » (sans lexème) et « polygamia » (L1062163). | `rel_2d18jbrw.json` |
| forme_de_composition | αἷμα → hémato- | aucune déclaration « dérivé du lexème » entre « hémato » (sans lexème) et « αἷμα » (L1101949). | `rel_2m4t0y62.json` |
| emprunt | πολύφωνος → polyphone | aucune déclaration « dérivé du lexème » entre « polyphone » (sans lexème) et « πολύφωνος » (sans lexème). | `rel_2mdayanj.json` |
| emprunt | psychologia → psychologie | aucune déclaration « dérivé du lexème » entre « psychologie » (L23845) et « psychologia » (sans lexème). | `rel_33mhbvjf.json` |
| forme_de_composition | θεός → théo- | aucune déclaration « dérivé du lexème » entre « théo » (sans lexème) et « θεός » (L1101950). | `rel_3gejs0nf.json` |
| forme_de_composition | σῶμα → -some | aucune déclaration « dérivé du lexème » entre « some » (sans lexème) et « σῶμα » (L1101921). | `rel_3hg91wrh.json` |
| forme_de_composition | φαγεῖν → -phage | aucune déclaration « dérivé du lexème » entre « phage » (sans lexème) et « φαγεῖν » (L1109170). | `rel_3rrx9dt4.json` |
| emprunt | θεοσοφία → théosophie | aucune déclaration « dérivé du lexème » entre « théosophie » (L1583380) et « θεοσοφία » (L1104280). | `rel_3wpcpqwr.json` |
| forme_de_composition | πᾶς → pan- | aucune déclaration « dérivé du lexème » entre « pan » (sans lexème) et « πᾶς » (sans lexème). | `rel_496mp28z.json` |
| forme_de_composition | ἀτμός → atmo- | aucune déclaration « dérivé du lexème » entre « atmo » (sans lexème) et « ἀτμός » (L1103287). | `rel_4gvwe8jp.json` |
| forme_de_composition | ἀήρ → aéro- | aucune déclaration « dérivé du lexème » entre « aéro » (sans lexème) et « ἀήρ » (L1101995). | `rel_4hax6yq7.json` |
| forme_de_composition | ἕτερος → hétéro- | aucune déclaration « dérivé du lexème » entre « hétéro » (sans lexème) et « ἕτερος » (L1110826). | `rel_4kkfajmj.json` |
| forme_de_composition | ἑκατόν → hecto- | aucune déclaration « dérivé du lexème » entre « hecto » (L20361) et « ἑκατόν » (sans lexème). | `rel_4z2a584y.json` |
| forme_de_composition | ἔξω → exo- | aucune déclaration « dérivé du lexème » entre « exo » (sans lexème) et « ἔξω » (L1073011). | `rel_52bmhq8r.json` |
| emprunt | αὐτονομία → autonomie | aucune déclaration « dérivé du lexème » entre « autonomie » (L13691) et « αὐτονομία » (L1103058). | `rel_54yv8ynz.json` |
| forme_de_composition | φίλος → -phile | aucune déclaration « dérivé du lexème » entre « phile » (sans lexème) et « φίλος » (L1110850). | `rel_57x1b0z5.json` |
| emprunt | astrologus → astrologue | aucune déclaration « dérivé du lexème » entre « astrologue » (L621686) et « astrologus » (L259510). | `rel_58f32tfq.json` |
| heritage | male → mal | aucune déclaration « dérivé du lexème » entre « mal » (L1571446) et « male » (L278230). | `rel_5avr638y.json` |
| emprunt | φιλόλογος → philologus | aucune déclaration « dérivé du lexème » entre « philologus » (L281318) et « φιλόλογος » (L1111190). | `rel_5bc7s2aj.json` |
| forme_de_composition | δίς → di- | aucune déclaration « dérivé du lexème » entre « di » (L700776) et « δίς » (sans lexème). | `rel_5egreyhz.json` |
| emprunt | etymologia → étymologie | aucune déclaration « dérivé du lexème » entre « étymologie » (L17663) et « etymologia » (L273145). | `rel_5jjesd4t.json` |
| emprunt | philosophia → philosophie | aucune déclaration « dérivé du lexème » entre « philosophie » (L19841) et « philosophia » (L281320). | `rel_6hz9rzcg.json` |
| forme_de_composition | ἔτυμον → étymo- | aucune déclaration « dérivé du lexème » entre « étymo » (sans lexème) et « ἔτυμον » (L305). | `rel_6msbs5pn.json` |
| derivation | νέμειν → νόμος | aucune déclaration « dérivé du lexème » entre « νόμος » (L1101939) et « νέμειν » (L1110170). | `rel_6pb5rtgt.json` |
| forme_de_composition | παλαιός → paléo- | aucune déclaration « dérivé du lexème » entre « paléo » (sans lexème) et « παλαιός » (L1110844). | `rel_6srz83z8.json` |
| forme_de_composition | ἔνδον → endo- | aucune déclaration « dérivé du lexème » entre « endo » (sans lexème) et « ἔνδον » (sans lexème). | `rel_6t05aeet.json` |
| emprunt | geographus → géographe | aucune déclaration « dérivé du lexème » entre « géographe » (L1576808) et « geographus » (L1052481). | `rel_75mt7q8e.json` |
| forme_de_composition | ξύλον → xylo- | aucune déclaration « dérivé du lexème » entre « xylo » (sans lexème) et « ξύλον » (L1103300). | `rel_7agz0jbj.json` |
| derivation | φιλόλογος → φιλολογία | aucune déclaration « dérivé du lexème » entre « φιλολογία » (L1103799) et « φιλόλογος » (L1111190). | `rel_7c3qea41.json` |
| emprunt | perimetros → périmètre | aucune déclaration « dérivé du lexème » entre « périmètre » (L21413) et « perimetros » (L281001). | `rel_7cex5dgy.json` |
| forme_de_composition | ἔθνος → ethno- | aucune déclaration « dérivé du lexème » entre « ethno » (sans lexème) et « ἔθνος » (L1101890). | `rel_7n55c8t8.json` |
| forme_de_composition | μῦθος → mytho- | aucune déclaration « dérivé du lexème » entre « mytho » (sans lexème) et « μῦθος » (L1102157). | `rel_804r7e7h.json` |
| forme_de_composition | μανία → -manie | aucune déclaration « dérivé du lexème » entre « manie » (sans lexème) et « μανία » (L1102623). | `rel_8c6qkvcm.json` |
| forme_de_composition | σκοπεῖν → -scopie | aucune déclaration « dérivé du lexème » entre « scopie » (sans lexème) et « σκοπεῖν » (L1110522). | `rel_8mkfmddt.json` |
| forme_de_composition | τόπος → topo- | aucune déclaration « dérivé du lexème » entre « topo » (sans lexème) et « τόπος » (L1102031). | `rel_8ndg3qdj.json` |
| forme_de_composition | αὐτός → auto- | aucune déclaration « dérivé du lexème » entre « auto » (L28893) et « αὐτός » (sans lexème). | `rel_8q8a71q0.json` |
| forme_de_composition | τύπος → -typie | aucune déclaration « dérivé du lexème » entre « typie » (sans lexème) et « τύπος » (L1102612). | `rel_91jb70km.json` |
| forme_de_composition | κράτος → -cratie | aucune déclaration « dérivé du lexème » entre « cratie » (sans lexème) et « κράτος » (L1102056). | `rel_93ddtn0x.json` |
| forme_de_composition | καρδία → cardio- | aucune déclaration « dérivé du lexème » entre « cardio » (sans lexème) et « καρδία » (L1101893). | `rel_9b0v0qwb.json` |
| derivation | πολύγαμος → πολυγαμία | aucune déclaration « dérivé du lexème » entre « πολυγαμία » (L1103964) et « πολύγαμος » (sans lexème). | `rel_9ds5j40g.json` |
| forme_de_composition | ἀντί → anti- | aucune déclaration « dérivé du lexème » entre « anti » (L28892) et « ἀντί » (sans lexème). | `rel_9ktyk7sc.json` |
| forme_de_composition | τέχνη → techno- | aucune déclaration « dérivé du lexème » entre « techno » (sans lexème) et « τέχνη » (L1102096). | `rel_9qy3h73b.json` |
| forme_de_composition | τάφος → -taphe | aucune déclaration « dérivé du lexème » entre « taphe » (sans lexème) et « τάφος » (L1102328). | `rel_9t86v8xs.json` |
| forme_de_composition | θέρμη → -thermie | aucune déclaration « dérivé du lexème » entre « thermie » (sans lexème) et « θέρμη » (L1103314). | `rel_a6sna1c6.json` |
| forme_de_composition | φαγεῖν → -phagie | aucune déclaration « dérivé du lexème » entre « phagie » (sans lexème) et « φαγεῖν » (L1109170). | `rel_a70qssvs.json` |
| heritage | inde → en- | aucune déclaration « dérivé du lexème » entre « en » (sans lexème) et « inde » (L276077). | `rel_a7f9d6qw.json` |
| emprunt | περίμετρος → perimetros | aucune déclaration « dérivé du lexème » entre « perimetros » (L281001) et « περίμετρος » (sans lexème). | `rel_aa3evsxy.json` |
| emprunt | ἀνθρωπόμορφος → anthropomorphe | aucune déclaration « dérivé du lexème » entre « anthropomorphe » (sans lexème) et « ἀνθρωπόμορφος » (sans lexème). | `rel_ac0cpybn.json` |
| forme_de_composition | ἀγρός → agro- | aucune déclaration « dérivé du lexème » entre « agro » (sans lexème) et « ἀγρός » (L1101935). | `rel_adjmfh0d.json` |
| forme_de_composition | θήκη → -thèque | aucune déclaration « dérivé du lexème » entre « thèque » (sans lexème) et « θήκη » (L1102279). | `rel_atw8535f.json` |
| emprunt | ἀστρολόγος → astrologus | aucune déclaration « dérivé du lexème » entre « astrologus » (L259510) et « ἀστρολόγος » (L1111283). | `rel_beb4t84k.json` |
| emprunt | ἀ- → a- | aucune déclaration « dérivé du lexème » entre « a » (sans lexème) et « ἀ » (L36457). | `rel_bj57cth9.json` |
| emprunt | astrologia → astrologie | aucune déclaration « dérivé du lexème » entre « astrologie » (L24148) et « astrologia » (L259509). | `rel_bk2ygshw.json` |
| forme_de_composition | ὄνομα → -onyme | aucune déclaration « dérivé du lexème » entre « onyme » (sans lexème) et « ὄνομα » (L1102060). | `rel_bnenpja9.json` |
| forme_de_composition | ἀνά → ana- | aucune déclaration « dérivé du lexème » entre « ana » (sans lexème) et « ἀνά » (sans lexème). | `rel_bt3sccp9.json` |
| forme_de_composition | ὕδωρ → hydro- | aucune déclaration « dérivé du lexème » entre « hydro » (L28894) et « ὕδωρ » (L2091). | `rel_bxt4arks.json` |
| emprunt | Biologie → biologie | aucune déclaration « dérivé du lexème » entre « biologie » (L17250) et « Biologie » (L6825). | `rel_c39gm1me.json` |
| forme_de_composition | φωνή → -phonie | aucune déclaration « dérivé du lexème » entre « phonie » (sans lexème) et « φωνή » (L1101957). | `rel_c8qrywvd.json` |
| forme_de_composition | μανία → -mane | aucune déclaration « dérivé du lexème » entre « mane » (sans lexème) et « μανία » (L1102623). | `rel_ca5wnkzh.json` |
| emprunt | geometres → géomètre | aucune déclaration « dérivé du lexème » entre « géomètre » (L1582412) et « geometres » (L274723). | `rel_cavw7j3n.json` |
| forme_de_composition | νόμος → -nomie | aucune déclaration « dérivé du lexème » entre « nomie » (sans lexème) et « νόμος » (L1101939). | `rel_cbfpszfn.json` |
| forme_de_composition | μορφή → -morphe | aucune déclaration « dérivé du lexème » entre « morphe » (sans lexème) et « μορφή » (L1102046). | `rel_cdqgpcah.json` |
| forme_de_composition | ὑπό → hypo- | aucune déclaration « dérivé du lexème » entre « hypo » (sans lexème) et « ὑπό » (sans lexème). | `rel_cg2g0k4z.json` |
| emprunt | λίτρα → litre | aucune déclaration « dérivé du lexème » entre « litre » (L18821) et « λίτρα » (L1103786). | `rel_cjk8y4r1.json` |
| forme_de_composition | δῆμος → démo- | aucune déclaration « dérivé du lexème » entre « démo » (sans lexème) et « δῆμος » (L1102123). | `rel_cqcsc3e8.json` |
| forme_de_composition | ἀρχή → -archie | aucune déclaration « dérivé du lexème » entre « archie » (sans lexème) et « ἀρχή » (L1101898). | `rel_ctv8w77h.json` |
| forme_de_composition | ἄλγος → -algie | aucune déclaration « dérivé du lexème » entre « algie » (sans lexème) et « ἄλγος » (L1102880). | `rel_cw2knhm1.json` |
| forme_de_composition | σφαῖρα → -sphère | aucune déclaration « dérivé du lexème » entre « sphère » (sans lexème) et « σφαῖρα » (L1103409). | `rel_cw8zq85z.json` |
| emprunt | ἀστρολογία → astrologia | aucune déclaration « dérivé du lexème » entre « astrologia » (L259509) et « ἀστρολογία » (L1103449). | `rel_d61tcxj7.json` |
| emprunt | ἀστρονομία → astronomia | aucune déclaration « dérivé du lexème » entre « astronomia » (L259511) et « ἀστρονομία » (L1103596). | `rel_d7rcj5n1.json` |
| forme_de_composition | ἰατρός → -iatre | aucune déclaration « dérivé du lexème » entre « iatre » (sans lexème) et « ἰατρός » (L1101891). | `rel_de5jx30f.json` |
| forme_de_composition | κάλλος → calli- | aucune déclaration « dérivé du lexème » entre « calli » (sans lexème) et « κάλλος » (L1102764). | `rel_dhdacw23.json` |
| forme_de_composition | σύν → syn- | aucune déclaration « dérivé du lexème » entre « syn » (sans lexème) et « σύν » (sans lexème). | `rel_e4pe7h9g.json` |
| forme_de_composition | γυνή → gynéco- | aucune déclaration « dérivé du lexème » entre « gynéco » (sans lexème) et « γυνή » (L1101926). | `rel_e7kthqye.json` |
| forme_de_composition | θεραπεία → -thérapie | aucune déclaration « dérivé du lexème » entre « thérapie » (sans lexème) et « θεραπεία » (L1103285). | `rel_egwxfycw.json` |
| forme_de_composition | κόσμος → cosmo- | aucune déclaration « dérivé du lexème » entre « cosmo » (sans lexème) et « κόσμος » (L1101841). | `rel_ehavkkaa.json` |
| forme_de_composition | χρῶμα → -chrome | aucune déclaration « dérivé du lexème » entre « chrome » (sans lexème) et « χρῶμα » (L1103315). | `rel_eny7qb69.json` |
| emprunt | πολυγραφία → polygraphie | aucune déclaration « dérivé du lexème » entre « polygraphie » (sans lexème) et « πολυγραφία » (sans lexème). | `rel_exh9k5dr.json` |
| emprunt | chronologia → chronologie | aucune déclaration « dérivé du lexème » entre « chronologie » (L17249) et « chronologia » (L263706). | `rel_exjcyp6v.json` |
| emprunt | διάμετρος → diametros | aucune déclaration « dérivé du lexème » entre « diametros » (L271803) et « διάμετρος » (L1103062). | `rel_f5qq65ks.json` |
| heritage | minare → mener | aucune déclaration « dérivé du lexème » entre « mener » (L9287) et « minare » (L278739). | `rel_f9a7fk3z.json` |
| forme_de_composition | ζῷον → zoo- | aucune déclaration « dérivé du lexème » entre « zoo » (sans lexème) et « ζῷον » (L729). | `rel_fb1a5pym.json` |
| emprunt | ὑδροφοβία → hydrophobia | aucune déclaration « dérivé du lexème » entre « hydrophobia » (L1053347) et « ὑδροφοβία » (sans lexème). | `rel_fbfw3wve.json` |
| forme_de_composition | νέος → néo- | aucune déclaration « dérivé du lexème » entre « néo » (sans lexème) et « νέος » (L9761). | `rel_fdw2sed3.json` |
| emprunt | monogramma → monogramme | aucune déclaration « dérivé du lexème » entre « monogramme » (sans lexème) et « monogramma » (L1058481). | `rel_ff5s19ay.json` |
| emprunt | technocracy → technocratie | aucune déclaration « dérivé du lexème » entre « technocratie » (sans lexème) et « technocracy » (L329113). | `rel_fgcbc2tv.json` |
| emprunt | καλλιγράφος → calligraphe | aucune déclaration « dérivé du lexème » entre « calligraphe » (L1576832) et « καλλιγράφος » (sans lexème). | `rel_fjjggvcs.json` |
| forme_de_composition | πρό → pro- | aucune déclaration « dérivé du lexème » entre « pro » (L27862) et « πρό » (L36089). | `rel_fkrygdv3.json` |
| forme_de_composition | ἰατρεία → -iatrie | aucune déclaration « dérivé du lexème » entre « iatrie » (sans lexème) et « ἰατρεία » (sans lexème). | `rel_fs8m0r7e.json` |
| forme_de_composition | φωνή → -phone | aucune déclaration « dérivé du lexème » entre « phone » (sans lexème) et « φωνή » (L1101957). | `rel_fsjx6dr7.json` |
| forme_de_composition | ἀρχαῖος → archéo- | aucune déclaration « dérivé du lexème » entre « archéo » (sans lexème) et « ἀρχαῖος » (L1110824). | `rel_g8jcdyc7.json` |
| emprunt | geographia → géographie | aucune déclaration « dérivé du lexème » entre « géographie » (L16438) et « geographia » (L274721). | `rel_gatd02r6.json` |
| forme_de_composition | ἱερός → hiéro- | aucune déclaration « dérivé du lexème » entre « hiéro » (sans lexème) et « ἱερός » (L1110978). | `rel_gbjma336.json` |
| emprunt | αὐτοκράτεια → autocratie | aucune déclaration « dérivé du lexème » entre « autocratie » (L1234531) et « αὐτοκράτεια » (sans lexème). | `rel_gey2qnrz.json` |
| forme_de_composition | πολύς → poly- | aucune déclaration « dérivé du lexème » entre « poly » (sans lexème) et « πολύς » (L1110848). | `rel_gfn9x1m2.json` |
| emprunt | χρονογράφος → chronographus | aucune déclaration « dérivé du lexème » entre « chronographus » (L263705) et « χρονογράφος » (L1102192). | `rel_h1nrm6dg.json` |
| forme_de_composition | ἵππος → hippo- | aucune déclaration « dérivé du lexème » entre « hippo » (sans lexème) et « ἵππος » (L5908). | `rel_h26xey2n.json` |
| forme_de_composition | πάλιν → palin- | aucune déclaration « dérivé du lexème » entre « palin » (sans lexème) et « πάλιν » (sans lexème). | `rel_h29zv6td.json` |
| heritage | ad- → a- | aucune déclaration « dérivé du lexème » entre « a » (sans lexème) et « ad » (L35193). | `rel_hjr51nnq.json` |
| emprunt | μονογαμία → monogamia | aucune déclaration « dérivé du lexème » entre « monogamia » (L1058044) et « μονογαμία » (L1130348). | `rel_hpzg802p.json` |
| forme_de_composition | centum → centi- | aucune déclaration « dérivé du lexème » entre « centi » (L20364) et « centum » (L263152). | `rel_j1xxr9jb.json` |
| forme_de_composition | mille → milli- | aucune déclaration « dérivé du lexème » entre « milli » (L20365) et « mille » (L278705). | `rel_jasdpacy.json` |
| forme_de_composition | στερεός → stéréo- | aucune déclaration « dérivé du lexème » entre « stéréo » (sans lexème) et « στερεός » (L1110886). | `rel_jd9p3zdc.json` |
| forme_de_composition | ὄνομα → -onymie | aucune déclaration « dérivé du lexème » entre « onymie » (sans lexème) et « ὄνομα » (L1102060). | `rel_jdgx4gwb.json` |
| emprunt | θεολογία → theologia | aucune déclaration « dérivé du lexème » entre « theologia » (L286348) et « θεολογία » (L1104355). | `rel_je9gqqme.json` |
| emprunt | αὐτόγραφος → autographe | aucune déclaration « dérivé du lexème » entre « autographe » (L12364) et « αὐτόγραφος » (sans lexème). | `rel_jjceh2g9.json` |
| emprunt | monogamia → monogamie | aucune déclaration « dérivé du lexème » entre « monogamie » (sans lexème) et « monogamia » (L1058044). | `rel_jmh6qapp.json` |
| derivation | τεχνολόγος → τεχνολογία | aucune déclaration « dérivé du lexème » entre « τεχνολογία » (L1103406) et « τεχνολόγος » (L1103727). | `rel_k3he10v5.json` |
| forme_de_composition | περί → péri- | aucune déclaration « dérivé du lexème » entre « péri » (sans lexème) et « περί » (L1549393). | `rel_kceksfcd.json` |
| emprunt | καλλιγραφία → calligraphie | aucune déclaration « dérivé du lexème » entre « calligraphie » (L24143) et « καλλιγραφία » (sans lexème). | `rel_kj9xbx9m.json` |
| emprunt | dialogus → dialogue | aucune déclaration « dérivé du lexème » entre « dialogue » (L20759) et « dialogus » (L271800). | `rel_kk26vgtd.json` |
| forme_de_composition | μόνος → mono- | aucune déclaration « dérivé du lexème » entre « mono » (sans lexème) et « μόνος » (L12250). | `rel_kk85z9fa.json` |
| forme_de_composition | διά → dia- | aucune déclaration « dérivé du lexème » entre « dia » (sans lexème) et « διά » (sans lexème). | `rel_m00kev05.json` |
| forme_de_composition | ὀδούς → odonto- | aucune déclaration « dérivé du lexème » entre « odonto » (sans lexème) et « ὀδούς » (L1101979). | `rel_mar953ac.json` |
| forme_de_composition | στενός → sténo- | aucune déclaration « dérivé du lexème » entre « sténo » (sans lexème) et « στενός » (L1111264). | `rel_medv2xam.json` |
| forme_de_composition | κρύσταλλος → cristallo- | aucune déclaration « dérivé du lexème » entre « cristallo » (sans lexème) et « κρύσταλλος » (L1102297). | `rel_mf33171d.json` |
| forme_de_composition | βιβλίον → biblio- | aucune déclaration « dérivé du lexème » entre « biblio » (sans lexème) et « βιβλίον » (L1102268). | `rel_mf76zr3f.json` |
| forme_de_composition | χρόνος → chrono- | aucune déclaration « dérivé du lexème » entre « chrono » (sans lexème) et « χρόνος » (L1102050). | `rel_mka9j5hc.json` |
| emprunt | δημοκρατία → démocratie | aucune déclaration « dérivé du lexème » entre « démocratie » (L17241) et « δημοκρατία » (L1101835). | `rel_mkx27epb.json` |
| forme_de_composition | σκοπεῖν → -scope | aucune déclaration « dérivé du lexème » entre « scope » (sans lexème) et « σκοπεῖν » (L1110522). | `rel_mt4ftntj.json` |
| forme_de_composition | ὀστέον → ostéo- | aucune déclaration « dérivé du lexème » entre « ostéo » (sans lexème) et « ὀστέον » (L1102085). | `rel_n6s44gsz.json` |
| forme_de_composition | λόγος → -logue | aucune déclaration « dérivé du lexème » entre « logue » (sans lexème) et « λόγος » (L1101900). | `rel_n700refq.json` |
| forme_de_composition | γλύφειν → -glyphe | aucune déclaration « dérivé du lexème » entre « glyphe » (sans lexème) et « γλύφειν » (L7961). | `rel_n7228rtg.json` |
| emprunt | anthropophagus → anthropophage | aucune déclaration « dérivé du lexème » entre « anthropophage » (sans lexème) et « anthropophagus » (L258079). | `rel_nn66xf12.json` |
| emprunt | barometer → baromètre | aucune déclaration « dérivé du lexème » entre « baromètre » (L27716) et « barometer » (L253904). | `rel_p7rh17z2.json` |
| forme_de_composition | μέγας → méga- | aucune déclaration « dérivé du lexème » entre « méga » (L20359) et « μέγας » (L1110814). | `rel_pf96wne4.json` |
| forme_de_composition | γῆ → géo- | aucune déclaration « dérivé du lexème » entre « géo » (L28895) et « γῆ » (L1101860). | `rel_psa6j2dj.json` |
| forme_de_composition | οὖς → oto- | aucune déclaration « dérivé du lexème » entre « oto » (sans lexème) et « οὖς » (L1102005). | `rel_q8vf8a4a.json` |
| forme_de_composition | μῦς → myo- | aucune déclaration « dérivé du lexème » entre « myo » (sans lexème) et « μῦς » (L1102618). | `rel_qcenbpjh.json` |
| emprunt | chronographus → chronographe | aucune déclaration « dérivé du lexème » entre « chronographe » (L1577162) et « chronographus » (L263705). | `rel_qnjydtfy.json` |
| forme_de_composition | μαντεία → -mancie | aucune déclaration « dérivé du lexème » entre « mancie » (sans lexème) et « μαντεία » (L1102597). | `rel_qt5pg1r2.json` |
| forme_de_composition | χίλιοι → kilo- | aucune déclaration « dérivé du lexème » entre « kilo » (L20360) et « χίλιοι » (sans lexème). | `rel_r6s5hhd0.json` |
| emprunt | θεοκρατία → théocratie | aucune déclaration « dérivé du lexème » entre « théocratie » (sans lexème) et « θεοκρατία » (L1104590). | `rel_r8n04vct.json` |
| forme_de_composition | ἄστρον → astro- | aucune déclaration « dérivé du lexème » entre « astro » (L31607) et « ἄστρον » (L1101870). | `rel_rhby96ec.json` |
| forme_de_composition | ὀλίγος → oligo- | aucune déclaration « dérivé du lexème » entre « oligo » (sans lexème) et « ὀλίγος » (L1110828). | `rel_rhmk4ach.json` |
| forme_de_composition | charta → carto- | aucune déclaration « dérivé du lexème » entre « carto » (sans lexème) et « charta » (L263508). | `rel_rmqsrdb8.json` |
| forme_de_composition | φίλος → philo- | aucune déclaration « dérivé du lexème » entre « philo » (sans lexème) et « φίλος » (L1110850). | `rel_rqv6dx2n.json` |
| forme_de_composition | ψυχή → psycho- | aucune déclaration « dérivé du lexème » entre « psycho » (sans lexème) et « ψυχή » (L1102048). | `rel_rtgr7y45.json` |
| emprunt | ξυλοφάγος → xylophagus | aucune déclaration « dérivé du lexème » entre « xylophagus » (sans lexème) et « ξυλοφάγος » (sans lexème). | `rel_rwbzarev.json` |
| forme_de_composition | σοφία → -sophie | aucune déclaration « dérivé du lexème » entre « sophie » (sans lexème) et « σοφία » (L1101933). | `rel_rxnj09j2.json` |
| emprunt | οἰκονομία → oeconomia | aucune déclaration « dérivé du lexème » entre « oeconomia » (L279937) et « οἰκονομία » (L1104011). | `rel_rymskqtn.json` |
| forme_de_composition | γάμος → -gamie | aucune déclaration « dérivé du lexème » entre « gamie » (sans lexème) et « γάμος » (L1102058). | `rel_s6g1jht9.json` |
| forme_de_composition | stratus → strato- | aucune déclaration « dérivé du lexème » entre « strato » (sans lexème) et « stratus » (L285384). | `rel_sbhcgwxd.json` |
| emprunt | ἀρχαιολογία → archéologie | aucune déclaration « dérivé du lexème » entre « archéologie » (L26871) et « ἀρχαιολογία » (sans lexème). | `rel_sdh0fy5r.json` |
| forme_de_composition | πέτρα → pétro- | aucune déclaration « dérivé du lexème » entre « pétro » (sans lexème) et « πέτρα » (L1101894). | `rel_sdsghd0x.json` |
| forme_de_composition | τύπος → -type | aucune déclaration « dérivé du lexème » entre « type » (sans lexème) et « τύπος » (L1102612). | `rel_sfmvr7w2.json` |
| forme_de_composition | δέκα → déca- | aucune déclaration « dérivé du lexème » entre « déca » (L20362) et « δέκα » (L1412206). | `rel_sgtkqw93.json` |
| forme_de_composition | οἶκος → éco- | aucune déclaration « dérivé du lexème » entre « éco » (sans lexème) et « οἶκος » (L309615). | `rel_snrnffjw.json` |
| emprunt | ismus → -isme | aucune déclaration « dérivé du lexème » entre « isme » (L29842) et « ismus » (sans lexème). | `rel_swbg52cx.json` |
| emprunt | -λογία → -logie | aucune déclaration « dérivé du lexème » entre « logie » (sans lexème) et « λογία » (sans lexème). | `rel_swxmxr5e.json` |
| forme_de_composition | χρῶμα → chromo- | aucune déclaration « dérivé du lexème » entre « chromo » (sans lexème) et « χρῶμα » (L1103315). | `rel_t35gyykm.json` |
| forme_de_composition | ὁμός → homo- | aucune déclaration « dérivé du lexème » entre « homo » (sans lexème) et « ὁμός » (L1111170). | `rel_t42556b6.json` |
| emprunt | telepathy → télépathie | aucune déclaration « dérivé du lexème » entre « télépathie » (sans lexème) et « telepathy » (L329138). | `rel_ta2203cv.json` |
| forme_de_composition | ὀφθαλμός → ophtalmo- | aucune déclaration « dérivé du lexème » entre « ophtalmo » (sans lexème) et « ὀφθαλμός » (L1101914). | `rel_taf7d5tm.json` |
| emprunt | πολύχρωμος → polychrome | aucune déclaration « dérivé du lexème » entre « polychrome » (sans lexème) et « πολύχρωμος » (sans lexème). | `rel_tazehpyh.json` |
| emprunt | xylophagus → xylophage | aucune déclaration « dérivé du lexème » entre « xylophage » (sans lexème) et « xylophagus » (sans lexème). | `rel_tf3wkgd4.json` |
| forme_de_composition | γονή → -gonie | aucune déclaration « dérivé du lexème » entre « gonie » (sans lexème) et « γονή » (L1104303). | `rel_tn5ehqgn.json` |
| forme_de_composition | τέτταρες → tétra- | aucune déclaration « dérivé du lexème » entre « tétra » (sans lexème) et « τέτταρες » (sans lexème). | `rel_tqt6jzp2.json` |
| forme_de_composition | πρῶτος → proto- | aucune déclaration « dérivé du lexème » entre « proto » (sans lexème) et « πρῶτος » (L204930). | `rel_tsvje9ek.json` |
| forme_de_composition | θερμός → thermo- | aucune déclaration « dérivé du lexème » entre « thermo » (sans lexème) et « θερμός » (L1111083). | `rel_ttee1zg9.json` |
| forme_de_composition | ὑπέρ → hyper- | aucune déclaration « dérivé du lexème » entre « hyper » (L11293) et « ὑπέρ » (L36097). | `rel_tz25yqy2.json` |
| emprunt | πολύμορφος → polymorphe | aucune déclaration « dérivé du lexème » entre « polymorphe » (sans lexème) et « πολύμορφος » (sans lexème). | `rel_v0ejpha6.json` |
| forme_de_composition | τῆλε → télé- | aucune déclaration « dérivé du lexème » entre « télé » (sans lexème) et « τῆλε » (sans lexème). | `rel_v5327a2j.json` |
| forme_de_composition | ναύτης → -naute | aucune déclaration « dérivé du lexème » entre « naute » (sans lexème) et « ναύτης » (L1103004). | `rel_v6y73ng7.json` |
| derivation | διαγράφειν → διάγραμμα | aucune déclaration « dérivé du lexème » entre « διάγραμμα » (L1103346) et « διαγράφειν » (sans lexème). | `rel_v7gqbbf2.json` |
| emprunt | moralis → moral | aucune déclaration « dérivé du lexème » entre « moral » (L16534) et « moralis » (L278907). | `rel_vdjr0nnw.json` |
| forme_de_composition | πάθος → patho- | aucune déclaration « dérivé du lexème » entre « patho » (sans lexème) et « πάθος » (L1102083). | `rel_veabfnnm.json` |
| forme_de_composition | δέρμα → dermato- | aucune déclaration « dérivé du lexème » entre « dermato » (sans lexème) et « δέρμα » (L1101877). | `rel_vf89vv29.json` |
| forme_de_composition | λίθος → litho- | aucune déclaration « dérivé du lexème » entre « litho » (sans lexème) et « λίθος » (L1102032). | `rel_vke6721r.json` |
| forme_de_composition | τύπος → typo- | aucune déclaration « dérivé du lexème » entre « typo » (sans lexème) et « τύπος » (L1102612). | `rel_vks8rkr9.json` |
| forme_de_composition | γένος → géno- | aucune déclaration « dérivé du lexème » entre « géno » (sans lexème) et « γένος » (L1102054). | `rel_vm8881je.json` |
| emprunt | philologus → philologue | aucune déclaration « dérivé du lexème » entre « philologue » (L1226420) et « philologus » (L281318). | `rel_vmpjh94f.json` |
| forme_de_composition | γεννάω → -gène | aucune déclaration « dérivé du lexème » entre « gène » (L1172654) et « γεννάω » (L1109141). | `rel_vp8ccz0p.json` |
| forme_de_composition | socius → socio- | aucune déclaration « dérivé du lexème » entre « socio » (L30078) et « socius » (L284958). | `rel_vsy2rmek.json` |
| emprunt | μονόχρωμος → monochrome | aucune déclaration « dérivé du lexème » entre « monochrome » (sans lexème) et « μονόχρωμος » (sans lexème). | `rel_vvj5bb36.json` |
| forme_de_composition | λέξις → lexico- | aucune déclaration « dérivé du lexème » entre « lexico » (sans lexème) et « λέξις » (L1103303). | `rel_w3zwz76p.json` |
| emprunt | astronomia → astronomie | aucune déclaration « dérivé du lexème » entre « astronomie » (L16435) et « astronomia » (L259511). | `rel_w9a7fgc3.json` |
| forme_de_composition | ἄνθρωπος → anthropo- | aucune déclaration « dérivé du lexème » entre « anthropo » (sans lexème) et « ἄνθρωπος » (L1101859). | `rel_wjzgjrp4.json` |
| emprunt | γεωγράφος → geographus | aucune déclaration « dérivé du lexème » entre « geographus » (L1052481) et « γεωγράφος » (sans lexème). | `rel_x2z8a2nz.json` |
| forme_de_composition | φῶς → photo- | aucune déclaration « dérivé du lexème » entre « photo » (sans lexème) et « φῶς » (L1101838). | `rel_x5tgj4mm.json` |
| forme_de_composition | γράμμα → -gramme | aucune déclaration « dérivé du lexème » entre « gramme » (sans lexème) et « γράμμα » (L1101872). | `rel_xaa9rpfz.json` |
| emprunt | theologia → théologie | aucune déclaration « dérivé du lexème » entre « théologie » (L23846) et « theologia » (L286348). | `rel_xb3y2gqd.json` |
| emprunt | diametros → diamètre | aucune déclaration « dérivé du lexème » entre « diamètre » (L20326) et « diametros » (L271803). | `rel_xfyvcwv6.json` |
| forme_de_composition | νόστος → nost- | aucune déclaration « dérivé du lexème » entre « nost » (sans lexème) et « νόστος » (L1104070). | `rel_xhe0yc4g.json` |
| emprunt | ἄφωνος → aphone | aucune déclaration « dérivé du lexème » entre « aphone » (sans lexème) et « ἄφωνος » (L1111558). | `rel_xmm9tm7q.json` |
| forme_de_composition | γαστήρ → gastro- | aucune déclaration « dérivé du lexème » entre « gastro » (sans lexème) et « γαστήρ » (L1101919). | `rel_xmwadjzh.json` |
| forme_de_composition | caedere → -cide | aucune déclaration « dérivé du lexème » entre « cide » (sans lexème) et « caedere » (L261461). | `rel_xq41xj9a.json` |
| forme_de_composition | βάρος → baro- | aucune déclaration « dérivé du lexème » entre « baro » (sans lexème) et « βάρος » (L1102067). | `rel_xw0yb8a3.json` |
| emprunt | palaeographia → paléographie | aucune déclaration « dérivé du lexème » entre « paléographie » (L1582908) et « palaeographia » (sans lexème). | `rel_y14pcvan.json` |
| forme_de_composition | ὀρθός → ortho- | aucune déclaration « dérivé du lexème » entre « ortho » (sans lexème) et « ὀρθός » (L1110980). | `rel_y3zrrx1m.json` |
| forme_de_composition | δρόμος → -drome | aucune déclaration « dérivé du lexème » entre « drome » (sans lexème) et « δρόμος » (L1101883). | `rel_y4fz7xh2.json` |
| emprunt | diagramma → diagramme | aucune déclaration « dérivé du lexème » entre « diagramme » (L28252) et « diagramma » (L1049133). | `rel_yb2gx0f7.json` |
| forme_de_composition | ψευδής → pseudo- | aucune déclaration « dérivé du lexème » entre « pseudo » (sans lexème) et « ψευδής » (L1110868). | `rel_yggwv0yt.json` |
| emprunt | φιλοσοφία → philosophia | aucune déclaration « dérivé du lexème » entre « philosophia » (L281320) et « φιλοσοφία » (L1102077). | `rel_yp0tevmg.json` |
| emprunt | ἀνθρωπολόγος → anthropologue | aucune déclaration « dérivé du lexème » entre « anthropologue » (L621776) et « ἀνθρωπολόγος » (sans lexème). | `rel_yrys1q10.json` |
| emprunt | microphone → microphone | aucune déclaration « dérivé du lexème » entre « microphone » (L1584176) et « microphone » (L323899). | `rel_ysv1tq7r.json` |
| emprunt | πολυγαμία → polygamia | aucune déclaration « dérivé du lexème » entre « polygamia » (L1062163) et « πολυγαμία » (L1103964). | `rel_z1901fyq.json` |
| emprunt | πρόλογος → prologus | aucune déclaration « dérivé du lexème » entre « prologus » (L282387) et « πρόλογος » (L1104503). | `rel_z31azkqy.json` |
| forme_de_composition | ἐπί → épi- | aucune déclaration « dérivé du lexème » entre « épi » (sans lexème) et « ἐπί » (L1452562). | `rel_z3hrq6hh.json` |
| forme_de_composition | ἰδέα → idéo- | aucune déclaration « dérivé du lexème » entre « idéo » (sans lexème) et « ἰδέα » (L1102022). | `rel_z7bqd1p1.json` |
| forme_de_composition | ἄρχειν → -arque | aucune déclaration « dérivé du lexème » entre « arque » (sans lexème) et « ἄρχειν » (L749469). | `rel_z7wvx736.json` |
| emprunt | γεωμέτρης → geometres | aucune déclaration « dérivé du lexème » entre « geometres » (L274723) et « γεωμέτρης » (sans lexème). | `rel_z83j4ey9.json` |
| forme_de_composition | καρδία → -cardie | aucune déclaration « dérivé du lexème » entre « cardie » (sans lexème) et « καρδία » (L1101893). | `rel_zdxjx8vd.json` |
| emprunt | ἀνθρωποφάγος → anthropophagus | aucune déclaration « dérivé du lexème » entre « anthropophagus » (L258079) et « ἀνθρωποφάγος » (L1112070). | `rel_zh3s0z4c.json` |
| forme_de_composition | γραφή → -graphie | aucune déclaration « dérivé du lexème » entre « graphie » (sans lexème) et « γραφή » (L1103188). | `rel_zjev6b0p.json` |

## 6. Compositions sans appui

Wikidata ne porte « combine les lexèmes » (P5238) que sur 7 mots du catalogue (mesures en *-mètre* et *décalitre*), revérifiés le 10 octobre 2026 sur les 308 lexèmes français. Toutes les autres analyses restent sans appui.

| Fasc. | Mot | Analyse | Type | Fichier |
|---|---|---|---|---|
| 1 | anthropobiologie | anthropo- + biologie | synchronique | `cmp_kfnwr7tg.json` |
| 1 | anthropographie | anthropo- + -graphie | synchronique | `cmp_w5m4hvfr.json` |
| 1 | anthropologie | anthropo- + -logie | synchronique | `cmp_sgtce5n1.json` |
| 1 | anthropologiste | anthropologie + -iste | synchronique | `cmp_mw5zngjf.json` |
| 1 | anthropologue | anthropo- + -logue | synchronique | `cmp_xg5f2tz5.json` |
| 1 | anthropomorphe | anthropo- + -morphe | synchronique | `cmp_zqpgcfpz.json` |
| 1 | anthropophage | anthropo- + -phage | synchronique | `cmp_11tw5w41.json` |
| 1 | anthropophobie | anthropo- + -phobie | synchronique | `cmp_ha47zfxz.json` |
| 1 | anthroposophie | anthropo- + -sophie | synchronique | `cmp_jhvdhv11.json` |
| 1 | biographe | bio- + -graphe | synchronique | `cmp_1vahd7zk.json` |
| 1 | biographie | bio- + -graphie | synchronique | `cmp_j5324f3h.json` |
| 1 | biographiste | biographie + -iste | synchronique | `cmp_wgah75j5.json` |
| 1 | biogène | bio- + -gène | synchronique | `cmp_tk8wmy27.json` |
| 1 | biologie | bio- + -logie | synchronique | `cmp_wabzjpv7.json` |
| 1 | biologiste | biologie + -iste | synchronique | `cmp_25pks50z.json` |
| 1 | biologue | bio- + -logue | synchronique | `cmp_38akffxf.json` |
| 1 | biomorphe | bio- + -morphe | synchronique | `cmp_h38g8hkw.json` |
| 1 | bionomie | bio- + -nomie | synchronique | `cmp_e36ptqrg.json` |
| 1 | biosphère | bio- + -sphère | synchronique | `cmp_pb115t10.json` |
| 1 | biothérapie | bio- + -thérapie | synchronique | `cmp_zxzv7vva.json` |
| 1 | neurobiologie | neuro- + biologie | synchronique | `cmp_x3ptehjt.json` |
| 1 | neurologie | neuro- + -logie | synchronique | `cmp_3v6gqhsh.json` |
| 1 | neurologiste | neurologie + -iste | synchronique | `cmp_k6fm6s7t.json` |
| 1 | neurologue | neuro- + -logue | synchronique | `cmp_b7canfg8.json` |
| 1 | neuropathie | neuro- + -pathie | synchronique | `cmp_s37nq4dj.json` |
| 1 | philologie | philo- + -logie | synchronique | `cmp_rft2v879.json` |
| 1 | philologue | philo- + -logue | synchronique | `cmp_hw3j7stx.json` |
| 1 | philosophie | philo- + -sophie | synchronique | `cmp_r61nrcds.json` |
| 1 | psychobiologie | psycho- + biologie | synchronique | `cmp_kakkz0zb.json` |
| 1 | psychographie | psycho- + -graphie | synchronique | `cmp_6g67yy4w.json` |
| 1 | psychogène | psycho- + -gène | synchronique | `cmp_0h5pte0p.json` |
| 1 | psychologie | psycho- + -logie | synchronique | `cmp_efab0878.json` |
| 1 | psychologiste | psychologie + -iste | synchronique | `cmp_a0ta1bbz.json` |
| 1 | psychologue | psycho- + -logue | synchronique | `cmp_rstc8vef.json` |
| 1 | psychopathie | psycho- + -pathie | synchronique | `cmp_t0a79ae5.json` |
| 1 | psychothérapie | psycho- + -thérapie | synchronique | `cmp_jkw3ynb0.json` |
| 1 | sociobiologie | socio- + biologie | synchronique | `cmp_0eszvc0w.json` |
| 1 | sociogramme | socio- + -gramme | synchronique | `cmp_80ch4xpj.json` |
| 1 | sociologie | socio- + -logie | synchronique | `cmp_exazgdws.json` |
| 1 | sociologiste | sociologie + -iste | synchronique | `cmp_37jdcbgw.json` |
| 1 | sociologue | socio- + -logue | synchronique | `cmp_rvmkk8cp.json` |
| 1 | sociopathie | socio- + -pathie | synchronique | `cmp_meckj6qh.json` |
| 1 | zoologie | zoo- + -logie | synchronique | `cmp_zkn8m35f.json` |
| 1 | zoologiste | zoologie + -iste | synchronique | `cmp_75x23ngm.json` |
| 1 | zoologue | zoo- + -logue | synchronique | `cmp_t8z1cdt1.json` |
| 1 | zoomorphe | zoo- + -morphe | synchronique | `cmp_90d6y3z5.json` |
| 1 | zoonomie | zoo- + -nomie | synchronique | `cmp_qc3ens4s.json` |
| 1 | zoophile | zoo- + -phile | synchronique | `cmp_1680apcw.json` |
| 1 | zoophobie | zoo- + -phobie | synchronique | `cmp_w59104ky.json` |
| 1 | zoothérapie | zoo- + -thérapie | synchronique | `cmp_ye5tqgbr.json` |
| 2 | agrobiologie | agro- + biologie | synchronique | `cmp_xkzkbscw.json` |
| 2 | agrologie | agro- + -logie | synchronique | `cmp_sm9mjket.json` |
| 2 | agronomie | agro- + -nomie | synchronique | `cmp_qsh9kgfv.json` |
| 2 | archéographie | archéo- + -graphie | synchronique | `cmp_bj40js6d.json` |
| 2 | archéologie | archéo- + -logie | synchronique | `cmp_rz39g02r.json` |
| 2 | archéologue | archéo- + -logue | synchronique | `cmp_p37g1zt0.json` |
| 2 | astrobiologie | astro- + biologie | synchronique | `cmp_cfsb2kt1.json` |
| 2 | astrologie | astro- + -logie | synchronique | `cmp_dtxas9pj.json` |
| 2 | astrologue | astro- + -logue | synchronique | `cmp_zcn0mnqz.json` |
| 2 | astronomie | astro- + -nomie | synchronique | `cmp_qwt0k5x2.json` |
| 2 | astrophobie | astro- + -phobie | synchronique | `cmp_etp236a2.json` |
| 2 | barogramme | baro- + -gramme | synchronique | `cmp_ptgntccp.json` |
| 2 | baromètre | baro- + -mètre | synchronique | `cmp_13ye97zx.json` |
| 2 | géobiologie | géo- + biologie | synchronique | `cmp_prx3qpm5.json` |
| 2 | géographe | géo- + -graphe | synchronique | `cmp_d6sm4n5g.json` |
| 2 | géographie | géo- + -graphie | synchronique | `cmp_ywc1pzmr.json` |
| 2 | géologie | géo- + -logie | synchronique | `cmp_kq4bhs7h.json` |
| 2 | géologiste | géologie + -iste | synchronique | `cmp_n3ak4se8.json` |
| 2 | géologue | géo- + -logue | synchronique | `cmp_zzk9dw6r.json` |
| 2 | géomètre | géo- + -mètre | synchronique | `cmp_s9fk7cmq.json` |
| 2 | géophage | géo- + -phage | synchronique | `cmp_86cx9w2r.json` |
| 2 | géophone | géo- + -phone | synchronique | `cmp_5y1bkrzg.json` |
| 2 | géosphère | géo- + -sphère | synchronique | `cmp_eyp7gdzy.json` |
| 2 | hydrobiologie | hydro- + biologie | synchronique | `cmp_6pxpa4q7.json` |
| 2 | hydrogramme | hydro- + -gramme | synchronique | `cmp_z2555ypq.json` |
| 2 | hydrographe | hydro- + -graphe | synchronique | `cmp_pfw8xsz3.json` |
| 2 | hydrographie | hydro- + -graphie | synchronique | `cmp_cteb54xa.json` |
| 2 | hydrogène | hydro- + -gène | synchronique | `cmp_4ph08pxd.json` |
| 2 | hydrologie | hydro- + -logie | synchronique | `cmp_6mr8k038.json` |
| 2 | hydrologiste | hydrologie + -iste | synchronique | `cmp_7y1eekb2.json` |
| 2 | hydrologue | hydro- + -logue | synchronique | `cmp_bqh9eg2p.json` |
| 2 | hydromorphe | hydro- + -morphe | synchronique | `cmp_xwcsb867.json` |
| 2 | hydrophile | hydro- + -phile | synchronique | `cmp_kjxv8g4n.json` |
| 2 | hydrophobie | hydro- + -phobie | synchronique | `cmp_vaet87vk.json` |
| 2 | hydrophone | hydro- + -phone | synchronique | `cmp_jwnwwd7p.json` |
| 2 | hydrosphère | hydro- + -sphère | synchronique | `cmp_a0kgcq9n.json` |
| 2 | hydrothérapie | hydro- + -thérapie | synchronique | `cmp_r3qfjbyh.json` |
| 2 | thermobiologie | thermo- + biologie | synchronique | `cmp_eserfz61.json` |
| 2 | thermogramme | thermo- + -gramme | synchronique | `cmp_6anj4sbn.json` |
| 2 | thermographe | thermo- + -graphe | synchronique | `cmp_apqqe2dm.json` |
| 2 | thermographie | thermo- + -graphie | synchronique | `cmp_v84049s2.json` |
| 2 | thermogène | thermo- + -gène | synchronique | `cmp_edmnscaj.json` |
| 2 | thermomètre | thermo- + -mètre | synchronique | `cmp_yparghy3.json` |
| 2 | thermophile | thermo- + -phile | synchronique | `cmp_4n334mm7.json` |
| 2 | thermophobie | thermo- + -phobie | synchronique | `cmp_dksdh7ey.json` |
| 2 | thermoscope | thermo- + -scope | synchronique | `cmp_8ghnjw68.json` |
| 2 | thermosphère | thermo- + -sphère | synchronique | `cmp_fk9nmna3.json` |
| 2 | thermothérapie | thermo- + -thérapie | synchronique | `cmp_g71n5674.json` |
| 2 | xylophage | xylo- + -phage | synchronique | `cmp_t618zpnf.json` |
| 2 | xylophone | xylo- + -phone | synchronique | `cmp_g3txdfwg.json` |
| 3 | bibliographe | biblio- + -graphe | synchronique | `cmp_7jrpxzan.json` |
| 3 | bibliographie | biblio- + -graphie | synchronique | `cmp_r3q220fv.json` |
| 3 | bibliologie | biblio- + -logie | synchronique | `cmp_3cy1n5jm.json` |
| 3 | bibliophile | biblio- + -phile | synchronique | `cmp_8jfhh3xx.json` |
| 3 | calligraphe | calli- + -graphe | synchronique | `cmp_jx4exfbb.json` |
| 3 | calligraphie | calli- + -graphie | synchronique | `cmp_3yhh8gsf.json` |
| 3 | calligraphiste | calligraphie + -iste | synchronique | `cmp_z8kx937g.json` |
| 3 | chronobiologie | chrono- + biologie | synchronique | `cmp_ry6j2kps.json` |
| 3 | chronogramme | chrono- + -gramme | synchronique | `cmp_j9zr9fyk.json` |
| 3 | chronographe | chrono- + -graphe | synchronique | `cmp_2cyexspf.json` |
| 3 | chronologie | chrono- + -logie | synchronique | `cmp_j06p6gck.json` |
| 3 | chronologiste | chronologie + -iste | synchronique | `cmp_zyq4r8an.json` |
| 3 | chronomètre | chrono- + -mètre | synchronique | `cmp_t6nqsmhq.json` |
| 3 | chronophobie | chrono- + -phobie | synchronique | `cmp_w89j2gm5.json` |
| 3 | chronoscope | chrono- + -scope | synchronique | `cmp_66c1h6hg.json` |
| 3 | diagramme | dia- + -gramme | synchronique | `cmp_3msxyh0f.json` |
| 3 | dialogue | dia- + -logue | synchronique | `cmp_69efafjd.json` |
| 3 | diamètre | dia- + -mètre | synchronique | `cmp_m1srpdmb.json` |
| 3 | microbiologie | micro- + biologie | synchronique | `cmp_52gvs1ew.json` |
| 3 | microbiologiste | microbiologie + -iste | synchronique | `cmp_zmy74ysq.json` |
| 3 | micrographe | micro- + -graphe | synchronique | `cmp_5cg219tr.json` |
| 3 | micrographie | micro- + -graphie | synchronique | `cmp_t2m7mgga.json` |
| 3 | micromètre | micro- + -mètre | synchronique | `cmp_c33rbenm.json` |
| 3 | microphone | micro- + -phone | synchronique | `cmp_33frha8p.json` |
| 3 | microscope | micro- + -scope | synchronique | `cmp_knpwmp2z.json` |
| 3 | paléographe | paléo- + -graphe | synchronique | `cmp_6xbv8qqp.json` |
| 3 | paléographie | paléo- + -graphie | synchronique | `cmp_pxpg2kyt.json` |
| 3 | photobiologie | photo- + biologie | synchronique | `cmp_tk22bbec.json` |
| 3 | photogramme | photo- + -gramme | synchronique | `cmp_g7ytpwbw.json` |
| 3 | photographe | photo- + -graphe | synchronique | `cmp_n0nbhcr5.json` |
| 3 | photographie | photo- + -graphie | synchronique | `cmp_apc5da0v.json` |
| 3 | photographiste | photographie + -iste | synchronique | `cmp_yg1046pa.json` |
| 3 | photogène | photo- + -gène | synchronique | `cmp_t66cvb4r.json` |
| 3 | photomètre | photo- + -mètre | synchronique | `cmp_600f452h.json` |
| 3 | photophile | photo- + -phile | synchronique | `cmp_fjc2twnh.json` |
| 3 | photophobie | photo- + -phobie | synchronique | `cmp_hq7d87hm.json` |
| 3 | photophone | photo- + -phone | synchronique | `cmp_dtk8njxc.json` |
| 3 | photothérapie | photo- + -thérapie | synchronique | `cmp_6ssrrcx2.json` |
| 3 | périmètre | péri- + -mètre | synchronique | `cmp_jap0hj46.json` |
| 3 | périscope | péri- + -scope | synchronique | `cmp_kecacr7d.json` |
| 3 | télescope | télé- + -scope | synchronique | `cmp_ydkddcws.json` |
| 3 | télégramme | télé- + -gramme | synchronique | `cmp_3671sdc1.json` |
| 3 | télégraphe | télé- + -graphe | synchronique | `cmp_akjqzt2q.json` |
| 3 | télégraphie | télé- + -graphie | synchronique | `cmp_kh91njfe.json` |
| 3 | télégraphiste | télégraphie + -iste | synchronique | `cmp_v3275bf0.json` |
| 3 | télémètre | télé- + -mètre | synchronique | `cmp_g7b0zpm0.json` |
| 3 | télépathie | télé- + -pathie | synchronique | `cmp_68sh1zhb.json` |
| 3 | téléphone | télé- + -phone | synchronique | `cmp_bmj3apcb.json` |
| 3 | étymologie | étymo- + -logie | synchronique | `cmp_t0jmxhyr.json` |
| 3 | étymologiste | étymologie + -iste | synchronique | `cmp_2em4pebm.json` |
| 4 | amener | a- + mener | synchronique | `cmp_bpxyhmrq.json` |
| 4 | amoral | a- + moral | synchronique | `cmp_hky4g031.json` |
| 4 | aphone | a- + -phone | synchronique | `cmp_3e5d6w39.json` |
| 4 | autochrome | auto- + -chrome | synchronique | `cmp_4vhkkpay.json` |
| 4 | autocratie | auto- + -cratie | synchronique | `cmp_fdbh8vds.json` |
| 4 | autographe | auto- + -graphe | synchronique | `cmp_jnkv92y3.json` |
| 4 | autonomie | auto- + -nomie | synchronique | `cmp_s9s5yttk.json` |
| 4 | cardiogramme | cardio- + -gramme | synchronique | `cmp_ybs030rv.json` |
| 4 | cardiographe | cardio- + -graphe | synchronique | `cmp_pncnem9d.json` |
| 4 | cardiographie | cardio- + -graphie | synchronique | `cmp_t2eyz1bs.json` |
| 4 | cardiologie | cardio- + -logie | synchronique | `cmp_p0tt965f.json` |
| 4 | cardiologue | cardio- + -logue | synchronique | `cmp_d1btpa4g.json` |
| 4 | cardiopathie | cardio- + -pathie | synchronique | `cmp_pw7967ct.json` |
| 4 | démocratie | démo- + -cratie | synchronique | `cmp_gcsdngga.json` |
| 4 | démographe | démo- + -graphe | synchronique | `cmp_ct64p3vz.json` |
| 4 | démographie | démo- + -graphie | synchronique | `cmp_ehs23rkh.json` |
| 4 | emmener | en- + mener | synchronique | `cmp_1r0eg16e.json` |
| 4 | malmener | mal + mener | synchronique | `cmp_410r20y9.json` |
| 4 | monochrome | mono- + -chrome | synchronique | `cmp_ak4vntdj.json` |
| 4 | monocratie | mono- + -cratie | synchronique | `cmp_akhee093.json` |
| 4 | monogamie | mono- + -gamie | synchronique | `cmp_chq1fv6h.json` |
| 4 | monogramme | mono- + -gramme | synchronique | `cmp_063q58db.json` |
| 4 | monographie | mono- + -graphie | synchronique | `cmp_qz31dc3c.json` |
| 4 | monologue | mono- + -logue | synchronique | `cmp_asf00eaa.json` |
| 4 | monomorphe | mono- + -morphe | synchronique | `cmp_8q6p7gtd.json` |
| 4 | monophobie | mono- + -phobie | synchronique | `cmp_07qdmhx7.json` |
| 4 | moraliste | moral + -iste | synchronique | `cmp_jcfqktx1.json` |
| 4 | polychrome | poly- + -chrome | synchronique | `cmp_76w2tnrg.json` |
| 4 | polygamie | poly- + -gamie | synchronique | `cmp_htn2vd6y.json` |
| 4 | polygraphe | poly- + -graphe | synchronique | `cmp_rkpq6qw7.json` |
| 4 | polygraphie | poly- + -graphie | synchronique | `cmp_jyn32a9d.json` |
| 4 | polymorphe | poly- + -morphe | synchronique | `cmp_b82fzjfs.json` |
| 4 | polyphone | poly- + -phone | synchronique | `cmp_nfp3ndf1.json` |
| 4 | prologue | pro- + -logue | synchronique | `cmp_cpnme4n5.json` |
| 4 | promener | pro- + mener | synchronique | `cmp_f37pnr16.json` |
| 4 | ramener | re- + amener | synchronique | `cmp_pp3mq10r.json` |
| 4 | technocratie | techno- + -cratie | synchronique | `cmp_mfy88d29.json` |
| 4 | technologie | techno- + -logie | synchronique | `cmp_bzy9yrhc.json` |
| 4 | technologue | techno- + -logue | synchronique | `cmp_st2kfaqv.json` |
| 4 | technophile | techno- + -phile | synchronique | `cmp_2d2rdm1n.json` |
| 4 | technophobie | techno- + -phobie | synchronique | `cmp_88wpqnx7.json` |
| 4 | théocratie | théo- + -cratie | synchronique | `cmp_t3c3yvtq.json` |
| 4 | théologie | théo- + -logie | synchronique | `cmp_6gjhcbd2.json` |
| 4 | théologue | théo- + -logue | synchronique | `cmp_45e2rpc4.json` |
| 4 | théosophie | théo- + -sophie | synchronique | `cmp_x7n6a8bd.json` |
| 4 | écologie | éco- + -logie | synchronique | `cmp_ky3266qx.json` |
| 4 | écologiste | écologie + -iste | synchronique | `cmp_eafydqk8.json` |
| 4 | écologue | éco- + -logue | synchronique | `cmp_vyfvxbkt.json` |
| 4 | économie | éco- + -nomie | synchronique | `cmp_hzjjj97y.json` |
| 4 | économiste | économie + -iste | synchronique | `cmp_ffaynhck.json` |
| 5 | achrome | a- + -chrome | synchronique | `cmp_v2x3degm.json` |
| 5 | agamie | a- + -gamie | synchronique | `cmp_4bnzpp48.json` |
| 5 | agraphie | a- + -graphie | synchronique | `cmp_yt9nn8f6.json` |
| 5 | alogie | a- + -logie | synchronique | `cmp_8dzgww8x.json` |
| 5 | amorphe | a- + -morphe | synchronique | `cmp_7g52393c.json` |
| 5 | anonyme | a- + -onyme | synchronique | `cmp_yx97tz0s.json` |
| 5 | anthropologique | anthropologie + -ique | synchronique | `cmp_kp8sbk64.json` |
| 5 | anthroponyme | anthropo- + -onyme | synchronique | `cmp_rdfj3nb3.json` |
| 5 | anthroponymie | anthropo- + -onymie | synchronique | `cmp_2a6dfprs.json` |
| 5 | antigène | anti- + -gène | synchronique | `cmp_1w4v716d.json` |
| 5 | antilogie | anti- + -logie | synchronique | `cmp_bs07vqxy.json` |
| 5 | antipathie | anti- + -pathie | synchronique | `cmp_ya6kw176.json` |
| 5 | antonyme | anti- + -onyme | synchronique | `cmp_1a1jf49b.json` |
| 5 | antonymie | anti- + -onymie | synchronique | `cmp_tb5884qv.json` |
| 5 | autonyme | auto- + -onyme | synchronique | `cmp_e3h3znv4.json` |
| 5 | autonymie | auto- + -onymie | synchronique | `cmp_f1h4jbbc.json` |
| 5 | barographe | baro- + -graphe | synchronique | `cmp_vdm0nnkr.json` |
| 5 | baroscope | baro- + -scope | synchronique | `cmp_xt37z7y9.json` |
| 5 | biographique | biographie + -ique | synchronique | `cmp_yachjfcx.json` |
| 5 | biologique | biologie + -ique | synchronique | `cmp_4tdj7jxg.json` |
| 5 | biomètre | bio- + -mètre | synchronique | `cmp_za7p4mgx.json` |
| 5 | biophile | bio- + -phile | synchronique | `cmp_zj7jz5ec.json` |
| 5 | bioscope | bio- + -scope | synchronique | `cmp_9qj7eeb3.json` |
| 5 | calligraphique | calligraphie + -ique | synchronique | `cmp_nfgf8xh7.json` |
| 5 | cardiomètre | cardio- + -mètre | synchronique | `cmp_30d1pvgr.json` |
| 5 | cardiophone | cardio- + -phone | synchronique | `cmp_gefgyyy7.json` |
| 5 | cardioscope | cardio- + -scope | synchronique | `cmp_67jtbq3f.json` |
| 5 | chronique | chrono- + -ique | synchronique | `cmp_1epq7c51.json` |
| 5 | chronographie | chrono- + -graphie | synchronique | `cmp_395f4mhk.json` |
| 5 | chronologique | chronologie + -ique | synchronique | `cmp_qt7desm8.json` |
| 5 | diaphone | dia- + -phone | synchronique | `cmp_676wrvv9.json` |
| 5 | diascope | dia- + -scope | synchronique | `cmp_hvnmt15w.json` |
| 5 | démonyme | démo- + -onyme | synchronique | `cmp_n69qd0hj.json` |
| 5 | géologique | géologie + -ique | synchronique | `cmp_t7xnf6rr.json` |
| 5 | géophile | géo- + -phile | synchronique | `cmp_gjfc1jj1.json` |
| 5 | géoscope | géo- + -scope | synchronique | `cmp_cxebwaqj.json` |
| 5 | homochrome | homo- + -chrome | synchronique | `cmp_g2jgcsxg.json` |
| 5 | homogamie | homo- + -gamie | synchronique | `cmp_n1tv4n1h.json` |
| 5 | homographe | homo- + -graphe | synchronique | `cmp_wbp53g70.json` |
| 5 | homogène | homo- + -gène | synchronique | `cmp_2y49bve2.json` |
| 5 | homologie | homo- + -logie | synchronique | `cmp_wbk98a05.json` |
| 5 | homologue | homo- + -logue | synchronique | `cmp_19mkjv5e.json` |
| 5 | homomorphe | homo- + -morphe | synchronique | `cmp_t687p7dy.json` |
| 5 | homonyme | homo- + -onyme | synchronique | `cmp_s7cw9v2r.json` |
| 5 | homonymie | homo- + -onymie | synchronique | `cmp_wqz9xn9r.json` |
| 5 | homophile | homo- + -phile | synchronique | `cmp_029m13kd.json` |
| 5 | homophobie | homo- + -phobie | synchronique | `cmp_2qd2dapk.json` |
| 5 | homophone | homo- + -phone | synchronique | `cmp_1p1zrwgt.json` |
| 5 | homosphère | homo- + -sphère | synchronique | `cmp_xm3287ja.json` |
| 5 | hydrique | hydro- + -ique | synchronique | `cmp_jpb3dpy6.json` |
| 5 | hydrologique | hydrologie + -ique | synchronique | `cmp_xq9jcpx6.json` |
| 5 | hydromètre | hydro- + -mètre | synchronique | `cmp_da73k07f.json` |
| 5 | hydronyme | hydro- + -onyme | synchronique | `cmp_v9yfxpxa.json` |
| 5 | hydronymie | hydro- + -onymie | synchronique | `cmp_ez020t5w.json` |
| 5 | hydroscope | hydro- + -scope | synchronique | `cmp_qqqegfav.json` |
| 5 | hétérochrome | hétéro- + -chrome | synchronique | `cmp_yt71q2xm.json` |
| 5 | hétérogamie | hétéro- + -gamie | synchronique | `cmp_0qqw32x4.json` |
| 5 | hétérographie | hétéro- + -graphie | synchronique | `cmp_c55w8n31.json` |
| 5 | hétérogène | hétéro- + -gène | synchronique | `cmp_ywnceya3.json` |
| 5 | hétérologue | hétéro- + -logue | synchronique | `cmp_8wjmqars.json` |
| 5 | hétéromorphe | hétéro- + -morphe | synchronique | `cmp_1fkbnd22.json` |
| 5 | hétéronomie | hétéro- + -nomie | synchronique | `cmp_wpday04q.json` |
| 5 | hétéronyme | hétéro- + -onyme | synchronique | `cmp_kpcy92kq.json` |
| 5 | microbiologique | microbiologie + -ique | synchronique | `cmp_1g9ywgg2.json` |
| 5 | monomètre | mono- + -mètre | synchronique | `cmp_3kvgbh45.json` |
| 5 | monophone | mono- + -phone | synchronique | `cmp_zs5qx22z.json` |
| 5 | neurologique | neurologie + -ique | synchronique | `cmp_9aw6dbb4.json` |
| 5 | photique | photo- + -ique | synchronique | `cmp_hmd0x2td.json` |
| 5 | photochrome | photo- + -chrome | synchronique | `cmp_64gme072.json` |
| 5 | photographique | photographie + -ique | synchronique | `cmp_2kzsnpdb.json` |
| 5 | polyscope | poly- + -scope | synchronique | `cmp_tg66g3jc.json` |
| 5 | pseudologie | pseudo- + -logie | synchronique | `cmp_qdby36wp.json` |
| 5 | pseudomorphe | pseudo- + -morphe | synchronique | `cmp_8he9ss4h.json` |
| 5 | pseudonyme | pseudo- + -onyme | synchronique | `cmp_5pf95d87.json` |
| 5 | pseudonymie | pseudo- + -onymie | synchronique | `cmp_m9vvnbd3.json` |
| 5 | psychique | psycho- + -ique | synchronique | `cmp_b38vttfk.json` |
| 5 | psychologique | psychologie + -ique | synchronique | `cmp_b6nnhbdn.json` |
| 5 | remener | re- + mener | synchronique | `cmp_tdpdt4gh.json` |
| 5 | sociologique | sociologie + -ique | synchronique | `cmp_v3t4vybd.json` |
| 5 | sympathie | syn- + -pathie | synchronique | `cmp_n33xwwhg.json` |
| 5 | syngamie | syn- + -gamie | synchronique | `cmp_rh4npfcr.json` |
| 5 | synonyme | syn- + -onyme | synchronique | `cmp_v4wa71w9.json` |
| 5 | synonymie | syn- + -onymie | synchronique | `cmp_gveth78m.json` |
| 5 | technique | techno- + -ique | synchronique | `cmp_g68newms.json` |
| 5 | thermique | thermo- + -ique | synchronique | `cmp_6yp91by7.json` |
| 5 | thermologie | thermo- + -logie | synchronique | `cmp_0fys723a.json` |
| 5 | thermophone | thermo- + -phone | synchronique | `cmp_70erzbce.json` |
| 5 | théonyme | théo- + -onyme | synchronique | `cmp_3rx8g09k.json` |
| 5 | topique | topo- + -ique | synchronique | `cmp_gqqr8e5b.json` |
| 5 | topographe | topo- + -graphe | synchronique | `cmp_de7a95j5.json` |
| 5 | topographie | topo- + -graphie | synchronique | `cmp_k649qw0w.json` |
| 5 | topologie | topo- + -logie | synchronique | `cmp_2x4mty33.json` |
| 5 | toponyme | topo- + -onyme | synchronique | `cmp_dttteacf.json` |
| 5 | toponymie | topo- + -onymie | synchronique | `cmp_6t5hvnwr.json` |
| 5 | télégraphique | télégraphie + -ique | synchronique | `cmp_mhej511t.json` |
| 5 | xylographe | xylo- + -graphe | synchronique | `cmp_rvkcn70y.json` |
| 5 | xylographie | xylo- + -graphie | synchronique | `cmp_h6yje93n.json` |
| 5 | xylophile | xylo- + -phile | synchronique | `cmp_kt8a1g2g.json` |
| 5 | zoographe | zoo- + -graphe | synchronique | `cmp_hx2anqyy.json` |
| 5 | zoographie | zoo- + -graphie | synchronique | `cmp_nsq3nf2z.json` |
| 5 | zoologique | zoologie + -ique | synchronique | `cmp_hqhvpfch.json` |
| 5 | zoomètre | zoo- + -mètre | synchronique | `cmp_6t8ksdk7.json` |
| 5 | zooscope | zoo- + -scope | synchronique | `cmp_wsn0dv06.json` |
| 5 | écologique | écologie + -ique | synchronique | `cmp_ecf6gp9g.json` |
| 5 | économique | économie + -ique | synchronique | `cmp_g71f96q4.json` |
| 5 | étymologique | étymologie + -ique | synchronique | `cmp_d1vjjxk5.json` |
| 6 | anthropogonie | anthropo- + -gonie | synchronique | `cmp_hme3ahpq.json` |
| 6 | antiphonie | anti- + -phonie | synchronique | `cmp_jxk4xhsd.json` |
| 6 | aphonie | a- + -phonie | synchronique | `cmp_m26tmcgn.json` |
| 6 | autocrate | auto- + -crate | synchronique | `cmp_se3vjatf.json` |
| 6 | autophonie | auto- + -phonie | synchronique | `cmp_w5mk05ks.json` |
| 6 | cosmique | cosmo- + -ique | synchronique | `cmp_5thw44h4.json` |
| 6 | cosmobiologie | cosmo- + biologie | synchronique | `cmp_7bxkc3wy.json` |
| 6 | cosmogonie | cosmo- + -gonie | synchronique | `cmp_s3rnr8gd.json` |
| 6 | cosmographe | cosmo- + -graphe | synchronique | `cmp_0vt0yz9m.json` |
| 6 | cosmographie | cosmo- + -graphie | synchronique | `cmp_ksfxfzwh.json` |
| 6 | cosmologie | cosmo- + -logie | synchronique | `cmp_y1vhnxs8.json` |
| 6 | cosmologique | cosmologie + -ique | synchronique | `cmp_qkgavywj.json` |
| 6 | cosmologiste | cosmologie + -iste | synchronique | `cmp_hn152j6s.json` |
| 6 | cosmologue | cosmo- + -logue | synchronique | `cmp_q0q4ttcr.json` |
| 6 | cosmosophie | cosmo- + -sophie | synchronique | `cmp_89e51bhw.json` |
| 6 | cryptogamie | crypto- + -gamie | synchronique | `cmp_md47te15.json` |
| 6 | cryptogramme | crypto- + -gramme | synchronique | `cmp_nvcv85mw.json` |
| 6 | cryptographe | crypto- + -graphe | synchronique | `cmp_p0svrbet.json` |
| 6 | cryptographie | crypto- + -graphie | synchronique | `cmp_fh741nsw.json` |
| 6 | cryptologie | crypto- + -logie | synchronique | `cmp_5y9kve5x.json` |
| 6 | cryptologue | crypto- + -logue | synchronique | `cmp_twyeegh9.json` |
| 6 | cryptonyme | crypto- + -onyme | synchronique | `cmp_x96rzbpm.json` |
| 6 | diaphonie | dia- + -phonie | synchronique | `cmp_h31385mw.json` |
| 6 | démocrate | démo- + -crate | synchronique | `cmp_dr6czvgz.json` |
| 6 | ethnique | ethno- + -ique | synchronique | `cmp_ynb080a9.json` |
| 6 | ethnobiologie | ethno- + biologie | synchronique | `cmp_483y4sth.json` |
| 6 | ethnographe | ethno- + -graphe | synchronique | `cmp_00bghajr.json` |
| 6 | ethnographie | ethno- + -graphie | synchronique | `cmp_a0seapyp.json` |
| 6 | ethnologie | ethno- + -logie | synchronique | `cmp_xf5719he.json` |
| 6 | ethnologique | ethnologie + -ique | synchronique | `cmp_8ensde5p.json` |
| 6 | ethnologue | ethno- + -logue | synchronique | `cmp_6s6s5tkq.json` |
| 6 | ethnonyme | ethno- + -onyme | synchronique | `cmp_62aap389.json` |
| 6 | ethnopsychologie | ethno- + psychologie | synchronique | `cmp_dxted83n.json` |
| 6 | graphologie | grapho- + -logie | synchronique | `cmp_hmyj5zy2.json` |
| 6 | graphologue | grapho- + -logue | synchronique | `cmp_0krsvn9v.json` |
| 6 | graphomètre | grapho- + -mètre | synchronique | `cmp_4nbvzy75.json` |
| 6 | graphothérapie | grapho- + -thérapie | synchronique | `cmp_xnsy1vns.json` |
| 6 | géogonie | géo- + -gonie | synchronique | `cmp_32228tbz.json` |
| 6 | homophonie | homo- + -phonie | synchronique | `cmp_p2zh31ty.json` |
| 6 | hétérophonie | hétéro- + -phonie | synchronique | `cmp_hg77zgtc.json` |
| 6 | idéogramme | idéo- + -gramme | synchronique | `cmp_4s4pf6y3.json` |
| 6 | idéographie | idéo- + -graphie | synchronique | `cmp_ke3c6hzx.json` |
| 6 | idéologie | idéo- + -logie | synchronique | `cmp_8jesqfth.json` |
| 6 | idéologique | idéologie + -ique | synchronique | `cmp_zytdfqjw.json` |
| 6 | idéologue | idéo- + -logue | synchronique | `cmp_9qavsqx3.json` |
| 6 | idéophone | idéo- + -phone | synchronique | `cmp_dbzgs9nz.json` |
| 6 | lexicographe | lexico- + -graphe | synchronique | `cmp_sb8rcvq3.json` |
| 6 | lexicographie | lexico- + -graphie | synchronique | `cmp_4sgk0mb8.json` |
| 6 | lexicologie | lexico- + -logie | synchronique | `cmp_g9pk42gq.json` |
| 6 | lexicologue | lexico- + -logue | synchronique | `cmp_qbq71s2j.json` |
| 6 | microphonie | micro- + -phonie | synchronique | `cmp_yrsmng9b.json` |
| 6 | monocrate | mono- + -crate | synchronique | `cmp_ddmpyevj.json` |
| 6 | monophonie | mono- + -phonie | synchronique | `cmp_geakp16j.json` |
| 6 | mythique | mytho- + -ique | synchronique | `cmp_xfj62zvb.json` |
| 6 | mythographe | mytho- + -graphe | synchronique | `cmp_cwtwb6f8.json` |
| 6 | mythographie | mytho- + -graphie | synchronique | `cmp_dpgw9enh.json` |
| 6 | mythologie | mytho- + -logie | synchronique | `cmp_f7np0a03.json` |
| 6 | mythologique | mythologie + -ique | synchronique | `cmp_p7cqmr2s.json` |
| 6 | mythologue | mytho- + -logue | synchronique | `cmp_gvfcy210.json` |
| 6 | orthochrome | ortho- + -chrome | synchronique | `cmp_8ghm5aeq.json` |
| 6 | orthographe | ortho- + -graphe | synchronique | `cmp_fxzkpp8e.json` |
| 6 | orthographie | ortho- + -graphie | synchronique | `cmp_th6ytsq6.json` |
| 6 | orthonyme | ortho- + -onyme | synchronique | `cmp_m8dpcm6c.json` |
| 6 | orthophonie | ortho- + -phonie | synchronique | `cmp_e047xn2e.json` |
| 6 | orthoscope | ortho- + -scope | synchronique | `cmp_n79xnzr3.json` |
| 6 | phonique | phono- + -ique | synchronique | `cmp_am8012dr.json` |
| 6 | phonogramme | phono- + -gramme | synchronique | `cmp_hd80jadz.json` |
| 6 | phonographe | phono- + -graphe | synchronique | `cmp_57map9ch.json` |
| 6 | phonographie | phono- + -graphie | synchronique | `cmp_bp00dz84.json` |
| 6 | phonologie | phono- + -logie | synchronique | `cmp_x4dw4v00.json` |
| 6 | phonologique | phonologie + -ique | synchronique | `cmp_hg8aazpq.json` |
| 6 | phonologue | phono- + -logue | synchronique | `cmp_dtdv3mj1.json` |
| 6 | phonomètre | phono- + -mètre | synchronique | `cmp_7krz904c.json` |
| 6 | phonophobie | phono- + -phobie | synchronique | `cmp_08dm4ksh.json` |
| 6 | photophonie | photo- + -phonie | synchronique | `cmp_0mnv4ee5.json` |
| 6 | polyphonie | poly- + -phonie | synchronique | `cmp_8cwzs7gh.json` |
| 6 | symphonie | syn- + -phonie | synchronique | `cmp_nfy1149m.json` |
| 6 | technocrate | techno- + -crate | synchronique | `cmp_hzvwytkz.json` |
| 6 | théocrate | théo- + -crate | synchronique | `cmp_bzf6e6dp.json` |
| 6 | théogonie | théo- + -gonie | synchronique | `cmp_zprzqd5j.json` |
| 6 | téléphonie | télé- + -phonie | synchronique | `cmp_re4j72qz.json` |
| 7 | anaglyphe | ana- + -glyphe | synchronique | `cmp_03z97h7q.json` |
| 7 | anagramme | ana- + -gramme | synchronique | `cmp_qw6k32w5.json` |
| 7 | analogie | ana- + -logie | synchronique | `cmp_f5zxsbyg.json` |
| 7 | analogue | ana- + -logue | synchronique | `cmp_66rtvz6x.json` |
| 7 | anarchie | a- + -archie | synchronique | `cmp_9b25amdb.json` |
| 7 | anarchique | anarchie + -ique | synchronique | `cmp_0hdtjmb4.json` |
| 7 | anarchiste | anarchie + -iste | synchronique | `cmp_1magvna5.json` |
| 7 | anatomie | ana- + -tomie | synchronique | `cmp_vc9y095j.json` |
| 7 | anatomique | anatomie + -ique | synchronique | `cmp_6btestf5.json` |
| 7 | anatomiste | anatomie + -iste | synchronique | `cmp_ajg3v0td.json` |
| 7 | anthropomancie | anthropo- + -mancie | synchronique | `cmp_8qf1gfh7.json` |
| 7 | astromancie | astro- + -mancie | synchronique | `cmp_f7d10gms.json` |
| 7 | atmomètre | atmo- + -mètre | synchronique | `cmp_cgctk22j.json` |
| 7 | atmosphère | atmo- + -sphère | synchronique | `cmp_s4h8yznd.json` |
| 7 | autarchie | auto- + -archie | synchronique | `cmp_30pcshsc.json` |
| 7 | autotomie | auto- + -tomie | synchronique | `cmp_256d4zb4.json` |
| 7 | bibliomancie | biblio- + -mancie | synchronique | `cmp_bqr5mxyp.json` |
| 7 | bibliothèque | biblio- + -thèque | synchronique | `cmp_9cp4htm0.json` |
| 7 | cardiotomie | cardio- + -tomie | synchronique | `cmp_a1tg2b10.json` |
| 7 | cartogramme | carto- + -gramme | synchronique | `cmp_65bnkwva.json` |
| 7 | cartographe | carto- + -graphe | synchronique | `cmp_0mggnaxf.json` |
| 7 | cartographie | carto- + -graphie | synchronique | `cmp_snntjhzk.json` |
| 7 | cartographique | cartographie + -ique | synchronique | `cmp_wjbgbbr8.json` |
| 7 | cartomancie | carto- + -mancie | synchronique | `cmp_s6sapkx7.json` |
| 7 | cartothèque | carto- + -thèque | synchronique | `cmp_m8hb33fn.json` |
| 7 | cristallographe | cristallo- + -graphe | synchronique | `cmp_npggy6dj.json` |
| 7 | cristallographie | cristallo- + -graphie | synchronique | `cmp_w3hjzcn7.json` |
| 7 | cristallomancie | cristallo- + -mancie | synchronique | `cmp_bpbs1db4.json` |
| 7 | ethnarchie | ethno- + -archie | synchronique | `cmp_97skt75v.json` |
| 7 | ethnarque | ethno- + -arque | synchronique | `cmp_ft3nddsb.json` |
| 7 | géoglyphe | géo- + -glyphe | synchronique | `cmp_ynh7rdfj.json` |
| 7 | géomancie | géo- + -mancie | synchronique | `cmp_pmk5c3bf.json` |
| 7 | hiérarchie | hiéro- + -archie | synchronique | `cmp_1end5fk1.json` |
| 7 | hiérarchique | hiérarchie + -ique | synchronique | `cmp_k20d7hmp.json` |
| 7 | hiérarque | hiéro- + -arque | synchronique | `cmp_kny33jds.json` |
| 7 | hiérocrate | hiéro- + -crate | synchronique | `cmp_w81z7ee5.json` |
| 7 | hiéroglyphe | hiéro- + -glyphe | synchronique | `cmp_2tays11a.json` |
| 7 | hiérogramme | hiéro- + -gramme | synchronique | `cmp_2t8wcfze.json` |
| 7 | hydromancie | hydro- + -mancie | synchronique | `cmp_vsh4m1rh.json` |
| 7 | hétérarchie | hétéro- + -archie | synchronique | `cmp_4fyma26h.json` |
| 7 | lithique | litho- + -ique | synchronique | `cmp_bbxzjzbz.json` |
| 7 | lithographe | litho- + -graphe | synchronique | `cmp_2vrzbjmz.json` |
| 7 | lithographie | litho- + -graphie | synchronique | `cmp_7tqv2sxh.json` |
| 7 | lithographique | lithographie + -ique | synchronique | `cmp_95nkxtvx.json` |
| 7 | lithographiste | lithographie + -iste | synchronique | `cmp_2e5ah7jc.json` |
| 7 | lithologie | litho- + -logie | synchronique | `cmp_vpzyp5yh.json` |
| 7 | lithomancie | litho- + -mancie | synchronique | `cmp_mv6wrp0d.json` |
| 7 | lithophage | litho- + -phage | synchronique | `cmp_ef4hcyhv.json` |
| 7 | lithophone | litho- + -phone | synchronique | `cmp_ac1kmf0w.json` |
| 7 | lithosphère | litho- + -sphère | synchronique | `cmp_px0mbjnj.json` |
| 7 | lithothèque | litho- + -thèque | synchronique | `cmp_ebv0z448.json` |
| 7 | lithothérapie | litho- + -thérapie | synchronique | `cmp_dp8mab5k.json` |
| 7 | lithotomie | litho- + -tomie | synchronique | `cmp_v2vqdhs7.json` |
| 7 | microlithe | micro- + -lithe | synchronique | `cmp_dmwz10vh.json` |
| 7 | microtomie | micro- + -tomie | synchronique | `cmp_2dktmx6f.json` |
| 7 | monarchie | mono- + -archie | synchronique | `cmp_e2pkwm3k.json` |
| 7 | monarchique | monarchie + -ique | synchronique | `cmp_dbzxst0y.json` |
| 7 | monarchiste | monarchie + -iste | synchronique | `cmp_8qn49j6v.json` |
| 7 | monarque | mono- + -arque | synchronique | `cmp_2h0g7kaf.json` |
| 7 | monolithe | mono- + -lithe | synchronique | `cmp_kd7rnt8c.json` |
| 7 | mégalithe | méga- + -lithe | synchronique | `cmp_v2nb69aw.json` |
| 7 | mégaphone | méga- + -phone | synchronique | `cmp_m10tcefc.json` |
| 7 | mégascope | méga- + -scope | synchronique | `cmp_aw5q58kf.json` |
| 7 | neurotomie | neuro- + -tomie | synchronique | `cmp_56krq7et.json` |
| 7 | oligarchie | oligo- + -archie | synchronique | `cmp_nyrpnktq.json` |
| 7 | oligarque | oligo- + -arque | synchronique | `cmp_rknvyhas.json` |
| 7 | phonolithe | phono- + -lithe | synchronique | `cmp_d00e8rbt.json` |
| 7 | phonothèque | phono- + -thèque | synchronique | `cmp_c6qe1mb4.json` |
| 7 | photothèque | photo- + -thèque | synchronique | `cmp_eg609t2b.json` |
| 7 | polyarchie | poly- + -archie | synchronique | `cmp_b3dt8k6p.json` |
| 7 | pétroglyphe | pétro- + -glyphe | synchronique | `cmp_2dhc7t2x.json` |
| 7 | pétrographe | pétro- + -graphe | synchronique | `cmp_hhg4aqq3.json` |
| 7 | pétrographie | pétro- + -graphie | synchronique | `cmp_wdak4qt0.json` |
| 7 | pétrologie | pétro- + -logie | synchronique | `cmp_6dt2sh2t.json` |
| 7 | pétrologue | pétro- + -logue | synchronique | `cmp_6r76ct04.json` |
| 7 | stratosphère | strato- + -sphère | synchronique | `cmp_4hvvyznn.json` |
| 7 | théarchie | théo- + -archie | synchronique | `cmp_nr9sw6cv.json` |
| 7 | théomancie | théo- + -mancie | synchronique | `cmp_jx5p259n.json` |
| 7 | xylothèque | xylo- + -thèque | synchronique | `cmp_42gkff25.json` |
| 7 | zoolithe | zoo- + -lithe | synchronique | `cmp_6vvq00xn.json` |
| 7 | zootomie | zoo- + -tomie | synchronique | `cmp_fr7jk0hg.json` |
| 7 | épigramme | épi- + -gramme | synchronique | `cmp_3v7gk9yj.json` |
| 7 | épigraphe | épi- + -graphe | synchronique | `cmp_e1xdngnd.json` |
| 7 | épigraphie | épi- + -graphie | synchronique | `cmp_rbkkbahe.json` |
| 7 | épigène | épi- + -gène | synchronique | `cmp_dw4165cq.json` |
| 7 | épilogue | épi- + -logue | synchronique | `cmp_cyec0932.json` |
| 7 | épiscope | épi- + -scope | synchronique | `cmp_sc3g487w.json` |
| 7 | épitaphe | épi- + -taphe | synchronique | `cmp_bnp9a4ad.json` |
| 7 | éponyme | épi- + -onyme | synchronique | `cmp_me5qwmaa.json` |
| 8 | autosome | auto- + -some | synchronique | `cmp_w5zpfeb0.json` |
| 8 | biotype | bio- + -type | synchronique | `cmp_xskzaeb8.json` |
| 8 | bradycardie | brady- + -cardie | synchronique | `cmp_r2g74sfj.json` |
| 8 | centilitre | centi- + litre | synchronique | `cmp_8338v3pg.json` |
| 8 | centimétrique | centimètre + -ique | synchronique | `cmp_a9158p4c.json` |
| 8 | chromique | chromo- + -ique | synchronique | `cmp_gey891nv.json` |
| 8 | chromogène | chromo- + -gène | synchronique | `cmp_js7h07ym.json` |
| 8 | chromolithographie | chromo- + lithographie | synchronique | `cmp_s81ynz09.json` |
| 8 | chromosome | chromo- + -some | synchronique | `cmp_59kmgw5y.json` |
| 8 | chromosphère | chromo- + -sphère | synchronique | `cmp_66r3b3xa.json` |
| 8 | chromothérapie | chromo- + -thérapie | synchronique | `cmp_45afydd1.json` |
| 8 | chromotypie | chromo- + -typie | synchronique | `cmp_j1tf0r79.json` |
| 8 | diarchie | di- + -archie | synchronique | `cmp_191nsrct.json` |
| 8 | digamie | di- + -gamie | synchronique | `cmp_vd1fgnj9.json` |
| 8 | digramme | di- + -gramme | synchronique | `cmp_xd3kapfr.json` |
| 8 | dimorphe | di- + -morphe | synchronique | `cmp_xpytkj9m.json` |
| 8 | diphone | di- + -phone | synchronique | `cmp_fxv7fpab.json` |
| 8 | décalogue | déca- + -logue | synchronique | `cmp_c29wctyv.json` |
| 8 | décamètre | déca- + -mètre | synchronique | `cmp_pn0rwr9b.json` |
| 8 | décilitre | déci- + litre | synchronique | `cmp_0dtyhr13.json` |
| 8 | hectolitre | hecto- + litre | synchronique | `cmp_77m2s1yy.json` |
| 8 | hectomètre | hecto- + -mètre | synchronique | `cmp_59fpksfp.json` |
| 8 | homotype | homo- + -type | synchronique | `cmp_wpemj51s.json` |
| 8 | kilolitre | kilo- + litre | synchronique | `cmp_h5x8wazd.json` |
| 8 | kilométrique | kilomètre + -ique | synchronique | `cmp_v0xq5hee.json` |
| 8 | lithotypie | litho- + -typie | synchronique | `cmp_s699bz16.json` |
| 8 | microlitre | micro- + litre | synchronique | `cmp_n0tkz9s3.json` |
| 8 | microsome | micro- + -some | synchronique | `cmp_66pawdjx.json` |
| 8 | millilitre | milli- + litre | synchronique | `cmp_95kgydab.json` |
| 8 | millimétrique | millimètre + -ique | synchronique | `cmp_k9c4pwz4.json` |
| 8 | monotype | mono- + -type | synchronique | `cmp_327qfqew.json` |
| 8 | monotypie | mono- + -typie | synchronique | `cmp_2snrtkrj.json` |
| 8 | nanolitre | nano- + litre | synchronique | `cmp_7rwcfrvq.json` |
| 8 | phototype | photo- + -type | synchronique | `cmp_s1qv81s6.json` |
| 8 | phototypie | photo- + -typie | synchronique | `cmp_vm00p9gs.json` |
| 8 | polysome | poly- + -some | synchronique | `cmp_kdv9yr42.json` |
| 8 | polytype | poly- + -type | synchronique | `cmp_g1nt7pan.json` |
| 8 | prototype | proto- + -type | synchronique | `cmp_qxkjfan9.json` |
| 8 | sténogramme | sténo- + -gramme | synchronique | `cmp_xcq7tz0q.json` |
| 8 | sténographe | sténo- + -graphe | synchronique | `cmp_8mnw0xaq.json` |
| 8 | sténographie | sténo- + -graphie | synchronique | `cmp_s7dt73e2.json` |
| 8 | sténographique | sténographie + -ique | synchronique | `cmp_2babpvfc.json` |
| 8 | sténotype | sténo- + -type | synchronique | `cmp_tg64jjmf.json` |
| 8 | sténotypie | sténo- + -typie | synchronique | `cmp_gssm3hbe.json` |
| 8 | stéréogramme | stéréo- + -gramme | synchronique | `cmp_jgrw6vqn.json` |
| 8 | stéréographie | stéréo- + -graphie | synchronique | `cmp_y8v00kvw.json` |
| 8 | stéréomètre | stéréo- + -mètre | synchronique | `cmp_vfdkzdcx.json` |
| 8 | stéréophonie | stéréo- + -phonie | synchronique | `cmp_f8nrcqmp.json` |
| 8 | stéréophonique | stéréophonie + -ique | synchronique | `cmp_nrz4b6dt.json` |
| 8 | stéréoscope | stéréo- + -scope | synchronique | `cmp_xt8abfrm.json` |
| 8 | stéréotype | stéréo- + -type | synchronique | `cmp_fhk4hh1w.json` |
| 8 | stéréotypie | stéréo- + -typie | synchronique | `cmp_d5rme24a.json` |
| 8 | tachycardie | tachy- + -cardie | synchronique | `cmp_bkrbqbtj.json` |
| 8 | tachygraphe | tachy- + -graphe | synchronique | `cmp_ecqfjv99.json` |
| 8 | tachygraphie | tachy- + -graphie | synchronique | `cmp_g8bq4m2h.json` |
| 8 | tachymètre | tachy- + -mètre | synchronique | `cmp_8pkk1j6s.json` |
| 8 | triarchie | tri- + -archie | synchronique | `cmp_06pdrjwk.json` |
| 8 | trichrome | tri- + -chrome | synchronique | `cmp_ntbahsg9.json` |
| 8 | trigamie | tri- + -gamie | synchronique | `cmp_k9rasbtx.json` |
| 8 | triglyphe | tri- + -glyphe | synchronique | `cmp_8tym32fs.json` |
| 8 | trigramme | tri- + -gramme | synchronique | `cmp_ee4q2sk0.json` |
| 8 | trilogie | tri- + -logie | synchronique | `cmp_ssg1e7nx.json` |
| 8 | trimorphe | tri- + -morphe | synchronique | `cmp_t0jjycwn.json` |
| 8 | trimètre | tri- + -mètre | synchronique | `cmp_25t829pz.json` |
| 8 | typique | typo- + -ique | synchronique | `cmp_c7taw036.json` |
| 8 | typographe | typo- + -graphe | synchronique | `cmp_cmk2z70m.json` |
| 8 | typographie | typo- + -graphie | synchronique | `cmp_fwxf8qax.json` |
| 8 | typographique | typographie + -ique | synchronique | `cmp_b4rwdqta.json` |
| 8 | typologie | typo- + -logie | synchronique | `cmp_nytrdmdm.json` |
| 8 | typomètre | typo- + -mètre | synchronique | `cmp_p9f4b8xp.json` |
| 8 | télétype | télé- + -type | synchronique | `cmp_yqajdrck.json` |
| 8 | tétragramme | tétra- + -gramme | synchronique | `cmp_ghg0k612.json` |
| 8 | tétralogie | tétra- + -logie | synchronique | `cmp_5x1d1ev1.json` |
| 8 | tétramètre | tétra- + -mètre | synchronique | `cmp_aj3108h8.json` |
| 8 | tétrarchie | tétra- + -archie | synchronique | `cmp_0b5nx362.json` |
| 8 | tétrarque | tétra- + -arque | synchronique | `cmp_4dmkwzem.json` |
| 8 | écotype | éco- + -type | synchronique | `cmp_kmrq8ktd.json` |
| 9 | cardialgie | cardio- + -algie | synchronique | `cmp_d6x7dy6m.json` |
| 9 | dermatoglyphe | dermato- + -glyphe | synchronique | `cmp_tdb0scmv.json` |
| 9 | dermatologie | dermato- + -logie | synchronique | `cmp_hhdmyhjy.json` |
| 9 | dermatologique | dermatologie + -ique | synchronique | `cmp_9rrc4nxn.json` |
| 9 | dermatologiste | dermatologie + -iste | synchronique | `cmp_nr7ehtep.json` |
| 9 | dermatologue | dermato- + -logue | synchronique | `cmp_wp7kr7fj.json` |
| 9 | endogamie | endo- + -gamie | synchronique | `cmp_yzajgtsq.json` |
| 9 | endogène | endo- + -gène | synchronique | `cmp_8w65s0aq.json` |
| 9 | endoscope | endo- + -scope | synchronique | `cmp_7qvkpmyn.json` |
| 9 | endoscopie | endo- + -scopie | synchronique | `cmp_6md0h6gk.json` |
| 9 | exobiologie | exo- + biologie | synchronique | `cmp_5gt8tmn9.json` |
| 9 | exogamie | exo- + -gamie | synchronique | `cmp_pqs9ajn5.json` |
| 9 | exogène | exo- + -gène | synchronique | `cmp_3rmpvvhy.json` |
| 9 | exosphère | exo- + -sphère | synchronique | `cmp_rasdyjr9.json` |
| 9 | gastralgie | gastro- + -algie | synchronique | `cmp_yd8kkgyz.json` |
| 9 | gastrique | gastro- + -ique | synchronique | `cmp_0bs0ccx8.json` |
| 9 | gastronomie | gastro- + -nomie | synchronique | `cmp_q5wy61nf.json` |
| 9 | gastropathie | gastro- + -pathie | synchronique | `cmp_02dbepp4.json` |
| 9 | gastroscope | gastro- + -scope | synchronique | `cmp_p3rjkszr.json` |
| 9 | gastroscopie | gastro- + -scopie | synchronique | `cmp_m52ca66w.json` |
| 9 | gastrotomie | gastro- + -tomie | synchronique | `cmp_tgc6f3pj.json` |
| 9 | gynécocratie | gynéco- + -cratie | synchronique | `cmp_ed0gcfca.json` |
| 9 | gynécologie | gynéco- + -logie | synchronique | `cmp_tqcgh0hq.json` |
| 9 | gynécologique | gynécologie + -ique | synchronique | `cmp_5kmxyq32.json` |
| 9 | gynécologiste | gynécologie + -iste | synchronique | `cmp_b5zk430n.json` |
| 9 | gynécologue | gynéco- + -logue | synchronique | `cmp_2f8k02a7.json` |
| 9 | hématique | hémato- + -ique | synchronique | `cmp_p0r5hqm2.json` |
| 9 | hématologie | hémato- + -logie | synchronique | `cmp_a647msrr.json` |
| 9 | hématologique | hématologie + -ique | synchronique | `cmp_1hbnr14t.json` |
| 9 | hématologiste | hématologie + -iste | synchronique | `cmp_dphx54w7.json` |
| 9 | hématologue | hémato- + -logue | synchronique | `cmp_efwtabmz.json` |
| 9 | hématophage | hémato- + -phage | synchronique | `cmp_0q5fetn0.json` |
| 9 | hématophobie | hémato- + -phobie | synchronique | `cmp_z3eehp3j.json` |
| 9 | microscopie | micro- + -scopie | synchronique | `cmp_mayvvvtz.json` |
| 9 | myalgie | myo- + -algie | synchronique | `cmp_5he0a02k.json` |
| 9 | myologie | myo- + -logie | synchronique | `cmp_np9bj1b1.json` |
| 9 | myopathie | myo- + -pathie | synchronique | `cmp_a5zfjzy0.json` |
| 9 | nostalgie | nost- + -algie | synchronique | `cmp_pynsrm6d.json` |
| 9 | névralgie | neuro- + -algie | synchronique | `cmp_8sth5gtc.json` |
| 9 | odontalgie | odonto- + -algie | synchronique | `cmp_gdr35t1m.json` |
| 9 | odontologie | odonto- + -logie | synchronique | `cmp_0j1kdv62.json` |
| 9 | ophtalmique | ophtalmo- + -ique | synchronique | `cmp_mrvvntga.json` |
| 9 | ophtalmologie | ophtalmo- + -logie | synchronique | `cmp_e5cryxyj.json` |
| 9 | ophtalmologique | ophtalmologie + -ique | synchronique | `cmp_6185ntwx.json` |
| 9 | ophtalmologiste | ophtalmologie + -iste | synchronique | `cmp_sk8r666j.json` |
| 9 | ophtalmologue | ophtalmo- + -logue | synchronique | `cmp_391z7spe.json` |
| 9 | ophtalmoscope | ophtalmo- + -scope | synchronique | `cmp_v6kph9f9.json` |
| 9 | ophtalmoscopie | ophtalmo- + -scopie | synchronique | `cmp_xqarpa91.json` |
| 9 | ostéologie | ostéo- + -logie | synchronique | `cmp_a3t8ne67.json` |
| 9 | ostéopathie | ostéo- + -pathie | synchronique | `cmp_256yct1h.json` |
| 9 | ostéotomie | ostéo- + -tomie | synchronique | `cmp_7de5s61z.json` |
| 9 | otalgie | oto- + -algie | synchronique | `cmp_hbtvrnax.json` |
| 9 | otologie | oto- + -logie | synchronique | `cmp_frkaaqj7.json` |
| 9 | otoscope | oto- + -scope | synchronique | `cmp_yxsxa7jh.json` |
| 9 | otoscopie | oto- + -scopie | synchronique | `cmp_e40pcwmb.json` |
| 9 | pathogène | patho- + -gène | synchronique | `cmp_k3pa6qrh.json` |
| 9 | pathologie | patho- + -logie | synchronique | `cmp_a87zcqha.json` |
| 9 | pathologique | pathologie + -ique | synchronique | `cmp_9zx9z5f8.json` |
| 9 | pathologiste | pathologie + -iste | synchronique | `cmp_9sqk7v9r.json` |
| 9 | phoniatre | phono- + -iatre | synchronique | `cmp_8gfx8nme.json` |
| 9 | phoniatrie | phono- + -iatrie | synchronique | `cmp_5v04capn.json` |
| 9 | psychiatre | psycho- + -iatre | synchronique | `cmp_n1897908.json` |
| 9 | psychiatrie | psycho- + -iatrie | synchronique | `cmp_wpsh869f.json` |
| 9 | stéréoscopie | stéréo- + -scopie | synchronique | `cmp_ayxat3c4.json` |
| 10 | anarchisme | anarchie + -isme | synchronique | `cmp_tnxq75y3.json` |
| 10 | anomie | a- + -nomie | synchronique | `cmp_q8v4n7sq.json` |
| 10 | anthropophagie | anthropo- + -phagie | synchronique | `cmp_n2p7brdg.json` |
| 10 | apathie | a- + -pathie | synchronique | `cmp_fm71ertq.json` |
| 10 | aphagie | a- + -phagie | synchronique | `cmp_qa2rcpm7.json` |
| 10 | astronaute | astro- + -naute | synchronique | `cmp_8rjk9we7.json` |
| 10 | athéisme | a- + théisme | synchronique | `cmp_az3r7cpx.json` |
| 10 | autisme | auto- + -isme | synchronique | `cmp_yvgx6x9c.json` |
| 10 | autodrome | auto- + -drome | synchronique | `cmp_zbe5bnnd.json` |
| 10 | aérodrome | aéro- + -drome | synchronique | `cmp_vmfkrhqk.json` |
| 10 | aérogramme | aéro- + -gramme | synchronique | `cmp_eq7krf84.json` |
| 10 | aérographe | aéro- + -graphe | synchronique | `cmp_rsgz9v9a.json` |
| 10 | aérolithe | aéro- + -lithe | synchronique | `cmp_q13128jn.json` |
| 10 | aérologie | aéro- + -logie | synchronique | `cmp_w8agzv5t.json` |
| 10 | aéronaute | aéro- + -naute | synchronique | `cmp_va3skhp1.json` |
| 10 | aérophagie | aéro- + -phagie | synchronique | `cmp_19tbzztp.json` |
| 10 | aérophobie | aéro- + -phobie | synchronique | `cmp_fv6mb3rp.json` |
| 10 | bibliomane | biblio- + -mane | synchronique | `cmp_3d63vsx6.json` |
| 10 | bibliomanie | biblio- + -manie | synchronique | `cmp_vcwcayqa.json` |
| 10 | biocide | bio- + -cide | synchronique | `cmp_pmawf1fy.json` |
| 10 | cosmodrome | cosmo- + -drome | synchronique | `cmp_xdmxq5d9.json` |
| 10 | cosmonaute | cosmo- + -naute | synchronique | `cmp_6qv3g0c5.json` |
| 10 | diathermie | dia- + -thermie | synchronique | `cmp_a440061r.json` |
| 10 | ethnocide | ethno- + -cide | synchronique | `cmp_15f5ymev.json` |
| 10 | graphomane | grapho- + -mane | synchronique | `cmp_fg2524gg.json` |
| 10 | graphomanie | grapho- + -manie | synchronique | `cmp_ff39gp98.json` |
| 10 | génocide | géno- + -cide | synchronique | `cmp_bwgy3efx.json` |
| 10 | génotype | géno- + -type | synchronique | `cmp_j21s1etr.json` |
| 10 | géophagie | géo- + -phagie | synchronique | `cmp_tnynhmvg.json` |
| 10 | géothermie | géo- + -thermie | synchronique | `cmp_9arcgket.json` |
| 10 | hippiatrie | hippo- + -iatrie | synchronique | `cmp_49rqacqf.json` |
| 10 | hippodrome | hippo- + -drome | synchronique | `cmp_8dqe397z.json` |
| 10 | hippologie | hippo- + -logie | synchronique | `cmp_jhb739dq.json` |
| 10 | hippophage | hippo- + -phage | synchronique | `cmp_3xx92cza.json` |
| 10 | hippophagie | hippo- + -phagie | synchronique | `cmp_cb15hj4s.json` |
| 10 | hippothérapie | hippo- + -thérapie | synchronique | `cmp_vjz7svvk.json` |
| 10 | hypergamie | hyper- + -gamie | synchronique | `cmp_82pcnsb0.json` |
| 10 | hyperonyme | hyper- + -onyme | synchronique | `cmp_5c8g14bx.json` |
| 10 | hyperphagie | hyper- + -phagie | synchronique | `cmp_zexnn6z9.json` |
| 10 | hyperthermie | hyper- + -thermie | synchronique | `cmp_hp36yj92.json` |
| 10 | hypogamie | hypo- + -gamie | synchronique | `cmp_ke8m5q5p.json` |
| 10 | hyponyme | hypo- + -onyme | synchronique | `cmp_vn973yxw.json` |
| 10 | hypothermie | hypo- + -thermie | synchronique | `cmp_fn1xna2h.json` |
| 10 | hématophagie | hémato- + -phagie | synchronique | `cmp_84ayy6x7.json` |
| 10 | monarchisme | monarchie + -isme | synchronique | `cmp_w4c5vs4k.json` |
| 10 | monisme | mono- + -isme | synchronique | `cmp_w66p97s3.json` |
| 10 | monomane | mono- + -mane | synchronique | `cmp_ktmm1db7.json` |
| 10 | monomanie | mono- + -manie | synchronique | `cmp_9qm1nsv9.json` |
| 10 | monothéisme | mono- + théisme | synchronique | `cmp_j6659m81.json` |
| 10 | moralisme | moral + -isme | synchronique | `cmp_5k9my8t6.json` |
| 10 | mythomane | mytho- + -mane | synchronique | `cmp_qbd6a7kf.json` |
| 10 | mythomanie | mytho- + -manie | synchronique | `cmp_dk3832qc.json` |
| 10 | néologie | néo- + -logie | synchronique | `cmp_f5jteyse.json` |
| 10 | néophobie | néo- + -phobie | synchronique | `cmp_eb6tfq0t.json` |
| 10 | palindrome | palin- + -drome | synchronique | `cmp_236mhnfw.json` |
| 10 | pangramme | pan- + -gramme | synchronique | `cmp_fbm9hv80.json` |
| 10 | panthéisme | pan- + théisme | synchronique | `cmp_h679nm1e.json` |
| 10 | polyphagie | poly- + -phagie | synchronique | `cmp_8fayfamh.json` |
| 10 | polythéisme | poly- + théisme | synchronique | `cmp_vnrckhqt.json` |
| 10 | prodrome | pro- + -drome | synchronique | `cmp_0f7t936p.json` |
| 10 | psychisme | psycho- + -isme | synchronique | `cmp_hzgs8kdq.json` |
| 10 | psychologisme | psychologie + -isme | synchronique | `cmp_2vkzhhen.json` |
| 10 | syndrome | syn- + -drome | synchronique | `cmp_bet8weaz.json` |
| 10 | théisme | théo- + -isme | synchronique | `cmp_g5t238tn.json` |
| 10 | théiste | théo- + -iste | synchronique | `cmp_9m1tkbd3.json` |
| 10 | xylophagie | xylo- + -phagie | synchronique | `cmp_wajfskf6.json` |
| 10 | écocide | éco- + -cide | synchronique | `cmp_a0bp6p3j.json` |
| 10 | écologisme | écologie + -isme | synchronique | `cmp_21kvmyps.json` |
| 10 | économisme | économie + -isme | synchronique | `cmp_2ev00vy5.json` |
| — | Biologie | βίος + λόγος | historique | `cmp_eh13hkzf.json` |
| — | barometer | βάρος + μέτρον | historique | `cmp_5wqca82n.json` |
| — | monogramma | μόνος + γράμμα | historique | `cmp_aczn9ytf.json` |
| — | palaeographia | παλαιός + γράφειν | historique | `cmp_6d71g3fg.json` |
| — | psychologia | ψυχή + λόγος | historique | `cmp_dp379k9n.json` |
| — | telescopium | τῆλε + σκοπεῖν | historique | `cmp_b9tk1yes.json` |
| — | Ökologie | οἶκος + λόγος | historique | `cmp_eb8j3t2f.json` |
| — | καλλιγράφος | κάλλος + γράφειν | historique | `cmp_cqmf0f95.json` |
| — | περίμετρος | περί + μέτρον | historique | `cmp_r4avqtsk.json` |
| — | πολύχρωμος | πολύς + χρῶμα | historique | `cmp_1dhzwxtf.json` |
| — | πρόλογος | πρό + λόγος | historique | `cmp_ds0ahxr9.json` |
| — | φιλοσοφία | φίλος + σοφία | historique | `cmp_mjnfz7m0.json` |
| — | φιλόλογος | φίλος + λόγος | historique | `cmp_60fcyk1w.json` |
| — | ἀρχαιολογία | ἀρχαῖος + λόγος | historique | `cmp_84kmr29p.json` |
| — | ἐτυμολογία | ἔτυμον + -λογία | historique | `cmp_0dr3c7s7.json` |

## 7. Attestations retirées

Dates lues dans le TLFi le 9 octobre 2026, retirées faute de P6684 dans Wikidata (revérifié le 10 octobre 2026 : aucun des 308 lexèmes français du catalogue ne porte P6684). Elles restent dans l’historique git (commit avant #31). À ressaisir si Wikidata reçoit la déclaration.

agamie 1814, agraphie 1865, agrologie 1836, agronomie 1372-1374 (intervalle), amener 950 (siecle), amoral 1885, amorphe 1784, anaglyphe 1495, anagramme 1571, analogie 1213, analogue 1503, anarchie 1372-1374 (intervalle), anarchique 1593, anarchisme 1840, anarchiste 1791, anatomie 1370, anatomique 1546, anatomiste 1503, anomie 1884, anonyme 1557, anthropologie 1516, anthropologiste 1808, anthropologue 1865, anthropomorphe 1811, anthroponymie 1938, anthropophage 1375, anthropophagie 1501 (siecle), antilogie 1623, antipathie 1555, antiphonie 1380, antonyme 1866 (avant), antonymie 1794 (avant), apathie 1375, aphone 1834, aphonie 1617, archéologie 1599, archéologue 1812, astrologie 1250 (vers), astrologue 1511, astronaute 1928, astronomie 1155 (vers), athéisme 1555, atmosphère 1665, autarchie 1896, autisme 1923, autocrate 1768, autocratie 1794 (avant), autodrome 1909, autographe 1553, autonomie 1596, aérodrome 1896, aérographe 1834, aérolithe 1834, aérologie 1696, aéronaute 1784, aérophagie 1891, aérophobie 1751, baromètre 1666, bibliographe 1665, bibliographie 1633, bibliologie 1845, bibliomane 1601 (siecle), bibliophile 1740, bibliothèque 1493, biographe 1693 (avant), biographie 1755, biographique 1800, biogène 1838, biologie 1802, biologique 1832, biologiste 1832, biologue 1832, biomètre 1832, bionomie 1838, biosphère 1838, biotype 1946, bradycardie 1917, calligraphe 1751, calligraphie 1569, calligraphique 1823, cardialgie 1546, cardiogramme 1922, cardiographe 1832, cardiographie 1793, cardiologie 1762, cardiologue 1920-1924 (intervalle), cardiopathie 1858, cartographie 1832, cartographique 1832, cartomancie 1803, cartothèque 1931, centilitre 1800, centimètre 1793, centimétrique 1912, chromique 1797, chromogène 1894, chromolithographie 1837, chromosome 1891, chromosphère 1877, chromotypie 1884, chronique 1314, chronogramme 1752, chronographe 1488, chronographie 1505 (avant), chronologie 1579, chronologique 1584, chronologiste 1560, chronomètre 1701, cosmique 1380 (vers), cosmogonie 1585, cosmographe 1361 (vers), cosmographie 1512, cosmologie 1582, cosmologique 1582, cosmologiste 1838, cosmonaute 1934, cristallographie 1772, cryptogamie 1771, cryptogramme 1867 (avant), cryptographe 1845, cryptographie 1624, dermatologie 1832, dermatologique 1845, dermatologiste 1845, dermatologue 1838, diagramme 1584, dialogue 1200 (vers), diamètre 1251 (siecle), diaphonie 1768, diascope 1960, diathermie 1912, digramme 1864, dimorphe 1826, décalitre 1795, décalogue 1401 (siecle), décamètre 1795, décilitre 1795, décimètre 1793, démocrate 1550 (vers), démocratie 1370, démographe 1861, démographie 1855, emmener 1100 (vers), endogamie 1890-1902 (intervalle), endogène 1813, endoscope 1858, endoscopie 1866, ethnarque 1569, ethnique 1201 (siecle), ethnographie 1819, ethnologie 1834, ethnologue 1870, exogamie 1874, exogène 1813, gastralgie 1825, gastrique 1560, gastronomie 1800, gastroscopie 1896, gastrotomie 1611, graphologie 1877, graphologue 1877, graphomètre 1597, gynécocratie 1565, gynécologie 1823, gynécologique 1873, gynécologue 1832, génocide 1948, génotype 1930, géographe 1532, géographie 1513, géologie 1751, géologique 1797, géologue 1797, géomancie 1301 (siecle), géomètre 1301 (siecle), géothermie 1867, hectolitre 1795, hectomètre 1795, hippiatrie 1534, hippodrome 1201 (siecle), hippologie 1858, hippophagie 1832, hiérarchie 1301 (siecle), hiérarchique 1401 (siecle), hiérarque 1551, hiéroglyphe 1546, homographe 1823, homogène 1516, homologie 1822, homologue 1585, homonyme 1534, homonymie 1534, homophone 1822, homophonie 1752, hydrique 1826, hydrographe 1548, hydrographie 1551, hydrogène 1787, hydrologie 1614, hydrologique 1832, hydromètre 1751, hydrophile 1902, hydrophobie 1314, hydroscope 1721, hydrosphère 1897, hydrothérapie 1840, hyperonyme 1970, hyperthermie 1877, hyponyme 1969, hypothermie 1889, hématique 1850, hématologie 1803, hétérogène 1370 (vers), hétérologue 1853, hétéromorphe 1816, hétéronomie 1867, hétéronyme 1893, idéogramme 1859, idéographie 1840, idéologie 1796, idéologique 1801, idéologue 1800 (vers), kilomètre 1795, kilométrique 1811, lexicographe 1578, lexicographie 1757, lexicologie 1748 (avant), lexicologue 1840, lithique 1787, lithographe 1752, lithographie 1649, lithographique 1816, lithologie 1742, lithophage 1694, lithosphère 1897, lithotomie 1612, malmener 1150 (vers), microbiologie 1883, microbiologique 1886, micrographe 1771, micrographie 1665, microlithe 1879, micromètre 1667, microphone 1721, microscope 1656, microscopie 1832, millilitre 1795, millimètre 1792, monarchie 1270 (vers), monarchique 1372-1374 (intervalle), monarchisme 1738, monarchiste 1550 (vers), monarque 1370-1372 (intervalle), monisme 1875, monochrome 1803, monogamie 1526, monogramme 1578, monographie 1800, monolithe 1532, monologue 1500-1503 (intervalle), monomane 1829, monomanie 1814, monophonie 1952, monothéisme 1804, monotype 1800, moralisme 1771, moraliste 1690, myologie 1629, myopathie 1884, mythique 1570, mythographe 1838, mythographie 1845-1846 (intervalle), mythologie 1403, mythologique 1480, mythologue 1546, mythomane 1905, mythomanie 1905, mégalithe 1867, mégaphone 1886, neurologie 1691, neurologique 1893, neurologiste 1888, neurologue 1907, nostalgie 1769, néologie 1759, névralgie 1801, odontalgie 1694, odontologie 1771, oligarchie 1370-1372 (intervalle), oligarque 1562 (vers), ophtalmique 1478, ophtalmologie 1753, ophtalmologique 1808, ophtalmoscope 1854, ophtalmoscopie 1650 (vers), orthographe 1529, orthographie 1245 (vers), orthophonie 1855, ostéologie 1594, otalgie 1578, otologie 1793, palindrome 1765, paléographe 1760, paléographie 1708, panthéisme 1712, pathogène 1878, pathologie 1550, pathologique 1552, philologie 1486, philologue 1534, philosophie 1175 (vers), phonique 1751, phonogramme 1890, phonographe 1844, phonographie 1842, phonolithe 1807, phonologie 1846, phonologique 1846, phonothèque 1929, photogramme 1903, photographe 1836, photographie 1832, photographique 1836, photographiste 1845, photogène 1832, photomètre 1792, photophobie 1812, photothèque 1939, phototype 1895, phototypie 1843, polychrome 1788, polygamie 1558, polygraphe 1536, polygraphie 1553, polymorphe 1805, polyphagie 1803, polyphone 1823, polyphonie 1862, polythéisme 1580, prodrome 1548, prologue 1175 (vers), promener 1365, prototype 1552, pseudonyme 1690, pseudonymie 1818, psychiatre 1802, psychiatrie 1842, psychique 1557, psychisme 1812, psychographie 1834, psychogène 1908, psychologie 1588, psychologique 1780, psychologisme 1840, psychologiste 1801, psychologue 1760, psychopathie 1877, psychothérapie 1888, périmètre 1538, périscope 1899, pétroglyphe 1890, pétrographie 1842, ramener 1101 (siecle), sociogramme 1947, sociologie 1839, sociologique 1839, sociologiste 1839, sociologue 1888, stratosphère 1909, sténographe 1792, sténographie 1771, sténographique 1775, sténotype 1909, sténotypie 1864, stéréographie 1721, stéréophonie 1944, stéréoscope 1841, stéréoscopie 1856, stéréotype 1796, stéréotypie 1802, sympathie 1409, symphonie 1120-1150 (intervalle), syndrome 1537, synonyme 1201 (siecle), synonymie 1582, tachycardie 1871, tachygraphe 1765, tachygraphie 1681, tachymètre 1811, technique 1684-1687 (intervalle), technocrate 1945, technocratie 1934, technologie 1656, technologue 1872, thermique 1847, thermogramme 1906, thermographe 1866, thermogène 1823, thermomètre 1624, thermoscope 1721, thermosphère 1894, thermothérapie 1876, théisme 1745, théiste 1705, théocratie 1679, théogonie 1556, théologie 1240 (vers), théosophie 1710, topique 1372-1374 (intervalle), topographie 1489-1494 (intervalle), topologie 1876, toponyme 1876, toponymie 1869, trichrome 1902, triglyphe 1526-1537 (intervalle), trigramme 1832, trilogie 1765, trimorphe 1823, trimètre 1556, typique 1495, typographe 1554, typographie 1572, typographique 1560, typologie 1841, typomètre 1836, télescope 1614-1636 (intervalle), télégramme 1863, télégraphe 1794, télégraphie 1798, télégraphique 1679, télégraphiste 1836, télémètre 1832, télépathie 1891, téléphone 1809, téléphonie 1836, télétype 1905, tétragramme 1549, tétralogie 1752, tétrarchie 1458, tétrarque 1213, xylographie 1771, xylophage 1803, xylophone 1868, zoologie 1750, zoologique 1754, zoologiste 1734, zoologue 1771, zoomorphe 1821, écologie 1910, écologique 1900, écologiste 1964, économie 1370-1372 (intervalle), économique 1265 (vers), économisme 1775, économiste 1767, écotype 1954, épigramme 1380 (vers), épigraphe 1694, épigraphie 1838, épilogue 1339-1348 (intervalle), épiscope 1955, épitaphe 1160 (vers), éponyme 1755, étymologie 1175 (vers), étymologique 1550 (vers), étymologiste 1578

## 8. Explications détaillées

Règle appliquée le 10 octobre 2026 : une date, un siècle, un lieu d’origine ou une personne n’est gardé que si Wikidata le porte. Les 493 explications des fascicules 5 à 10, rédigées le 10 octobre 2026, suivent cette règle dès l’écriture : elles ne donnent que le sens des éléments, leurs étymons tels que le catalogue les saisit, et le sens du mot ; seuls autisme et monisme citent un emprunt, à l’allemand, que Wikidata déclare (P5191).

- Gardés, portés par Wikidata : téléphone (Bell, 1876 : P61, P575 de Q11035) ; photophone (Bell et Tainter, 1880 : description de Q1302202) ; télescope (vers 1607, Lippershey, Janssen, Metius : P571, P61 de Q4213) ; télégraphe (Chappe, années 1790 : Q3546615, 1791) ; autochrome (frères Lumière : P61 de Q355674) ; anthroposophie (Steiner, début du XXe siècle : P112, P571 de Q184719) ; géographe et géographie (Ératosthène, géographe : P106 de Q43182) ; moraliste (La Bruyère : P106 de Q314975) ; écologie (emprunt à l’allemand Ökologie : P5191 de L20728) ; hydrogène (nommé d’après l’eau : P138 de Q556) ; Micrographia, Hooke 1665 (Q2469465) ; Sociobiology, Wilson 1975 (Q1760020) ; Société théosophique, 1875, Blavatsky (Q327625) ; Oeconomicus, Xénophon (Q2555163).
- Retirés, non portés : datations de mots (télépathie, sociologie, biologie, géologie, télégramme, photographie, technocratie, psychologie, thermomètre, hydrothérapie, archéologie, microphone, photomètre, bibliophile, hydrographie, démographie, xylophone, thermothérapie, psychothérapie, périscope, microscope, photographe, zoonomie, agronomie, astronomie, bibliographie, théocratie, chronomètre, zoologie, étymologie, démocratie) ; attributions (Demisiani, Myers, Moreno, Comte, Lamarck et Treviranus, Suess et Vernadski, Pythagore, Cicéron, Lavoisier, Guillard, Montfaucon, Haeckel, Herschel, Josèphe, La Rochefoucauld) ; dates de l’autochrome (1903, 1907) ; crues du Nil ; tsars ; Moyen Âge (polyphonie) ; fibre optique.
- Reste à revoir : les affirmations linguistiques des explications (« passe par le latin », « d’après télégraphe »…) suivent les relations, elles-mêmes souvent sans appui (§ 5) ; elles n’ont pas été retirées.

## 9. Langues

`referentiels/langues/` : latin, début −800 (P580 de Q397). Grec ancien laissé vide : Wikidata déclare −1500 à −300, en contradiction avec sa propre description (« jusqu’au VIe siècle »). Français, anglais, allemand, italien : aucune période dans Wikidata. Wikidata ne déclare pas la filiation français ← latin ni italien ← latin telle que le catalogue la saisit (`langue_parente_id`). Codes ISO et écritures concordent.

## 10. Reste hors Wikidata

- Translittérations grecques (`grc-savante`) : générées par règle, non vérifiées.
- Gloses, définitions, sens littéraux, explications : rédigés, jamais sourcés (par conception).
- Silhouettes des langues anglais, allemand, italien : à relire sur planche (#15).
- Compilateur : ne publie que le premier maillon des chaînes et aucune analyse historique (#29).
- Contribuer à Wikidata (créer les lexèmes, déclarer P5191, P5238, P6684) est la seule voie pour publier ce qui reste : à décider, rien n’a été écrit dans Wikidata.
