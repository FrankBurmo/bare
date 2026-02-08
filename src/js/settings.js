/**
 * Bare Browser - Settings Management
 * 
 * Håndterer brukerinnstillinger og synkronisering med backend.
 */

const { invoke } = window.__TAURI__.core;

/**
 * Laster innstillinger fra backend
 */
async function loadSettings() {
    try {
        const settings = await invoke('get_settings');
        setSettings(settings);
        applySettings();
    } catch (error) {
        console.error('Kunne ikke laste innstillinger:', error);
        setSettings({ ...DEFAULT_SETTINGS });
    }
}

/**
 * Anvender innstillinger på UI
 */
function applySettings() {
    const settings = getSettings();
    if (!settings) return;
    
    // Tema
    let effectiveTheme = settings.theme;
    if (effectiveTheme === 'system') {
        effectiveTheme = window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    document.documentElement.setAttribute('data-theme', effectiveTheme);
    updateThemeButton(effectiveTheme);
    
    // Skriftstørrelse
    document.documentElement.style.setProperty('--base-font-size', `${settings.font_size}%`);
    document.body.style.fontSize = `${settings.font_size}%`;
    
    // Zoom
    elements.content.style.transform = `scale(${settings.zoom / 100})`;
    elements.content.style.transformOrigin = 'top center';
    updateZoomDisplay(settings.zoom);
    
    // Skrifttype
    document.body.className = `font-${settings.font_family}`;
    
    // Innholdsbredde
    document.documentElement.style.setProperty('--content-max-width', `${settings.content_width}px`);
    
    // Oppdater innstillingspanel-kontroller
    updateSettingsPanel(settings);
}

/**
 * Oppdaterer innstillingspanel-kontrollene
 * @param {Object} settings - Innstillinger
 */
function updateSettingsPanel(settings) {
    if (elements.settingTheme) {
        elements.settingTheme.value = settings.theme;
    }
    if (elements.settingFontFamily) {
        elements.settingFontFamily.value = settings.font_family;
    }
    if (elements.settingFontSize) {
        elements.settingFontSize.value = settings.font_size;
        elements.settingFontSizeValue.textContent = `${settings.font_size}%`;
    }
    if (elements.settingContentWidth) {
        elements.settingContentWidth.value = settings.content_width;
        elements.settingContentWidthValue.textContent = `${settings.content_width}px`;
    }
    if (elements.settingConversionMode) {
        elements.settingConversionMode.value = settings.conversion_mode;
    }
    if (elements.settingReadability) {
        elements.settingReadability.checked = settings.readability_enabled;
    }
}

/**
 * Oppdaterer en enkelt innstilling
 * @param {string} key - Innstillingsnøkkel
 * @param {any} value - Ny verdi
 */
async function updateSetting(key, value) {
    try {
        const params = {};
        params[key] = value;
        const newSettings = await invoke('update_settings', { params });
        setSettings(newSettings);
        applySettings();
    } catch (error) {
        showStatus(`Kunne ikke oppdatere innstilling: ${error}`, true);
    }
}

/**
 * Bytter til neste tema i syklusen
 */
async function toggleTheme() {
    const settings = getSettings();
    const currentIndex = THEMES.indexOf(settings.theme);
    const nextTheme = THEMES[(currentIndex + 1) % THEMES.length];
    await updateSetting('theme', nextTheme);
}

/**
 * Zoomer inn
 */
async function zoomIn() {
    try {
        const newSettings = await invoke('zoom_in');
        setSettings(newSettings);
        applySettings();
    } catch (error) {
        showStatus(`Kunne ikke zoome inn: ${error}`, true);
    }
}

/**
 * Zoomer ut
 */
async function zoomOut() {
    try {
        const newSettings = await invoke('zoom_out');
        setSettings(newSettings);
        applySettings();
    } catch (error) {
        showStatus(`Kunne ikke zoome ut: ${error}`, true);
    }
}

/**
 * Tilbakestiller zoom til 100%
 */
async function zoomReset() {
    try {
        const newSettings = await invoke('zoom_reset');
        setSettings(newSettings);
        applySettings();
    } catch (error) {
        showStatus(`Kunne ikke tilbakestille zoom: ${error}`, true);
    }
}

/**
 * Toggles innstillingspanelet og håndterer state
 */
function toggleSettingsPanel() {
    toggleSettingsPanelUI();
}

// ===== Onboarding =====

/**
 * Viser onboarding-modalen hvis brukeren ikke har fullført den
 * @returns {Promise<boolean>} True hvis onboarding ble vist
 */
async function checkAndShowOnboarding() {
    const settings = getSettings();
    if (!settings || settings.onboarding_completed) {
        return false;
    }
    
    return new Promise((resolve) => {
        elements.onboardingOverlay.classList.remove('hidden');
        
        elements.btnOnboardingConfirm.addEventListener('click', async () => {
            const selected = document.querySelector('input[name="onboarding-conversion"]:checked');
            if (selected) {
                await updateSetting('conversion_mode', selected.value);
            }
            await updateSetting('onboarding_completed', true);
            elements.onboardingOverlay.classList.add('hidden');
            resolve(true);
        }, { once: true });
    });
}
