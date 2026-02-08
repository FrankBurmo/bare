/**
 * Bare Browser - Navigation
 * 
 * Håndterer navigasjon, URL-lasting og filåpning.
 */

const { invoke: invokeNav } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;

// ===== Home =====

/**
 * Navigerer til startsiden
 */
async function goHome() {
    showLoading();
    startFooterLoading();
    updateFooterStatus('Laster startside...');
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
        showError(`Kunne ikke laste startsiden: ${error}`);
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
    updateFooterStatus('Laster på nytt...');
    
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
        await loadUrl(path, addHistory);
        return;
    }
    
    // Gemini-URLer
    if (path.startsWith(GEMINI_SCHEME)) {
        await loadGeminiUrl(path, addHistory);
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
        renderContent(result.html, result.title, result.was_converted);
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
        
        if (result.was_converted) {
            showStatus('HTML konvertert til markdown', false);
        } else {
            showStatus('Markdown lastet fra nettverket', false);
        }
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
                showError('Konvertering avbrutt av brukeren');
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
        renderContent(result.html, result.title, true);
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
        showStatus('HTML konvertert til markdown', false);
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
        await loadUrl(href);
        return;
    }
    
    // Gemini-URLer
    if (href.startsWith(GEMINI_SCHEME)) {
        await loadGeminiUrl(href);
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
            } else {
                await loadUrl(resolvedUrl);
            }
        } catch (error) {
            showError(`Kunne ikke løse URL: ${error}`);
        }
    } else if (currentPath) {
        // Lokal fil - løs relativt til den
        const basePath = currentPath.substring(0, currentPath.lastIndexOf(/[\\/]/) + 1);
        const newPath = basePath + href;
        await loadPath(newPath);
    } else {
        showError('Kan ikke navigere til relativ lenke uten en base-URL');
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
        renderContent(result.html, result.title, result.was_converted);
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
        showStatus('Gemini-side lastet', false);
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
        renderContent(result.html, result.title, result.was_converted);
        setCurrentPath(null);
        setCurrentUrl(result.url || url);
        
        if (result.url) {
            elements.urlBar.value = result.url;
        }
        
        addToHistory(result.url || url);
        updateNavigationButtons();
        updateFooter(result.url || url, true);
        updateBookmarkButton();
        showStatus('Gemini-side lastet', false);
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
        showError(`Kunne ikke åpne fil: ${error}`);
    }
}

// ===== URL Bar Handling =====

/**
 * Håndterer submit fra URL-bar
 */
async function handleUrlSubmit() {
    const input = elements.urlBar.value.trim();
    
    if (!input) {
        await goHome();
        return;
    }
    
    // Absolutte URLer
    if (input.startsWith('http://') || input.startsWith('https://')) {
        await loadUrl(input);
        return;
    }
    
    // Gemini-URLer
    if (input.startsWith(GEMINI_SCHEME)) {
        await loadGeminiUrl(input);
        return;
    }
    
    // Lokale stier
    if (input.startsWith('/') || input.match(/^[a-zA-Z]:\\/)) {
        await loadPath(input);
    } else if (input.startsWith('file://')) {
        const path = input.replace('file://', '');
        await loadPath(path);
    } else {
        // Anta HTTPS for alt annet
        const urlWithScheme = 'https://' + input;
        await loadUrl(urlWithScheme);
    }
}
