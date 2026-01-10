//! Bokmerke-håndtering for Bare
//!
//! Lagrer og henter bokmerker fra JSON-fil.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

/// Feil som kan oppstå ved bokmerke-operasjoner
#[derive(Debug, Error)]
pub enum BookmarkError {
    #[error("Kunne ikke lese bokmerker: {0}")]
    Read(String),

    #[error("Kunne ikke lagre bokmerker: {0}")]
    Write(String),

    #[error("Bokmerke finnes allerede: {0}")]
    AlreadyExists(String),

    #[error("Bokmerke ikke funnet: {0}")]
    NotFound(String),
}

/// Et enkelt bokmerke
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bookmark {
    /// Unik ID for bokmerket
    pub id: String,
    /// Tittel på bokmerket
    pub title: String,
    /// URL eller filsti
    pub url: String,
    /// Tidspunkt bokmerket ble opprettet (Unix timestamp)
    pub created_at: u64,
}

/// Samling av alle bokmerker
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BookmarkStore {
    pub bookmarks: Vec<Bookmark>,
}

impl BookmarkStore {
    /// Last bokmerker fra fil
    pub fn load(path: &PathBuf) -> Result<Self, BookmarkError> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path).map_err(|e| BookmarkError::Read(e.to_string()))?;

        serde_json::from_str(&content).map_err(|e| BookmarkError::Read(e.to_string()))
    }

    /// Lagre bokmerker til fil
    pub fn save(&self, path: &PathBuf) -> Result<(), BookmarkError> {
        // Opprett overordnede mapper hvis de ikke finnes
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| BookmarkError::Write(e.to_string()))?;
        }

        let content =
            serde_json::to_string_pretty(self).map_err(|e| BookmarkError::Write(e.to_string()))?;

        fs::write(path, content).map_err(|e| BookmarkError::Write(e.to_string()))
    }

    /// Legg til et nytt bokmerke
    pub fn add(&mut self, bookmark: Bookmark) -> Result<(), BookmarkError> {
        // Sjekk om bokmerket allerede finnes (basert på URL)
        if self.bookmarks.iter().any(|b| b.url == bookmark.url) {
            return Err(BookmarkError::AlreadyExists(bookmark.url));
        }

        self.bookmarks.push(bookmark);
        Ok(())
    }

    /// Fjern et bokmerke basert på ID
    pub fn remove(&mut self, id: &str) -> Result<(), BookmarkError> {
        let original_len = self.bookmarks.len();
        self.bookmarks.retain(|b| b.id != id);

        if self.bookmarks.len() == original_len {
            return Err(BookmarkError::NotFound(id.to_string()));
        }

        Ok(())
    }

    /// Hent alle bokmerker
    pub fn list(&self) -> &[Bookmark] {
        &self.bookmarks
    }

    /// Sjekk om en URL er bokmerket
    pub fn is_bookmarked(&self, url: &str) -> bool {
        self.bookmarks.iter().any(|b| b.url == url)
    }
}

/// Hent stien til bokmerke-filen
pub fn get_bookmarks_path() -> PathBuf {
    // Bruk brukerens config-mappe
    let config_dir = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    config_dir.join("bare").join("bookmarks.json")
}

/// Generer en unik ID for et bokmerke
pub fn generate_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("bm_{}", timestamp)
}

/// Hent nåværende Unix timestamp
pub fn current_timestamp() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_bookmark(url: &str, title: &str) -> Bookmark {
        Bookmark {
            id: generate_id(),
            title: title.to_string(),
            url: url.to_string(),
            created_at: current_timestamp(),
        }
    }

    #[test]
    fn test_add_bookmark() {
        let mut store = BookmarkStore::default();
        let bookmark = create_test_bookmark("https://example.com", "Example");

        assert!(store.add(bookmark).is_ok());
        assert_eq!(store.bookmarks.len(), 1);
    }

    #[test]
    fn test_add_duplicate_bookmark() {
        let mut store = BookmarkStore::default();
        let bookmark1 = create_test_bookmark("https://example.com", "Example 1");
        let bookmark2 = create_test_bookmark("https://example.com", "Example 2");

        assert!(store.add(bookmark1).is_ok());
        assert!(matches!(
            store.add(bookmark2),
            Err(BookmarkError::AlreadyExists(_))
        ));
    }

    #[test]
    fn test_remove_bookmark() {
        let mut store = BookmarkStore::default();
        let bookmark = create_test_bookmark("https://example.com", "Example");
        let id = bookmark.id.clone();

        store.add(bookmark).unwrap();
        assert!(store.remove(&id).is_ok());
        assert_eq!(store.bookmarks.len(), 0);
    }

    #[test]
    fn test_is_bookmarked() {
        let mut store = BookmarkStore::default();
        let bookmark = create_test_bookmark("https://example.com", "Example");

        store.add(bookmark).unwrap();
        assert!(store.is_bookmarked("https://example.com"));
        assert!(!store.is_bookmarked("https://other.com"));
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("bookmarks.json");

        let mut store = BookmarkStore::default();
        store
            .add(create_test_bookmark("https://example.com", "Example"))
            .unwrap();

        store.save(&path).unwrap();

        let loaded = BookmarkStore::load(&path).unwrap();
        assert_eq!(loaded.bookmarks.len(), 1);
        assert_eq!(loaded.bookmarks[0].url, "https://example.com");
    }
}
