/* Quiz data — Textes B1: La Vie Quotidienne */
/* Sentences drawn from the chapter's dialogues and texts. */

const QUIZ_DATA = {

FILL_IN_BLANK: [
    // 01 — Métro
    {
        section: "01",
        sentence_fr: "Vous devez prendre la ___ 12 en direction de Front Populaire.",
        answer: "ligne",
        hint: "the (metro) line"
    },
    {
        section: "01",
        sentence_fr: "Votre ticket est ___ pour tout le trajet.",
        answer: "valable",
        hint: "valid"
    },
    // 02 — Viennoiserie
    {
        section: "02",
        sentence_fr: "Ils sont tout chauds, ils viennent de sortir du ___.",
        answer: "four",
        hint: "the oven"
    },
    {
        section: "02",
        sentence_fr: "___ fait combien ?",
        answer: "Ça",
        hint: "How much does that come to?"
    },
    // 03 — Directions Toulouse
    {
        section: "03",
        sentence_fr: "C'est une rue ___ très connue.",
        answer: "commerçante",
        hint: "a shopping street"
    },
    {
        section: "03",
        sentence_fr: "Vous allez ___ sur une grande place.",
        answer: "déboucher",
        hint: "to come out onto / to open onto"
    },
    // 05 — Appel agence
    {
        section: "05",
        sentence_fr: "Il est loué ___. Par contre, la cuisine est entièrement équipée.",
        answer: "vide",
        hint: "unfurnished (lit. empty)"
    },
    {
        section: "05",
        sentence_fr: "Vos ___ doivent être au moins trois fois le montant du loyer.",
        answer: "revenus",
        hint: "income"
    },
    // 06 — Discussion quartier
    {
        section: "06",
        sentence_fr: "Y'a une ambiance de ___ ! C'est vivant.",
        answer: "ouf",
        hint: "amazing (verlan for 'fou')"
    },
    {
        section: "06",
        sentence_fr: "Au-dessus de la rue de Charonne, c'est ___.",
        answer: "peinard",
        hint: "peaceful / chill (informal)"
    },
    // 07 — Boulangerie
    {
        section: "07",
        sentence_fr: "Notre spécialité maison, c'est le ___.",
        answer: "Paris-Noisette",
        hint: "a house specialty with hazelnuts"
    },
    {
        section: "07",
        sentence_fr: "On le saupoudre de noisettes ___ et d'un peu de fleur de sel.",
        answer: "torréfiées",
        hint: "toasted / roasted"
    },
    // 08 — Taxi hôtel
    {
        section: "08",
        sentence_fr: "Il vaut mieux prévoir de la ___.",
        answer: "marge",
        hint: "to allow extra time"
    },
    {
        section: "08",
        sentence_fr: "C'est un tarif ___, tous les taxis parisiens officiels doivent l'appliquer.",
        answer: "réglementé",
        hint: "regulated"
    },
    // 10 — Rentrée CM2
    {
        section: "10",
        sentence_fr: "Elle avait peur d'avoir un maître plus ___ que l'année dernière.",
        answer: "sévère",
        hint: "strict"
    },
    {
        section: "10",
        sentence_fr: "Les fractions et les problèmes de ___, c'est nouveau pour lui.",
        answer: "géométrie",
        hint: "geometry"
    }
],

LISTENING: [
    {
        type: "topic",
        audio_src: "audio/01_paris_metro/lines/04_marc.mp3",
        transcript: "Oui, il faut changer à la station Marcadet-Poissonniers. C\u2019est à trois arrêts d\u2019ici.",
        options: [
            "Giving directions for changing metro lines",
            "Explaining how to buy a metro ticket",
            "Describing the history of the Paris metro",
            "Warning about pickpockets on public transport"
        ],
        answer: 0
    },
    {
        type: "topic",
        audio_src: "audio/05_appel_agence/lines/09_thomas.mp3",
        transcript: "C\u2019est vrai que le bas de la rue est animé avec les bars et les restaurants. Mais l\u2019appartement est plus haut sur la rue, c\u2019est beaucoup plus calme. Et puis la chambre donne sur la cour, donc vous n\u2019entendez presque rien.",
        options: [
            "Describing what shops are available in the neighborhood",
            "Explaining the apartment\u2019s layout and number of rooms",
            "Reassuring a tenant that the apartment is quiet despite a lively street",
            "Recommending nearby bars and restaurants"
        ],
        answer: 2
    },
    {
        type: "topic",
        audio_src: "audio/07_boulangerie/lines/06_monsieur_duval.mp3",
        transcript: "C\u2019est une pâte feuilletée croustillante avec une mousse à la noisette du Piémont et un cœur de caramel au beurre salé. On le saupoudre de noisettes torréfiées et d\u2019un peu de fleur de sel.",
        options: [
            "Listing the prices on the bakery menu",
            "Describing the ingredients of a specialty pastry",
            "Explaining how to make croissants at home",
            "Recommending a chocolate dessert from Belgium"
        ],
        answer: 1
    },
    {
        type: "topic",
        audio_src: "audio/08_taxi_hotel/lines/06_emilie.mp3",
        transcript: "Depuis notre quartier, près de la Gare du Nord, il faut compter entre trente et quarante-cinq minutes pour l\u2019aéroport. Ça dépend du terminal.",
        options: [
            "Giving the fixed fare for a taxi to the airport",
            "Explaining how to take the train to the airport",
            "Estimating travel time from a hotel to the airport",
            "Describing the location of the hotel near Gare du Nord"
        ],
        answer: 2
    },
    {
        type: "next",
        audio_src: "audio/02_viennoiserie/lines/03_antoine.mp3",
        transcript: "Les pains au chocolat viennent de sortir du four, ils sont encore tout chauds. Et nos chaussons aux pommes sont très bons aussi.",
        options: [
            "La cliente passe sa commande",
            "Le boulanger ferme la boutique",
            "La cliente se plaint du prix",
            "Le boulanger s\u2019excuse de ne plus avoir de croissants"
        ],
        answer: 0
    },
    {
        type: "next",
        audio_src: "audio/03_directions_toulouse/lines/01_nadia.mp3",
        transcript: "Pardon, monsieur, je suis un peu perdue. Est-ce que vous connaissez la Place du Capitole ?",
        options: [
            "L\u2019habitant confirme et demande si elle est à pied",
            "L\u2019habitant dit qu\u2019il ne connaît pas la ville",
            "Nadia décide de prendre un taxi",
            "L\u2019habitant lui vend un plan de la ville"
        ],
        answer: 0
    },
    {
        type: "next",
        audio_src: "audio/06_discussion_quartier/lines/06_yasmine.mp3",
        transcript: "Ah carrément ! Y\u2019a le marché de la Bastille le jeudi et le dimanche, il est génial. Les fruits et légumes sont super frais, et y\u2019a un fromager là-bas, il est incroyable.",
        options: [
            "Camille pose des questions sur les restaurants du coin",
            "Yasmine propose de déménager dans un autre quartier",
            "Camille décide que le quartier est trop cher",
            "Yasmine se plaint du bruit du marché"
        ],
        answer: 0
    },
    {
        type: "next",
        audio_src: "audio/10_rentree_cm2/lines/05_olivier.mp3",
        transcript: "Chloé aussi commence à s\u2019adapter. Ce qui l\u2019a surprise, c\u2019est la quantité de devoirs. En CM1, elle avait une demi-heure de travail le soir, mais maintenant c\u2019est presque une heure.",
        options: [
            "Un autre parent partage un problème scolaire similaire",
            "Olivier annonce que sa fille change d\u2019école",
            "Les parents décident d\u2019annuler les devoirs",
            "Chloé arrive et demande de rentrer à la maison"
        ],
        answer: 0
    }
],

DRAG_DROP: [
    // 01 — Métro
    {
        section: "01",
        sentence_fr: "Vous allez voir des ___ orange avec le numéro de la ligne.",
        options: ["panneaux", "flèches", "couloirs", "tickets"],
        answer: "panneaux"
    },
    {
        section: "01",
        sentence_fr: "Le métro parisien est très ___, on s'y habitue vite.",
        options: ["fiable", "valable", "agréable", "disponible"],
        answer: "fiable"
    },
    // 02 — Viennoiserie
    {
        section: "02",
        sentence_fr: "C'est pour ___ ou pour manger ici ?",
        options: ["emporter", "partir", "sortir", "prendre"],
        answer: "emporter"
    },
    // 03 — Directions
    {
        section: "03",
        sentence_fr: "Continuez tout ___ jusqu'au bout de la rue.",
        options: ["droit", "seul", "près", "vite"],
        answer: "droit"
    },
    {
        section: "03",
        sentence_fr: "C'est une rue commerçante très connue, vous ne pouvez pas la ___.",
        options: ["rater", "trouver", "chercher", "manquer"],
        answer: "rater"
    },
    // 05 — Appel agence
    {
        section: "05",
        sentence_fr: "La chambre donne sur la ___, donc vous n'entendez presque rien.",
        options: ["cour", "rue", "place", "cuisine"],
        answer: "cour"
    },
    {
        section: "05",
        sentence_fr: "Il me faudrait votre ___ de travail et vos trois derniers bulletins de salaire.",
        options: ["contrat", "dossier", "avis", "relevé"],
        answer: "contrat"
    },
    // 06 — Discussion quartier
    {
        section: "06",
        sentence_fr: "Y'a pas mal de petits restos ___ sans se ruiner.",
        options: ["sympa", "chouette", "génial", "nickel"],
        answer: "sympa"
    },
    {
        section: "06",
        sentence_fr: "Tu vas te plaire ici, c'est ___ !",
        options: ["carrément", "franchement", "grave", "hyper"],
        answer: "carrément"
    },
    // 07 — Boulangerie
    {
        section: "07",
        sentence_fr: "Votre ___ est magnifique. Tout a l'air délicieux.",
        options: ["vitrine", "boutique", "cuisine", "recette"],
        answer: "vitrine"
    },
    {
        section: "07",
        sentence_fr: "On utilise du chocolat noir à soixante-douze pour cent, importé d'un artisan ___.",
        options: ["belge", "français", "suisse", "italien"],
        answer: "belge"
    },
    // 08 — Taxi
    {
        section: "08",
        sentence_fr: "Le chauffeur viendra vous ___ devant l'entrée principale.",
        options: ["chercher", "attendre", "trouver", "appeler"],
        answer: "chercher"
    },
    {
        section: "08",
        sentence_fr: "Le pourboire n'est pas ___, mais c'est toujours apprécié.",
        options: ["obligatoire", "interdit", "recommandé", "réglementé"],
        answer: "obligatoire"
    },
    // 10 — Rentrée CM2
    {
        section: "10",
        sentence_fr: "Le ___ simple, le subjonctif... Il trouve ça très difficile.",
        options: ["passé", "présent", "futur", "premier"],
        answer: "passé"
    }
]

};
