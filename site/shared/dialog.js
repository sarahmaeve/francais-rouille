/* Français Rouillé — Dialog Audio Player */
/* No dependencies. Handles per-line and sequential playback. */

(function() {
    'use strict';

    var currentAudio = null;
    var currentBtn = null;
    var playingAll = false;
    var playAllBtn = null;
    var playAllQueue = [];
    var playAllTimer = null;

    function stopPlayAll() {
        playingAll = false;
        if (playAllTimer) { clearTimeout(playAllTimer); playAllTimer = null; }
        if (playAllBtn) { playAllBtn.classList.remove('playing'); playAllBtn = null; }
        playAllQueue = [];
    }

    function toggleAudio(btn) {
        var audioId = btn.dataset.audio;
        var audio = document.getElementById(audioId);
        stopPlayAll();

        if (currentAudio && currentAudio !== audio) {
            currentAudio.pause();
            currentAudio.currentTime = 0;
            currentBtn.classList.remove('playing');
        }

        if (audio.paused) {
            audio.play();
            btn.classList.add('playing');
            currentAudio = audio;
            currentBtn = btn;
        } else {
            audio.pause();
            audio.currentTime = 0;
            btn.classList.remove('playing');
            currentAudio = null;
            currentBtn = null;
        }

        audio.onended = function() {
            btn.classList.remove('playing');
            currentAudio = null;
            currentBtn = null;
        };
    }

    function playAll(btn) {
        if (playingAll) {
            if (currentAudio && !currentAudio.paused) {
                currentAudio.pause();
                btn.classList.remove('playing');
                if (currentBtn) currentBtn.classList.remove('playing');
                return;
            }
            if (currentAudio && currentAudio.paused) {
                currentAudio.play();
                btn.classList.add('playing');
                if (currentBtn) currentBtn.classList.add('playing');
                return;
            }
        }

        if (currentAudio) {
            currentAudio.pause();
            currentAudio.currentTime = 0;
            if (currentBtn) currentBtn.classList.remove('playing');
        }

        playAllQueue = Array.from(document.querySelectorAll('.dialogue audio'));
        if (playAllQueue.length === 0) return;

        playingAll = true;
        playAllBtn = btn;
        btn.classList.add('playing');
        playNextInSequence();
    }

    function playNextInSequence() {
        if (!playingAll || playAllQueue.length === 0) {
            stopPlayAll();
            if (currentBtn) { currentBtn.classList.remove('playing'); currentBtn = null; }
            currentAudio = null;
            return;
        }

        var audio = playAllQueue.shift();
        var lineDiv = audio.closest('.line');
        var lineBtn = lineDiv ? lineDiv.querySelector('.play-btn') : null;

        if (currentBtn && currentBtn !== playAllBtn) {
            currentBtn.classList.remove('playing');
        }
        if (lineBtn) lineBtn.classList.add('playing');

        currentBtn = lineBtn;
        currentAudio = audio;
        audio.currentTime = 0;
        audio.onended = function() {
            if (lineBtn) lineBtn.classList.remove('playing');
            playAllTimer = setTimeout(playNextInSequence, 750);
        };
        audio.play();
    }

    // Bind event listeners once DOM is ready
    document.addEventListener('DOMContentLoaded', function() {
        // Per-line play buttons
        document.querySelectorAll('.play-btn[data-audio]').forEach(function(btn) {
            btn.addEventListener('click', function() { toggleAudio(btn); });
        });

        // Play-all button
        var playAllButton = document.querySelector('.play-combined .play-btn');
        if (playAllButton) {
            playAllButton.addEventListener('click', function() { playAll(playAllButton); });
        }
    });
})();
