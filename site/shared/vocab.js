/* Français Rouillé — Shared vocabulary parser */
/* Parses vocabulaire.html pages into {french, english, section} entries. */

function fetchVocab(url) {
    return fetch(url).then(function(resp) {
        return resp.text();
    }).then(function(html) {
        var parser = new DOMParser();
        var doc = parser.parseFromString(html, 'text/html');
        var sections = doc.querySelectorAll('.vocab-section');
        var entries = [];
        sections.forEach(function(section) {
            var heading = section.querySelector('h2');
            var sectionName = heading ? heading.textContent.trim() : '';
            var rows = section.querySelectorAll('tr');
            rows.forEach(function(row) {
                var cells = row.querySelectorAll('td');
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
    });
}
