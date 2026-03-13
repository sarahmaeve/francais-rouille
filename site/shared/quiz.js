/* Français Rouillé — Quiz Engine */
/* No dependencies. Works with any chapter's vocabulaire.html. */

// --- Utilities ---

function shuffle(arr) {
    const a = arr.slice();
    for (let i = a.length - 1; i > 0; i--) {
        const j = Math.floor(Math.random() * (i + 1));
        [a[i], a[j]] = [a[j], a[i]];
    }
    return a;
}

function normalizeAccents(str) {
    return str.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toLowerCase().trim();
}

function hasAccentDifference(input, answer) {
    return normalizeAccents(input) === normalizeAccents(answer) &&
           input.trim().toLowerCase() !== answer.toLowerCase();
}

function pickDistractors(correctEntry, pool, n) {
    const filtered = pool.filter(e => e.french !== correctEntry.french);
    const shuffled = shuffle(filtered);
    return shuffled.slice(0, n);
}

// --- Vocabulary Parser ---

async function fetchVocab(url) {
    const resp = await fetch(url);
    const html = await resp.text();
    const parser = new DOMParser();
    const doc = parser.parseFromString(html, 'text/html');
    const sections = doc.querySelectorAll('.vocab-section');
    const entries = [];

    sections.forEach(section => {
        const heading = section.querySelector('h2');
        const sectionName = heading ? heading.textContent.trim() : '';
        const rows = section.querySelectorAll('tr');
        rows.forEach(row => {
            const cells = row.querySelectorAll('td');
            if (cells.length >= 2) {
                entries.push({
                    french: cells[0].textContent.trim(),
                    english: cells[1].textContent.trim(),
                    section: sectionName
                });
            }
        });
    });

    return entries;
}

// --- Quiz Engine ---

class QuizEngine {
    constructor(containerEl, app) {
        this.container = containerEl;
        this.app = app;
        this.score = 0;
        this.total = 0;
        this.current = 0;
        this.questions = [];
    }

    showProgress() {
        const pct = this.total > 0 ? Math.round((this.current / this.total) * 100) : 0;
        this._progressPct = pct;
        return `
            <div class="quiz-status">
                <span>Question ${Math.min(this.current + 1, this.total)} / ${this.total}</span>
                <span>${this.score} correct</span>
            </div>
            <div class="quiz-progress">
                <div class="quiz-progress-fill"></div>
            </div>
        `;
    }

    applyProgress() {
        const fill = this.container.querySelector('.quiz-progress-fill');
        if (fill) fill.style.width = this._progressPct + '%';
    }

    showScore(quizType, restartCallback) {
        const pct = this.total > 0 ? Math.round((this.score / this.total) * 100) : 0;
        let message = '';
        if (pct === 100) message = 'Parfait !';
        else if (pct >= 80) message = 'Excellent travail !';
        else if (pct >= 60) message = 'Bon travail, continuez !';
        else message = 'Continuez à pratiquer !';

        this.container.innerHTML = `
            <div class="quiz-score">
                <div class="quiz-score-number">${this.score} / ${this.total}</div>
                <div class="quiz-score-label">${pct}%</div>
                <div class="quiz-score-message">${message}</div>
                <div class="quiz-score-actions">
                    <button class="quiz-btn quiz-restart">Recommencer</button>
                </div>
            </div>
        `;
        this.container.querySelector('.quiz-restart').addEventListener('click', restartCallback);
    }

    // --- Multiple Choice ---

    loadMultipleChoice(vocabData, count) {
        this.score = 0;
        this.current = 0;
        const selected = shuffle(vocabData).slice(0, count || 10);
        this.total = selected.length;
        this.questions = selected.map(entry => {
            const frToEn = Math.random() > 0.5;
            const distractors = pickDistractors(entry, vocabData, 3);
            const options = shuffle([entry, ...distractors]);
            return { entry, frToEn, options };
        });
        this.renderMCQuestion();
    }

    renderMCQuestion() {
        if (this.current >= this.total) {
            this.showScore('mc', () => this.app.startMC());
            return;
        }

        const q = this.questions[this.current];
        const prompt = q.frToEn
            ? `Que signifie <strong>${q.entry.french}</strong> en anglais ?`
            : `Comment dit-on <strong>${q.entry.english}</strong> en français ?`;

        const optionsHtml = q.options.map((opt, i) => {
            const label = q.frToEn ? opt.english : opt.french;
            return `<button class="quiz-option" data-index="${i}">${label}</button>`;
        }).join('');

        this.container.innerHTML = `
            ${this.showProgress()}
            <div class="quiz-card">
                <div class="quiz-prompt">${prompt}</div>
                <div class="quiz-options">${optionsHtml}</div>
                <div class="quiz-feedback" id="quiz-feedback"></div>
            </div>
        `;

        this.applyProgress();
        const self = this;
        this.container.querySelectorAll('.quiz-option').forEach(function(btn) {
            btn.addEventListener('click', function() {
                self.checkMC(parseInt(btn.dataset.index, 10));
            });
        });
    }

    checkMC(selectedIndex) {
        const q = this.questions[this.current];
        const correctIndex = q.options.indexOf(q.entry);
        const buttons = this.container.querySelectorAll('.quiz-option');
        const feedback = document.getElementById('quiz-feedback');

        buttons.forEach(btn => btn.disabled = true);
        buttons[correctIndex].classList.add('correct');

        if (selectedIndex === correctIndex) {
            this.score++;
            feedback.className = 'quiz-feedback correct';
            feedback.textContent = 'Correct !';
        } else {
            buttons[selectedIndex].classList.add('wrong');
            const correctLabel = q.frToEn ? q.entry.english : q.entry.french;
            feedback.className = 'quiz-feedback wrong';
            feedback.innerHTML = `La bonne réponse : <strong>${correctLabel}</strong>`;
        }

        this.current++;
        const nextBtn = document.createElement('button');
        nextBtn.className = 'quiz-btn quiz-next';
        nextBtn.textContent = this.current >= this.total ? 'Voir le score' : 'Suivant';
        nextBtn.onclick = () => this.renderMCQuestion();
        this.container.querySelector('.quiz-card').appendChild(nextBtn);
    }

    // --- Fill in the Blank ---

    loadFillInBlank(data) {
        this.score = 0;
        this.current = 0;
        this.questions = shuffle(data.slice());
        this.total = this.questions.length;
        this.renderFITBQuestion();
    }

    renderFITBQuestion() {
        if (this.current >= this.total) {
            this.showScore('fitb', () => this.app.startFITB());
            return;
        }

        const q = this.questions[this.current];
        const parts = q.sentence_fr.split('___');
        const sentenceHtml = parts[0] +
            `<input type="text" class="quiz-fitb-input" id="fitb-input" autocomplete="off" autocapitalize="none" spellcheck="false">` +
            (parts[1] || '');

        this.container.innerHTML = `
            ${this.showProgress()}
            <div class="quiz-card">
                <div class="quiz-fitb-sentence">${sentenceHtml}</div>
                <div class="quiz-fitb-actions">
                    <button class="quiz-btn" id="fitb-check">Vérifier</button>
                    <button class="quiz-hint" id="fitb-hint-btn">Indice</button>
                    <span class="quiz-hint-text" id="fitb-hint"></span>
                </div>
                <div class="quiz-feedback" id="quiz-feedback"></div>
            </div>
        `;

        this.applyProgress();
        const self = this;
        document.getElementById('fitb-check').addEventListener('click', function() { self.checkFITB(); });
        document.getElementById('fitb-hint-btn').addEventListener('click', function() { self.showHint(); });

        const input = document.getElementById('fitb-input');
        input.focus();
        input.addEventListener('keydown', e => {
            if (e.key === 'Enter') this.checkFITB();
        });
    }

    showHint() {
        const q = this.questions[this.current];
        document.getElementById('fitb-hint').textContent = q.hint;
    }

    checkFITB() {
        const q = this.questions[this.current];
        const input = document.getElementById('fitb-input');
        const checkBtn = document.getElementById('fitb-check');
        const feedback = document.getElementById('quiz-feedback');
        const userAnswer = input.value.trim();

        if (!userAnswer) return;

        input.disabled = true;
        checkBtn.disabled = true;

        const exactMatch = userAnswer.toLowerCase() === q.answer.toLowerCase();
        const accentMatch = hasAccentDifference(userAnswer, q.answer);
        const normalizedMatch = normalizeAccents(userAnswer) === normalizeAccents(q.answer);

        if (exactMatch) {
            this.score++;
            input.classList.add('correct');
            feedback.className = 'quiz-feedback correct';
            feedback.textContent = 'Correct !';
        } else if (accentMatch) {
            this.score++;
            input.classList.add('correct');
            feedback.className = 'quiz-feedback accent-note';
            feedback.innerHTML = `Presque ! Attention à l'accent : <strong>${q.answer}</strong>`;
        } else if (normalizedMatch) {
            this.score++;
            input.classList.add('correct');
            feedback.className = 'quiz-feedback accent-note';
            feedback.innerHTML = `Correct ! Forme exacte : <strong>${q.answer}</strong>`;
        } else {
            input.classList.add('wrong');
            feedback.className = 'quiz-feedback wrong';
            feedback.innerHTML = `La bonne réponse : <strong>${q.answer}</strong>`;
        }

        this.current++;
        const nextBtn = document.createElement('button');
        nextBtn.className = 'quiz-btn quiz-next';
        nextBtn.textContent = this.current >= this.total ? 'Voir le score' : 'Suivant';
        nextBtn.onclick = () => this.renderFITBQuestion();
        this.container.querySelector('.quiz-card').appendChild(nextBtn);
    }

    // --- Drag and Drop ---

    loadDragDrop(data) {
        this.score = 0;
        this.current = 0;
        this.questions = shuffle(data.slice());
        this.total = this.questions.length;
        this.selectedChip = null;
        this.renderDDQuestion();
    }

    renderDDQuestion() {
        if (this.current >= this.total) {
            this.showScore('dd', () => this.app.startDD());
            return;
        }

        const q = this.questions[this.current];
        const parts = q.sentence_fr.split('___');
        const sentenceHtml = parts[0] +
            `<span class="quiz-dropzone" id="dd-dropzone"></span>` +
            (parts[1] || '');

        const chipsHtml = shuffle(q.options).map((word, i) =>
            `<button class="quiz-chip" draggable="true" data-word="${word}" data-index="${i}">${word}</button>`
        ).join('');

        this.container.innerHTML = `
            ${this.showProgress()}
            <div class="quiz-card">
                <div class="quiz-dd-sentence">${sentenceHtml}</div>
                <div class="quiz-chips" id="dd-chips">${chipsHtml}</div>
                <div class="quiz-feedback" id="quiz-feedback"></div>
            </div>
        `;

        this.applyProgress();
        this.selectedChip = null;
        this.initDragDrop(q);
    }

    initDragDrop(question) {
        const dropzone = document.getElementById('dd-dropzone');
        const chips = this.container.querySelectorAll('.quiz-chip');
        const self = this;

        // Drag and drop (desktop)
        chips.forEach(chip => {
            chip.addEventListener('dragstart', e => {
                e.dataTransfer.setData('text/plain', chip.dataset.word);
                chip.style.opacity = '0.5';
            });
            chip.addEventListener('dragend', () => {
                chip.style.opacity = '1';
            });
        });

        dropzone.addEventListener('dragover', e => {
            e.preventDefault();
            dropzone.classList.add('dragover');
        });

        dropzone.addEventListener('dragleave', () => {
            dropzone.classList.remove('dragover');
        });

        dropzone.addEventListener('drop', e => {
            e.preventDefault();
            dropzone.classList.remove('dragover');
            const word = e.dataTransfer.getData('text/plain');
            self.checkDD(word, question);
        });

        // Click-to-select (mobile fallback + desktop alternative)
        chips.forEach(chip => {
            chip.addEventListener('click', () => {
                if (chip.classList.contains('used')) return;

                // Deselect previous
                chips.forEach(c => c.classList.remove('selected'));
                chip.classList.add('selected');
                self.selectedChip = chip.dataset.word;
            });
        });

        dropzone.addEventListener('click', () => {
            if (self.selectedChip) {
                self.checkDD(self.selectedChip, question);
            }
        });
    }

    checkDD(word, question) {
        const dropzone = document.getElementById('dd-dropzone');
        const feedback = document.getElementById('quiz-feedback');
        const chips = this.container.querySelectorAll('.quiz-chip');

        if (word === question.answer) {
            this.score++;
            dropzone.textContent = word;
            dropzone.classList.add('correct');
            feedback.className = 'quiz-feedback correct';
            feedback.textContent = 'Correct !';

            chips.forEach(c => {
                c.classList.add('used');
                c.classList.remove('selected');
            });

            this.current++;
            const nextBtn = document.createElement('button');
            nextBtn.className = 'quiz-btn quiz-next';
            nextBtn.textContent = this.current >= this.total ? 'Voir le score' : 'Suivant';
            nextBtn.onclick = () => this.renderDDQuestion();
            this.container.querySelector('.quiz-card').appendChild(nextBtn);
        } else {
            dropzone.classList.add('wrong');
            dropzone.textContent = word;
            feedback.className = 'quiz-feedback wrong';
            feedback.textContent = 'Essayez encore !';

            // Find and mark the wrong chip
            chips.forEach(c => {
                if (c.dataset.word === word) {
                    c.classList.add('used');
                }
                c.classList.remove('selected');
            });
            this.selectedChip = null;

            setTimeout(() => {
                dropzone.classList.remove('wrong');
                dropzone.textContent = '';
            }, 600);
        }
    }
}

// --- Quiz App (page-level controller) ---

class QuizApp {
    constructor(vocabUrl, quizData) {
        this.vocabUrl = vocabUrl;
        this.fillInBlank = quizData.FILL_IN_BLANK || [];
        this.dragDrop = quizData.DRAG_DROP || [];
        this.vocabData = [];
        this.engine = new QuizEngine(document.getElementById('quiz-container'), this);
        this.currentTab = null;

        this.initTabs();
    }

    initTabs() {
        document.querySelectorAll('.quiz-tab').forEach(tab => {
            tab.addEventListener('click', () => {
                const type = tab.dataset.type;
                this.switchTab(type);
            });
        });
    }

    switchTab(type) {
        document.querySelectorAll('.quiz-tab').forEach(t => t.classList.remove('active'));
        document.querySelector(`.quiz-tab[data-type="${type}"]`).classList.add('active');
        this.currentTab = type;

        if (type === 'mc') this.startMC();
        else if (type === 'fitb') this.startFITB();
        else if (type === 'dd') this.startDD();
    }

    async startMC() {
        if (this.vocabData.length === 0) {
            this.engine.container.innerHTML = '<p>Chargement du vocabulaire...</p>';
            try {
                this.vocabData = await fetchVocab(this.vocabUrl);
            } catch (e) {
                this.engine.container.innerHTML =
                    '<div class="quiz-card"><p>Impossible de charger le vocabulaire.</p>' +
                    '<p class="quiz-error-hint">Vérifiez que la page de vocabulaire est disponible.</p></div>';
                return;
            }
        }
        const count = Math.min(10, this.vocabData.length);
        this.engine = new QuizEngine(document.getElementById('quiz-container'), this);
        this.engine.loadMultipleChoice(this.vocabData, count);
    }

    startFITB() {
        if (this.fillInBlank.length === 0) {
            this.engine.container.innerHTML =
                '<div class="quiz-card"><p>Pas encore de questions pour ce type.</p></div>';
            return;
        }
        this.engine = new QuizEngine(document.getElementById('quiz-container'), this);
        this.engine.loadFillInBlank(this.fillInBlank);
    }

    startDD() {
        if (this.dragDrop.length === 0) {
            this.engine.container.innerHTML =
                '<div class="quiz-card"><p>Pas encore de questions pour ce type.</p></div>';
            return;
        }
        this.engine = new QuizEngine(document.getElementById('quiz-container'), this);
        this.engine.loadDragDrop(this.dragDrop);
    }
}

// --- Auto-init ---
// Expects: <div id="quiz-container" data-vocab-url="vocabulaire.html"></div>
// and a global QUIZ_DATA object loaded from quiz-data.js before this script.

document.addEventListener('DOMContentLoaded', function() {
    var container = document.getElementById('quiz-container');
    if (container && typeof QUIZ_DATA !== 'undefined') {
        var vocabUrl = container.dataset.vocabUrl || 'vocabulaire.html';
        var app = new QuizApp(vocabUrl, QUIZ_DATA);
        app.switchTab('mc');
    }
});

// --- Self-tests (run via browser console: runTests()) ---

function runTests() {
    let passed = 0;
    let failed = 0;

    function assert(condition, name) {
        if (condition) {
            console.log('PASS: ' + name);
            passed++;
        } else {
            console.error('FAIL: ' + name);
            failed++;
        }
    }

    // shuffle
    const arr = [1, 2, 3, 4, 5];
    const s = shuffle(arr);
    assert(s.length === 5, 'shuffle preserves length');
    assert(arr.join() === '1,2,3,4,5', 'shuffle does not mutate original');
    assert(s.sort().join() === '1,2,3,4,5', 'shuffle preserves elements');

    // normalizeAccents
    assert(normalizeAccents('chaudière') === 'chaudiere', 'normalizeAccents removes accent');
    assert(normalizeAccents('RÉSUMÉ') === 'resume', 'normalizeAccents lowercases and strips');
    assert(normalizeAccents('hello') === 'hello', 'normalizeAccents no-op on plain text');
    assert(normalizeAccents(' Café ') === 'cafe', 'normalizeAccents trims');

    // hasAccentDifference
    assert(hasAccentDifference('chaudiere', 'chaudière') === true, 'hasAccentDifference detects missing accent');
    assert(hasAccentDifference('chaudière', 'chaudière') === false, 'hasAccentDifference exact match');
    assert(hasAccentDifference('wrong', 'chaudière') === false, 'hasAccentDifference different word');

    // pickDistractors
    const pool = [
        { french: 'a' }, { french: 'b' }, { french: 'c' },
        { french: 'd' }, { french: 'e' }
    ];
    const d = pickDistractors({ french: 'a' }, pool, 3);
    assert(d.length === 3, 'pickDistractors returns requested count');
    assert(d.every(x => x.french !== 'a'), 'pickDistractors excludes correct');

    console.log(`\nResults: ${passed} passed, ${failed} failed`);
    return failed === 0;
}
