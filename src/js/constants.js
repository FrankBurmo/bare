/**
 * Bare Browser - Konstanter
 * 
 * Sentral lagring av alle konstanter og magic values.
 */

// App-versjon (oppdateres fra backend ved init)
let APP_VERSION = 'v0.1.0';

/**
 * Setter app-versjonen
 * @param {string} version - Versjonsnummer uten 'v' prefix
 */
function setAppVersion(version) {
    APP_VERSION = 'v' + version;
}

/**
 * Henter app-versjonen
 * @returns {string} Formatert versjon med 'v' prefix
 */
function getAppVersion() {
    return APP_VERSION;
}

// Historikk
const MAX_HISTORY_SIZE = 50;

// Zoom-grenser
const ZOOM_MIN = 50;
const ZOOM_MAX = 200;
const ZOOM_DEFAULT = 100;

// Skriftstørrelse-grenser
const FONT_SIZE_MIN = 70;
const FONT_SIZE_MAX = 150;
const FONT_SIZE_DEFAULT = 100;

// Innholdsbredde-grenser
const CONTENT_WIDTH_MIN = 400;
const CONTENT_WIDTH_MAX = 1200;
const CONTENT_WIDTH_DEFAULT = 800;

// Spesielle verdier
const HOME_PATH = '__home__';
const CONVERSION_PROMPT_PREFIX = 'CONVERSION_PROMPT:';
const GEMINI_SCHEME = 'gemini://';
const GEMINI_INPUT_PROMPT_PREFIX = 'GEMINI_INPUT_PROMPT:';
const GEMINI_SENSITIVE_INPUT_PROMPT_PREFIX = 'GEMINI_SENSITIVE_INPUT_PROMPT:';

// Status bar timeout (ms)
const STATUS_TIMEOUT = 3000;

// Tema-alternativer
const THEMES = ['light', 'dark', 'system'];

// Standard innstillinger
const DEFAULT_SETTINGS = {
    theme: 'light',
    font_size: FONT_SIZE_DEFAULT,
    zoom: ZOOM_DEFAULT,
    font_family: 'system',
    content_width: CONTENT_WIDTH_DEFAULT,
    conversion_mode: 'convert-all',
    readability_enabled: true,
    onboarding_completed: false,
};
