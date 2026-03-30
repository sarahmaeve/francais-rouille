(function () {
    var slides = document.querySelectorAll('.hero-img');
    if (slides.length < 2) return;

    var current = 0;
    var interval = 6000;

    setInterval(function () {
        slides[current].classList.remove('active');
        current = (current + 1) % slides.length;
        slides[current].classList.add('active');
    }, interval);
})();
