/**
 * Bare Browser - DOM Utilities
 * 
 * Sentralisert tilgang til DOM-elementer og utility-funksjoner.
 */

// ===== DOM Elements =====
const elements = {
    // Toolbar
    urlBar: document.getElementById('url-bar'),
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
    
    // Hovedinnhold
    content: document.getElementById('content'),
    
    // Status og footer
    statusBar: document.getElementById('status-bar'),
    statusMessage: document.getElementById('status-message'),
    footerInfo: document.getElementById('footer-info'),
    zoomLevel: document.getElementById('zoom-level'),
    
    // Søk
    searchBar: document.getElementById('search-bar'),
    searchInput: document.getElementById('search-input'),
    searchCount: document.getElementById('search-count'),
    btnSearchPrev: document.getElementById('btn-search-prev'),
    btnSearchNext: document.getElementById('btn-search-next'),
    btnSearchClose: document.getElementById('btn-search-close'),
    
    // Bokmerke-panel
    bookmarksPanel: document.getElementById('bookmarks-panel'),
    bookmarksList: document.getElementById('bookmarks-list'),
    btnCloseBookmarks: document.getElementById('btn-close-bookmarks'),
    
    // Innstillinger-panel
    settingsPanel: document.getElementById('settings-panel'),
    btnCloseSettings: document.getElementById('btn-close-settings'),
    
    // Innstillinger-kontroller
    settingTheme: document.getElementById('setting-theme'),
    settingFontFamily: document.getElementById('setting-font-family'),
    settingFontSize: document.getElementById('setting-font-size'),
    settingFontSizeValue: document.getElementById('setting-font-size-value'),
    settingContentWidth: document.getElementById('setting-content-width'),
    settingContentWidthValue: document.getElementById('setting-content-width-value'),
    settingConversionMode: document.getElementById('setting-conversion-mode'),
    settingReadability: document.getElementById('setting-readability'),
};

// ===== Utility Functions =====

/**
 * Escaper HTML for sikker visning
 * @param {string} text - Tekst å escape
 * @returns {string} Escaped HTML
 */
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

/**
 * Sjekker om et input-felt har fokus
 * @returns {boolean}
 */
function isInputFocused() {
    const active = document.activeElement;
    return active && (active.tagName === 'INPUT' || active.tagName === 'TEXTAREA' || active.isContentEditable);
}

/**
 * Henter markdown-body elementet fra content
 * @returns {Element|null}
 */
function getMarkdownBody() {
    return elements.content.querySelector('.markdown-body');
}
