const { invoke: invokePalette } = window.__TAURI__.core;

const commandPaletteState = {
    isOpen: false,
    results: [],
    selectedIndex: 0,
};

function isCommandPaletteOpen() {
    return commandPaletteState.isOpen;
}

function openCommandPalette() {
    if (commandPaletteState.isOpen) return;
    commandPaletteState.isOpen = true;
    commandPaletteState.selectedIndex = 0;
    elements.commandPaletteOverlay.classList.remove('hidden');
    elements.commandPaletteInput.value = '';
    elements.commandPaletteResults.innerHTML = '';
    elements.commandPaletteInput.focus();
    loadCommandPaletteResults('');
}

function closeCommandPalette() {
    if (!commandPaletteState.isOpen) return;
    commandPaletteState.isOpen = false;
    elements.commandPaletteOverlay.classList.add('hidden');
    elements.commandPaletteInput.value = '';
    commandPaletteState.results = [];
}

function fuzzyMatch(query, text) {
    if (!query) return { match: true, score: 0, indices: [] };
    const q = query.toLowerCase();
    const t = text.toLowerCase();
    if (t.includes(q)) {
        const idx = t.indexOf(q);
        const indices = [];
        for (let i = idx; i < idx + q.length; i++) indices.push(i);
        return { match: true, score: 100 - (idx === 0 ? 50 : 0) - (t.length - q.length), indices };
    }
    let qi = 0;
    let score = 0;
    const indices = [];
    let consecutive = 0;
    for (let ti = 0; ti < t.length && qi < q.length; ti++) {
        if (t[ti] === q[qi]) {
            indices.push(ti);
            score += 10 + consecutive * 5;
            if (ti === 0) score += 20;
            consecutive++;
            qi++;
        } else {
            consecutive = 0;
        }
    }
    if (qi < q.length) return { match: false, score: 0, indices: [] };
    return { match: true, score, indices };
}

function highlightMatches(text, indices) {
    if (!indices || indices.length === 0) return escapeHtml(text);
    let result = '';
    let inHighlight = false;
    for (let i = 0; i < text.length; i++) {
        if (indices.includes(i)) {
            if (!inHighlight) {
                result += '<strong class="command-palette-highlight">';
                inHighlight = true;
            }
        } else {
            if (inHighlight) {
                result += '</strong>';
                inHighlight = false;
            }
        }
        result += escapeHtml(text[i]);
    }
    if (inHighlight) result += '</strong>';
    return result;
}

function getBuiltInActions() {
    return [
        { type: 'action', icon: '\u2190', label: t('commandPalette.goHome'), shortcut: 'G', action: () => goHome() },
        { type: 'action', icon: '\u21BB', label: t('commandPalette.reload'), shortcut: 'F5', action: () => reloadPage() },
        { type: 'action', icon: '\u2B62', label: t('commandPalette.forward'), shortcut: 'Alt+\u2192', action: () => goForward() },
        { type: 'action', icon: '\u2B60', label: t('commandPalette.back'), shortcut: 'Alt+\u2190', action: () => goBack() },
        { type: 'action', icon: '\u21F1', label: t('commandPalette.openFile'), shortcut: 'Ctrl+O', action: () => openFileDialog() },
        { type: 'action', icon: '\u2606', label: t('commandPalette.bookmark'), shortcut: 'Ctrl+D', action: () => toggleBookmark() },
        { type: 'action', icon: '\u29C9', label: t('commandPalette.bookmarks'), shortcut: 'Ctrl+B', action: () => toggleBookmarksPanel() },
        { type: 'action', icon: '\u2699', label: t('commandPalette.settings'), action: () => toggleSettingsPanelUI() },
        { type: 'action', icon: '\u2295', label: t('commandPalette.zoomIn'), shortcut: 'Ctrl++', action: () => zoomIn() },
        { type: 'action', icon: '\u229F', label: t('commandPalette.zoomOut'), shortcut: 'Ctrl+-', action: () => zoomOut() },
        { type: 'action', icon: '\u29BF', label: t('commandPalette.zoomReset'), shortcut: 'Ctrl+0', action: () => zoomReset() },
        { type: 'action', icon: '\u2318', label: t('commandPalette.search'), shortcut: 'Ctrl+F', action: () => openSearch() },
        { type: 'action', icon: '\u2318', label: t('commandPalette.focusUrl'), shortcut: 'Ctrl+L', action: () => { elements.urlBar.focus(); elements.urlBar.select(); } },
        { type: 'action', icon: '\u2139', label: t('commandPalette.about'), action: () => showAboutDialog() },
    ];
}

async function loadCommandPaletteResults(query) {
    const results = [];

    const builtIn = getBuiltInActions();
    for (const item of builtIn) {
        const { match, score, indices } = fuzzyMatch(query, item.label);
        if (match) {
            results.push({ ...item, score, indices });
        }
    }

    try {
        const bookmarks = await invokePalette('get_bookmarks');
        for (const bm of bookmarks) {
            const search = bm.title + ' ' + bm.url;
            const { match, score, indices } = fuzzyMatch(query, search);
            if (match) {
                results.push({
                    type: 'bookmark',
                    icon: '\u2605',
                    label: bm.title,
                    detail: bm.url,
                    score,
                    indices,
                    action: () => navigateToBookmark(bm.url),
                });
            }
        }
    } catch (_) {}

    const history = state.history || [];
    for (const entry of history) {
        const { match, score, indices } = fuzzyMatch(query, entry);
        if (match) {
            results.push({
                type: 'history',
                icon: '\u231A',
                label: entry,
                score,
                indices,
                action: () => loadPath(entry),
            });
        }
    }

    results.sort((a, b) => b.score - a.score);

    commandPaletteState.results = results;
    commandPaletteState.selectedIndex = 0;
    renderCommandPaletteResults(results, query);
}

function renderCommandPaletteResults(results, query) {
    if (results.length === 0) {
        elements.commandPaletteResults.innerHTML = `<div class="command-palette-empty">${t('commandPalette.empty')}</div>`;
        return;
    }

    elements.commandPaletteResults.innerHTML = results.map((item, i) => {
        const selected = i === commandPaletteState.selectedIndex ? ' command-palette-item-selected' : '';
        const detail = item.detail ? `<span class="command-palette-item-detail">${escapeHtml(item.detail)}</span>` : '';
        return `<div class="command-palette-item${selected}" data-index="${i}" tabindex="-1">
            <span class="command-palette-item-icon">${item.icon}</span>
            <span class="command-palette-item-label">${highlightMatches(item.label, item.indices)}</span>
            ${detail}
        </div>`;
    }).join('');
}

function selectCommandPaletteItem(index) {
    const results = commandPaletteState.results;
    if (index < 0 || index >= results.length) return;
    commandPaletteState.selectedIndex = index;

    const items = elements.commandPaletteResults.querySelectorAll('.command-palette-item');
    items.forEach((el, i) => {
        el.classList.toggle('command-palette-item-selected', i === index);
    });

    const selected = elements.commandPaletteResults.querySelector('.command-palette-item-selected');
    if (selected) selected.scrollIntoView({ block: 'nearest' });
}

function executeCommandPaletteItem(index) {
    const results = commandPaletteState.results;
    if (index < 0 || index >= results.length) return;
    closeCommandPalette();
    results[index].action();
}

function handleCommandPaletteKeydown(e) {
    if (!isCommandPaletteOpen()) return false;

    if (e.key === 'ArrowDown') {
        e.preventDefault();
        selectCommandPaletteItem(Math.min(commandPaletteState.selectedIndex + 1, commandPaletteState.results.length - 1));
        return true;
    }
    if (e.key === 'ArrowUp') {
        e.preventDefault();
        selectCommandPaletteItem(Math.max(commandPaletteState.selectedIndex - 1, 0));
        return true;
    }
    if (e.key === 'Enter') {
        e.preventDefault();
        executeCommandPaletteItem(commandPaletteState.selectedIndex);
        return true;
    }
    if (e.key === 'Escape') {
        e.preventDefault();
        closeCommandPalette();
        return true;
    }
    return false;
}
