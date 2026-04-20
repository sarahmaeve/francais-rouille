/* Français Rouillé — Feature Flags management UI */
/* Depends on flags.js (window.FrFlags). */

(function () {
    'use strict';

    /** Known flags — add entries here as new experiments are created. */
    var KNOWN_FLAGS = [
        { name: 'new-dining', description: 'Experimental dining dialog (B1)' },
        { name: 'grammar', description: 'Grammar tips page (B2)' },
        { name: 'voicemail-trial', description: 'Voicemail / répondeur format trial (B1)' }
    ];

    function render() {
        var active = window.FrFlags.active();
        var list = document.getElementById('flag-list');
        var empty = document.getElementById('empty-state');

        // Merge known flags with any extra active flags.
        var allNames = KNOWN_FLAGS.map(function (f) { return f.name; });
        Object.keys(active).forEach(function (name) {
            if (allNames.indexOf(name) === -1) {
                KNOWN_FLAGS.push({ name: name, description: 'Custom flag' });
                allNames.push(name);
            }
        });

        if (KNOWN_FLAGS.length === 0) {
            empty.hidden = false;
            return;
        }

        empty.hidden = true;
        list.innerHTML = '';

        KNOWN_FLAGS.forEach(function (flag) {
            var row = document.createElement('div');
            row.className = 'flag-row';

            var cb = document.createElement('input');
            cb.type = 'checkbox';
            cb.id = 'flag-' + flag.name;
            cb.checked = !!active[flag.name];

            var label = document.createElement('label');
            label.htmlFor = cb.id;
            label.textContent = flag.name;

            var desc = document.createElement('span');
            desc.className = 'flag-status';
            desc.textContent = flag.description;

            cb.addEventListener('change', function () {
                active[flag.name] = cb.checked;
                window.FrFlags.save(active);
            });

            row.appendChild(cb);
            row.appendChild(label);
            row.appendChild(desc);
            list.appendChild(row);
        });
    }

    function addManualFlag() {
        var input = document.getElementById('flag-input');
        var name = input.value.trim();
        if (!name) return;

        var active = window.FrFlags.active();
        active[name] = true;
        window.FrFlags.save(active);
        input.value = '';
        render();
    }

    document.getElementById('add-flag-btn').addEventListener('click', addManualFlag);

    document.getElementById('flag-input').addEventListener('keydown', function (e) {
        if (e.key === 'Enter') {
            e.preventDefault();
            addManualFlag();
        }
    });

    document.addEventListener('DOMContentLoaded', render);
})();
