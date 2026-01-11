/**
 * Bare Browser - State Management
 * 
 * Sentralisert tilstandshåndtering for applikasjonen.
 */

const state = {
    // Navigasjonshistorikk
    history: [],
    historyIndex: -1,
    
    // Nåværende lokasjon
    currentPath: null,
    currentUrl: null,
    currentTitle: null,
    
    // Brukerinnstillinger
    settings: null,
    
    // Søk
    searchMatches: [],
    currentMatchIndex: -1,
};

// ===== Getters =====

/**
 * Henter hele state-objektet (for debugging)
 * @returns {Object} Hele state
 */
function getState() {
    return state;
}

/**
 * Henter nåværende URL eller path
 * @returns {string|null} Aktiv URL/path
 */
function getCurrentLocation() {
    return state.currentUrl || state.currentPath;
}

/**
 * Sjekker om vi er på startsiden
 * @returns {boolean} True hvis på hjem
 */
function isHome() {
    return state.currentPath === HOME_PATH;
}

// ===== Setters =====

/**
 * Setter nåværende path
 * @param {string|null} path - Filsti
 */
function setCurrentPath(path) {
    state.currentPath = path;
}

/**
 * Setter nåværende URL
 * @param {string|null} url - URL
 */
function setCurrentUrl(url) {
    state.currentUrl = url;
}

/**
 * Setter nåværende tittel
 * @param {string|null} title - Sidetittel
 */
function setCurrentTitle(title) {
    state.currentTitle = title;
}

/**
 * Setter brukerinnstillinger
 * @param {Object} settings - Innstillingsobjekt
 */
function setSettings(settings) {
    state.settings = settings;
}

/**
 * Henter brukerinnstillinger
 * @returns {Object|null} Innstillinger
 */
function getSettings() {
    return state.settings;
}

// ===== History Management =====

/**
 * Legger til en sti i historikken
 * @param {string} path - Sti å legge til
 */
function addToHistory(path) {
    // Fjern fremtidig historikk hvis vi har navigert tilbake
    if (state.historyIndex < state.history.length - 1) {
        state.history = state.history.slice(0, state.historyIndex + 1);
    }
    
    state.history.push(path);
    state.historyIndex = state.history.length - 1;
    state.currentPath = path;
    
    // Begrens historikk-størrelse
    if (state.history.length > MAX_HISTORY_SIZE) {
        state.history.shift();
        state.historyIndex--;
    }
}

/**
 * Kan vi gå tilbake i historikken?
 * @returns {boolean}
 */
function canGoBack() {
    return state.historyIndex > 0;
}

/**
 * Kan vi gå fremover i historikken?
 * @returns {boolean}
 */
function canGoForward() {
    return state.historyIndex < state.history.length - 1;
}

/**
 * Gå tilbake i historikken
 * @returns {string|null} Forrige sti eller null
 */
function historyBack() {
    if (canGoBack()) {
        state.historyIndex--;
        return state.history[state.historyIndex];
    }
    return null;
}

/**
 * Gå fremover i historikken
 * @returns {string|null} Neste sti eller null
 */
function historyForward() {
    if (canGoForward()) {
        state.historyIndex++;
        return state.history[state.historyIndex];
    }
    return null;
}

/**
 * Henter historikk-indeks
 * @returns {number}
 */
function getHistoryIndex() {
    return state.historyIndex;
}

// ===== Search State =====

/**
 * Nullstiller søketilstand
 */
function resetSearchState() {
    state.searchMatches = [];
    state.currentMatchIndex = -1;
}

/**
 * Setter søketreff
 * @param {Array} matches - Array av highlight-elementer
 */
function setSearchMatches(matches) {
    state.searchMatches = matches;
}

/**
 * Henter søketreff
 * @returns {Array}
 */
function getSearchMatches() {
    return state.searchMatches;
}

/**
 * Setter nåværende treff-indeks
 * @param {number} index
 */
function setCurrentMatchIndex(index) {
    state.currentMatchIndex = index;
}

/**
 * Henter nåværende treff-indeks
 * @returns {number}
 */
function getCurrentMatchIndex() {
    return state.currentMatchIndex;
}

/**
 * Gå til neste søketreff
 * @returns {number} Ny indeks
 */
function nextMatch() {
    if (state.searchMatches.length === 0) return -1;
    state.currentMatchIndex = (state.currentMatchIndex + 1) % state.searchMatches.length;
    return state.currentMatchIndex;
}

/**
 * Gå til forrige søketreff
 * @returns {number} Ny indeks
 */
function prevMatch() {
    if (state.searchMatches.length === 0) return -1;
    state.currentMatchIndex = (state.currentMatchIndex - 1 + state.searchMatches.length) % state.searchMatches.length;
    return state.currentMatchIndex;
}
