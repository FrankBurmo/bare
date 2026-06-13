/**
 * Bare Browser - Link Hints (Vimium-style)
 * 
 * Tillater navigering av lenker med tastaturet ved å trykke 'f'.
 */

let isHintMode = false;
let hintElements = [];
let currentHintInput = '';

/**
 * Starter hint-modus
 */
function enterHintMode() {
    if (isHintMode) return;
    
    const links = Array.from(document.querySelectorAll('.markdown-body a'));
    const visibleLinks = links.filter(link => {
        const rect = link.getBoundingClientRect();
        return rect.top >= 0 && rect.left >= 0 && 
               rect.bottom <= window.innerHeight && 
               rect.right <= window.innerWidth;
    });

    if (visibleLinks.length === 0) return;

    isHintMode = true;
    currentHintInput = '';
    hintElements = [];

    const alphabet = 'asdfjklghweruiopvbnm';
    
    visibleLinks.forEach((link, index) => {
        const hint = generateHint(index, alphabet);
        const rect = link.getBoundingClientRect();
        
        const hintEl = document.createElement('div');
        hintEl.className = 'link-hint';
        hintEl.textContent = hint.toUpperCase();
        hintEl.style.top = (rect.top + window.scrollY) + 'px';
        hintEl.style.left = (rect.left + window.scrollX) + 'px';
        
        document.body.appendChild(hintEl);
        
        hintElements.push({
            hint: hint.toLowerCase(),
            element: hintEl,
            link: link
        });
    });
}

/**
 * Genererer en unik hint-streng basert på indeks
 */
function generateHint(index, alphabet) {
    const base = alphabet.length;
    if (index < base) return alphabet[index];
    
    return alphabet[Math.floor(index / base) - 1] + alphabet[index % base];
}

/**
 * Avslutter hint-modus
 */
function exitHintMode() {
    isHintMode = false;
    currentHintInput = '';
    hintElements.forEach(item => item.element.remove());
    hintElements = [];
}

/**
 * Håndterer tastetrykk i hint-modus
 */
function handleHintKey(key) {
    if (!isHintMode) return false;

    if (key === 'Escape') {
        exitHintMode();
        return true;
    }

    if (key === 'Backspace') {
        currentHintInput = currentHintInput.slice(0, -1);
        updateHints();
        return true;
    }

    if (key.length === 1 && /[a-z]/i.test(key)) {
        currentHintInput += key.toLowerCase();
        
        const matches = hintElements.filter(item => item.hint.startsWith(currentHintInput));
        
        if (matches.length === 1 && matches[0].hint === currentHintInput) {
            matches[0].link.click();
            exitHintMode();
        } else if (matches.length === 0) {
            exitHintMode();
        } else {
            updateHints();
        }
        return true;
    }

    return false;
}

/**
 * Oppdaterer visning av hints basert på input
 */
function updateHints() {
    hintElements.forEach(item => {
        if (item.hint.startsWith(currentHintInput)) {
            item.element.style.opacity = '1';
            // Marker treffende del?
        } else {
            item.element.style.opacity = '0.2';
        }
    });
}

// Eksporter funksjoner hvis nødvendig, eller bare la dem være globale siden vi ikke har moduler
window.enterHintMode = enterHintMode;
window.handleHintKey = handleHintKey;
window.isHintMode = () => isHintMode;
