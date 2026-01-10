/**
 * Bare Browser - Frontend Application
 * 
 * Minimal JavaScript for å håndtere UI-interaksjoner og kommunikasjon med Tauri backend.
 */

// ===== Tauri API =====
const { invoke } = window.__TAURI__.core;
const { open } = window.__TAURI__.dialog;

// ===== DOM Elements =====
const elements = {
    urlBar: document.getElementById('url-bar'),
    content: document.getElementById('content'),
    statusBar: document.getElementById('status-bar'),
    statusMessage: document.getElementById('status-message'),
    footerInfo: document.getElementById('footer-info'),
    zoomLevel: document.getElementById('zoom-level'),
    btnBack: document.getElementById('btn-back'),
    btnForward: document.getElementById('btn-forward'),
    btnHome: document.getElementById('btn-home'),
    btnOpen: document.getElementById('btn-open'),
    btnTheme: document.getElementById('btn-theme'),
    btnBookmark: document.getElementById('btn-bookmark'),
    btnBookmarks: document.getElementById('btn-bookmarks'),
    btnZoomIn: document.getElementById('btn-zoom-in'),
    btnZoomOut: document.getElementById('btn-zoom-out'),
    btnSettings: document.getElementById('btn-settings'),
    // Søk
    searchBar: document.getElementById('search-bar'),
    searchInput: document.getElementById('search-input'),
    searchCount: document.getElementById('search-count'),
    btnSearchPrev: document.getElementById('btn-search-prev'),
    btnSearchNext: document.getElementById('btn-search-next'),
    btnSearchClose: document.getElementById('btn-search-close'),
    // Paneler
    bookmarksPanel: document.getElementById('bookmarks-panel'),
    bookmarksList: document.getElementById('bookmarks-list'),
    btnCloseBookmarks: document.getElementById('btn-close-bookmarks'),
    settingsPanel: document.getElementById('settings-panel'),
    btnCloseSettings: document.getElementById('btn-close-settings'),
    // Innstillinger
    settingTheme: document.getElementById('setting-theme'),
    settingFontFamily: document.getElementById('setting-font-family'),
    settingFontSize: document.getElementById('setting-font-size'),
    settingFontSizeValue: document.getElementById('setting-font-size-value'),
    settingContentWidth: document.getElementById('setting-content-width'),
    settingContentWidthValue: document.getElementById('setting-content-width-value'),
    settingConversionMode: document.getElementById('setting-conversion-mode'),
    settingReadability: document.getElementById('setting-readability'),
};

// ===== State =====
const state = {
    history: [],
    historyIndex: -1,
    currentPath: null,
    currentUrl: null,
    currentTitle: null,
    settings: null,
    // Søk
    searchMatches: [],
    currentMatchIndex: -1,
    originalContent: '',
};

// ===== Settings Management =====
async function loadSettings() {
    try {
        state.settings = await invoke('get_settings');
        applySettings();
    } catch (error) {
        console.error('Kunne ikke laste innstillinger:', error);
        state.settings = {
            theme: 'light',
            font_size: 100,
            zoom: 100,
            font_family: 'system',
            content_width: 800,
            conversion_mode: 'convert-all',
            readability_enabled: true,
        };
    }
}

function applySettings() {
    if (!state.settings) return;
    
    // Tema
    let effectiveTheme = state.settings.theme;
    if (effectiveTheme === 'system') {
        effectiveTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    document.documentElement.setAttribute('data-theme', effectiveTheme);
    elements.btnTheme.textContent = effectiveTheme === 'dark' ? '☀️' : '🌙';
    
    // Skriftstørrelse
    document.documentElement.style.setProperty('--base-font-size', `${state.settings.font_size}%`);
    document.body.style.fontSize = `${state.settings.font_size}%`;
    
    // Zoom
    elements.content.style.transform = `scale(${state.settings.zoom / 100})`;
    elements.content.style.transformOrigin = 'top center';
    elements.zoomLevel.textContent = `${state.settings.zoom}%`;
    
    // Skrifttype
    document.body.className = `font-${state.settings.font_family}`;
    
    // Innholdsbredde
    document.documentElement.style.setProperty('--content-max-width', `${state.settings.content_width}px`);
    
    // Oppdater innstillingspanel
    if (elements.settingTheme) {
        elements.settingTheme.value = state.settings.theme;
    }
    if (elements.settingFontFamily) {
        elements.settingFontFamily.value = state.settings.font_family;
    }
    if (elements.settingFontSize) {
        elements.settingFontSize.value = state.settings.font_size;
        elements.settingFontSizeValue.textContent = `${state.settings.font_size}%`;
    }
    if (elements.settingContentWidth) {
        elements.settingContentWidth.value = state.settings.content_width;
        elements.settingContentWidthValue.textContent = `${state.settings.content_width}px`;
    }
    
    // Konverteringsinnstillinger
    if (elements.settingConversionMode) {
        elements.settingConversionMode.value = state.settings.conversion_mode;
    }
    if (elements.settingReadability) {
        elements.settingReadability.checked = state.settings.readability_enabled;
    }
}

async function updateSetting(key, value) {
    try {
        const params = {};
        params[key] = value;
        state.settings = await invoke('update_settings', { params });
        applySettings();
    } catch (error) {
        showStatus(`Kunne ikke oppdatere innstilling: ${error}`, true);
    }
}

async function toggleTheme() {
    const themes = ['light', 'dark', 'system'];
    const currentIndex = themes.indexOf(state.settings.theme);
    const nextTheme = themes[(currentIndex + 1) % themes.length];
    await updateSetting('theme', nextTheme);
}

async function zoomIn() {
    try {
        state.settings = await invoke('zoom_in');
        applySettings();
    } catch (error) {
        showStatus(`Kunne ikke zoome inn: ${error}`, true);
    }
}

async function zoomOut() {
    try {
        state.settings = await invoke('zoom_out');
        applySettings();
    } catch (error) {
        showStatus(`Kunne ikke zoome ut: ${error}`, true);
    }
}

async function zoomReset() {
    try {
        state.settings = await invoke('zoom_reset');
        applySettings();
    } catch (error) {
        showStatus(`Kunne ikke tilbakestille zoom: ${error}`, true);
    }
}

// ===== Bookmark Management =====
async function loadBookmarks() {
    try {
        const bookmarks = await invoke('get_bookmarks');
        renderBookmarksList(bookmarks);
    } catch (error) {
        console.error('Kunne ikke laste bokmerker:', error);
    }
}

function renderBookmarksList(bookmarks) {
    if (bookmarks.length === 0) {
        elements.bookmarksList.innerHTML = '<p class="empty-message">Ingen bokmerker ennå</p>';
        return;
    }
    
    elements.bookmarksList.innerHTML = bookmarks.map(b => `
        <div class="bookmark-item" data-url="${escapeHtml(b.url)}" data-id="${b.id}">
            <div class="bookmark-info">
                <div class="bookmark-title">${escapeHtml(b.title)}</div>
                <div class="bookmark-url">${escapeHtml(b.url)}</div>
            </div>
            <button class="bookmark-delete" data-id="${b.id}" title="Slett">✕</button>
        </div>
    `).join('');
}

async function toggleBookmark() {
    const url = state.currentUrl || state.currentPath;
    if (!url || url === '__home__') {
        showStatus('Kan ikke bokmerke denne siden', true);
        return;
    }
    
    try {
        const isBookmarked = await invoke('is_bookmarked', { url });
        
        if (isBookmarked) {
            // Finn og fjern bokmerket
            const bookmarks = await invoke('get_bookmarks');
            const bookmark = bookmarks.find(b => b.url === url);
            if (bookmark) {
                await invoke('remove_bookmark', { id: bookmark.id });
                elements.btnBookmark.textContent = '☆';
                elements.btnBookmark.classList.remove('bookmarked');
                showStatus('Bokmerke fjernet');
            }
        } else {
            const title = state.currentTitle || url;
            await invoke('add_bookmark', { title, url });
            elements.btnBookmark.textContent = '★';
            elements.btnBookmark.classList.add('bookmarked');
            showStatus('Bokmerke lagt til');
        }
        
        await loadBookmarks();
    } catch (error) {
        showStatus(`Kunne ikke oppdatere bokmerke: ${error}`, true);
    }
}

async function updateBookmarkButton() {
    const url = state.currentUrl || state.currentPath;
    if (!url || url === '__home__') {
        elements.btnBookmark.textContent = '☆';
        elements.btnBookmark.classList.remove('bookmarked');
        return;
    }
    
    try {
        const isBookmarked = await invoke('is_bookmarked', { url });
        elements.btnBookmark.textContent = isBookmarked ? '★' : '☆';
        elements.btnBookmark.classList.toggle('bookmarked', isBookmarked);
    } catch (error) {
        console.error('Kunne ikke sjekke bokmerke-status:', error);
    }
}

function toggleBookmarksPanel() {
    const isVisible = !elements.bookmarksPanel.classList.contains('hidden');
    elements.bookmarksPanel.classList.toggle('hidden', isVisible);
    elements.settingsPanel.classList.add('hidden');
    
    if (!isVisible) {
        loadBookmarks();
    }
}

function toggleSettingsPanel() {
    const isVisible = !elements.settingsPanel.classList.contains('hidden');
    elements.settingsPanel.classList.toggle('hidden', isVisible);
    elements.bookmarksPanel.classList.add('hidden');
}

// ===== Search Functionality =====
function openSearch() {
    elements.searchBar.classList.remove('hidden');
    elements.searchInput.focus();
    elements.searchInput.select();
}

function closeSearch() {
    elements.searchBar.classList.add('hidden');
    clearSearchHighlights();
    state.searchMatches = [];
    state.currentMatchIndex = -1;
    elements.searchCount.textContent = '';
}

function clearSearchHighlights() {
    const highlights = elements.content.querySelectorAll('.search-highlight');
    highlights.forEach(el => {
        const parent = el.parentNode;
        parent.replaceChild(document.createTextNode(el.textContent), el);
        parent.normalize();
    });
}

function performSearch() {
    const query = elements.searchInput.value.trim().toLowerCase();
    
    clearSearchHighlights();
    state.searchMatches = [];
    state.currentMatchIndex = -1;
    
    if (!query) {
        elements.searchCount.textContent = '';
        return;
    }
    
    const markdownBody = elements.content.querySelector('.markdown-body');
    if (!markdownBody) return;
    
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
                state.searchMatches.push(highlight);
                
                // Oppdater node-referansen for videre søk
                node = highlight.nextSibling;
                if (!node) break;
                startIndex = 0;
            } catch (e) {
                startIndex = index + 1;
            }
        }
    });
    
    if (state.searchMatches.length > 0) {
        state.currentMatchIndex = 0;
        highlightCurrentMatch();
    }
    
    updateSearchCount();
}

function highlightCurrentMatch() {
    state.searchMatches.forEach((el, i) => {
        el.classList.toggle('current', i === state.currentMatchIndex);
    });
    
    if (state.searchMatches[state.currentMatchIndex]) {
        state.searchMatches[state.currentMatchIndex].scrollIntoView({
            behavior: 'smooth',
            block: 'center'
        });
    }
}

function updateSearchCount() {
    if (state.searchMatches.length === 0) {
        elements.searchCount.textContent = 'Ingen treff';
    } else {
        elements.searchCount.textContent = `${state.currentMatchIndex + 1} av ${state.searchMatches.length}`;
    }
}

function searchNext() {
    if (state.searchMatches.length === 0) return;
    state.currentMatchIndex = (state.currentMatchIndex + 1) % state.searchMatches.length;
    highlightCurrentMatch();
    updateSearchCount();
}

function searchPrev() {
    if (state.searchMatches.length === 0) return;
    state.currentMatchIndex = (state.currentMatchIndex - 1 + state.searchMatches.length) % state.searchMatches.length;
    highlightCurrentMatch();
    updateSearchCount();
}

// ===== Status Bar =====
function showStatus(message, isError = false) {
    elements.statusMessage.textContent = message;
    elements.statusBar.classList.remove('hidden', 'error');
    if (isError) {
        elements.statusBar.classList.add('error');
    }
    
    setTimeout(() => {
        elements.statusBar.classList.add('hidden');
    }, 3000);
}

function hideStatus() {
    elements.statusBar.classList.add('hidden');
}

// ===== Content Rendering =====
function renderContent(html, title, wasConverted = false) {
    const convertedBadge = wasConverted ? '<span class="converted-badge" title="Konvertert fra HTML">📄→📝</span>' : '';
    elements.content.innerHTML = `<div class="markdown-body">${convertedBadge}${html}</div>`;
    state.currentTitle = title;
    
    if (title) {
        document.title = `${title} - Bare`;
    } else {
        document.title = 'Bare';
    }
    
    updateBookmarkButton();
}

function showError(message) {
    elements.content.innerHTML = `
        <div class="markdown-body">
            <h1>⚠️ Feil</h1>
            <p>${escapeHtml(message)}</p>
            <p><a href="#" onclick="goHome(); return false;">Gå til startsiden</a></p>
        </div>
    `;
    showStatus(message, true);
}

function showLoading() {
    elements.content.innerHTML = '<div class="loading"><p>Laster...</p></div>';
}

// ===== Navigation =====
function updateNavigationButtons() {
    elements.btnBack.disabled = state.historyIndex <= 0;
    elements.btnForward.disabled = state.historyIndex >= state.history.length - 1;
}

function addToHistory(path) {
    if (state.historyIndex < state.history.length - 1) {
        state.history = state.history.slice(0, state.historyIndex + 1);
    }
    
    state.history.push(path);
    state.historyIndex = state.history.length - 1;
    state.currentPath = path;
    
    if (state.history.length > 50) {
        state.history.shift();
        state.historyIndex--;
    }
    
    updateNavigationButtons();
}

async function goBack() {
    if (state.historyIndex > 0) {
        state.historyIndex--;
        const path = state.history[state.historyIndex];
        await loadPath(path, false);
    }
}

async function goForward() {
    if (state.historyIndex < state.history.length - 1) {
        state.historyIndex++;
        const path = state.history[state.historyIndex];
        await loadPath(path, false);
    }
}

async function goHome() {
    showLoading();
    try {
        const result = await invoke('get_welcome_content');
        renderContent(result.html, result.title);
        elements.urlBar.value = '';
        state.currentUrl = null;
        state.currentPath = '__home__';
        addToHistory('__home__');
    } catch (error) {
        showError(`Kunne ikke laste startsiden: ${error}`);
    }
}

// ===== File Loading =====
async function loadPath(path, addHistory = true) {
    if (path === '__home__') {
        await goHome();
        return;
    }
    
    // Sjekk om dette er en URL eller lokal fil
    if (path.startsWith('http://') || path.startsWith('https://')) {
        // Dette er en URL - bruk loadUrl
        await loadUrl(path, addHistory);
        return;
    }
    
    showLoading();
    elements.urlBar.value = path;
    
    try {
        const result = await invoke('open_file', { path });
        renderContent(result.html, result.title);
        state.currentPath = path;
        state.currentUrl = result.url || null;
        
        if (addHistory) {
            addToHistory(path);
        }
        
        updateFooter(path);
    } catch (error) {
        showError(error);
    }
}

// ===== URL Loading =====
async function loadUrl(url, addHistory = true) {
    showLoading();
    elements.urlBar.value = url;
    
    try {
        const result = await invoke('fetch_url', { url });
        renderContent(result.html, result.title, result.was_converted);
        state.currentPath = null;
        state.currentUrl = result.url || url;
        
        if (result.url) {
            elements.urlBar.value = result.url;
        }
        
        if (addHistory) {
            addToHistory(result.url || url);
        }
        
        updateFooter(result.url || url, result.was_converted);
        
        if (result.was_converted) {
            showStatus('HTML konvertert til markdown', false);
        } else {
            showStatus('Markdown lastet fra nettverket', false);
        }
    } catch (error) {
        // Sjekk om dette er en konverteringsprompt
        if (typeof error === 'string' && error.startsWith('CONVERSION_PROMPT:')) {
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

// ===== Konvertering =====
async function convertAndLoad(url, addHistory = true) {
    showLoading();
    elements.urlBar.value = url;
    
    try {
        const result = await invoke('convert_url', { url });
        renderContent(result.html, result.title, true);
        state.currentPath = null;
        state.currentUrl = result.url || url;
        
        if (result.url) {
            elements.urlBar.value = result.url;
        }
        
        if (addHistory) {
            addToHistory(result.url || url);
        }
        
        updateFooter(result.url || url, true);
        showStatus('HTML konvertert til markdown', false);
    } catch (error) {
        showError(error);
    }
}

// ===== Link Resolution =====
async function resolveAndNavigate(href) {
    if (href.startsWith('http://') || href.startsWith('https://')) {
        await loadUrl(href);
        return;
    }
    
    if (href.startsWith('file://')) {
        const path = href.replace('file://', '');
        await loadPath(path);
        return;
    }
    
    if (state.currentUrl) {
        try {
            const resolvedUrl = await invoke('resolve_url', {
                baseUrl: state.currentUrl,
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
    } else if (state.currentPath) {
        const basePath = state.currentPath.substring(0, state.currentPath.lastIndexOf(/[\\/]/) + 1);
        const newPath = basePath + href;
        await loadPath(newPath);
    } else {
        showError('Kan ikke navigere til relativ lenke uten en base-URL');
    }
}

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
async function handleUrlSubmit() {
    const input = elements.urlBar.value.trim();
    
    if (!input) {
        await goHome();
        return;
    }
    
    if (input.startsWith('http://') || input.startsWith('https://')) {
        await loadUrl(input);
        return;
    }
    
    if (input.startsWith('/') || input.match(/^[a-zA-Z]:\\/)) {
        await loadPath(input);
    } else if (input.startsWith('file://')) {
        const path = input.replace('file://', '');
        await loadPath(path);
    } else {
        const urlWithScheme = 'https://' + input;
        await loadUrl(urlWithScheme);
    }
}

// ===== Footer =====
function updateFooter(path, wasConverted = false) {
    if (path && path !== '__home__') {
        const filename = path.split(/[\\/]/).pop();
        const conversionIndicator = wasConverted ? ' (konvertert)' : '';
        elements.footerInfo.textContent = `Bare v0.1.0 — ${filename}${conversionIndicator}`;
    } else {
        elements.footerInfo.textContent = 'Bare v0.1.0';
    }
}

// ===== Utilities =====
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// ===== Event Listeners =====
function initEventListeners() {
    // URL bar
    elements.urlBar.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            handleUrlSubmit();
        }
    });
    
    // Navigation buttons
    elements.btnBack.addEventListener('click', goBack);
    elements.btnForward.addEventListener('click', goForward);
    elements.btnHome.addEventListener('click', goHome);
    elements.btnOpen.addEventListener('click', openFileDialog);
    elements.btnTheme.addEventListener('click', toggleTheme);
    
    // Bokmerker
    elements.btnBookmark.addEventListener('click', toggleBookmark);
    elements.btnBookmarks.addEventListener('click', toggleBookmarksPanel);
    elements.btnCloseBookmarks.addEventListener('click', () => elements.bookmarksPanel.classList.add('hidden'));
    
    // Bokmerke-liste klikk
    elements.bookmarksList.addEventListener('click', async (e) => {
        const deleteBtn = e.target.closest('.bookmark-delete');
        if (deleteBtn) {
            e.stopPropagation();
            const id = deleteBtn.dataset.id;
            try {
                await invoke('remove_bookmark', { id });
                await loadBookmarks();
                await updateBookmarkButton();
                showStatus('Bokmerke slettet');
            } catch (error) {
                showStatus(`Kunne ikke slette bokmerke: ${error}`, true);
            }
            return;
        }
        
        const item = e.target.closest('.bookmark-item');
        if (item) {
            const url = item.dataset.url;
            elements.bookmarksPanel.classList.add('hidden');
            
            if (url.startsWith('file://')) {
                await loadPath(url.replace('file://', ''));
            } else if (url.startsWith('http://') || url.startsWith('https://')) {
                await loadUrl(url);
            } else {
                await loadPath(url);
            }
        }
    });
    
    // Zoom
    elements.btnZoomIn.addEventListener('click', zoomIn);
    elements.btnZoomOut.addEventListener('click', zoomOut);
    
    // Innstillinger
    elements.btnSettings.addEventListener('click', toggleSettingsPanel);
    elements.btnCloseSettings.addEventListener('click', () => elements.settingsPanel.classList.add('hidden'));
    
    elements.settingTheme.addEventListener('change', (e) => updateSetting('theme', e.target.value));
    elements.settingFontFamily.addEventListener('change', (e) => updateSetting('font_family', e.target.value));
    elements.settingFontSize.addEventListener('input', (e) => {
        elements.settingFontSizeValue.textContent = `${e.target.value}%`;
    });
    elements.settingFontSize.addEventListener('change', (e) => updateSetting('font_size', parseInt(e.target.value)));
    elements.settingContentWidth.addEventListener('input', (e) => {
        elements.settingContentWidthValue.textContent = `${e.target.value}px`;
    });
    elements.settingContentWidth.addEventListener('change', (e) => updateSetting('content_width', parseInt(e.target.value)));
    
    // Konverteringsinnstillinger
    elements.settingConversionMode.addEventListener('change', (e) => updateSetting('conversion_mode', e.target.value));
    elements.settingReadability.addEventListener('change', (e) => updateSetting('readability_enabled', e.target.checked));
    
    // Søk
    elements.searchInput.addEventListener('input', performSearch);
    elements.searchInput.addEventListener('keydown', (e) => {
        if (e.key === 'Enter') {
            if (e.shiftKey) {
                searchPrev();
            } else {
                searchNext();
            }
        }
        if (e.key === 'Escape') {
            closeSearch();
        }
    });
    elements.btnSearchNext.addEventListener('click', searchNext);
    elements.btnSearchPrev.addEventListener('click', searchPrev);
    elements.btnSearchClose.addEventListener('click', closeSearch);
    
    // Keyboard shortcuts
    document.addEventListener('keydown', (e) => {
        // Ctrl+O: Åpne fil
        if (e.ctrlKey && e.key === 'o') {
            e.preventDefault();
            openFileDialog();
        }
        
        // Ctrl+F: Søk
        if (e.ctrlKey && e.key === 'f') {
            e.preventDefault();
            openSearch();
        }
        
        // Ctrl+D: Bokmerke
        if (e.ctrlKey && e.key === 'd') {
            e.preventDefault();
            toggleBookmark();
        }
        
        // Ctrl+B: Vis bokmerker
        if (e.ctrlKey && e.key === 'b') {
            e.preventDefault();
            toggleBookmarksPanel();
        }
        
        // Ctrl++: Zoom inn
        if (e.ctrlKey && (e.key === '+' || e.key === '=')) {
            e.preventDefault();
            zoomIn();
        }
        
        // Ctrl+-: Zoom ut
        if (e.ctrlKey && e.key === '-') {
            e.preventDefault();
            zoomOut();
        }
        
        // Ctrl+0: Tilbakestill zoom
        if (e.ctrlKey && e.key === '0') {
            e.preventDefault();
            zoomReset();
        }
        
        // Alt+Left: Tilbake
        if (e.altKey && e.key === 'ArrowLeft') {
            e.preventDefault();
            goBack();
        }
        
        // Alt+Right: Fremover
        if (e.altKey && e.key === 'ArrowRight') {
            e.preventDefault();
            goForward();
        }
        
        // Ctrl+L: Fokuser URL-bar
        if (e.ctrlKey && e.key === 'l') {
            e.preventDefault();
            elements.urlBar.focus();
            elements.urlBar.select();
        }
        
        // g: Gå hjem (kun når ikke i input)
        if (e.key === 'g' && !isInputFocused()) {
            e.preventDefault();
            goHome();
        }
        
        // j: Scroll ned
        if (e.key === 'j' && !isInputFocused()) {
            elements.content.scrollBy({ top: 100, behavior: 'smooth' });
        }
        
        // k: Scroll opp
        if (e.key === 'k' && !isInputFocused()) {
            elements.content.scrollBy({ top: -100, behavior: 'smooth' });
        }
        
        // G: Scroll til bunnen
        if (e.key === 'G' && !isInputFocused()) {
            elements.content.scrollTo({ top: elements.content.scrollHeight, behavior: 'smooth' });
        }
        
        // gg: Scroll til toppen (vi bruker bare Home-tasten her)
        if (e.key === 'Home' && !isInputFocused()) {
            elements.content.scrollTo({ top: 0, behavior: 'smooth' });
        }
        
        // Escape: Lukk paneler og søk
        if (e.key === 'Escape') {
            elements.urlBar.blur();
            elements.bookmarksPanel.classList.add('hidden');
            elements.settingsPanel.classList.add('hidden');
            closeSearch();
        }
    });
    
    // Handle links in content
    elements.content.addEventListener('click', async (e) => {
        const link = e.target.closest('a');
        if (link) {
            const href = link.getAttribute('href');
            
            if (href.startsWith('#')) {
                return;
            }
            
            e.preventDefault();
            await resolveAndNavigate(href);
        }
    });
}

function isInputFocused() {
    const active = document.activeElement;
    return active && (active.tagName === 'INPUT' || active.tagName === 'TEXTAREA' || active.isContentEditable);
}

// ===== Initialization =====
async function init() {
    await loadSettings();
    initEventListeners();
    updateNavigationButtons();
    await goHome();
}

// Start applikasjonen
document.addEventListener('DOMContentLoaded', init);

// Eksporter funksjoner for global bruk
window.goHome = goHome;
window.goBack = goBack;
window.goForward = goForward;
