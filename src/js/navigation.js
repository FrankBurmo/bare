/**
 * Bare Browser - Navigation
 * 
 * Håndterer navigasjon, URL-lasting og filåpning.
 */

const { invoke: invokeNav } = window.__TAURI__.core;
const { invoke: invokeSuggestions } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;

// ===== PDF Helpers =====

/**
 * Sjekker om en URL peker til en PDF-fil
 */
function isPdfUrl(url) {
    try {
        const pathname = new URL(url).pathname;
        return pathname.toLowerCase().endsWith('.pdf');
    } catch {
        return url.split('?')[0].toLowerCase().endsWith('.pdf');
    }
}

/**
 * Åpner en URL i systemets standard program
 */
async function openExternally(url) {
    await window.__TAURI__.opener.openUrl(url);
}

// ===== Home =====

/**
 * Navigerer til startsiden
 */
async function goHome() {
    showLoading();
    startFooterLoading();
    updateFooterStatus(t('footer.loadingHome'));
    try {
        const result = await invokeNav('get_welcome_content');
        renderContent(result.html, result.title);
        elements.urlBar.value = '';
        setCurrentUrl(null);
        setCurrentPath(HOME_PATH);
        addToHistory(HOME_PATH);
        updateNavigationButtons();
        updateFooter(HOME_PATH);
        updateBookmarkButton();
        stopFooterLoading();
    } catch (error) {
        showError(`${t('status.loadHomeError')}: ${error}`);
        stopFooterLoading();
    }
}

// ===== Reload =====

/**
 * Laster gjeldende side på nytt
 */
async function reloadPage() {
    const currentPath = getCurrentPath();
    if (!currentPath || currentPath === HOME_PATH) {
        await goHome();
        return;
    }
    
    // Legg til loading-animasjon på reload-knappen
    elements.btnReload.classList.add('loading');
    startFooterLoading();
    updateFooterStatus(t('footer.reloading'));
    
    try {
        await loadPath(currentPath, false);
    } finally {
        elements.btnReload.classList.remove('loading');
    }
}

// ===== Back/Forward =====

/**
 * Navigerer tilbake i historikken
 */
async function goBack() {
    const path = historyBack();
    if (path) {
        await loadPath(path, false);
    }
}

/**
 * Navigerer fremover i historikken
 */
async function goForward() {
    const path = historyForward();
    if (path) {
        await loadPath(path, false);
    }
}

// ===== File Loading =====

/**
 * Laster en fil eller URL basert på sti
 * @param {string} path - Sti eller URL å laste
 * @param {boolean} addHistory - Om stien skal legges til historikken
 */
async function loadPath(path, addHistory = true) {
    if (path === HOME_PATH) {
        await goHome();
        return;
    }
    
    // Sjekk om dette er en URL eller lokal fil
    if (path.startsWith('http://') || path.startsWith('https://')) {
        if (isPdfUrl(path)) {
            await openExternally(path);
            return;
        }
        await loadUrl(path, addHistory);
        return;
    }
    
    // Gemini-URLer
    if (path.startsWith(GEMINI_SCHEME)) {
        await loadGeminiUrl(path, addHistory);
        return;
    }
    
    // Gopher-URLer
    if (path.startsWith(GOPHER_SCHEME)) {
        await loadGopherUrl(path, addHistory);
        return;
    }
    
    // Lokal PDF-fil
    if (path.toLowerCase().endsWith('.pdf')) {
        await window.__TAURI__.opener.openPath(path);
        return;
    }
    
    showLoading();
    elements.urlBar.value = path;
    
    try {
        const result = await invokeNav('open_file', { path });
        renderContent(result.html, result.title);
        setCurrentPath(path);
        setCurrentUrl(result.url || null);
        
        if (addHistory) {
            addToHistory(path);
        }
        
        updateNavigationButtons();
        updateFooter(path);
        updateBookmarkButton();
    } catch (error) {
        showError(error);
    }
}

// ===== URL Loading =====

/**
 * Laster innhold fra en URL
 * @param {string} url - URL å laste
 * @param {boolean} addHistory - Om URL skal legges til historikken
 */
async function loadUrl(url, addHistory = true) {
    showLoading();
    startFooterLoading();
    elements.urlBar.value = url;
    
    try {
        const result = await invokeNav('fetch_url', { url });
        renderContent(result.html, result.title);
        setCurrentPath(null);
        setCurrentUrl(result.url || url);
        
        if (result.url) {
            elements.urlBar.value = result.url;
        }
        
        if (addHistory) {
            addToHistory(result.url || url);
        }
        
        updateNavigationButtons();
        updateFooter(result.url || url, result.was_converted);
        updateBookmarkButton();
    } catch (error) {
        stopFooterLoading();
        // Sjekk om dette er en konverteringsprompt
        if (typeof error === 'string' && error.startsWith(CONVERSION_PROMPT_PREFIX)) {
            // Bruk indexOf for å unngå splitting av URL-er som inneholder ':'
            const withoutPrefix = error.substring(CONVERSION_PROMPT_PREFIX.length);
            const lastColon = withoutPrefix.lastIndexOf(':http');
            let message, promptUrl;
            if (lastColon !== -1) {
                message = withoutPrefix.substring(0, lastColon);
                promptUrl = withoutPrefix.substring(lastColon + 1);
            } else {
                // Fallback: bruk opprinnelig URL
                message = withoutPrefix;
                promptUrl = url;
            }
            
            // Gjenopprett URL i adressefeltet mens brukeren velger
            elements.urlBar.value = url;
            
            if (confirm(message)) {
                await convertAndLoad(promptUrl, addHistory);
            } else {
                showError(t('status.conversionCancelled'));
            }
        } else {
            showError(error);
        }
    }
}

// ===== URL Conversion =====

/**
 * Konverterer og laster en URL
 * @param {string} url - URL å konvertere
 * @param {boolean} addHistory - Om URL skal legges til historikken
 */
async function convertAndLoad(url, addHistory = true) {
    showLoading();
    startFooterLoading();
    elements.urlBar.value = url;
    
    try {
        const result = await invokeNav('convert_url', { url });
        renderContent(result.html, result.title);
        setCurrentPath(null);
        setCurrentUrl(result.url || url);
        
        if (result.url) {
            elements.urlBar.value = result.url;
        }
        
        if (addHistory) {
            addToHistory(result.url || url);
        }
        
        updateNavigationButtons();
        updateFooter(result.url || url, true);
        updateBookmarkButton();
    } catch (error) {
        stopFooterLoading();
        showError(error);
    }
}

// ===== Link Resolution =====

/**
 * Løser og navigerer til en lenke
 * @param {string} href - Lenke å følge
 */
async function resolveAndNavigate(href) {
    // Absolutte URLer
    if (href.startsWith('http://') || href.startsWith('https://')) {
        if (isPdfUrl(href)) {
            await openExternally(href);
            return;
        }
        await loadUrl(href);
        return;
    }
    
    // Gemini-URLer
    if (href.startsWith(GEMINI_SCHEME)) {
        await loadGeminiUrl(href);
        return;
    }
    
    // Gopher-URLer
    if (href.startsWith(GOPHER_SCHEME)) {
        await loadGopherUrl(href);
        return;
    }
    
    // File URLs
    if (href.startsWith('file://')) {
        const path = href.replace('file://', '');
        await loadPath(path);
        return;
    }
    
    // Relativ URL - løs basert på nåværende lokasjon
    const currentUrl = state.currentUrl;
    const currentPath = state.currentPath;
    
    if (currentUrl) {
        try {
            // Bruk riktig resolver basert på protokoll
            let resolvedUrl;
            if (currentUrl.startsWith(GEMINI_SCHEME)) {
                resolvedUrl = await invokeNav('resolve_gemini_url', {
                    baseUrl: currentUrl,
                    relativeUrl: href
                });
            } else if (currentUrl.startsWith(GOPHER_SCHEME)) {
                resolvedUrl = await invokeNav('resolve_gopher_url', {
                    baseUrl: currentUrl,
                    relativeUrl: href
                });
            } else {
                resolvedUrl = await invokeNav('resolve_url', {
                    baseUrl: currentUrl,
                    relativeUrl: href
                });
            }
            
            if (resolvedUrl.startsWith('file://')) {
                const path = resolvedUrl.replace('file://', '');
                await loadPath(path);
            } else if (resolvedUrl.startsWith(GEMINI_SCHEME)) {
                await loadGeminiUrl(resolvedUrl);
            } else if (resolvedUrl.startsWith(GOPHER_SCHEME)) {
                await loadGopherUrl(resolvedUrl);
            } else {
                if (isPdfUrl(resolvedUrl)) {
                    await openExternally(resolvedUrl);
                    return;
                }
                await loadUrl(resolvedUrl);
            }
        } catch (error) {
            showError(`${t('status.urlResolveError')}: ${error}`);
        }
    } else if (currentPath) {
        // Lokal fil - løs relativt til den
        const basePath = currentPath.substring(0, currentPath.lastIndexOf(/[\\/]/) + 1);
        const newPath = basePath + href;
        await loadPath(newPath);
    } else {
        showError(t('status.noBaseUrl'));
    }
}

// ===== Gemini Loading =====

/**
 * Laster innhold fra en Gemini-URL
 * @param {string} url - Gemini-URL å laste (gemini://...)
 * @param {boolean} addHistory - Om URL skal legges til historikken
 */
async function loadGeminiUrl(url, addHistory = true) {
    showLoading();
    startFooterLoading();
    elements.urlBar.value = url;
    
    try {
        const result = await invokeNav('fetch_gemini', { url });
        renderContent(result.html, result.title);
        setCurrentPath(null);
        setCurrentUrl(result.url || url);
        
        if (result.url) {
            elements.urlBar.value = result.url;
        }
        
        if (addHistory) {
            addToHistory(result.url || url);
        }
        
        updateNavigationButtons();
        updateFooter(result.url || url, true);
        updateBookmarkButton();
    } catch (error) {
        stopFooterLoading();
        
        // Sjekk om dette er en input-prompt
        if (typeof error === 'string' && error.startsWith(GEMINI_INPUT_PROMPT_PREFIX)) {
            const prompt = error.substring(GEMINI_INPUT_PROMPT_PREFIX.length);
            showGeminiInputDialog(prompt, url, false);
        } else if (typeof error === 'string' && error.startsWith(GEMINI_SENSITIVE_INPUT_PROMPT_PREFIX)) {
            const prompt = error.substring(GEMINI_SENSITIVE_INPUT_PROMPT_PREFIX.length);
            showGeminiInputDialog(prompt, url, true);
        } else {
            showError(error);
        }
    }
}

/**
 * Sender brukerinput til en Gemini-server og laster resultatet
 * @param {string} url - Original Gemini-URL som ba om input
 * @param {string} input - Brukerens input-tekst
 */
async function submitGeminiInput(url, input) {
    showLoading();
    startFooterLoading();
    
    try {
        const result = await invokeNav('submit_gemini_input', { url, input });
        renderContent(result.html, result.title);
        setCurrentPath(null);
        setCurrentUrl(result.url || url);
        
        if (result.url) {
            elements.urlBar.value = result.url;
        }
        
        addToHistory(result.url || url);
        updateNavigationButtons();
        updateFooter(result.url || url, true);
        updateBookmarkButton();
    } catch (error) {
        stopFooterLoading();
        
        // Sjekk om dette er enda en input-prompt
        if (typeof error === 'string' && error.startsWith(GEMINI_INPUT_PROMPT_PREFIX)) {
            const prompt = error.substring(GEMINI_INPUT_PROMPT_PREFIX.length);
            showGeminiInputDialog(prompt, url, false);
        } else if (typeof error === 'string' && error.startsWith(GEMINI_SENSITIVE_INPUT_PROMPT_PREFIX)) {
            const prompt = error.substring(GEMINI_SENSITIVE_INPUT_PROMPT_PREFIX.length);
            showGeminiInputDialog(prompt, url, true);
        } else {
            showError(error);
        }
    }
}

// ===== Gopher Loading =====

/**
 * Laster innhold fra en Gopher-URL
 * @param {string} url - Gopher-URL å laste (gopher://...)
 * @param {boolean} addHistory - Om URL skal legges til historikken
 */
async function loadGopherUrl(url, addHistory = true) {
    showLoading();
    startFooterLoading();
    elements.urlBar.value = url;
    
    try {
        const result = await invokeNav('fetch_gopher', { url });
        renderContent(result.html, result.title);
        setCurrentPath(null);
        setCurrentUrl(result.url || url);
        
        if (result.url) {
            elements.urlBar.value = result.url;
        }
        
        if (addHistory) {
            addToHistory(result.url || url);
        }
        
        updateNavigationButtons();
        updateFooter(result.url || url, true);
        updateBookmarkButton();
    } catch (error) {
        stopFooterLoading();
        
        // Sjekk om dette er en søke-prompt
        if (typeof error === 'string' && error.startsWith(GOPHER_SEARCH_PROMPT_PREFIX)) {
            const searchUrl = error.substring(GOPHER_SEARCH_PROMPT_PREFIX.length);
            showGopherSearchDialog(searchUrl);
        } else {
            showError(error);
        }
    }
}

/**
 * Sender et Gopher-søk og laster resultatet
 * @param {string} url - Gopher-søke-URL
 * @param {string} query - Søkestreng
 */
async function submitGopherSearch(url, query) {
    showLoading();
    startFooterLoading();
    
    try {
        const result = await invokeNav('gopher_search', { url, query });
        renderContent(result.html, result.title);
        setCurrentPath(null);
        setCurrentUrl(result.url || url);
        
        if (result.url) {
            elements.urlBar.value = result.url;
        }
        
        addToHistory(result.url || url);
        updateNavigationButtons();
        updateFooter(result.url || url, true);
        updateBookmarkButton();
    } catch (error) {
        stopFooterLoading();
        showError(error);
    }
}

// ===== File Dialog =====

/**
 * Åpner fil-dialog for å velge en markdown-fil
 */
async function openFileDialog() {
    try {
        const selected = await open({
            multiple: false,
            filters: [{
                name: 'Markdown',
                extensions: ['md', 'markdown']
            }]
        });
        
        if (selected) {
            await loadPath(selected);
        }
    } catch (error) {
        showError(`${t('status.openFileError')}: ${error}`);
    }
}

// ===== URL Bar Handling =====

/** Debounce-timer for adressforslag */
let suggestionDebounceTimer = null;

/**
 * Håndterer submit fra URL-bar
 */
async function handleUrlSubmit() {
    const input = elements.urlBar.value.trim();
    hideSuggestions();
    
    if (!input) {
        await goHome();
        return;
    }
    
    // Sjekk om brukeren har valgt et forslag
    if (state.selectedSuggestionIndex >= 0 && state.suggestions[state.selectedSuggestionIndex]) {
        const suggestion = state.suggestions[state.selectedSuggestionIndex];
        if (suggestion.url) {
            await loadPath(suggestion.url);
        } else if (suggestion.searchUrl) {
            await loadUrl(suggestion.searchUrl);
        }
        return;
    }
    
    // Absolutte URLer
    if (input.startsWith('http://') || input.startsWith('https://')) {
        if (isPdfUrl(input)) {
            await openExternally(input);
            return;
        }
        await loadUrl(input);
        return;
    }
    
    // Gemini-URLer
    if (input.startsWith(GEMINI_SCHEME)) {
        await loadGeminiUrl(input);
        return;
    }
    
    // Gopher-URLer
    if (input.startsWith(GOPHER_SCHEME)) {
        await loadGopherUrl(input);
        return;
    }
    
    // Lokale stier
    if (input.startsWith('/') || input.match(/^[a-zA-Z]:\\/)) {
        await loadPath(input);
    } else if (input.startsWith('file://')) {
        const path = input.replace('file://', '');
        await loadPath(path);
    } else if (looksLikeUrl(input)) {
        // Ser ut som en URL - legg til https://
        const urlWithScheme = 'https://' + input;
        if (isPdfUrl(urlWithScheme)) {
            await openExternally(urlWithScheme);
            return;
        }
        await loadUrl(urlWithScheme);
    } else {
        // Ikke en URL - bruk søkemotor
        await searchWithEngine(input);
    }
}

/**
 * Sjekker om en streng ser ut som en URL
 * @param {string} input - Tastaturinput å sjekke
 * @returns {boolean} True hvis det ser ut som en URL
 */
function looksLikeUrl(input) {
    // Inneholder mellomrom = søkestreng
    if (input.includes(' ')) return false;
    // Inneholder protokoll
    if (/^[a-zA-Z]+:\/\//.test(input)) return true;
    // Inneholder domene med punktum (f.eks. example.com, example.co.uk)
    if (/^[a-zA-Z0-9]([a-zA-Z0-9-]*[a-zA-Z0-9])?(\.[a-zA-Z]{2,})+/.test(input)) return true;
    // Lokal filsti
    if (input.startsWith('/') || /^[a-zA-Z]:\\/.test(input)) return true;
    return false;
}

/**
 * Søker med den valgte søkemotoren
 * @param {string} query - Søkestreng
 */
async function searchWithEngine(query) {
    const settings = getSettings();
    const engineKey = (settings && settings.search_engine) || 'duckduckgo';
    const engine = SEARCH_ENGINES[engineKey] || SEARCH_ENGINES.duckduckgo;
    const searchUrl = engine.url + encodeURIComponent(query);
    await loadUrl(searchUrl);
}

/**
 * Oppdaterer adressforslag basert på input
 * @param {string} query - Søkestreng fra bruker
 */
async function updateSuggestions(query) {
    if (!query || query.length === 0) {
        hideSuggestions();
        return;
    }
    
    const results = [];
    const q = query.toLowerCase();
    
    // Søk i historikk
    const history = state.history || [];
    for (const entry of history) {
        const { match, score, indices } = fuzzyMatch(query, entry);
        if (match) {
            results.push({
                type: 'history',
                icon: '\u231A',
                label: entry,
                url: entry,
                score: score + 50, // Historikk får prioritet
                indices,
            });
        }
    }
    
    // Søk i bokmerker
    try {
        const bookmarks = await invokeSuggestions('get_bookmarks');
        for (const bm of bookmarks) {
            const search = bm.title + ' ' + bm.url;
            const { match, score, indices } = fuzzyMatch(query, search);
            if (match) {
                // Unngå duplikater fra historikk
                const isDuplicate = results.some(r => r.url === bm.url);
                if (!isDuplicate) {
                    results.push({
                        type: 'bookmark',
                        icon: '\u2605',
                        label: bm.title,
                        detail: bm.url,
                        url: bm.url,
                        score: score + 100, // Bokmerker får høyest prioritet
                        indices,
                    });
                }
            }
        }
    } catch (_) {}
    
    // Sorter etter score (høyest først)
    results.sort((a, b) => b.score - a.score);
    
    // Begrens antall resultater
    const limitedResults = results.slice(0, MAX_SUGGESTIONS);
    
    // Legg til søkemotor-forslag hvis input ikke er en URL
    let searchSuggestion = null;
    if (!looksLikeUrl(query) && !query.startsWith('/') && !/^[a-zA-Z]:\\/.test(query)) {
        const settings = getSettings();
        const engineKey = (settings && settings.search_engine) || 'duckduckgo';
        const engine = SEARCH_ENGINES[engineKey] || SEARCH_ENGINES.duckduckgo;
        searchSuggestion = {
            type: 'search',
            icon: '\u{1F50D}',
            label: `Søk med ${engine.name}`,
            searchUrl: engine.url + encodeURIComponent(query),
        };
    }
    
    state.suggestions = limitedResults;
    state.selectedSuggestionIndex = -1;
    
    renderSuggestions(limitedResults, searchSuggestion, query);
}

/**
 * Rendrer adressforslag i dropdown
 * @param {Array} results - Forslagsresultater
 * @param {Object|null} searchSuggestion - Søkemotor-forslag
 * @param {string} query - Opprinnelig søkestreng
 */
function renderSuggestions(results, searchSuggestion, query) {
    if (results.length === 0 && !searchSuggestion) {
        hideSuggestions();
        return;
    }
    
    let html = '';
    
    for (let i = 0; i < results.length; i++) {
        const item = results[i];
        const selected = i === state.selectedSuggestionIndex ? ' url-suggestion-item-selected' : '';
        const detail = item.detail ? `<span class="url-suggestion-detail">${escapeHtml(item.detail)}</span>` : '';
        const label = highlightMatches(item.label, item.indices);
        html += `<div class="url-suggestion-item${selected}" data-index="${i}" tabindex="-1">
            <span class="url-suggestion-icon">${item.icon}</span>
            <span class="url-suggestion-label">${label}</span>
            ${detail}
        </div>`;
    }
    
    if (searchSuggestion) {
        const searchSelected = state.selectedSuggestionIndex === results.length ? ' url-suggestion-search-selected' : '';
        html += `<div class="url-suggestion-search${searchSelected}" data-index="${results.length}" tabindex="-1">
            <span class="url-suggestion-search-icon">${searchSuggestion.icon}</span>
            <span class="url-suggestion-label">${escapeHtml(searchSuggestion.label)}</span>
        </div>`;
    }
    
    elements.urlSuggestions.innerHTML = html;
    elements.urlSuggestions.classList.remove('hidden');
    state.suggestionsVisible = true;
    
    // Legg til klikk-handlere
    const items = elements.urlSuggestions.querySelectorAll('.url-suggestion-item, .url-suggestion-search');
    items.forEach((el) => {
        el.addEventListener('click', () => {
            const idx = parseInt(el.dataset.index);
            selectSuggestion(idx);
        });
    });
}

/**
 * Skjuler adressforslag-dropdown
 */
function hideSuggestions() {
    elements.urlSuggestions.classList.add('hidden');
    elements.urlSuggestions.innerHTML = '';
    state.suggestions = [];
    state.selectedSuggestionIndex = -1;
    state.suggestionsVisible = false;
}

/**
 * Velger et forslag
 * @param {number} index - Indeks i forslagslisten
 */
function selectSuggestion(index) {
    const allItems = [...state.suggestions];
    
    // Sjekk om det er søkemotor-forslag
    const searchItem = allItems.length <= index ? {
        type: 'search',
        searchUrl: elements.urlSuggestions.querySelector('.url-suggestion-search')?.dataset?.searchUrl,
    } : null;
    
    if (searchItem && searchItem.searchUrl) {
        hideSuggestions();
        loadUrl(searchItem.searchUrl);
        return;
    }
    
    if (index >= 0 && index < state.suggestions.length) {
        const suggestion = state.suggestions[index];
        hideSuggestions();
        if (suggestion.url) {
            loadPath(suggestion.url);
        }
    }
}

/**
 * Håndterer tastatur navigasjon i adressforslag
 * @param {KeyboardEvent} e - Tastaturhendelse
 * @returns {boolean} True hvis hendelsen ble håndtert
 */
function handleSuggestionsKeydown(e) {
    if (!state.suggestionsVisible) return false;
    
    const totalItems = state.suggestions.length + 
        (elements.urlSuggestions.querySelector('.url-suggestion-search') ? 1 : 0);
    
    if (e.key === 'ArrowDown') {
        e.preventDefault();
        state.selectedSuggestionIndex = Math.min(
            state.selectedSuggestionIndex + 1,
            totalItems - 1
        );
        updateSuggestionSelection();
        return true;
    }
    
    if (e.key === 'ArrowUp') {
        e.preventDefault();
        state.selectedSuggestionIndex = Math.max(
            state.selectedSuggestionIndex - 1,
            -1
        );
        updateSuggestionSelection();
        return true;
    }
    
    if (e.key === 'Escape') {
        e.preventDefault();
        hideSuggestions();
        return true;
    }
    
    return false;
}

/**
 * Oppdaterer visuelt valg av forslag
 */
function updateSuggestionSelection() {
    const items = elements.urlSuggestions.querySelectorAll('.url-suggestion-item, .url-suggestion-search');
    items.forEach((el, i) => {
        el.classList.toggle('url-suggestion-item-selected', i === state.selectedSuggestionIndex);
        el.classList.toggle('url-suggestion-search-selected', i === state.selectedSuggestionIndex);
    });
    
    // Rull til valgt element
    const selected = elements.urlSuggestions.querySelector('.url-suggestion-item-selected, .url-suggestion-search-selected');
    if (selected) {
        selected.scrollIntoView({ block: 'nearest' });
    }
}

/**
 * Håndterer input-endringer i URL-bar med debounce
 */
function handleUrlBarInput() {
    const query = elements.urlBar.value;
    
    if (suggestionDebounceTimer) {
        clearTimeout(suggestionDebounceTimer);
    }
    
    suggestionDebounceTimer = setTimeout(() => {
        updateSuggestions(query);
    }, SUGGESTION_DEBOUNCE_MS);
}
