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
        console.error(t('status.loadSettingsError') + ':', error);
    }
}

/**
 * Rendrer bokmerkelisten
 * @param {Array} bookmarks - Array av bokmerke-objekter
 */
function renderBookmarksList(bookmarks) {
    if (bookmarks.length === 0) {
        elements.bookmarksList.innerHTML = `<p class="empty-message" data-i18n="bookmarks.empty">${t('bookmarks.empty')}</p>`;
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
        showStatus(t('status.noBaseUrl'), true);
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
                showStatus(t('bookmarks.removed'));
            }
        } else {
            const title = state.currentTitle || url;
            await invokeBookmarks('add_bookmark', { title, url });
            updateBookmarkButtonUI(true);
            showStatus(t('bookmarks.added'));
        }
        
        await loadBookmarks();
    } catch (error) {
        showStatus(`${t('status.settingsError')}: ${error}`, true);
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
        console.error(t('status.settingsError') + ':', error);
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
        showStatus(t('bookmarks.removed'));
    } catch (error) {
        showStatus(`${t('status.settingsError')}: ${error}`, true);
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
