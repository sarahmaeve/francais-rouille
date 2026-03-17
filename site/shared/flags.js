/* Français Rouillé — Feature Flags (client-side gate) */
/* No dependencies. Unhides elements gated behind data-flag attributes. */

(function () {
    'use strict';

    var STORAGE_KEY = 'fr-flags';

    /** Return the set of active flag names. */
    function activeFlags() {
        var flags = {};

        // 1. localStorage (persistent)
        try {
            var stored = localStorage.getItem(STORAGE_KEY);
            if (stored) {
                stored.split(',').forEach(function (f) {
                    var name = f.trim();
                    if (name) flags[name] = true;
                });
            }
        } catch (_) { /* private browsing */ }

        // 2. URL ?flags=a,b,c (session override)
        try {
            var params = new URLSearchParams(window.location.search);
            var urlFlags = params.get('flags');
            if (urlFlags) {
                urlFlags.split(',').forEach(function (f) {
                    var name = f.trim();
                    if (name) flags[name] = true;
                });
            }
        } catch (_) { /* old browser */ }

        return flags;
    }

    /** Save flags to localStorage. */
    function saveFlags(flagObj) {
        try {
            var names = Object.keys(flagObj).filter(function (k) { return flagObj[k]; });
            if (names.length) {
                localStorage.setItem(STORAGE_KEY, names.join(','));
            } else {
                localStorage.removeItem(STORAGE_KEY);
            }
        } catch (_) { /* private browsing */ }
    }

    /** Unhide all elements whose data-flag value is in the active set. */
    function applyFlags() {
        var flags = activeFlags();
        var gated = document.querySelectorAll('[data-flag]');
        gated.forEach(function (el) {
            if (flags[el.getAttribute('data-flag')]) {
                el.classList.remove('flag-hidden');
            }
        });
    }

    // Expose for flags.html management page.
    window.FrFlags = {
        active: activeFlags,
        save: saveFlags
    };

    document.addEventListener('DOMContentLoaded', applyFlags);
})();
