/**
 * Bare Browser - Event Handlers
 * 
 * Sentralisert registrering av alle event listeners.
 */

/**
 * Initialiserer alle event listeners
 */
function initEventListeners() {
    initToolbarEvents();
    initBookmarkEvents();
    initSettingsEvents();
    initSearchEvents();
    initGeminiInputEvents();
    initKeyboardShortcuts();
    initContentEvents();
}

// ===== Toolbar Events =====

function initToolbarEvents() {
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
    elements.btnReload.addEventListener('click', reloadPage);
    elements.btnOpen.addEventListener('click', openFileDialog);
    elements.btnTheme.addEventListener('click', () => {
        toggleTheme();
        closeDropdownMenu();
    });
    
    // Zoom
    elements.btnZoomIn.addEventListener('click', zoomIn);
    elements.btnZoomOut.addEventListener('click', zoomOut);
    
    // Dropdown meny (3-prikks)
    elements.btnMenu.addEventListener('click', (e) => {
        e.stopPropagation();
        toggleDropdownMenu();
    });
    
    // Om-dialog
    elements.btnAbout.addEventListener('click', showAboutDialog);
    elements.btnCloseAbout.addEventListener('click', closeAboutDialog);
    elements.aboutOverlay.addEventListener('click', (e) => {
        if (e.target === elements.aboutOverlay) {
            closeAboutDialog();
        }
    });
    
    // Lukk dropdown når man klikker utenfor
    document.addEventListener('click', (e) => {
        if (!elements.dropdownMenu.contains(e.target) && !elements.btnMenu.contains(e.target)) {
            closeDropdownMenu();
        }
    });
}

// ===== Bookmark Events =====

function initBookmarkEvents() {
    elements.btnBookmark.addEventListener('click', toggleBookmark);
    elements.btnBookmarks.addEventListener('click', toggleBookmarksPanel);
    elements.btnCloseBookmarks.addEventListener('click', closeBookmarksPanel);
    
    // Bokmerke-liste klikk (delegert)
    elements.bookmarksList.addEventListener('click', async (e) => {
        // Slett-knapp
        const deleteBtn = e.target.closest('.bookmark-delete');
        if (deleteBtn) {
            e.stopPropagation();
            const id = deleteBtn.dataset.id;
            await deleteBookmark(id);
            return;
        }
        
        // Bokmerke-item
        const item = e.target.closest('.bookmark-item');
        if (item) {
            const url = item.dataset.url;
            await navigateToBookmark(url);
        }
    });
}

// ===== Settings Events =====

function initSettingsEvents() {
    elements.btnSettings.addEventListener('click', toggleSettingsPanel);
    elements.btnCloseSettings.addEventListener('click', closeSettingsPanel);
    
    // Tema
    elements.settingTheme.addEventListener('change', (e) => {
        updateSetting('theme', e.target.value);
    });
    
    // Skrifttype
    elements.settingFontFamily.addEventListener('change', (e) => {
        updateSetting('font_family', e.target.value);
    });
    
    // Skriftstørrelse
    elements.settingFontSize.addEventListener('input', (e) => {
        elements.settingFontSizeValue.textContent = `${e.target.value}%`;
    });
    elements.settingFontSize.addEventListener('change', (e) => {
        updateSetting('font_size', parseInt(e.target.value));
    });
    
    // Innholdsbredde
    elements.settingContentWidth.addEventListener('input', (e) => {
        elements.settingContentWidthValue.textContent = `${e.target.value}px`;
    });
    elements.settingContentWidth.addEventListener('change', (e) => {
        updateSetting('content_width', parseInt(e.target.value));
    });
    
    // Konverteringsinnstillinger
    elements.settingConversionMode.addEventListener('change', (e) => {
        updateSetting('conversion_mode', e.target.value);
    });
    elements.settingReadability.addEventListener('change', (e) => {
        updateSetting('readability_enabled', e.target.checked);
    });
}

// ===== Search Events =====

function initSearchEvents() {
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
}

// ===== Gemini Input Events =====

function initGeminiInputEvents() {
    // Send-knapp
    elements.btnGeminiInputSend.addEventListener('click', handleGeminiInputSubmit);
    
    // Avbryt-knapp
    elements.btnGeminiInputCancel.addEventListener('click', closeGeminiInputDialog);
    
    // Lukk-knapp (X)
    elements.btnCloseGeminiInput.addEventListener('click', closeGeminiInputDialog);
    
    // Enter i inputfelt
    elements.geminiInputField.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') {
            handleGeminiInputSubmit();
        }
    });
    
    // Klikk utenfor dialog
    elements.geminiInputOverlay.addEventListener('click', (e) => {
        if (e.target === elements.geminiInputOverlay) {
            closeGeminiInputDialog();
        }
    });
}

// ===== Keyboard Shortcuts =====

function initKeyboardShortcuts() {
    document.addEventListener('keydown', (e) => {
        // F5: Last siden på nytt
        if (e.key === 'F5') {
            e.preventDefault();
            reloadPage();
        }
        
        // Ctrl+R: Last siden på nytt
        if (e.ctrlKey && e.key === 'r') {
            e.preventDefault();
            reloadPage();
        }
        
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
        
        // Vim-lignende navigasjon (kun når ikke i input)
        if (!isInputFocused()) {
            // g: Gå hjem
            if (e.key === 'g') {
                e.preventDefault();
                goHome();
            }
            
            // j: Scroll ned
            if (e.key === 'j') {
                elements.content.scrollBy({ top: 100, behavior: 'smooth' });
            }
            
            // k: Scroll opp
            if (e.key === 'k') {
                elements.content.scrollBy({ top: -100, behavior: 'smooth' });
            }
            
            // G: Scroll til bunnen
            if (e.key === 'G') {
                elements.content.scrollTo({ top: elements.content.scrollHeight, behavior: 'smooth' });
            }
            
            // Home: Scroll til toppen
            if (e.key === 'Home') {
                elements.content.scrollTo({ top: 0, behavior: 'smooth' });
            }
        }
        
        // Escape: Lukk paneler, søk, meny og dialoger
        if (e.key === 'Escape') {
            elements.urlBar.blur();
            closeAllPanels();
            closeSearch();
            closeDropdownMenu();
            closeAboutDialog();
            closeGeminiInputDialog();
        }
    });
}

// ===== Content Events =====

function initContentEvents() {
    // Handle links in content
    elements.content.addEventListener('click', async (e) => {
        const link = e.target.closest('a');
        if (link) {
            const href = link.getAttribute('href');
            
            // Ignorer fragment-lenker
            if (href.startsWith('#')) {
                return;
            }
            
            e.preventDefault();
            await resolveAndNavigate(href);
        }
    });
}
