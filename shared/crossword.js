/* Français Rouillé — Crossword Engine */
/* No dependencies. Pure static HTML/CSS/JS. */

function normalizeForGrid(str) {
    return str.normalize('NFD').replace(/[\u0300-\u036f]/g, '').toUpperCase().trim();
}

class CrosswordEngine {
    constructor(container, puzzle) {
        this.container = container;
        this.puzzle = puzzle;
        this.rows = puzzle.rows;
        this.cols = puzzle.cols;

        // Build solution grid and cell metadata
        this.solution = Array.from({ length: this.rows }, () => Array(this.cols).fill(null));
        this.cellWords = Array.from({ length: this.rows }, () =>
            Array.from({ length: this.cols }, () => ({ across: null, down: null }))
        );

        // Assign clue numbers by scanning top-to-bottom, left-to-right
        this.assignClueNumbers();

        // Fill solution grid and cell-word mapping
        for (const w of this.puzzle.words) {
            const len = w.answer.length;
            const letters = normalizeForGrid(w.answer);
            for (let i = 0; i < len; i++) {
                const r = w.dir === 'across' ? w.row : w.row + i;
                const c = w.dir === 'across' ? w.col + i : w.col;
                this.solution[r][c] = letters[i];
                this.cellWords[r][c][w.dir] = w.num;
            }
        }

        this.activeDir = 'across';
        this.activeWord = null;
        this.activeRow = -1;
        this.activeCol = -1;

        this.build();
    }

    assignClueNumbers() {
        // Determine which cells start a word
        const starts = new Map(); // "row,col" -> number
        let num = 1;

        // Collect all starting cells
        const startCells = new Set();
        for (const w of this.puzzle.words) {
            startCells.add(`${w.row},${w.col}`);
        }

        // Scan in reading order
        for (let r = 0; r < this.rows; r++) {
            for (let c = 0; c < this.cols; c++) {
                const key = `${r},${c}`;
                if (startCells.has(key) && !starts.has(key)) {
                    starts.set(key, num);
                    num++;
                }
            }
        }

        // Assign numbers to words
        for (const w of this.puzzle.words) {
            w.num = starts.get(`${w.row},${w.col}`);
        }
    }

    build() {
        // Build grid table
        const table = document.createElement('table');
        table.className = 'cw-table';

        this.inputs = Array.from({ length: this.rows }, () => Array(this.cols).fill(null));

        for (let r = 0; r < this.rows; r++) {
            const tr = document.createElement('tr');
            for (let c = 0; c < this.cols; c++) {
                const td = document.createElement('td');

                if (this.solution[r][c] !== null) {
                    td.className = 'cw-cell';

                    // Clue number
                    const cw = this.cellWords[r][c];
                    const numHere = this.getClueNumber(r, c);
                    if (numHere !== null) {
                        const span = document.createElement('span');
                        span.className = 'cw-number';
                        span.textContent = numHere;
                        td.appendChild(span);
                    }

                    const input = document.createElement('input');
                    input.type = 'text';
                    input.maxLength = 1;
                    input.autocomplete = 'off';
                    input.autocapitalize = 'characters';
                    input.spellcheck = false;
                    input.dataset.row = r;
                    input.dataset.col = c;

                    input.addEventListener('focus', () => this.onFocus(r, c));
                    input.addEventListener('click', (e) => {
                        e.stopPropagation();
                        this.onClick(r, c);
                    });
                    input.addEventListener('input', (e) => this.onInput(r, c, e));
                    input.addEventListener('keydown', (e) => this.onKeydown(r, c, e));

                    td.appendChild(input);
                    this.inputs[r][c] = input;
                } else {
                    td.className = 'cw-black';
                }

                tr.appendChild(td);
            }
            table.appendChild(tr);
        }

        // Build clue lists
        const cluesDiv = document.createElement('div');
        cluesDiv.className = 'cw-clues';

        const acrossCol = document.createElement('div');
        acrossCol.className = 'cw-clues-col';
        acrossCol.innerHTML = '<h3>Horizontalement</h3>';
        const acrossList = document.createElement('ul');
        acrossList.className = 'cw-clue-list';

        const downCol = document.createElement('div');
        downCol.className = 'cw-clues-col';
        downCol.innerHTML = '<h3>Verticalement</h3>';
        const downList = document.createElement('ul');
        downList.className = 'cw-clue-list';

        const sortedWords = this.puzzle.words.slice().sort((a, b) => a.num - b.num);
        for (const w of sortedWords) {
            const li = document.createElement('li');
            li.className = 'cw-clue';
            li.dataset.num = w.num;
            li.dataset.dir = w.dir;
            li.innerHTML = `<span class="cw-clue-num">${w.num}.</span> ${w.clue}`;
            li.addEventListener('click', () => this.selectWord(w.num, w.dir));

            if (w.dir === 'across') acrossList.appendChild(li);
            else downList.appendChild(li);
        }

        acrossCol.appendChild(acrossList);
        downCol.appendChild(downList);
        cluesDiv.appendChild(acrossCol);
        cluesDiv.appendChild(downCol);

        // Actions
        const actionsDiv = document.createElement('div');
        actionsDiv.className = 'cw-actions';

        const verifyBtn = document.createElement('button');
        verifyBtn.className = 'cw-btn';
        verifyBtn.textContent = 'Vérifier';
        verifyBtn.addEventListener('click', () => this.verify());

        const revealBtn = document.createElement('button');
        revealBtn.className = 'cw-btn-secondary';
        revealBtn.textContent = 'Révéler';
        revealBtn.addEventListener('click', () => this.reveal());

        const resetBtn = document.createElement('button');
        resetBtn.className = 'cw-btn-secondary';
        resetBtn.textContent = 'Recommencer';
        resetBtn.addEventListener('click', () => this.reset());

        actionsDiv.appendChild(verifyBtn);
        actionsDiv.appendChild(revealBtn);
        actionsDiv.appendChild(resetBtn);

        // Feedback
        const feedbackDiv = document.createElement('div');
        feedbackDiv.className = 'cw-feedback';
        feedbackDiv.id = 'cw-feedback';

        // Assemble
        const layout = document.createElement('div');
        layout.className = 'cw-layout';
        layout.appendChild(table);
        layout.appendChild(cluesDiv);
        layout.appendChild(actionsDiv);
        layout.appendChild(feedbackDiv);

        this.container.innerHTML = '';
        this.container.appendChild(layout);
    }

    getClueNumber(r, c) {
        for (const w of this.puzzle.words) {
            if (w.row === r && w.col === c) return w.num;
        }
        return null;
    }

    getWord(num, dir) {
        return this.puzzle.words.find(w => w.num === num && w.dir === dir);
    }

    getWordCells(word) {
        const cells = [];
        const len = word.answer.length;
        for (let i = 0; i < len; i++) {
            const r = word.dir === 'across' ? word.row : word.row + i;
            const c = word.dir === 'across' ? word.col + i : word.col;
            cells.push({ row: r, col: c });
        }
        return cells;
    }

    // --- Interaction ---

    onFocus(r, c) {
        // Just set active position, don't toggle direction
        this.activeRow = r;
        this.activeCol = c;
    }

    onClick(r, c) {
        const cw = this.cellWords[r][c];

        if (this.activeRow === r && this.activeCol === c) {
            // Clicking same cell: toggle direction if both available
            if (cw.across !== null && cw.down !== null) {
                this.activeDir = this.activeDir === 'across' ? 'down' : 'across';
            }
        } else {
            // Clicking new cell: prefer current direction if available
            if (cw[this.activeDir] === null) {
                this.activeDir = cw.across !== null ? 'across' : 'down';
            }
        }

        this.activeRow = r;
        this.activeCol = c;
        this.activeWord = cw[this.activeDir];
        this.highlightActive();
    }

    selectWord(num, dir) {
        const word = this.getWord(num, dir);
        if (!word) return;

        this.activeDir = dir;
        this.activeWord = num;
        this.activeRow = word.row;
        this.activeCol = word.col;

        // Focus first empty cell, or first cell
        const cells = this.getWordCells(word);
        let target = cells[0];
        for (const cell of cells) {
            const input = this.inputs[cell.row][cell.col];
            if (input && !input.value) {
                target = cell;
                break;
            }
        }

        this.activeRow = target.row;
        this.activeCol = target.col;
        this.highlightActive();
        const input = this.inputs[target.row][target.col];
        if (input) input.focus();
    }

    highlightActive() {
        // Clear all highlights
        for (let r = 0; r < this.rows; r++) {
            for (let c = 0; c < this.cols; c++) {
                if (this.inputs[r][c]) {
                    const td = this.inputs[r][c].closest('td');
                    td.classList.remove('cw-active-cell', 'cw-active-word');
                }
            }
        }

        // Clear clue highlights
        this.container.querySelectorAll('.cw-clue.active').forEach(el => el.classList.remove('active'));

        if (this.activeWord === null) return;

        // Highlight active word cells
        const word = this.getWord(this.activeWord, this.activeDir);
        if (word) {
            const cells = this.getWordCells(word);
            for (const cell of cells) {
                const input = this.inputs[cell.row][cell.col];
                if (input) {
                    input.closest('td').classList.add('cw-active-word');
                }
            }

            // Highlight clue
            const clueEl = this.container.querySelector(
                `.cw-clue[data-num="${word.num}"][data-dir="${word.dir}"]`
            );
            if (clueEl) {
                clueEl.classList.add('active');
                clueEl.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
            }
        }

        // Highlight active cell
        const activeInput = this.inputs[this.activeRow][this.activeCol];
        if (activeInput) {
            activeInput.closest('td').classList.add('cw-active-cell');
        }
    }

    onInput(r, c, e) {
        const input = this.inputs[r][c];
        // Only allow letters
        const val = input.value.replace(/[^a-zA-ZÀ-ÿ]/g, '');
        input.value = val ? val[val.length - 1].toUpperCase() : '';

        // Clear verification state
        const td = input.closest('td');
        td.classList.remove('cw-correct', 'cw-wrong', 'cw-revealed');

        if (input.value) {
            this.advanceFocus(r, c, 1);
        }
    }

    onKeydown(r, c, e) {
        switch (e.key) {
            case 'Backspace':
                e.preventDefault();
                const input = this.inputs[r][c];
                if (input.value) {
                    input.value = '';
                    input.closest('td').classList.remove('cw-correct', 'cw-wrong', 'cw-revealed');
                } else {
                    this.advanceFocus(r, c, -1);
                    const prev = this.inputs[this.activeRow][this.activeCol];
                    if (prev) {
                        prev.value = '';
                        prev.closest('td').classList.remove('cw-correct', 'cw-wrong', 'cw-revealed');
                    }
                }
                break;

            case 'ArrowRight':
                e.preventDefault();
                this.activeDir = 'across';
                this.moveFocus(r, c, 0, 1);
                break;
            case 'ArrowLeft':
                e.preventDefault();
                this.activeDir = 'across';
                this.moveFocus(r, c, 0, -1);
                break;
            case 'ArrowDown':
                e.preventDefault();
                this.activeDir = 'down';
                this.moveFocus(r, c, 1, 0);
                break;
            case 'ArrowUp':
                e.preventDefault();
                this.activeDir = 'down';
                this.moveFocus(r, c, -1, 0);
                break;

            case 'Tab':
                e.preventDefault();
                this.tabToNextWord(e.shiftKey);
                break;
        }
    }

    advanceFocus(r, c, delta) {
        let nr = r, nc = c;
        if (this.activeDir === 'across') nc += delta;
        else nr += delta;

        if (nr >= 0 && nr < this.rows && nc >= 0 && nc < this.cols && this.inputs[nr][nc]) {
            this.activeRow = nr;
            this.activeCol = nc;
            this.activeWord = this.cellWords[nr][nc][this.activeDir] || this.activeWord;
            this.inputs[nr][nc].focus();
            this.highlightActive();
        }
    }

    moveFocus(r, c, dr, dc) {
        let nr = r + dr, nc = c + dc;
        // Skip black cells
        while (nr >= 0 && nr < this.rows && nc >= 0 && nc < this.cols) {
            if (this.inputs[nr][nc]) {
                this.activeRow = nr;
                this.activeCol = nc;
                const cw = this.cellWords[nr][nc];
                if (cw[this.activeDir] !== null) {
                    this.activeWord = cw[this.activeDir];
                }
                this.inputs[nr][nc].focus();
                this.highlightActive();
                return;
            }
            nr += dr;
            nc += dc;
        }
    }

    tabToNextWord(reverse) {
        const sortedWords = this.puzzle.words.slice().sort((a, b) => {
            if (a.num !== b.num) return a.num - b.num;
            return a.dir === 'across' ? -1 : 1;
        });

        let currentIdx = sortedWords.findIndex(
            w => w.num === this.activeWord && w.dir === this.activeDir
        );

        if (currentIdx === -1) currentIdx = 0;

        const step = reverse ? -1 : 1;
        const nextIdx = (currentIdx + step + sortedWords.length) % sortedWords.length;
        const nextWord = sortedWords[nextIdx];

        this.selectWord(nextWord.num, nextWord.dir);
    }

    // --- Verification ---

    verify() {
        let correct = 0;
        let total = 0;
        let filled = 0;

        for (let r = 0; r < this.rows; r++) {
            for (let c = 0; c < this.cols; c++) {
                if (this.solution[r][c] !== null) {
                    total++;
                    const input = this.inputs[r][c];
                    const td = input.closest('td');
                    td.classList.remove('cw-correct', 'cw-wrong');

                    const val = normalizeForGrid(input.value);
                    if (val) {
                        filled++;
                        if (val === this.solution[r][c]) {
                            correct++;
                            td.classList.add('cw-correct');
                        } else {
                            td.classList.add('cw-wrong');
                        }
                    } else {
                        td.classList.add('cw-wrong');
                    }
                }
            }
        }

        const feedback = document.getElementById('cw-feedback');
        if (correct === total) {
            feedback.className = 'cw-feedback correct';
            feedback.textContent = 'Parfait ! Toutes les cases sont correctes !';
        } else {
            feedback.className = 'cw-feedback partial';
            feedback.textContent = `${correct} / ${total} cases correctes.`;
            if (filled < total) {
                feedback.textContent += ` (${total - filled} cases vides)`;
            }
        }
    }

    reveal() {
        for (let r = 0; r < this.rows; r++) {
            for (let c = 0; c < this.cols; c++) {
                if (this.solution[r][c] !== null) {
                    const input = this.inputs[r][c];
                    const td = input.closest('td');
                    input.value = this.solution[r][c];
                    td.classList.remove('cw-correct', 'cw-wrong');
                    td.classList.add('cw-revealed');
                }
            }
        }
        const feedback = document.getElementById('cw-feedback');
        feedback.className = 'cw-feedback partial';
        feedback.textContent = 'Solution révélée.';
    }

    reset() {
        for (let r = 0; r < this.rows; r++) {
            for (let c = 0; c < this.cols; c++) {
                if (this.inputs[r][c]) {
                    this.inputs[r][c].value = '';
                    const td = this.inputs[r][c].closest('td');
                    td.classList.remove('cw-correct', 'cw-wrong', 'cw-revealed');
                }
            }
        }
        const feedback = document.getElementById('cw-feedback');
        feedback.className = 'cw-feedback';
        feedback.textContent = '';
    }
}

// --- Self-tests (run via browser console: runCrosswordTests()) ---

function runCrosswordTests() {
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

    // normalizeForGrid
    assert(normalizeForGrid('chaudière') === 'CHAUDIERE', 'normalizeForGrid strips accents');
    assert(normalizeForGrid('Café') === 'CAFE', 'normalizeForGrid uppercase + strip');
    assert(normalizeForGrid('BAIL') === 'BAIL', 'normalizeForGrid no-op on plain uppercase');

    // Test crossing consistency: cells shared by two words must have the same letter
    const testPuzzle = {
        rows: 5, cols: 5,
        words: [
            { num: 1, dir: 'across', row: 0, col: 0, answer: 'ABC', clue: 'test' },
            { num: 1, dir: 'down', row: 0, col: 0, answer: 'AXY', clue: 'test' }
        ]
    };
    const container = document.createElement('div');
    const engine = new CrosswordEngine(container, testPuzzle);
    assert(engine.solution[0][0] === 'A', 'Crossing cell has consistent letter');
    assert(engine.solution[0][1] === 'B', 'Across cell B');
    assert(engine.solution[1][0] === 'X', 'Down cell X');

    // Test white cell count
    let whiteCells = 0;
    for (let r = 0; r < 5; r++) {
        for (let c = 0; c < 5; c++) {
            if (engine.solution[r][c] !== null) whiteCells++;
        }
    }
    assert(whiteCells === 5, 'Correct white cell count (3 + 3 - 1 crossing = 5)');

    console.log(`\nResults: ${passed} passed, ${failed} failed`);
    return failed === 0;
}
