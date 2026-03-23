/* Français Rouillé — Dashboard */

(function () {
    'use strict';

    var API = '/api/stats';
    var token = '';
    var period = 'week';

    var tokenPrompt = document.getElementById('token-prompt');
    var tokenInput = document.getElementById('token-input');
    var dashboard = document.getElementById('dashboard');
    var loading = document.getElementById('loading');
    var error = document.getElementById('error');
    var cards = document.getElementById('cards');
    var status = document.getElementById('status');
    var refreshBtn = document.getElementById('refresh-btn');

    // Read token from URL query parameter if present.
    var params = new URLSearchParams(window.location.search);
    if (params.get('token')) {
        token = params.get('token');
        showDashboard();
    }

    // Token input: submit on Enter.
    tokenInput.addEventListener('keydown', function (e) {
        if (e.key === 'Enter') {
            token = tokenInput.value.trim();
            if (token) {
                showDashboard();
            }
        }
    });

    // Period buttons.
    var periodBtns = document.querySelectorAll('[data-period]');
    periodBtns.forEach(function (btn) {
        // Set initial active state based on default period.
        if (btn.getAttribute('data-period') === period) {
            btn.classList.add('dash-btn-active');
        } else {
            btn.classList.remove('dash-btn-active');
        }
        btn.addEventListener('click', function () {
            period = btn.getAttribute('data-period');
            periodBtns.forEach(function (b) {
                b.classList.remove('dash-btn-active');
            });
            btn.classList.add('dash-btn-active');
            fetchStats(false);
        });
    });

    // Refresh button.
    refreshBtn.addEventListener('click', function () {
        fetchStats(true);
    });

    function showDashboard() {
        tokenPrompt.classList.add('dash-hidden');
        dashboard.classList.remove('dash-hidden');
        fetchStats(false);
    }

    function fetchStats(bustCache) {
        loading.classList.remove('dash-hidden');
        error.classList.add('dash-hidden');
        cards.classList.add('dash-hidden');

        var url = API + '?token=' + encodeURIComponent(token) +
            '&period=' + encodeURIComponent(period);
        if (bustCache) {
            url += '&bust=1';
        }

        fetch(url)
            .then(function (res) {
                if (res.status === 401) {
                    throw new Error('Invalid token');
                }
                if (!res.ok) {
                    throw new Error('Server error (' + res.status + ')');
                }
                return res.json();
            })
            .then(function (data) {
                loading.classList.add('dash-hidden');
                render(data);
                cards.classList.remove('dash-hidden');
                status.textContent = 'Last fetched: ' + new Date().toLocaleTimeString();
            })
            .catch(function (err) {
                loading.classList.add('dash-hidden');
                error.textContent = err.message;
                error.classList.remove('dash-hidden');

                // If unauthorized, show token prompt again.
                if (err.message === 'Invalid token') {
                    dashboard.classList.add('dash-hidden');
                    tokenPrompt.classList.remove('dash-hidden');
                    tokenInput.value = '';
                    tokenInput.focus();
                }
            });
    }

    function render(data) {
        // Total pageviews.
        setText('stat-pageviews', formatNumber(data.total_pageviews));

        // Average time on page.
        if (data.avg_time_on_page != null) {
            setText('stat-avg-time', formatDuration(data.avg_time_on_page));
            setText('stat-avg-time-detail', data.avg_time_on_page + ' seconds');
        } else {
            setText('stat-avg-time', '—');
            setText('stat-avg-time-detail', 'No data');
        }

        // Most active page.
        if (data.most_active_page) {
            setText('stat-active-page', formatNumber(data.most_active_page.views) + ' views');
            setText('stat-active-page-detail', data.most_active_page.page);
        } else {
            setText('stat-active-page', '—');
            setText('stat-active-page-detail', 'No data');
        }

        // Top audio file.
        if (data.top_audio_file) {
            setText('stat-audio-file', formatNumber(data.top_audio_file.completions) + ' plays');
            setText('stat-audio-file-detail', data.top_audio_file.file);
        } else {
            setText('stat-audio-file', '—');
            setText('stat-audio-file-detail', 'No data');
        }

        // Top audio page (chapter completions).
        if (data.top_audio_page) {
            setText('stat-audio-page', formatNumber(data.top_audio_page.completions) + ' completions');
            setText('stat-audio-page-detail', data.top_audio_page.page);
        } else {
            setText('stat-audio-page', '—');
            setText('stat-audio-page-detail', 'No data');
        }
    }

    function setText(id, text) {
        document.getElementById(id).textContent = text;
    }

    function formatNumber(n) {
        if (n == null) return '0';
        return Number(n).toLocaleString();
    }

    function formatDuration(seconds) {
        if (seconds < 60) return seconds + 's';
        var m = Math.floor(seconds / 60);
        var s = seconds % 60;
        return m + 'm ' + s + 's';
    }
})();
