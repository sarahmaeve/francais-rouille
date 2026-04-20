# Recurring Characters

Canonical reference for characters who appear in more than one chapter.
Edit this file whenever a character changes — it is the source of
truth the writer and the audio pipeline are supposed to agree with.

When a new chapter introduces a brand-new character, leave them out of
this file. Add them only when they recur in a second chapter, or when
they're meant to recur (the narrator / guide character types below).

## Why this file exists

Two recurring problems this file is here to prevent:

1. **Nationality drift.** Maeve was "irlandaise" in `b1-appartement`
   and "américaine" in `b2-chansons/03_films_mousquetaires` for a long
   time. Reconciled to American in 2026 after the tension around
   `b1-appartement/03_poste_courrier` (a package coming from the US)
   made the Irish framing incoherent.
2. **Name collisions.** "Camille" is the name of at least three
   distinct characters across the repo. The voice pipeline (step-2
   content-addressed audio) assigns voices by character name, so
   silent collisions would give three unrelated people the same voice.
   Distinguishing suffixes and an explicit `voices.toml` override are
   how we resolve this.

---

## Maeve Carrick

- **Nationality:** American
- **Age / life stage:** late 20s / early 30s, early professional career
- **Domain:** designer — worked in design roles; joins Atelier Clairac
  (Nantes) as a designer in `b1-travail`. Earlier Lyon life in
  `b1-appartement` frames her as a tenant without specifying her work.
- **Partner:** Irene (see below). They are preparing a PACS in
  `b1-paperasse/04_mairie_acte_naissance`.
- **Family:** mother and grandmother in the US. Grandmother turns 90
  on 1 November in the `b1-travail` RTT email.
- **Register:** careful polite French with strangers, casual with
  friends; accepts "tu" easily in a laid-back workplace culture.
- **Location arc:**
  - **Lyon (Croix-Rousse) — earlier timeline.** `b1-appartement`
    (heating, post office, lost cat, fibre syndic), `b1-messagerie/
    01_rappel_medecin` (Lyon cabinet médical), `b2-chansons/
    03_films_mousquetaires` (discussing films with Irene).
  - **Nantes — later timeline.** `b1-travail` (new job at Atelier
    Clairac, Ludo asks about her trip from Lyon), `b1-paperasse`
    (préfecture de la Loire-Atlantique, mairie de Nantes, PACS
    preparation).
- **Recurring voice (recommendation for step-2 `voices.toml`):**
  `fr-FR-Studio-A` (warm, clear, handles liaisons well).

## Irene

- **Nationality:** American
- **Age / life stage:** close to Maeve's; consistently framed as a
  student or young adult.
- **Domain:** cinephile, film/cultural interests. In `b2-chansons/
  03_films_mousquetaires` she's "plus critique sur le cinéma
  français"; in `b2-librairie-occitane` she follows the culture and
  history of Occitania with genuine curiosity.
- **Partner:** Maeve (see above). No surname given in any chapter —
  leave ambiguous.
- **Location arc:**
  - Shares the Lyon flat with Maeve in `b1-appartement` (05, 06).
  - Travels to Toulouse as a tourist in `b2-librairie-occitane` (02,
    03, 04).
  - Lives with Maeve in Nantes in `b1-paperasse/04` (PACS together).
  - Receives a voicemail from her Paris friend Camille in
    `b1-messagerie/03_camille_annule`.
- **Register:** sustains a B2-level register in cultural discussions;
  informal "tu" with friends.
- **Recurring voice (recommendation):** a different voice from Maeve,
  so the learner can tell them apart on joint lines. A candidate from
  the Chirp3-HD female pool, e.g. `fr-FR-Chirp3-HD-Kore`.

## Bruno (Lyon plumber/chauffagiste)

- **Role:** independent plumber / heating technician working in Lyon.
  Casual, direct, explains things simply.
- **Appearances:**
  - `b1-appartement/02_reparation_chauffage` (fixes Maeve's heating —
    vanne trois voies, purging the system).
  - `b1-messagerie/04_plombier_bruno` (voicemail confirming a leak
    repair visit in the Croix-Rousse area).
- **Register:** highly informal — "ouais", "du coup", dropping
  negations, approximations ("dans les une heure"). A good contrast
  with the polite register of the admin scenes.
- **Recurring voice:** a distinct male fallback voice — any of the
  Chirp3-HD male pool works (e.g. `fr-FR-Chirp3-HD-Enceladus`).

## Camilles — disambiguation

There are **three** unrelated characters named Camille. This is the
single biggest voice-collision risk in the repo.

### 1. Camille (Paris 11e — the flat-hunter)

- Appears in `b1-vie-quotidienne/05_appel_agence` and
  `06_discussion_quartier` as the future tenant looking for an
  apartment in Paris 11e.
- Same person as in `b1-messagerie/03_camille_annule`: now an
  Irene's friend in Paris, leaves a casual voicemail cancelling a
  dinner.
- **Register:** informal, warm, Parisian ("coucou ma belle", "gros
  bisous").
- **Voice candidate:** a Chirp3-HD female, distinct from Maeve and
  Irene.
- **Handle in voices.toml:** `"Camille"` — the default Camille if no
  surname is given.

### 2. Camille Perret (Nantes designer)

- Appears in `b1-travail/04_dejeuner_cantine` and `05_reunion_lundi`
  as a senior designer at Atelier Clairac, direct and funny, 35.
- Always referred to with the surname in the character block:
  "Camille Perret". In dialogue, usually just "Camille".
- **Register:** workplace informal. French "tu" with colleagues,
  dry humour.
- **Voice candidate:** distinct from the Paris Camille. A Neural2
  voice would contrast well (e.g. `fr-FR-Neural2-F`).
- **Handle in voices.toml:** `"Camille Perret"` — always include the
  surname so this Camille resolves separately.

### 3. Camille (referenced only)

- Named in `b1-travail/02_message_vocal_camille` as the recipient of
  Maeve's voicemail — she is the same person as Camille Perret above,
  since the scene is inside Atelier Clairac.
- Not a separate character — just a cross-reference.

**Writing rule:** when introducing a character named Camille in a new
chapter, always check this file first and either (a) attach them to
an existing Camille via surname or context, or (b) pick a different
first name. Do not silently add a fourth Camille.

## Paul & Alice Dumas (American tourists / residents in Brittany)

- **Nationality:** American
- **Setting:** living in rural Brittany, shopping at La Madeleine
  (Saint-Malo) in `b1-centre-commercial`.
- **Relationship:** married couple.
- **Surname:** Paul's is **Dumas** — established in
  `b1-messagerie/05_banque_securite` (BNP Paribas calls "monsieur
  Dumas"). Alice shares it by implication.
- **Register:** American English internally, polite French in public;
  early-B1-level French accuracy in the dialogues.
- **Recurring voice:**
  - Alice: Chirp3-HD female, distinct from Maeve/Irene/Camille.
  - Paul: Studio-D or a Chirp3-HD male.

## The narrator/guide archetype (Isabelle)

- **Isabelle** in `b2-toulouse-medieval` is a guide and medieval
  historian. Appears in 01, 02, 03, 04 of that chapter. Delivers
  monologues with occasional tourist Q&A.
- **Register:** formal, explanatory, literary spoken French with
  compound tenses and relative pronouns (the B2 level grammar
  showcase).
- **Voice candidate:** `fr-FR-Studio-A` works well for long-form
  explanation.

---

## Conventions for future chapters

1. **Before introducing a recurring character, check this file.**
   Add them here when they cross a chapter boundary.
2. **Nationality is forever.** Decide once, write it here, stick to
   it. Drift is how we got the Irish/American tension.
3. **Give surnames to distinguish homonyms.** "Camille" by itself is
   now ambiguous; "Camille Perret" resolves cleanly.
4. **The character-block description must start with a gendered
   article or noun** so the TTS gender detector can pick it up.
   Concretely, every `- Name — description` line's description
   should begin with `une`, `la`, `l'X` (feminine noun), `un`, `le`,
   or similar, so `detect_gender` returns `Female` or `Male` rather
   than falling back to alternating assignment. Bad:
   `- Nadia Lefort — désormais senior data-viz engineer`. Good:
   `- Nadia Lefort — une senior data-viz engineer`. The gender
   detector runs *only* on that first word — if it misses, the
   character ends up with an arbitrary voice that may not match
   their actual gender. Run `cargo run -- voice-report <chapter>`
   after authoring to confirm every speaker shows `[Female]` or
   `[Male]`, not `[fallback]`.
5. **Voice drift across chapters is accepted for now.** Recurring
   characters (Maeve, Camille, Nadia) get different voices in
   different dialogs because voice assignment runs per-dialog
   (first speaker of each gender gets the preferred Studio voice;
   subsequent speakers get a Chirp3-HD fallback). The learner hears
   consistent voices *within* a dialog but not across. A future
   `voices.toml` could pin recurring characters to specific voices
   globally; it hasn't been built yet.
6. **English translations must match.** When you edit a character
   description in French, update the `_en` counterpart in the same
   commit.
