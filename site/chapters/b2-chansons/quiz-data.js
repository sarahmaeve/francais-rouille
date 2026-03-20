/* Quiz data — Chansons Populaires */
/* Listening comprehension: 8 questions from chapter dialogues. */

const QUIZ_DATA = {

LISTENING: [
    {
        type: "topic",
        audio_src: "audio/02_chansons_indila/lines/06_yasmine.mp3",
        transcript: "Rafra\u00eechissant, peut-\u00eatre, mais frustrant aussi. Parce que \u00ab Mini World \u00bb, l\u2019album, il y avait des p\u00e9pites incroyables. \u00ab S.O.S. \u00bb est magnifique, \u00ab Tourner dans le vide \u00bb aussi. Et on attend la suite depuis une \u00e9ternit\u00e9.",
        options: [
            "Complaining that a concert was cancelled at the last minute",
            "Expressing frustration about a talented artist who released great music but then went silent",
            "Recommending a new album that just came out",
            "Criticizing an artist for releasing too many albums"
        ],
        answer: 1
    },
    {
        type: "topic",
        audio_src: "audio/03_films_mousquetaires/lines/07_irene.mp3",
        transcript: "Ah \u00e7a oui, les chor\u00e9graphies de combat sont impressionnantes. C\u2019est ce qui m\u2019a emp\u00each\u00e9e de d\u00e9crocher compl\u00e8tement. Mais entre les sc\u00e8nes d\u2019action, il y a des longueurs.",
        options: [
            "Explaining why she stopped watching a TV series",
            "Praising a film\u2019s action scenes while criticizing its slow pacing",
            "Describing the plot of a new action movie",
            "Comparing two different martial arts styles"
        ],
        answer: 1
    },
    {
        type: "topic",
        audio_src: "audio/04_album_zaz/lines/04_rebecca.mp3",
        transcript: "Ce n\u2019est pas que \u00e7a ne m\u2019a pas plu. C\u2019est juste que\u2026 ce n\u2019est plus la Zaz que j\u2019aimais. Le premier album, celui avec \u00ab Je veux \u00bb, c\u2019\u00e9tait brut, c\u2019\u00e9tait de la chanson de rue avec une \u00e9nergie folle.",
        options: [
            "Recommending a friend listen to a new artist",
            "Complaining about the price of concert tickets",
            "Explaining that an artist has changed too much from her raw, energetic beginnings",
            "Praising an album for its innovative new sound"
        ],
        answer: 2
    },
    {
        type: "topic",
        audio_src: "audio/02_chansons_indila/lines/12_hugo.mp3",
        transcript: "Eh, ce n\u2019est pas ce que je dis ! Je dis juste que quand la production est trop propre, \u00e7a enl\u00e8ve l\u2019\u00e9motion. Regarde les vieux enregistrements d\u2019Oum Kalthoum \u2014 c\u2019est brut, c\u2019est imparfait, et c\u2019est justement pour \u00e7a que c\u2019est bouleversant.",
        options: [
            "Defending a friend who was criticized for their musical taste",
            "Explaining how to set up a home recording studio",
            "Arguing that overly polished music production removes emotional authenticity",
            "Describing the history of Egyptian popular music"
        ],
        answer: 2
    },
    {
        type: "next",
        audio_src: "audio/03_films_mousquetaires/lines/08_maeve.mp3",
        transcript: "C\u2019est vrai que c\u2019est dense. Mais moi, j\u2019avais lu le r\u00e9sum\u00e9 avant, donc j\u2019ai pu suivre. Et puis, il faut parler d\u2019Eva Green. Elle est incroyable en Milady. C\u2019est la meilleure chose dans les deux films.",
        options: [
            "L\u2019autre personne exprime son accord total sur le talent d\u2019Eva Green",
            "Maeve propose d\u2019aller voir un autre film ce soir",
            "Irene change de sujet pour parler d\u2019un concert",
            "Maeve reconna\u00eet que le film \u00e9tait d\u00e9cevant"
        ],
        answer: 0
    },
    {
        type: "next",
        audio_src: "audio/04_album_zaz/lines/11_matthieu.mp3",
        transcript: "C\u2019est le premier album qu\u2019elle sort chez t\u00f4t ou tard. Peut-\u00eatre que le changement de label a influenc\u00e9 la direction artistique. Mais moi, je vois \u00e7a comme une \u00e9volution naturelle. Brel aussi a commenc\u00e9 dans les cabarets et il a fini par faire des albums orchestraux.",
        options: [
            "Rebecca r\u00e9agit en trouvant la comparaison avec Brel exag\u00e9r\u00e9e",
            "Matthieu s\u2019excuse d\u2019avoir \u00e9t\u00e9 trop critique",
            "Rebecca propose d\u2019\u00e9couter du Brel \u00e0 la place",
            "Matthieu avoue qu\u2019il n\u2019aime plus vraiment Zaz"
        ],
        answer: 0
    },
    {
        type: "next",
        audio_src: "audio/02_chansons_indila/lines/09_yasmine.mp3",
        transcript: "C\u2019est un peu le pi\u00e8ge du tube, \u00e7a. Quand tu as un titre qui explose comme \u00e7a, tout le monde juge l\u2019artiste sur cette seule chanson. Moi, ce que j\u2019aime chez Indila, c\u2019est son m\u00e9lange de cultures musicales.",
        options: [
            "Hugo admet que le m\u00e9lange culturel est int\u00e9ressant mais critique la production",
            "L\u00e9a annonce qu\u2019elle va arr\u00eater d\u2019\u00e9couter Indila",
            "Yasmine met la chanson pour que tout le monde l\u2019\u00e9coute",
            "Hugo propose d\u2019aller \u00e0 un concert d\u2019Indila"
        ],
        answer: 0
    },
    {
        type: "next",
        audio_src: "audio/03_films_mousquetaires/lines/12_maeve.mp3",
        transcript: "Oui, c\u2019est \u00e7a. Et la fin laisse la porte ouverte pour un troisi\u00e8me film. Tu crois qu\u2019ils vont le faire ?",
        options: [
            "Irene exprime des doutes \u00e0 cause des r\u00e9sultats mitig\u00e9s au box-office",
            "Maeve r\u00e9pond qu\u2019elle a d\u00e9j\u00e0 achet\u00e9 des billets",
            "Irene propose de lire le roman de Dumas \u00e0 la place",
            "Maeve annonce qu\u2019Eva Green a quitt\u00e9 le projet"
        ],
        answer: 0
    }
]

};
