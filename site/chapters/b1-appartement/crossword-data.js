/*
    Grid layout (14 x 14):

         0  1  2  3  4  5  6  7  8  9 10 11 12 13
    0    .  .  .  .  .  L  O  Y  E  R  .  .  .  .
    1    F  T  .  .  .  O  .  .  .  .  .  .  .  .
    2    I  U  .  B  .  C  H  A  U  D  I  E  R  E
    3    B  Y  .  A  .  A  .  .  .  .  .  .  .  .
    4    R  A  D  I  A  T  E  U  R  .  .  .  .  .
    5    E  U  .  L  .  A  .  .  .  L  .  .  .  .
    6    .  .  .  .  .  I  .  .  .  A  .  .  .  .
    7    .  .  .  .  .  R  .  .  .  I  .  .  .  .
    8    .  .  .  .  P  E  L  O  U  S  E  .  .  .
    9    .  .  .  .  A  .  .  .  .  S  .  .  .  .
   10    .  .  .  .  L  .  .  .  .  E  .  .  .  .
   11    .  .  .  .  I  .  .  .  .  .  .  .  .  .
   12    .  .  .  .  E  .  .  .  .  .  .  .  .  .
   13    .  .  .  .  R  .  .  .  .  .  .  .  .  .
*/

var PUZZLE = {
    rows: 14,
    cols: 14,
    words: [
        // Across
        {
            num: 0, dir: "across", row: 0, col: 5,
            answer: "LOYER",
            clue: "Il monte chaque année, mais personne ne le prend pour l'ascenseur"
        },
        {
            num: 0, dir: "across", row: 2, col: 5,
            answer: "CHAUDIERE",
            clue: "Elle fait bouillir l'eau sans jamais préparer de thé"
        },
        {
            num: 0, dir: "across", row: 4, col: 0,
            answer: "RADIATEUR",
            clue: "Froid quand il tombe en panne, brûlant quand il se réveille"
        },
        {
            num: 0, dir: "across", row: 8, col: 4,
            answer: "PELOUSE",
            clue: "Verte et interdite : le syndic vous surveille si vous marchez dessus"
        },
        // Down
        {
            num: 0, dir: "down", row: 0, col: 5,
            answer: "LOCATAIRE",
            clue: "Il paye pour habiter chez quelqu'un d'autre — c'est la vie !"
        },
        {
            num: 0, dir: "down", row: 1, col: 0,
            answer: "FIBRE",
            clue: "Optique et rapide, tout l'immeuble la veut"
        },
        {
            num: 0, dir: "down", row: 1, col: 1,
            answer: "TUYAU",
            clue: "Il transporte l'eau et aussi les bons conseils"
        },
        {
            num: 0, dir: "down", row: 2, col: 3,
            answer: "BAIL",
            clue: "Quatre lettres qui vous attachent à un appartement pour des années"
        },
        {
            num: 0, dir: "down", row: 5, col: 9,
            answer: "LAISSE",
            clue: "Le chien n'en veut pas, le syndic en exige une"
        },
        {
            num: 0, dir: "down", row: 8, col: 4,
            answer: "PALIER",
            clue: "Vous y croisez vos voisins entre deux étages"
        }
    ]
};
