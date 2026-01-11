/**
 * Bare Browser - Search Functionality
 * 
 * Håndterer tekstsøk i rendret innhold med highlighting.
 */

/**
 * Åpner søkefeltet
 */
function openSearch() {
    elements.searchBar.classList.remove('hidden');
    elements.searchInput.focus();
    elements.searchInput.select();
}

/**
 * Lukker søkefeltet og rydder opp
 */
function closeSearch() {
    elements.searchBar.classList.add('hidden');
    clearSearchHighlights();
    resetSearchState();
    elements.searchCount.textContent = '';
}

/**
 * Fjerner alle søkehighlights fra innholdet
 */
function clearSearchHighlights() {
    const highlights = elements.content.querySelectorAll('.search-highlight');
    highlights.forEach(el => {
        const parent = el.parentNode;
        parent.replaceChild(document.createTextNode(el.textContent), el);
        parent.normalize();
    });
}

/**
 * Utfører søk i innholdet
 */
function performSearch() {
    const query = elements.searchInput.value.trim().toLowerCase();
    
    clearSearchHighlights();
    resetSearchState();
    
    if (!query) {
        elements.searchCount.textContent = '';
        return;
    }
    
    const markdownBody = getMarkdownBody();
    if (!markdownBody) return;
    
    // Finn alle tekstnoder
    const walker = document.createTreeWalker(
        markdownBody,
        NodeFilter.SHOW_TEXT,
        null,
        false
    );
    
    const textNodes = [];
    while (walker.nextNode()) {
        textNodes.push(walker.currentNode);
    }
    
    // Highlight treff
    const matches = [];
    textNodes.forEach(node => {
        const text = node.textContent;
        const lowerText = text.toLowerCase();
        let startIndex = 0;
        let index;
        
        while ((index = lowerText.indexOf(query, startIndex)) !== -1) {
            const range = document.createRange();
            range.setStart(node, index);
            range.setEnd(node, index + query.length);
            
            const highlight = document.createElement('span');
            highlight.className = 'search-highlight';
            
            try {
                range.surroundContents(highlight);
                matches.push(highlight);
                
                // Oppdater node-referansen for videre søk
                node = highlight.nextSibling;
                if (!node) break;
                startIndex = 0;
            } catch (e) {
                // Range krysser element-grenser, hopp over
                startIndex = index + 1;
            }
        }
    });
    
    setSearchMatches(matches);
    
    if (matches.length > 0) {
        setCurrentMatchIndex(0);
        highlightCurrentMatch();
    }
    
    updateSearchCount();
}

/**
 * Highlighter nåværende søketreff
 */
function highlightCurrentMatch() {
    const matches = getSearchMatches();
    const currentIndex = getCurrentMatchIndex();
    
    matches.forEach((el, i) => {
        el.classList.toggle('current', i === currentIndex);
    });
    
    if (matches[currentIndex]) {
        matches[currentIndex].scrollIntoView({
            behavior: 'smooth',
            block: 'center'
        });
    }
}

/**
 * Oppdaterer søketeller-visningen
 */
function updateSearchCount() {
    const matches = getSearchMatches();
    const currentIndex = getCurrentMatchIndex();
    
    if (matches.length === 0) {
        elements.searchCount.textContent = 'Ingen treff';
    } else {
        elements.searchCount.textContent = `${currentIndex + 1} av ${matches.length}`;
    }
}

/**
 * Går til neste søketreff
 */
function searchNext() {
    if (getSearchMatches().length === 0) return;
    nextMatch();
    highlightCurrentMatch();
    updateSearchCount();
}

/**
 * Går til forrige søketreff
 */
function searchPrev() {
    if (getSearchMatches().length === 0) return;
    prevMatch();
    highlightCurrentMatch();
    updateSearchCount();
}
