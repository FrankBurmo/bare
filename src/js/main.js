/**
 * Bare Browser - Main Entry Point
 * 
 * Initialiserer applikasjonen og orkestrerer alle moduler.
 */

const { invoke: invokeMain } = window.__TAURI__.core;

/**
 * Initialiserer applikasjonen
 */
async function init() {
    // Hent versjon fra backend
    try {
        const version = await invokeMain('get_app_version');
        setAppVersion(version);
        updateFooter(null);
    } catch (error) {
        console.error('Kunne ikke hente app-versjon:', error);
    }
    
    // Last innstillinger
    await loadSettings();
    
    // Registrer event listeners
    initEventListeners();
    
    // Oppdater UI
    updateNavigationButtons();
    
    // Naviger til startsiden
    await goHome();
}

// Start applikasjonen når DOM er klar
document.addEventListener('DOMContentLoaded', init);

// Eksporter funksjoner for global bruk (onclick-handlers i HTML)
window.goHome = goHome;
window.goBack = goBack;
window.goForward = goForward;
