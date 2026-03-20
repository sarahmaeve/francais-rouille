/* Français Rouillé — Flashcard Engine */
/* Requires vocab.js (fetchVocab) to be loaded first. */

// --- Utilities ---

function fcShuffle(arr) {
    var a = arr.slice();
    for (var i = a.length - 1; i > 0; i--) {
        var j = Math.floor(Math.random() * (i + 1));
        var tmp = a[i]; a[i] = a[j]; a[j] = tmp;
    }
    return a;
}

// --- localStorage State ---

function fcStorageKey(chapterId, french) {
    return 'fc:' + chapterId + ':' + french;
}

function fcGetState(chapterId, french) {
    try {
        var raw = localStorage.getItem(fcStorageKey(chapterId, french));
        if (raw) return JSON.parse(raw);
    } catch (e) { /* ignore */ }
    return { box: 1, correct: 0, wrong: 0 };
}

function fcSetState(chapterId, french, state) {
    try {
        localStorage.setItem(fcStorageKey(chapterId, french), JSON.stringify(state));
    } catch (e) { /* ignore */ }
}

function fcClearAll(chapterId, entries) {
    entries.forEach(function(e) {
        try { localStorage.removeItem(fcStorageKey(chapterId, e.french)); }
        catch (err) { /* ignore */ }
    });
}

// --- Flashcard App ---

function FlashcardApp(container, vocabUrl, chapterId) {
    this.container = container;
    this.vocabUrl = vocabUrl;
    this.chapterId = chapterId;
    this.allEntries = [];
    this.deck = [];
    this.current = 0;
    this.sessionCorrect = 0;
    this.sessionTotal = 0;
    this.direction = 'fr-en'; // or 'en-fr'
    this.sectionFilter = 'all';
    this.isFlipped = false;

    this.init();
}

FlashcardApp.prototype.init = function() {
    var self = this;
    this.container.innerHTML = '<p>Chargement du vocabulaire\u2026</p>';
    fetchVocab(this.vocabUrl).then(function(entries) {
        self.allEntries = entries;
        self.render();
    }).catch(function() {
        self.container.innerHTML =
            '<p>Impossible de charger le vocabulaire. V\u00e9rifiez que la page est disponible.</p>';
    });
};

FlashcardApp.prototype.render = function() {
    this.buildDeck();
    this.current = 0;
    this.sessionCorrect = 0;
    this.sessionTotal = 0;
    this.renderControls();
    this.renderCard();
};

FlashcardApp.prototype.getSections = function() {
    var seen = {};
    var sections = [];
    this.allEntries.forEach(function(e) {
        if (!seen[e.section]) {
            seen[e.section] = true;
            sections.push(e.section);
        }
    });
    return sections;
};

FlashcardApp.prototype.buildDeck = function() {
    var self = this;
    var filtered = this.allEntries;
    if (this.sectionFilter !== 'all') {
        filtered = filtered.filter(function(e) { return e.section === self.sectionFilter; });
    }

    // Sort: unlearned cards first (box < 3), then known (box 3)
    // Within each group, shuffle
    var unlearned = [];
    var known = [];
    filtered.forEach(function(e) {
        var state = fcGetState(self.chapterId, e.french);
        if (state.box === 3) {
            known.push(e);
        } else {
            unlearned.push(e);
        }
    });

    this.deck = [].concat(fcShuffle(unlearned), fcShuffle(known));
};

FlashcardApp.prototype.getBoxCounts = function() {
    var self = this;
    var entries = this.allEntries;
    if (this.sectionFilter !== 'all') {
        entries = entries.filter(function(e) { return e.section === self.sectionFilter; });
    }
    var counts = { remaining: 0, known: 0 };
    entries.forEach(function(e) {
        var state = fcGetState(self.chapterId, e.french);
        if (state.box === 3) {
            counts.known++;
        } else {
            counts.remaining++;
        }
    });
    return counts;
};

FlashcardApp.prototype.renderControls = function() {
    var self = this;
    var sections = this.getSections();

    // Build controls area (above the card container)
    var controlsEl = document.getElementById('fc-controls');
    if (!controlsEl) return;

    var sectionOpts = '<option value="all">Toutes les sections</option>';
    sections.forEach(function(s) {
        var sel = s === self.sectionFilter ? ' selected' : '';
        sectionOpts += '<option value="' + s.replace(/"/g, '&quot;') + '"' + sel + '>' + s + '</option>';
    });

    var dirLabel = this.direction === 'fr-en' ? 'FR \u2192 EN' : 'EN \u2192 FR';

    controlsEl.innerHTML =
        '<label>Section : <select id="fc-section-select">' + sectionOpts + '</select></label>' +
        '<button class="fc-direction-toggle" id="fc-dir-btn">' + dirLabel + '</button>';

    document.getElementById('fc-section-select').addEventListener('change', function() {
        self.sectionFilter = this.value;
        self.render();
    });

    document.getElementById('fc-dir-btn').addEventListener('click', function() {
        self.direction = self.direction === 'fr-en' ? 'en-fr' : 'fr-en';
        self.render();
    });
};

FlashcardApp.prototype.renderCard = function() {
    if (this.current >= this.deck.length) {
        this.renderDone();
        return;
    }

    var entry = this.deck[this.current];
    var front = this.direction === 'fr-en' ? entry.french : entry.english;
    var back = this.direction === 'fr-en' ? entry.english : entry.french;
    var counts = this.getBoxCounts();
    var total = this.deck.length;
    var pct = total > 0 ? Math.round((this.current / total) * 100) : 0;

    this.isFlipped = false;
    var self = this;

    this.container.innerHTML =
        '<div class="fc-stats">' +
            '<div class="fc-stat fc-stat-new"><span class="fc-stat-num">' + counts.remaining + '</span><span class="fc-stat-label">\u00c0 revoir</span></div>' +
            '<div class="fc-stat fc-stat-known"><span class="fc-stat-num">' + counts.known + '</span><span class="fc-stat-label">Acquis</span></div>' +
        '</div>' +
        '<div class="fc-status">' +
            '<span>Carte ' + (this.current + 1) + ' / ' + total + '</span>' +
            '<span>' + this.sessionCorrect + ' acquis cette session</span>' +
        '</div>' +
        '<div class="fc-progress"><div class="fc-progress-fill" id="fc-progress-fill"></div></div>' +
        '<div class="fc-card-area">' +
            '<div class="fc-card" id="fc-card">' +
                '<div class="fc-card-face fc-front">' +
                    '<div class="fc-card-word">' + this.escapeHtml(front) + '</div>' +
                    '<div class="fc-card-section">' + this.escapeHtml(entry.section) + '</div>' +
                    '<div class="fc-tap-hint">Cliquez pour retourner</div>' +
                '</div>' +
                '<div class="fc-card-face fc-back">' +
                    '<div class="fc-card-word">' + this.escapeHtml(back) + '</div>' +
                    '<div class="fc-card-section">' + this.escapeHtml(entry.section) + '</div>' +
                '</div>' +
            '</div>' +
        '</div>' +
        '<div class="fc-grade" id="fc-grade"></div>';

    // Apply progress bar
    var fill = document.getElementById('fc-progress-fill');
    if (fill) fill.style.width = pct + '%';

    // Card flip handler
    var card = document.getElementById('fc-card');
    card.addEventListener('click', function() {
        if (!self.isFlipped) {
            card.classList.add('flipped');
            self.isFlipped = true;
            self.showGradeButtons();
        }
    });
};

FlashcardApp.prototype.showGradeButtons = function() {
    var gradeEl = document.getElementById('fc-grade');
    var self = this;

    gradeEl.innerHTML =
        '<button class="fc-btn-again" id="fc-again">\u00c0 revoir</button>' +
        '<button class="fc-btn-know" id="fc-know">Acquis</button>';

    document.getElementById('fc-again').addEventListener('click', function() {
        self.grade(false);
    });
    document.getElementById('fc-know').addEventListener('click', function() {
        self.grade(true);
    });
};

FlashcardApp.prototype.grade = function(known) {
    var entry = this.deck[this.current];
    var state = fcGetState(this.chapterId, entry.french);

    if (known) {
        state.box = 3;
        state.correct = (state.correct || 0) + 1;
        this.sessionCorrect++;
    } else {
        state.box = 1;
        state.wrong = (state.wrong || 0) + 1;
    }

    fcSetState(this.chapterId, entry.french, state);
    this.sessionTotal++;
    this.current++;
    this.renderCard();
};

FlashcardApp.prototype.renderDone = function() {
    var counts = this.getBoxCounts();
    var total = counts.remaining + counts.known;
    var knownPct = total > 0 ? Math.round((counts.known / total) * 100) : 0;

    var message = '';
    if (knownPct === 100) message = 'Tout est acquis \u2014 bravo\u00a0!';
    else if (knownPct >= 70) message = 'Excellent travail\u00a0! Continuez \u00e0 r\u00e9viser les cartes restantes.';
    else if (knownPct >= 40) message = 'Bon d\u00e9but\u00a0! Revenez r\u00e9guli\u00e8rement pour progresser.';
    else message = 'Continuez \u00e0 pratiquer\u00a0! La r\u00e9p\u00e9tition est la cl\u00e9.';

    var self = this;

    this.container.innerHTML =
        '<div class="fc-done">' +
            '<div class="fc-stats">' +
                '<div class="fc-stat fc-stat-new"><span class="fc-stat-num">' + counts.remaining + '</span><span class="fc-stat-label">\u00c0 revoir</span></div>' +
                '<div class="fc-stat fc-stat-known"><span class="fc-stat-num">' + counts.known + '</span><span class="fc-stat-label">Acquis</span></div>' +
            '</div>' +
            '<div class="fc-done-number">' + counts.known + ' / ' + total + '</div>' +
            '<div class="fc-done-label">' + knownPct + '% acquis</div>' +
            '<div class="fc-done-message">' + message + '</div>' +
            '<div class="fc-done-actions">' +
                '<button class="fc-btn" id="fc-restart">Recommencer</button>' +
                '<button class="fc-btn-secondary" id="fc-reset">R\u00e9initialiser</button>' +
            '</div>' +
        '</div>';

    document.getElementById('fc-restart').addEventListener('click', function() {
        self.render();
    });
    document.getElementById('fc-reset').addEventListener('click', function() {
        fcClearAll(self.chapterId, self.allEntries);
        self.render();
    });
};

FlashcardApp.prototype.escapeHtml = function(str) {
    var div = document.createElement('div');
    div.appendChild(document.createTextNode(str));
    return div.innerHTML;
};

// --- Auto-init ---
// Expects: <div id="fc-container" data-vocab-url="vocabulaire.html" data-chapter="chapter-slug"></div>

document.addEventListener('DOMContentLoaded', function() {
    var container = document.getElementById('fc-container');
    if (container) {
        var vocabUrl = container.dataset.vocabUrl || 'vocabulaire.html';
        var chapterId = container.dataset.chapter || 'default';
        new FlashcardApp(container, vocabUrl, chapterId);
    }
});
