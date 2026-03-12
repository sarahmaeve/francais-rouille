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
