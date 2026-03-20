/* Quiz data — La Vie en Appartement */
/* Sentences drawn from the chapter's dialogues and texts. */

const QUIZ_DATA = {

FILL_IN_BLANK: [
    {
        section: "01",
        sentence_fr: "Je me ___ de vous contacter au sujet d'un problème urgent.",
        answer: "permets",
        hint: "I take the liberty of (polite)"
    },
    {
        section: "02",
        sentence_fr: "La ___ trois voies est bloquée.",
        answer: "vanne",
        hint: "a valve"
    },
    {
        section: "02",
        sentence_fr: "Faut que je ___ tout le système et que je remplace la vanne.",
        answer: "purge",
        hint: "to bleed (a radiator/system)"
    },
    {
        section: "02",
        sentence_fr: "C'est un problème de ___, c'est pas de votre faute.",
        answer: "vétusté",
        hint: "wear and tear, obsolescence"
    },
    {
        section: "03",
        sentence_fr: "Je viens pour un courrier ___ que j'attends depuis trois semaines.",
        answer: "recommandé",
        hint: "registered (letter)"
    },
    {
        section: "03",
        sentence_fr: "Il faudrait faire une ___ en ligne sur le site de La Poste.",
        answer: "réclamation",
        hint: "a complaint, a claim"
    },
    {
        section: "05",
        sentence_fr: "Les chats adorent se ___ là-dedans.",
        answer: "planquer",
        hint: "to hide (informal)"
    },
    {
        section: "05",
        sentence_fr: "Mettez sa ___ sur le balcon.",
        answer: "litière",
        hint: "litter (box)"
    },
    {
        section: "05",
        sentence_fr: "Il revenait comme si de rien n'___.",
        answer: "était",
        hint: "as if nothing had happened"
    },
    {
        section: "06",
        sentence_fr: "Le câble doit passer par la colonne ___ de l'immeuble.",
        answer: "montante",
        hint: "the riser (vertical cable shaft)"
    },
    {
        section: "06",
        sentence_fr: "Il doit utiliser les chemins de ___ existants.",
        answer: "câble",
        hint: "cable runs/trays"
    },
    {
        section: "06",
        sentence_fr: "Il me faut la lettre d'Orange avec le plan d'installation, et une copie de votre ___.",
        answer: "bail",
        hint: "a lease"
    }
],

LISTENING: [
    /* Type "topic": listen, then pick the English description of what the speaker is saying */
    {
        type: "topic",
        audio_src: "audio/02_reparation_chauffage/lines/05_bruno.mp3",
        transcript: "Ah ouais, voilà. Bon alors, le problème c\u2019est que la vanne trois voies est bloquée, et en plus y\u2019a de l\u2019air dans le circuit.",
        options: [
            "Diagnosing a broken heating valve and air in the pipes",
            "Describing how to install a new boiler",
            "Complaining about the building\u2019s electricity",
            "Asking the tenant for permission to begin work"
        ],
        answer: 0
    },
    {
        type: "topic",
        audio_src: "audio/03_poste_courrier/lines/07_maeve.mp3",
        transcript: "Mais trois semaines, c\u2019est quand même très long, non ? Ce sont des documents importants. J\u2019en ai besoin pour une démarche administrative.",
        options: [
            "Asking for directions to the nearest post office",
            "Expressing frustration about a delayed package with important documents",
            "Requesting a refund for a damaged parcel",
            "Explaining why she needs to change her address"
        ],
        answer: 1
    },
    {
        type: "topic",
        audio_src: "audio/05_chat_perdu/lines/07_karim.mp3",
        transcript: "Vous avez regardé dans les traboules ? Y\u2019en a une juste à côté de l\u2019immeuble. Les chats adorent se planquer là-dedans.",
        options: [
            "Warning the neighbors about a dangerous animal",
            "Describing where he last saw a lost dog",
            "Suggesting hidden passageways as a place to look for a lost cat",
            "Recommending a veterinarian in the neighborhood"
        ],
        answer: 2
    },
    {
        type: "topic",
        audio_src: "audio/06_fibre_syndic/lines/04_monsieur_reynaud.mp3",
        transcript: "Oui, en effet, c\u2019est la procédure. Le câble doit passer par la colonne montante de l\u2019immeuble, et toute intervention dans les parties communes nécessite un accord.",
        options: [
            "Refusing a tenant\u2019s request to install internet",
            "Explaining that cable work in shared areas requires building approval",
            "Describing the cost of a fiber optic subscription",
            "Asking about the tenant\u2019s current internet provider"
        ],
        answer: 1
    },
    /* Type "next": listen, then pick (in French) what might logically come next */
    {
        type: "next",
        audio_src: "audio/02_reparation_chauffage/lines/06_maeve.mp3",
        transcript: "Pardon, je n\u2019ai pas bien compris. La vanne… trois voies ? Et purger, ça veut dire quoi exactement ?",
        options: [
            "Le technicien réexplique avec des mots plus simples",
            "La locataire appelle le propriétaire pour se plaindre",
            "Le technicien part chercher un autre collègue",
            "La locataire propose de faire la réparation elle-même"
        ],
        answer: 0
    },
    {
        type: "next",
        audio_src: "audio/03_poste_courrier/lines/03_maeve.mp3",
        transcript: "Oui, le voici. C\u2019est un envoi recommandé avec accusé de réception.",
        options: [
            "Le guichetier recherche le colis dans le système",
            "Le guichetier demande de revenir un autre jour",
            "Maeve décide de partir sans attendre",
            "Le guichetier propose d\u2019acheter un timbre"
        ],
        answer: 0
    },
    {
        type: "next",
        audio_src: "audio/05_chat_perdu/lines/01_maeve.mp3",
        transcript: "Hélène ! Karim ! Excusez-moi de vous déranger, mais est-ce que vous avez vu un chat gris ce matin ? Le nôtre a disparu.",
        options: [
            "Ils appellent la police pour signaler la disparition",
            "Une voisine demande des détails sur l\u2019apparence du chat",
            "Ils refusent de l\u2019aider et rentrent chez eux",
            "Maeve décide d\u2019adopter un nouveau chat"
        ],
        answer: 1
    },
    {
        type: "next",
        audio_src: "audio/06_fibre_syndic/lines/09_irene.mp3",
        transcript: "Quinze jours ? C\u2019est le délai habituel ?",
        options: [
            "Il confirme et explique comment accélérer les choses",
            "Il annonce que la fibre est interdite dans l\u2019immeuble",
            "Irene décide de changer d\u2019opérateur",
            "Il demande à Irene de déménager au rez-de-chaussée"
        ],
        answer: 0
    }
],

DRAG_DROP: [
    {
        section: "01",
        sentence_fr: "Les radiateurs restent ___ dans toutes les pièces.",
        options: ["froids", "chauds", "ouverts", "cassés"],
        answer: "froids"
    },
    {
        section: "02",
        sentence_fr: "Bon, je vais jeter un ___ à la chaudière.",
        options: ["œil", "coup", "regard", "mot"],
        answer: "œil"
    },
    {
        section: "02",
        sentence_fr: "Quand y'a de l'air dans les radiateurs, l'eau chaude peut pas ___ correctement.",
        options: ["circuler", "chauffer", "monter", "couler"],
        answer: "circuler"
    },
    {
        section: "03",
        sentence_fr: "Vous avez le numéro de ___ ?",
        options: ["suivi", "courrier", "compte", "dossier"],
        answer: "suivi"
    },
    {
        section: "03",
        sentence_fr: "Ils répondent sous dix jours ___.",
        options: ["ouvrés", "complets", "entiers", "normaux"],
        answer: "ouvrés"
    },
    {
        section: "04",
        sentence_fr: "Les chiens doivent être tenus en ___ dans les parties communes.",
        options: ["laisse", "cage", "main", "ordre"],
        answer: "laisse"
    },
    {
        section: "05",
        sentence_fr: "Vous avez regardé dans les ___ ?",
        options: ["traboules", "escaliers", "couloirs", "jardins"],
        answer: "traboules"
    },
    {
        section: "05",
        sentence_fr: "Les gens sont super ___ dessus.",
        options: ["réactifs", "contents", "gentils", "rapides"],
        answer: "réactifs"
    },
    {
        section: "06",
        sentence_fr: "Les copropriétaires ne peuvent pas s'opposer à l'installation de la ___.",
        options: ["fibre", "chaudière", "colonne", "vanne"],
        answer: "fibre"
    },
    {
        section: "06",
        sentence_fr: "Prévenez vos voisins du troisième le jour de l'installation, c'est une question de ___.",
        options: ["courtoisie", "sécurité", "règlement", "prudence"],
        answer: "courtoisie"
    }
]

};
