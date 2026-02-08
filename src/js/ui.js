/**
 * Bare Browser - UI Management
 * 
 * Håndterer status bar, loading states, footer og panel-visning.
 */

const { listen } = window.__TAURI__.event;

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

// ===== Footer Loading Status (Netscape-stil) =====

let loadingStepIndex = 0;
let progressAnimFrame = null;
let progressTarget = 0;
let progressCurrent = 0;
let footerResetTimeout = null;

/** Status-steg mapping til progress-prosent */
const LOADING_STEP_PROGRESS = {
    'lookup': 15,
    'connect': 30,
    'transfer': 60,
    'convert': 75,
    'render': 90,
    'done': 100,
    'error': 0,
    'waiting': 45,
    'stopped': 0,
};

/**
 * Parser en loading-status melding og gir tilbake progress-steg
 * @param {string} msg - Status-melding fra backend
 * @returns {string} Steg-nøkkel
 */
function parseLoadingStep(msg) {
    if (msg.startsWith('Slår opp')) return 'lookup';
    if (msg.startsWith('Kobler til')) return 'connect';
    if (msg.startsWith('Overfører data')) return 'transfer';
    if (msg.startsWith('Konverterer')) return 'convert';
    if (msg.startsWith('Rendrer')) return 'render';
    if (msg.startsWith('Dokument: Ferdig')) return 'done';
    if (msg.startsWith('Feil')) return 'error';
    if (msg.startsWith('Venter')) return 'waiting';
    if (msg.startsWith('Stoppet')) return 'stopped';
    return 'transfer';
}

/**
 * Starter loading-indikatoren i footer
 */
function startFooterLoading() {
    if (footerResetTimeout) {
        clearTimeout(footerResetTimeout);
        footerResetTimeout = null;
    }
    loadingStepIndex = 0;
    progressCurrent = 0;
    progressTarget = 0;
    elements.footerStatus.textContent = 'Kobler til...';
    elements.footerProgress.classList.add('active');
    elements.footerProgressBar.style.width = '0%';
}

/**
 * Oppdaterer loading-status i footer med Netscape-lignende meldinger
 * @param {string} message - Statusmelding fra backend
 */
function updateLoadingStatus(message) {
    elements.footerStatus.textContent = message;
    
    const step = parseLoadingStep(message);
    progressTarget = LOADING_STEP_PROGRESS[step] || 50;
    
    // Animer progress-baren jevnt
    animateProgress();
    
    // Hvis ferdig, planlegg reset
    if (step === 'done') {
        footerResetTimeout = setTimeout(() => {
            stopFooterLoading();
        }, 1500);
    }
}

/**
 * Animerer progress-bar mot target
 */
function animateProgress() {
    if (progressAnimFrame) {
        cancelAnimationFrame(progressAnimFrame);
    }
    
    function step() {
        if (progressCurrent < progressTarget) {
            // Rask start, saktere mot mål (ease-out)
            const diff = progressTarget - progressCurrent;
            const increment = Math.max(1, diff * 0.3);
            progressCurrent = Math.min(progressTarget, progressCurrent + increment);
            elements.footerProgressBar.style.width = progressCurrent + '%';
            progressAnimFrame = requestAnimationFrame(step);
        }
    }
    
    progressAnimFrame = requestAnimationFrame(step);
}

/**
 * Stopper loading-indikatoren og resetter footer
 */
function stopFooterLoading() {
    if (progressAnimFrame) {
        cancelAnimationFrame(progressAnimFrame);
        progressAnimFrame = null;
    }
    elements.footerStatus.textContent = 'Klar';
    elements.footerProgress.classList.remove('active');
    elements.footerProgressBar.style.width = '0%';
    progressCurrent = 0;
    progressTarget = 0;
}

/**
 * Initialiserer lytting på loading-status events fra Tauri-backend
 */
async function initLoadingStatusListener() {
    await listen('loading-status', (event) => {
        updateLoadingStatus(event.payload);
    });
}

// ===== Content States =====

/**
 * Viser loading-indikator i content-området
 */
function showLoading() {
    elements.content.innerHTML = '<div class=\"loading\"><p>[ LASTER... ]</p></div>';
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
    closeDropdownMenu();
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

// ===== Gemini Input Dialog =====

/** Lagrer nåværende Gemini input URL */
let geminiInputUrl = null;

/**
 * Viser Gemini input-dialog
 * @param {string} prompt - Serverens prompt-tekst
 * @param {string} url - Gemini-URL som ba om input
 * @param {boolean} sensitive - Om input er sensitiv (passord)
 */
function showGeminiInputDialog(prompt, url, sensitive = false) {
    geminiInputUrl = url;
    elements.geminiInputPrompt.textContent = prompt || 'Serveren ber om input:';
    elements.geminiInputField.value = '';
    elements.geminiInputField.type = sensitive ? 'password' : 'text';
    elements.geminiInputField.placeholder = sensitive ? 'Skriv inn (skjult)...' : 'Skriv inn tekst...';
    elements.geminiInputOverlay.classList.remove('hidden');
    
    // Fokuser inputfeltet
    setTimeout(() => elements.geminiInputField.focus(), 100);
}

/**
 * Lukker Gemini input-dialog
 */
function closeGeminiInputDialog() {
    elements.geminiInputOverlay.classList.add('hidden');
    geminiInputUrl = null;
    elements.geminiInputField.value = '';
}

/**
 * Håndterer submit fra Gemini input-dialog
 */
async function handleGeminiInputSubmit() {
    const input = elements.geminiInputField.value;
    const url = geminiInputUrl;
    
    if (!url) return;
    
    closeGeminiInputDialog();
    
    if (input !== null && input !== undefined) {
        await submitGeminiInput(url, input);
    }
}
