/**
 * Bare Browser - Bookmark Management
 * 
 * Håndterer bokmerker: CRUD-operasjoner og UI-oppdateringer.
 */

const { invoke: invokeBookmarks } = window.__TAURI__.core;

/**
 * Laster bokmerker fra backend og oppdaterer UI
 */
async function loadBookmarks() {
    try {
        const bookmarks = await invokeBookmarks('get_bookmarks');
        renderBookmarksList(bookmarks);
    } catch (error) {
        console.error('Kunne ikke laste bokmerker:', error);
    }
}

/**
 * Rendrer bokmerkelisten
 * @param {Array} bookmarks - Array av bokmerke-objekter
 */
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

/**
 * Toggler bokmerke for nåværende side
 */
async function toggleBookmark() {
    const url = getCurrentLocation();
    if (!url || url === HOME_PATH) {
        showStatus('Kan ikke bokmerke denne siden', true);
        return;
    }
    
    try {
        const isBookmarked = await invokeBookmarks('is_bookmarked', { url });
        
        if (isBookmarked) {
            // Finn og fjern bokmerket
            const bookmarks = await invokeBookmarks('get_bookmarks');
            const bookmark = bookmarks.find(b => b.url === url);
            if (bookmark) {
                await invokeBookmarks('remove_bookmark', { id: bookmark.id });
                updateBookmarkButtonUI(false);
                showStatus('Bokmerke fjernet');
            }
        } else {
            const title = state.currentTitle || url;
            await invokeBookmarks('add_bookmark', { title, url });
            updateBookmarkButtonUI(true);
            showStatus('Bokmerke lagt til');
        }
        
        await loadBookmarks();
    } catch (error) {
        showStatus(`Kunne ikke oppdatere bokmerke: ${error}`, true);
    }
}

/**
 * Oppdaterer bokmerkeknappen basert på nåværende side
 */
async function updateBookmarkButton() {
    const url = getCurrentLocation();
    if (!url || url === HOME_PATH) {
        updateBookmarkButtonUI(false);
        return;
    }
    
    try {
        const isBookmarked = await invokeBookmarks('is_bookmarked', { url });
        updateBookmarkButtonUI(isBookmarked);
    } catch (error) {
        console.error('Kunne ikke sjekke bokmerke-status:', error);
    }
}

/**
 * Oppdaterer bokmerkeknappens utseende
 * @param {boolean} isBookmarked - Om siden er bokmerket
 */
function updateBookmarkButtonUI(isBookmarked) {
    elements.btnBookmark.textContent = isBookmarked ? '★' : '☆';
    elements.btnBookmark.classList.toggle('bookmarked', isBookmarked);
}

/**
 * Toggler bokmerke-panelet
 */
function toggleBookmarksPanel() {
    const isNowVisible = toggleBookmarksPanelUI();
    if (isNowVisible) {
        loadBookmarks();
    }
}

/**
 * Sletter et bokmerke etter ID
 * @param {string} id - Bokmerke-ID
 */
async function deleteBookmark(id) {
    try {
        await invokeBookmarks('remove_bookmark', { id });
        await loadBookmarks();
        await updateBookmarkButton();
        showStatus('Bokmerke slettet');
    } catch (error) {
        showStatus(`Kunne ikke slette bokmerke: ${error}`, true);
    }
}

/**
 * Navigerer til et bokmerke
 * @param {string} url - Bokmerke-URL
 */
async function navigateToBookmark(url) {
    closeBookmarksPanel();
    
    if (url.startsWith('file://')) {
        await loadPath(url.replace('file://', ''));
    } else if (url.startsWith('http://') || url.startsWith('https://')) {
        await loadUrl(url);
    } else {
        await loadPath(url);
    }
}
