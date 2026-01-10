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
    btnBack: document.getElementById('btn-back'),
    btnForward: document.getElementById('btn-forward'),
    btnHome: document.getElementById('btn-home'),
    btnOpen: document.getElementById('btn-open'),
    btnTheme: document.getElementById('btn-theme'),
};

// ===== State =====
const state = {
    history: [],
    historyIndex: -1,
    currentPath: null,
    currentUrl: null, // URL for den gjeldende siden (for relativ URL-oppløsning)
    theme: localStorage.getItem('bare-theme') || 'light',
};

// ===== Theme Management =====
function initTheme() {
    // Sjekk system preferanse hvis ingen lagret preferanse
    if (!localStorage.getItem('bare-theme')) {
        const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
        state.theme = prefersDark ? 'dark' : 'light';
    }
    applyTheme();
}

function applyTheme() {
    document.documentElement.setAttribute('data-theme', state.theme);
    elements.btnTheme.textContent = state.theme === 'dark' ? '☀️' : '🌙';
}

function toggleTheme() {
    state.theme = state.theme === 'dark' ? 'light' : 'dark';
    localStorage.setItem('bare-theme', state.theme);
    applyTheme();
}

// ===== Status Bar =====
function showStatus(message, isError = false) {
    elements.statusMessage.textContent = message;
    elements.statusBar.classList.remove('hidden', 'error');
    if (isError) {
        elements.statusBar.classList.add('error');
    }
    
    // Auto-hide etter 3 sekunder
    setTimeout(() => {
        elements.statusBar.classList.add('hidden');
    }, 3000);
}

function hideStatus() {
    elements.statusBar.classList.add('hidden');
}

// ===== Content Rendering =====
function renderContent(html, title) {
    elements.content.innerHTML = `<div class="markdown-body">${html}</div>`;
    
    // Oppdater tittel
    if (title) {
        document.title = `${title} - Bare`;
    } else {
        document.title = 'Bare';
    }
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
    // Fjern fremtidig historikk hvis vi navigerer fra midten
    if (state.historyIndex < state.history.length - 1) {
        state.history = state.history.slice(0, state.historyIndex + 1);
    }
    
    state.history.push(path);
    state.historyIndex = state.history.length - 1;
    state.currentPath = path;
    
    // Begrens historikk til 50 elementer
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
        renderContent(result.html, result.title);
        state.currentPath = null;
        state.currentUrl = result.url || url;
        
        // Oppdater URL-bar med endelig URL (etter redirects)
        if (result.url) {
            elements.urlBar.value = result.url;
        }
        
        if (addHistory) {
            addToHistory(result.url || url);
        }
        
        updateFooter(result.url || url);
        showStatus('Innhold lastet fra nettverket', false);
    } catch (error) {
        showError(error);
    }
}

// ===== Link Resolution =====
async function resolveAndNavigate(href) {
    // Sjekk om det er en absolutt URL
    if (href.startsWith('http://') || href.startsWith('https://')) {
        await loadUrl(href);
        return;
    }
    
    // Sjekk om det er en lokal fil-referanse
    if (href.startsWith('file://')) {
        const path = href.replace('file://', '');
        await loadPath(path);
        return;
    }
    
    // Relativ URL - må ha en base-URL
    if (state.currentUrl) {
        try {
            const resolvedUrl = await invoke('resolve_url', {
                baseUrl: state.currentUrl,
                relativeUrl: href
            });
            
            // Sjekk om det er en fil eller HTTP URL
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
        // Relativ sti basert på lokal fil
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
    
    // Sjekk om det er en HTTP/HTTPS URL
    if (input.startsWith('http://') || input.startsWith('https://')) {
        await loadUrl(input);
        return;
    }
    
    // Sjekk om det er en lokal fil
    if (input.startsWith('/') || input.match(/^[a-zA-Z]:\\/)) {
        await loadPath(input);
    } else if (input.startsWith('file://')) {
        const path = input.replace('file://', '');
        await loadPath(path);
    } else {
        // Prøv å behandle som URL (legg til https://)
        const urlWithScheme = 'https://' + input;
        await loadUrl(urlWithScheme);
    }
}

// ===== Footer =====
function updateFooter(path) {
    if (path && path !== '__home__') {
        const filename = path.split(/[\\/]/).pop();
        elements.footerInfo.textContent = `Bare v0.1.0 — ${filename}`;
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
    
    // Keyboard shortcuts
    document.addEventListener('keydown', (e) => {
        // Ctrl+O: Åpne fil
        if (e.ctrlKey && e.key === 'o') {
            e.preventDefault();
            openFileDialog();
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
        
        // Escape: Fjern fokus fra URL-bar
        if (e.key === 'Escape') {
            elements.urlBar.blur();
        }
    });
    
    // Handle links in content
    elements.content.addEventListener('click', async (e) => {
        const link = e.target.closest('a');
        if (link) {
            const href = link.getAttribute('href');
            
            // Interne anker-lenker
            if (href.startsWith('#')) {
                return; // La nettleseren håndtere dette
            }
            
            // Forhindre standard navigasjon
            e.preventDefault();
            
            // Naviger til lenken
            await resolveAndNavigate(href);
        }
    });
}

// ===== Initialization =====
async function init() {
    initTheme();
    initEventListeners();
    updateNavigationButtons();
    
    // Last velkomstside
    await goHome();
}

// Start applikasjonen
document.addEventListener('DOMContentLoaded', init);

// Eksporter funksjoner for global bruk
window.goHome = goHome;
window.goBack = goBack;
window.goForward = goForward;
