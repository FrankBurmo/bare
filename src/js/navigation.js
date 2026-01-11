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
    } catch (error) {
        showError(`Kunne ikke laste startsiden: ${error}`);
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
        // Sjekk om dette er en konverteringsprompt
        if (typeof error === 'string' && error.startsWith(CONVERSION_PROMPT_PREFIX)) {
            const parts = error.split(':');
            const message = parts[1];
            const promptUrl = parts[2];
            
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
            const resolvedUrl = await invokeNav('resolve_url', {
                baseUrl: currentUrl,
                relativeUrl: href
            });
            
            if (resolvedUrl.startsWith('file://')) {
                const path = resolvedUrl.replace('file://', '');
                await loadPath(path);
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
