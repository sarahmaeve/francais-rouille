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

// Create an element with an optional class and text content. Author text is
// always set via textContent (never interpolated into innerHTML), so French
// punctuation and quotes can't be misread as markup.
function el(tag, className, text) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text != null) node.textContent = text;
    return node;
}

// Render a points value rounded to at most 2 decimals (3 -> "3", 1.333 -> "1.33").
function formatPoints(n) {
    return String(Math.round(n * 100) / 100);
}

// --- Reading-comprehension scoring (deterministic; see docs/READING.md) ---

// Jaccard overlap of a selected sentence-id set with one accepted set.
function jaccardOverlap(selected, acceptedArr) {
    const sel = selected instanceof Set ? selected : new Set(selected);
    const acc = new Set(acceptedArr);
    let inter = 0;
    sel.forEach(function(x) { if (acc.has(x)) inter++; });
    const union = sel.size + acc.size - inter;
    return union === 0 ? 0 : inter / union;
}

function scoreMcSingle(selected, answer, points) {
    return selected === answer ? points : 0;
}

function scoreMultiSelect(selected, answer, points, method) {
    const ans = new Set(answer);
    const sel = new Set(selected);
    if (method === 'all_or_nothing') {
        if (sel.size !== ans.size) return 0;
        let all = true;
        sel.forEach(function(x) { if (!ans.has(x)) all = false; });
        return all ? points : 0;
    }
    let correct = 0;
    let incorrect = 0;
    sel.forEach(function(x) { if (ans.has(x)) correct++; else incorrect++; });
    return points * Math.max(0, (correct - incorrect) / ans.size);
}

function scoreTfng(selected, answer, points) {
    const keys = Object.keys(answer);
    if (keys.length === 0) return 0;
    let correct = 0;
    keys.forEach(function(k) { if (selected[k] === answer[k]) correct++; });
    return points * (correct / keys.length);
}

function scoreHighlight(selected, accepted, minOverlap, points) {
    const threshold = typeof minOverlap === 'number' ? minOverlap : 0.6;
    for (let i = 0; i < accepted.length; i++) {
        if (jaccardOverlap(selected, accepted[i]) >= threshold) return points;
    }
    return 0;
}

// --- Vocabulary Parser ---
// fetchVocab(url) is provided by vocab.js (loaded before this script).

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

    // --- Listening Comprehension ---

    loadListening(data) {
        this.score = 0;
        this.current = 0;
        this.questions = shuffle(data.slice());
        this.total = this.questions.length;
        this.renderListeningQuestion();
    }

    renderListeningQuestion() {
        if (this.current >= this.total) {
            this.showScore('listen', () => this.app.startListening());
            return;
        }

        const q = this.questions[this.current];
        const promptText = q.type === 'topic'
            ? 'Listen, then choose the best description of what the speaker is saying.'
            : 'Écoutez, puis choisissez ce qui pourrait suivre dans la conversation.';

        const optionsHtml = q.options.map((opt, i) =>
            `<button class="quiz-option" data-index="${i}">${opt}</button>`
        ).join('');

        this.container.innerHTML = `
            ${this.showProgress()}
            <div class="quiz-card">
                <div class="quiz-listen-prompt">${promptText}</div>
                <div class="quiz-listen-audio">
                    <button class="quiz-listen-btn" id="listen-play">
                        <svg class="quiz-listen-icon" viewBox="0 0 24 24" width="22" height="22">
                            <polygon points="6,3 20,12 6,21" fill="currentColor"/>
                        </svg>
                        <span>Écouter</span>
                    </button>
                    <audio id="listen-audio" preload="auto"></audio>
                </div>
                <div class="quiz-options">${optionsHtml}</div>
                <div class="quiz-feedback" id="quiz-feedback"></div>
            </div>
        `;

        // Set audio src via DOM property (avoids string interpolation in HTML)
        document.getElementById('listen-audio').src = q.audio_src;

        this.applyProgress();

        const audio = document.getElementById('listen-audio');
        const playBtn = document.getElementById('listen-play');
        playBtn.addEventListener('click', () => {
            if (audio.paused) {
                audio.currentTime = 0;
                audio.play();
                playBtn.classList.add('playing');
            } else {
                audio.pause();
                audio.currentTime = 0;
                playBtn.classList.remove('playing');
            }
        });
        audio.addEventListener('ended', () => {
            playBtn.classList.remove('playing');
        });

        this.container.querySelectorAll('.quiz-option').forEach(btn => {
            btn.addEventListener('click', () => {
                this.checkListening(parseInt(btn.dataset.index, 10));
            });
        });
    }

    checkListening(selectedIndex) {
        const q = this.questions[this.current];
        const buttons = this.container.querySelectorAll('.quiz-option');
        const feedback = document.getElementById('quiz-feedback');

        buttons.forEach(btn => { btn.disabled = true; });
        buttons[q.answer].classList.add('correct');

        if (selectedIndex === q.answer) {
            this.score++;
            feedback.className = 'quiz-feedback correct';
            feedback.textContent = 'Correct !';
        } else {
            buttons[selectedIndex].classList.add('wrong');
            feedback.className = 'quiz-feedback wrong';
            feedback.innerHTML = `La bonne réponse : <strong>${q.options[q.answer]}</strong>`;
        }

        // Show transcript after answering
        if (q.transcript) {
            const transcriptEl = document.createElement('div');
            transcriptEl.className = 'quiz-listen-transcript';
            const label = document.createElement('span');
            label.className = 'quiz-listen-transcript-label';
            label.textContent = 'Transcription : ';
            transcriptEl.appendChild(label);
            transcriptEl.appendChild(document.createTextNode(q.transcript));
            this.container.querySelector('.quiz-card').appendChild(transcriptEl);
        }

        this.current++;
        const nextBtn = document.createElement('button');
        nextBtn.className = 'quiz-btn quiz-next';
        nextBtn.textContent = this.current >= this.total ? 'Voir le score' : 'Suivant';
        nextBtn.onclick = () => this.renderListeningQuestion();
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

// --- Reading Comprehension Engine ---
// A distinct rendering mode: render a passage once, show all its items
// together, grade them on a single submit with weighted/partial credit.

const TFNG_LABELS = { V: 'Vrai', F: 'Faux', NP: 'Non précisé' };
const TFNG_ORDER = ['V', 'F', 'NP'];

class ReadingEngine {
    constructor(container, app) {
        this.container = container;
        this.app = app;
        this.passages = [];
        this.passage = null;
        this.index = 0;
        this.responses = {};   // item id -> user selection
        this.itemNodes = {};   // item id -> { item, body, feedback }
        this.graded = false;
    }

    start(readingData) {
        this.passages = (readingData && readingData.passages) || [];
        if (this.passages.length === 0) {
            const card = el('div', 'quiz-card');
            card.appendChild(el('p', null, 'Pas encore de texte pour ce chapitre.'));
            this.container.replaceChildren(card);
            return;
        }
        if (this.passages.length === 1) {
            this.renderPassage(0);
        } else {
            this.renderMenu();
        }
    }

    renderMenu() {
        const card = el('div', 'quiz-card');
        card.appendChild(el('div', 'quiz-prompt', 'Choisissez un texte :'));
        const list = el('div', 'quiz-reading-menu');
        this.passages.forEach((p, i) => {
            const btn = el('button', 'quiz-option');
            btn.appendChild(el('span', 'quiz-reading-menu-title', p.title || ('Texte ' + (i + 1))));
            const bits = [];
            if (p.level) bits.push(p.level);
            bits.push(p.items.length + ' questions');
            if (p.sources.length > 1) bits.push('sources appariées');
            btn.appendChild(el('span', 'quiz-reading-menu-meta', bits.join(' · ')));
            btn.addEventListener('click', () => this.renderPassage(i));
            list.appendChild(btn);
        });
        card.appendChild(list);
        this.container.replaceChildren(card);
    }

    renderPassage(index) {
        this.index = index;
        this.passage = this.passages[index];
        this.responses = {};
        this.itemNodes = {};
        this.graded = false;

        const wrap = el('div', 'quiz-reading');

        const passageEl = el('div', 'quiz-reading-passage');
        if (this.passage.title) {
            passageEl.appendChild(el('h2', 'quiz-reading-title', this.passage.title));
        }
        this.passage.sources.forEach(src => passageEl.appendChild(this.buildSource(src)));
        wrap.appendChild(passageEl);

        const itemsEl = el('ol', 'quiz-reading-items');
        this.passage.items.forEach((item, i) => itemsEl.appendChild(this.buildItem(item, i)));
        wrap.appendChild(itemsEl);

        const actions = el('div', 'quiz-reading-actions');
        this.submitBtn = el('button', 'quiz-btn', 'Vérifier');
        this.submitBtn.addEventListener('click', () => this.grade());
        actions.appendChild(this.submitBtn);
        wrap.appendChild(actions);

        this.resultEl = el('div', 'quiz-reading-result');
        wrap.appendChild(this.resultEl);

        this.container.replaceChildren(wrap);
    }

    buildSource(source) {
        const art = el('article', 'quiz-source');
        if (source.title) art.appendChild(el('h3', 'quiz-source-title', source.title));
        const body = el('p', 'quiz-source-text');
        source.sentences.forEach((sentence, i) => {
            const span = el('span', 'quiz-sentence', sentence);
            span.dataset.sid = String(i);
            body.appendChild(span);
            body.appendChild(document.createTextNode(' '));
        });
        art.appendChild(body);
        return art;
    }

    buildItem(item, i) {
        const li = el('li', 'quiz-item');
        const prompt = el('div', 'quiz-item-prompt');
        prompt.appendChild(el('span', 'quiz-item-num', (i + 1) + '.'));
        prompt.appendChild(document.createTextNode(' ' + (item.prompt || '')));
        li.appendChild(prompt);

        let body;
        switch (item.type) {
            case 'mc_single': body = this.buildMcSingle(item); break;
            case 'multi_select': body = this.buildMultiSelect(item); break;
            case 'true_false_notgiven': body = this.buildTfng(item); break;
            case 'highlight_span': body = this.buildHighlight(item); break;
            default: body = el('div', 'quiz-item-unsupported', '(type non pris en charge : ' + item.type + ')');
        }
        li.appendChild(body);

        const feedback = el('div', 'quiz-feedback');
        li.appendChild(feedback);

        this.itemNodes[item.id] = { item: item, body: body, feedback: feedback };
        return li;
    }

    buildMcSingle(item) {
        const wrap = el('div', 'quiz-options');
        this.responses[item.id] = null;
        item.options.forEach(opt => {
            const btn = el('button', 'quiz-option', opt.text);
            btn.dataset.oid = opt.id;
            btn.addEventListener('click', () => {
                if (this.graded) return;
                wrap.querySelectorAll('.quiz-option').forEach(b => b.classList.remove('chosen'));
                btn.classList.add('chosen');
                this.responses[item.id] = opt.id;
            });
            wrap.appendChild(btn);
        });
        return wrap;
    }

    buildMultiSelect(item) {
        const wrap = el('div', 'quiz-options');
        this.responses[item.id] = [];
        item.options.forEach(opt => {
            const btn = el('button', 'quiz-option', opt.text);
            btn.dataset.oid = opt.id;
            btn.addEventListener('click', () => {
                if (this.graded) return;
                const arr = this.responses[item.id];
                const at = arr.indexOf(opt.id);
                if (at === -1) {
                    arr.push(opt.id);
                    btn.classList.add('chosen');
                } else {
                    arr.splice(at, 1);
                    btn.classList.remove('chosen');
                }
            });
            wrap.appendChild(btn);
        });
        return wrap;
    }

    buildTfng(item) {
        const wrap = el('div', 'quiz-tfng');
        this.responses[item.id] = {};
        item.statements.forEach(st => {
            const row = el('div', 'quiz-tfng-row');
            row.dataset.stid = st.id;
            row.appendChild(el('div', 'quiz-tfng-text', st.text));
            const choices = el('div', 'quiz-tfng-choices');
            TFNG_ORDER.forEach(val => {
                const b = el('button', 'quiz-tfng-btn', TFNG_LABELS[val]);
                b.dataset.val = val;
                b.addEventListener('click', () => {
                    if (this.graded) return;
                    choices.querySelectorAll('.quiz-tfng-btn').forEach(x => x.classList.remove('chosen'));
                    b.classList.add('chosen');
                    this.responses[item.id][st.id] = val;
                });
                choices.appendChild(b);
            });
            row.appendChild(choices);
            wrap.appendChild(row);
        });
        return wrap;
    }

    buildHighlight(item) {
        const wrap = el('div', 'quiz-highlight');
        this.responses[item.id] = [];
        const source = this.sourceFor(item);
        if (!source) {
            wrap.appendChild(el('div', 'quiz-item-unsupported', '(source introuvable)'));
            return wrap;
        }
        source.sentences.forEach((sentence, i) => {
            const btn = el('button', 'quiz-sentence-option', sentence);
            btn.dataset.sid = String(i);
            btn.addEventListener('click', () => {
                if (this.graded) return;
                const arr = this.responses[item.id];
                const at = arr.indexOf(i);
                if (at === -1) {
                    arr.push(i);
                    btn.classList.add('chosen');
                } else {
                    arr.splice(at, 1);
                    btn.classList.remove('chosen');
                }
            });
            wrap.appendChild(btn);
        });
        return wrap;
    }

    sourceFor(item) {
        const sources = this.passage.sources;
        if (item.source) return sources.find(s => s.id === item.source) || null;
        return sources.length === 1 ? sources[0] : null;
    }

    grade() {
        if (this.graded) return;
        this.graded = true;

        let earned = 0;
        let max = 0;
        this.passage.items.forEach(item => {
            const node = this.itemNodes[item.id];
            max += item.points;
            let got = 0;
            switch (item.type) {
                case 'mc_single': got = this.gradeMcSingle(item, node); break;
                case 'multi_select': got = this.gradeMultiSelect(item, node); break;
                case 'true_false_notgiven': got = this.gradeTfng(item, node); break;
                case 'highlight_span': got = this.gradeHighlight(item, node); break;
                default: got = 0;
            }
            earned += got;
            this.showItemFeedback(node, got, item.points);
        });

        this.submitBtn.disabled = true;
        this.showResult(earned, max);
    }

    gradeMcSingle(item, node) {
        const sel = this.responses[item.id];
        node.body.querySelectorAll('.quiz-option').forEach(b => {
            b.disabled = true;
            b.classList.remove('chosen');
            if (b.dataset.oid === item.answer) b.classList.add('correct');
            else if (b.dataset.oid === sel) b.classList.add('wrong');
        });
        return scoreMcSingle(sel, item.answer, item.points);
    }

    gradeMultiSelect(item, node) {
        const sel = this.responses[item.id];
        const answer = new Set(item.answer);
        const chosen = new Set(sel);
        node.body.querySelectorAll('.quiz-option').forEach(b => {
            b.disabled = true;
            b.classList.remove('chosen');
            const oid = b.dataset.oid;
            if (answer.has(oid)) b.classList.add('correct');
            else if (chosen.has(oid)) b.classList.add('wrong');
        });
        return scoreMultiSelect(sel, item.answer, item.points, item.scoring || 'partial');
    }

    gradeTfng(item, node) {
        const sel = this.responses[item.id];
        node.body.querySelectorAll('.quiz-tfng-row').forEach(row => {
            const stid = row.dataset.stid;
            const correctVal = item.answer[stid];
            row.querySelectorAll('.quiz-tfng-btn').forEach(b => {
                b.disabled = true;
                b.classList.remove('chosen');
                if (b.dataset.val === correctVal) b.classList.add('correct');
                else if (b.dataset.val === sel[stid]) b.classList.add('wrong');
            });
        });
        return scoreTfng(sel, item.answer, item.points);
    }

    gradeHighlight(item, node) {
        const sel = this.responses[item.id];
        const evidence = new Set();
        item.accepted.forEach(span => span.forEach(s => evidence.add(s)));
        const chosen = new Set(sel);
        node.body.querySelectorAll('.quiz-sentence-option').forEach(b => {
            b.disabled = true;
            b.classList.remove('chosen');
            const sid = parseInt(b.dataset.sid, 10);
            if (evidence.has(sid)) b.classList.add('correct');
            else if (chosen.has(sid)) b.classList.add('wrong');
        });
        return scoreHighlight(sel, item.accepted, item.min_overlap, item.points);
    }

    showItemFeedback(node, got, points) {
        const fb = node.feedback;
        if (got >= points) {
            fb.className = 'quiz-feedback correct';
            fb.textContent = 'Correct (+' + formatPoints(got) + ')';
        } else if (got > 0) {
            fb.className = 'quiz-feedback accent-note';
            fb.textContent = 'Partiel : ' + formatPoints(got) + ' / ' + formatPoints(points);
        } else {
            fb.className = 'quiz-feedback wrong';
            fb.textContent = 'Incorrect (0 / ' + formatPoints(points) + ')';
        }
    }

    showResult(earned, max) {
        const pct = max > 0 ? Math.round((earned / max) * 100) : 0;
        let message;
        if (pct === 100) message = 'Parfait !';
        else if (pct >= 80) message = 'Excellent travail !';
        else if (pct >= 60) message = 'Bon travail, continuez !';
        else message = 'Continuez à pratiquer !';

        const score = el('div', 'quiz-score');
        score.appendChild(el('div', 'quiz-score-number', formatPoints(earned) + ' / ' + formatPoints(max)));
        score.appendChild(el('div', 'quiz-score-label', pct + '%'));
        score.appendChild(el('div', 'quiz-score-message', message));

        const actions = el('div', 'quiz-score-actions');
        const again = el('button', 'quiz-btn', 'Recommencer');
        again.addEventListener('click', () => this.renderPassage(this.index));
        actions.appendChild(again);
        if (this.passages.length > 1) {
            const other = el('button', 'quiz-btn-secondary', 'Autre texte');
            other.addEventListener('click', () => this.renderMenu());
            actions.appendChild(other);
        }
        score.appendChild(actions);

        this.resultEl.replaceChildren(score);
        this.resultEl.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    }
}

// --- Quiz App (page-level controller) ---

class QuizApp {
    constructor(vocabUrl, quizData) {
        const container = document.getElementById('quiz-container');
        this.vocabUrl = vocabUrl;
        this.fillInBlank = quizData.FILL_IN_BLANK || [];
        this.dragDrop = quizData.DRAG_DROP || [];
        this.listening = quizData.LISTENING || [];
        this.vocabData = [];
        // Reading data is fetched lazily from its own JSON file (the same file
        // the Rust `verify-quiz` validator checks).
        this.readingUrl = container.dataset.readingUrl || 'reading-data.json';
        this.readingData = null;
        this.engine = new QuizEngine(container, this);
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
        // A reading-only page may have no tab bar at all.
        const tab = document.querySelector(`.quiz-tab[data-type="${type}"]`);
        if (tab) tab.classList.add('active');
        this.currentTab = type;

        if (type === 'mc') this.startMC();
        else if (type === 'fitb') this.startFITB();
        else if (type === 'dd') this.startDD();
        else if (type === 'listen') this.startListening();
        else if (type === 'reading') this.startReading();
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

    startListening() {
        if (this.listening.length === 0) {
            this.engine.container.innerHTML =
                '<div class="quiz-card"><p>Pas encore de questions pour ce type.</p></div>';
            return;
        }
        this.engine = new QuizEngine(document.getElementById('quiz-container'), this);
        this.engine.loadListening(this.listening);
    }

    async startReading() {
        const container = document.getElementById('quiz-container');
        if (!this.readingData) {
            container.innerHTML = '<div class="quiz-card"><p>Chargement des textes…</p></div>';
            try {
                const resp = await fetch(this.readingUrl);
                if (!resp.ok) throw new Error('HTTP ' + resp.status);
                this.readingData = await resp.json();
            } catch (e) {
                container.innerHTML =
                    '<div class="quiz-card"><p>Impossible de charger les textes.</p>' +
                    '<p class="quiz-error-hint">Vérifiez que reading-data.json est disponible.</p></div>';
                return;
            }
        }
        this.reading = new ReadingEngine(container, this);
        this.reading.start(this.readingData);
    }
}

// --- Auto-init ---
// Expects: <div id="quiz-container" data-vocab-url="vocabulaire.html"></div>
// and a global QUIZ_DATA object loaded from quiz-data.js before this script.

document.addEventListener('DOMContentLoaded', function() {
    var container = document.getElementById('quiz-container');
    if (!container) return;
    // QUIZ_DATA is optional: a chapter with only a reading tab needs no quiz-data.js.
    var data = (typeof QUIZ_DATA !== 'undefined') ? QUIZ_DATA : {};
    var vocabUrl = container.dataset.vocabUrl || 'vocabulaire.html';
    var app = new QuizApp(vocabUrl, data);
    // Open whichever tab is marked active, else the first one present, else the
    // container's declared default (a reading-only page sets data-default-tab).
    var active = document.querySelector('.quiz-tab.active') || document.querySelector('.quiz-tab');
    app.switchTab(active ? active.dataset.type : (container.dataset.defaultTab || 'mc'));
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

    // Reading-comprehension scoring (see docs/READING.md)
    const approx = (a, b) => Math.abs(a - b) < 1e-9;

    assert(scoreMcSingle('B', 'B', 1) === 1, 'mc_single correct');
    assert(scoreMcSingle('A', 'B', 1) === 0, 'mc_single wrong');
    assert(scoreMcSingle(null, 'B', 1) === 0, 'mc_single unanswered');

    assert(scoreMultiSelect(['A', 'B', 'D'], ['A', 'B', 'D'], 3, 'partial') === 3, 'multi all correct');
    assert(approx(scoreMultiSelect(['A', 'B'], ['A', 'B', 'D'], 3, 'partial'), 2), 'multi 2/3 correct');
    assert(approx(scoreMultiSelect(['A', 'B', 'C'], ['A', 'B', 'D'], 3, 'partial'), 1), 'multi penalizes wrong pick');
    assert(scoreMultiSelect(['A', 'C'], ['A', 'B', 'D'], 3, 'partial') === 0, 'multi never below zero');
    assert(scoreMultiSelect(['A', 'B'], ['A', 'B', 'D'], 3, 'all_or_nothing') === 0, 'multi AON incomplete');
    assert(scoreMultiSelect(['A', 'B', 'D'], ['A', 'B', 'D'], 3, 'all_or_nothing') === 3, 'multi AON complete');

    const tfngKey = { a: 'F', b: 'V', c: 'NP', d: 'F' };
    assert(scoreTfng(tfngKey, tfngKey, 4) === 4, 'tfng all correct');
    assert(scoreTfng({ a: 'F', b: 'V', c: 'V', d: 'F' }, tfngKey, 4) === 3, 'tfng 3/4');
    assert(scoreTfng({}, tfngKey, 4) === 0, 'tfng unanswered');

    assert(scoreHighlight([7], [[7], [7, 8]], 0.6, 1) === 1, 'highlight exact single');
    assert(scoreHighlight([7, 8], [[7], [7, 8]], 0.6, 1) === 1, 'highlight exact pair');
    assert(scoreHighlight([8], [[7], [7, 8]], 0.6, 1) === 0, 'highlight insufficient overlap');
    assert(scoreHighlight([], [[7]], 0.6, 1) === 0, 'highlight empty selection');
    assert(approx(jaccardOverlap([7, 8], [7, 8, 9]), 2 / 3), 'jaccard 2/3');

    console.log(`\nResults: ${passed} passed, ${failed} failed`);
    return failed === 0;
}
