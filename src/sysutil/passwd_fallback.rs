// SPDX-FileCopyrightText: 2022 The ReGreet Authors
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Fallback user discovery via `/etc/passwd` and `/etc/login.defs`.
//!
//! Used when D-Bus (AccountsService) is unavailable, e.g. in demo mode
//! or on systems without AccountsService running.

use std::collections::HashMap;
use std::fs::read_to_string;
use std::path::Path;

use pwd::Passwd;
use tracing::{debug, warn};

/// UID boundaries for "normal" users, read from `/etc/login.defs`.
#[derive(Debug)]
struct UidBounds {
    min: u64,
    max: u64,
}

impl Default for UidBounds {
    fn default() -> Self {
        // Conventional defaults from Linux login.defs
        Self {
            min: 1000,
            max: 60000,
        }
    }
}

impl UidBounds {
    /// Parse UID_MIN and UID_MAX from `/etc/login.defs` content.
    fn parse_login_defs(content: &str) -> Self {
        let mut min = None;
        let mut max = None;

        for line in content.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let mut parts = line.split_whitespace();
            match (parts.next(), parts.next()) {
                (Some("UID_MIN"), Some(val)) => min = val.parse().ok(),
                (Some("UID_MAX"), Some(val)) => max = val.parse().ok(),
                _ => {}
            };
        }

        Self {
            min: min.unwrap_or(1000),
            max: max.unwrap_or(60000),
        }
    }

    /// Read UID bounds from the first available login.defs path.
    fn from_system() -> Self {
        for path in &["/etc/login.defs", "/usr/etc/login.defs"] {
            if Path::new(path).exists() {
                if let Ok(content) = read_to_string(path) {
                    debug!("Reading UID bounds from {path}");
                    return Self::parse_login_defs(&content);
                }
            }
        }
        warn!("No login.defs found, using default UID bounds");
        Self::default()
    }

    fn contains(&self, uid: u64) -> bool {
        (self.min..=self.max).contains(&uid)
    }
}

/// Discover regular users by reading `/etc/passwd` directly.
///
/// Returns `(users, shells)` maps:
/// - `users`: maps full name (from gecos) → system username
/// - `shells`: maps system username → shell path
pub fn discover_users() -> (HashMap<String, String>, HashMap<String, String>) {
    let bounds = UidBounds::from_system();
    debug!("Using UID bounds: {}–{}", bounds.min, bounds.max);

    let mut users = HashMap::new();
    let mut shells = HashMap::new();

    for entry in Passwd::iter().filter(|e| bounds.contains(e.uid.into())) {
        // Extract full name from gecos field (first comma-separated part).
        let full_name = entry
            .gecos
            .as_deref()
            .filter(|g| !g.is_empty())
            .map(|g| g.split(',').next().unwrap_or(g).trim().to_string())
            .unwrap_or_else(|| entry.name.clone());

        debug!(
            "Found user '{}' (UID {}) with full name '{}'",
            entry.name, entry.uid, full_name
        );

        users.insert(full_name, entry.name.clone());

        if !entry.shell.is_empty() {
            shells.insert(entry.name, entry.shell.clone());
        }
    }

    (users, shells)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_login_defs_basic() {
        let content = "UID_MIN 1000\nUID_MAX 60000";
        let bounds = UidBounds::parse_login_defs(content);
        assert_eq!(bounds.min, 1000);
        assert_eq!(bounds.max, 60000);
    }

    #[test]
    fn test_parse_login_defs_with_comments() {
        let content = "# Comment\nUID_MIN\t500\n\nUID_MAX\t30000\n# Another comment";
        let bounds = UidBounds::parse_login_defs(content);
        assert_eq!(bounds.min, 500);
        assert_eq!(bounds.max, 30000);
    }

    #[test]
    fn test_parse_login_defs_missing_max() {
        let content = "UID_MIN 2000";
        let bounds = UidBounds::parse_login_defs(content);
        assert_eq!(bounds.min, 2000);
        assert_eq!(bounds.max, 60000); // default
    }

    #[test]
    fn test_parse_login_defs_empty() {
        let bounds = UidBounds::parse_login_defs("");
        assert_eq!(bounds, UidBounds::default());
    }

    #[test]
    fn test_uid_bounds_contains() {
        let bounds = UidBounds {
            min: 1000,
            max: 60000,
        };
        assert!(!bounds.contains(0));    // root
        assert!(!bounds.contains(999));  // just below
        assert!(bounds.contains(1000));  // at min
        assert!(bounds.contains(50000)); // middle
        assert!(bounds.contains(60000)); // at max
        assert!(!bounds.contains(60001)); // just above
    }
}
