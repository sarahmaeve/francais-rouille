/* Français Rouillé — Lightweight analytics */
/* No dependencies. Sends events to /api/event (same-origin D1 endpoint). */

(function () {
    'use strict';

    var ENDPOINT = '/api/event';
    var SESSION_KEY = 'fr-session';

    /** Get or create an anonymous session ID. */
    function sessionId() {
        try {
            var id = localStorage.getItem(SESSION_KEY);
            if (id) return id;
            id = crypto.randomUUID ? crypto.randomUUID()
                : Math.random().toString(36).slice(2) + Date.now().toString(36);
            localStorage.setItem(SESSION_KEY, id);
            return id;
        } catch (_) {
            // Private browsing — generate a per-pageload ID.
            return Math.random().toString(36).slice(2);
        }
    }

    var session = sessionId();
    var pageEnteredAt = Date.now();
    var pagePath = location.pathname;

    /** Send an event. Uses sendBeacon on unload, fetch otherwise. */
    function send(event, value) {
        var payload = JSON.stringify({
            session: session,
            event: event,
            page: pagePath,
            value: value || null
        });

        // Prefer sendBeacon for reliability on page unload.
        if (navigator.sendBeacon) {
            navigator.sendBeacon(ENDPOINT, new Blob([payload], { type: 'application/json' }));
        } else {
            fetch(ENDPOINT, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: payload,
                keepalive: true
            }).catch(function () {});
        }
    }

    // ── Page view ───────────────────────────────────────────────────

    send('pageview');

    // ── Time on page ────────────────────────────────────────────────

    // Send time-on-page when the user leaves. visibilitychange fires
    // more reliably than beforeunload on mobile.
    function sendTimeOnPage() {
        var seconds = Math.round((Date.now() - pageEnteredAt) / 1000);
        if (seconds > 0 && seconds < 7200) { // ignore if > 2 hours (stale tab)
            send('time_on_page', String(seconds));
        }
    }

    document.addEventListener('visibilitychange', function () {
        if (document.visibilityState === 'hidden') {
            sendTimeOnPage();
        }
    });

    // Fallback for desktop browsers that don't fire visibilitychange on close.
    window.addEventListener('pagehide', sendTimeOnPage);

    // ── Audio events ────────────────────────────────────────────────

    document.addEventListener('DOMContentLoaded', function () {
        // Track audio completions.
        var audios = document.querySelectorAll('audio');
        audios.forEach(function (audio) {
            audio.addEventListener('ended', function () {
                send('audio_complete', audio.getAttribute('src') || audio.id);
            });
        });

        // Track play button clicks (first play only per element).
        var playBtns = document.querySelectorAll('.play-btn[data-audio]');
        playBtns.forEach(function (btn) {
            var played = false;
            btn.addEventListener('click', function () {
                if (!played) {
                    played = true;
                    send('audio_play', btn.getAttribute('data-audio'));
                }
            });
        });
    });
})();
