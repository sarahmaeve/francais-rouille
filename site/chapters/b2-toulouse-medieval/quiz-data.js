/* Quiz data — Toulouse Médiévale */
/* Listening comprehension: 8 questions from chapter dialogues. */

const QUIZ_DATA = {

LISTENING: [
    {
        type: "topic",
        audio_src: "audio/01_histoire_cathares/lines/03_isabelle.mp3",
        transcript: "Mais Toulouse \u00e9tait aussi au c\u0153ur d\u2019un mouvement religieux qui allait bouleverser toute la r\u00e9gion : le catharisme. Les Cathares \u2014 qu\u2019on appelait aussi les \u00ab Bonshommes \u00bb ou les \u00ab Parfaits \u00bb \u2014 pr\u00eachaient une forme de christianisme radicalement diff\u00e9rente de celle de Rome.",
        options: [
            "Introducing the Cathar religious movement and its beliefs",
            "Describing the Roman architecture of medieval Toulouse",
            "Explaining the political system of the counts of Toulouse",
            "Listing the main churches built during the Middle Ages"
        ],
        answer: 0
    },
    {
        type: "topic",
        audio_src: "audio/02_basilique_saint_sernin/lines/03_isabelle.mp3",
        transcript: "La construction de l\u2019\u00e9difice actuel a commenc\u00e9 vers 1080 et s\u2019est poursuivie pendant plus de deux si\u00e8cles. L\u2019\u00e9glise a \u00e9t\u00e9 b\u00e2tie pour accueillir les reliques de saint Saturnin et pour recevoir les p\u00e8lerins qui se rendaient \u00e0 Saint-Jacques-de-Compostelle.",
        options: [
            "Describing the interior decorations of a Gothic church",
            "Explaining when and why the basilica was built: for relics and pilgrims to Compostela",
            "Recounting the martyrdom of Saint Saturnin",
            "Comparing two different churches in Toulouse"
        ],
        answer: 1
    },
    {
        type: "topic",
        audio_src: "audio/03_questions_cathares/lines/04_isabelle.mp3",
        transcript: "La croisade des Albigeois a dur\u00e9 vingt ans, de 1209 \u00e0 1229. Mais les pers\u00e9cutions ont continu\u00e9 bien apr\u00e8s. Le premier grand massacre a eu lieu \u00e0 B\u00e9ziers, en 1209, o\u00f9 les crois\u00e9s ont tu\u00e9 des milliers de personnes.",
        options: [
            "Describing a religious ceremony held inside the basilica",
            "Explaining the construction timeline of a medieval fortress",
            "Recounting the duration of the Albigensian Crusade and the massacre at B\u00e9ziers",
            "Discussing the founding of the Dominican order in Toulouse"
        ],
        answer: 2
    },
    {
        type: "topic",
        audio_src: "audio/04_dominicains_jacobins/lines/05_isabelle.mp3",
        transcript: "Mais entrons. La merveille se trouve \u00e0 l\u2019int\u00e9rieur : le fameux \u00ab palmier des Jacobins \u00bb. C\u2019est une colonne unique de vingt-huit m\u00e8tres de haut dont les nervures s\u2019ouvrent en \u00e9ventail au sommet, comme les branches d\u2019un palmier.",
        options: [
            "Describing the relics of Saint Thomas Aquinas",
            "Explaining why the Dominicans chose Toulouse",
            "Describing a famous palm-shaped column inside the church",
            "Recounting the construction challenges of a medieval building"
        ],
        answer: 2
    },
    {
        type: "next",
        audio_src: "audio/01_histoire_cathares/lines/04_isabelle.mp3",
        transcript: "Les Cathares rejetaient la hi\u00e9rarchie de l\u2019\u00c9glise catholique, ses sacrements et sa richesse. Ils vivaient dans la pauvret\u00e9 et la simplicit\u00e9. Leur message s\u00e9duisait beaucoup de gens, y compris des nobles et des bourgeois du Languedoc.",
        options: [
            "La guide explique pourquoi Rome a r\u00e9agi avec violence",
            "La guide d\u00e9crit les costumes des p\u00e8lerins m\u00e9di\u00e9vaux",
            "Un touriste demande o\u00f9 se trouve le restaurant le plus proche",
            "La guide recommande un livre sur l\u2019architecture gothique"
        ],
        answer: 0
    },
    {
        type: "next",
        audio_src: "audio/03_questions_cathares/lines/01_thomas.mp3",
        transcript: "Excusez-moi, Isabelle. Vous avez dit que les Cathares rejetaient l\u2019\u00c9glise catholique. Mais est-ce qu\u2019ils se consid\u00e9raient quand m\u00eame comme chr\u00e9tiens ?",
        options: [
            "La guide confirme et d\u00e9crit leur sacrement, le consolamentum",
            "La guide change de sujet et parle de l\u2019architecture",
            "Thomas d\u00e9cide de visiter une autre ville",
            "La guide avoue qu\u2019elle ne conna\u00eet pas la r\u00e9ponse"
        ],
        answer: 0
    },
    {
        type: "next",
        audio_src: "audio/03_questions_cathares/lines/05_thomas.mp3",
        transcript: "C\u2019est effroyable. Et Toulouse, elle a \u00e9t\u00e9 attaqu\u00e9e aussi ?",
        options: [
            "La guide raconte les si\u00e8ges de Toulouse et la mort de Simon de Montfort",
            "La guide r\u00e9pond que Toulouse n\u2019a jamais \u00e9t\u00e9 touch\u00e9e par la croisade",
            "Thomas propose de visiter le ch\u00e2teau de Carcassonne",
            "La guide sugg\u00e8re de faire une pause caf\u00e9"
        ],
        answer: 0
    },
    {
        type: "next",
        audio_src: "audio/04_dominicains_jacobins/lines/02_isabelle.mp3",
        transcript: "En 1215, un pr\u00eatre castillan nomm\u00e9 Dominique de Guzm\u00e1n est arriv\u00e9 \u00e0 Toulouse avec une mission : combattre l\u2019h\u00e9r\u00e9sie cathare, non pas par les armes, mais par la parole et par l\u2019exemple.",
        options: [
            "La guide explique qu\u2019il a fond\u00e9 un ordre de fr\u00e8res pr\u00eacheurs",
            "La guide raconte que Dominique a quitt\u00e9 Toulouse imm\u00e9diatement",
            "Un touriste demande si on peut entrer dans l\u2019\u00e9glise",
            "La guide d\u00e9crit les vitraux de la cath\u00e9drale"
        ],
        answer: 0
    }
]

};
