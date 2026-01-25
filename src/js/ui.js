/**
 * Bare Browser - UI Management
 * 
 * Håndterer status bar, loading states, footer og panel-visning.
 */

// ===== Status Bar =====

let statusTimeout = null;

/**
 * Viser en statusmelding
 * @param {string} message - Melding å vise
 * @param {boolean} isError - Om det er en feilmelding
 */
function showStatus(message, isError = false) {
    // Rydd opp tidligere timeout
    if (statusTimeout) {
        clearTimeout(statusTimeout);
    }
    
    elements.statusMessage.textContent = message;
    elements.statusBar.classList.remove('hidden', 'error');
    
    if (isError) {
        elements.statusBar.classList.add('error');
    }
    
    statusTimeout = setTimeout(() => {
        elements.statusBar.classList.add('hidden');
        statusTimeout = null;
    }, STATUS_TIMEOUT);
}

/**
 * Skjuler statusmeldingen umiddelbart
 */
function hideStatus() {
    if (statusTimeout) {
        clearTimeout(statusTimeout);
        statusTimeout = null;
    }
    elements.statusBar.classList.add('hidden');
}

// ===== Content States =====

/**
 * Viser loading-indikator i content-området
 */
function showLoading() {
    elements.content.innerHTML = '<div class="loading"><p>Laster...</p></div>';
}

/**
 * Viser en feilmelding i content-området
 * @param {string} message - Feilmelding
 */
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

/**
 * Rendrer innhold i content-området
 * @param {string} html - HTML-innhold
 * @param {string|null} title - Sidetittel
 * @param {boolean} wasConverted - Om innholdet ble konvertert fra HTML
 */
function renderContent(html, title, wasConverted = false) {
    const convertedBadge = wasConverted ? '<span class="converted-badge" title="Konvertert fra HTML">📄→📝</span>' : '';
    elements.content.innerHTML = `<div class="markdown-body">${convertedBadge}${html}</div>`;
    
    setCurrentTitle(title);
    
    if (title) {
        document.title = `${title} - Bare`;
    } else {
        document.title = 'Bare';
    }
}

// ===== Footer =====

/**
 * Oppdaterer footer-informasjon
 * @param {string} path - Nåværende sti/URL
 * @param {boolean} wasConverted - Om innholdet ble konvertert
 */
function updateFooter(path, wasConverted = false) {
    if (path && path !== HOME_PATH) {
        const filename = path.split(/[\\/]/).pop();
        const conversionIndicator = wasConverted ? ' (konvertert)' : '';
        elements.footerInfo.textContent = `Bare ${getAppVersion()} — ${filename}${conversionIndicator}`;
    } else {
        elements.footerInfo.textContent = `Bare ${getAppVersion()}`;
    }
}

/**
 * Oppdaterer status-tekst i footer
 * @param {string} status - Statustekst
 */
function updateFooterStatus(status) {
    if (elements.footerStatus) {
        elements.footerStatus.textContent = status;
    }
}

/**
 * Oppdaterer zoom-nivå visning i footer og meny
 * @param {number} zoomLevel - Zoom-prosent
 */
function updateZoomDisplay(zoomLevel) {
    elements.zoomLevel.textContent = `${zoomLevel}%`;
    updateMenuZoomLevel(zoomLevel);
}

// ===== Navigation Buttons =====

/**
 * Oppdaterer navigasjonsknappenes tilstand
 */
function updateNavigationButtons() {
    elements.btnBack.disabled = !canGoBack();
    elements.btnForward.disabled = !canGoForward();
}

// ===== Panel Management =====

/**
 * Lukker alle side-paneler
 */
function closeAllPanels() {
    elements.bookmarksPanel.classList.add('hidden');
    elements.settingsPanel.classList.add('hidden');
}

/**
 * Toggles bokmerke-panelet
 * @returns {boolean} Om panelet nå er synlig
 */
function toggleBookmarksPanelUI() {
    const isVisible = !elements.bookmarksPanel.classList.contains('hidden');
    elements.bookmarksPanel.classList.toggle('hidden', isVisible);
    elements.settingsPanel.classList.add('hidden');
    return !isVisible;
}

/**
 * Toggles innstillinger-panelet
 * @returns {boolean} Om panelet nå er synlig
 */
function toggleSettingsPanelUI() {
    const isVisible = !elements.settingsPanel.classList.contains('hidden');
    elements.settingsPanel.classList.toggle('hidden', isVisible);
    elements.bookmarksPanel.classList.add('hidden');
    return !isVisible;
}

/**
 * Lukker bokmerke-panelet
 */
function closeBookmarksPanel() {
    elements.bookmarksPanel.classList.add('hidden');
}

/**
 * Lukker innstillinger-panelet
 */
function closeSettingsPanel() {
    elements.settingsPanel.classList.add('hidden');
}

// ===== Theme Button =====

/**
 * Oppdaterer tema-knappens ikon
 * @param {string} theme - Aktivt tema ('light' eller 'dark')
 */
function updateThemeButton(theme) {
    const icon = theme === 'dark' ? '☀️' : '🌙';
    elements.btnTheme.innerHTML = `<span class="menu-icon">${icon}</span><span>Bytt tema</span>`;
}

// ===== Dropdown Menu =====

/**
 * Toggler dropdown-menyen
 */
function toggleDropdownMenu() {
    const isVisible = !elements.dropdownMenu.classList.contains('hidden');
    elements.dropdownMenu.classList.toggle('hidden', isVisible);
}

/**
 * Lukker dropdown-menyen
 */
function closeDropdownMenu() {
    elements.dropdownMenu.classList.add('hidden');
}

/**
 * Oppdaterer zoom-nivå i menyen
 * @param {number} zoomLevel - Zoom-prosent
 */
function updateMenuZoomLevel(zoomLevel) {
    if (elements.menuZoomLevel) {
        elements.menuZoomLevel.textContent = `${zoomLevel}%`;
    }
}

// ===== Om-dialog =====

/**
 * Viser Om-dialogen
 */
function showAboutDialog() {
    // Oppdater versjonsnummer
    elements.aboutVersion.textContent = getAppVersion();
    elements.aboutOverlay.classList.remove('hidden');
    closeDropdownMenu();
}

/**
 * Lukker Om-dialogen
 */
function closeAboutDialog() {
    elements.aboutOverlay.classList.add('hidden');
}
