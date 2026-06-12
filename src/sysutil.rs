// SPDX-FileCopyrightText: 2022 The ReGreet Authors
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Helper for system utilities like users and sessions

mod accounts_service;
mod passwd_fallback;

use std::collections::{HashMap, HashSet};
use std::env;
use std::error::Error;
use std::path::Path;

use freedesktop_entry_parser::Entry;
use glob::glob;
use shlex::Shlex;
use zbus::Connection;

use self::accounts_service::AccountsServiceProxy;
use self::accounts_service::UserProxy;
use crate::config::Config;
use crate::constants::SESSION_DIRS;


/// XDG data directory variable name (parent directory for X11/Wayland sessions)
const XDG_DIR_ENV_VAR: &str = "XDG_DATA_DIRS";

#[derive(Clone, Copy)]
pub enum SessionType {
    X11,
    Wayland,
    Unknown,
}

#[derive(Clone)]
pub struct SessionInfo {
    pub command: Vec<String>,
    pub sess_type: SessionType,
}

// Convenient aliases for used maps
type UserMap = HashMap<String, String>;
type ShellMap = HashMap<String, String>;
type SessionMap = HashMap<String, SessionInfo>;

/// Stores info of all regular users and sessions
pub struct SysUtil {
    /// Maps a user's full name to their system username
    pub users: UserMap,
    /// Maps a system username to their shell
    pub shells: ShellMap,
    /// Maps a session's full name to its command
    pub sessions: SessionMap,
}

impl SysUtil {
    pub async fn new(config: &Config) -> Result<Self, Box<dyn Error>> {
        let (users, shells) = match Self::try_dbus_users().await {
            Some(result) => {
                debug!("Using D-Bus AccountsService for user discovery");
                result
            }
            None => {
                warn!("D-Bus AccountsService unavailable, falling back to /etc/passwd");
                passwd_fallback::discover_users()
            }
        };

        let sessions = match Self::init_sessions(config).await {
            Ok(sessions) => sessions,
            Err(err) => {
                warn!("Failed to discover sessions: {err}");
                HashMap::new()
            }
        };

        Ok(Self {
            users,
            shells,
            sessions,
        })
    }

    /// Try to discover users via D-Bus AccountsService.
    ///
    /// Returns `Some((users, shells))` on success, or `None` if D-Bus is unavailable
    /// or AccountsService is not running.
    async fn try_dbus_users() -> Option<(UserMap, ShellMap)> {
        let dbus_system_conn = match Connection::system().await {
            Ok(conn) => conn,
            Err(err) => {
                debug!("D-Bus system connection failed: {err}");
                return None;
            }
        };

        let accounts_proxy = match AccountsServiceProxy::new(&dbus_system_conn).await {
            Ok(proxy) => proxy,
            Err(err) => {
                debug!("AccountsService proxy creation failed: {err}");
                return None;
            }
        };

        let user_paths = match accounts_proxy.list_cached_users().await {
            Ok(paths) => paths,
            Err(err) => {
                debug!("Failed to list cached users via D-Bus: {err}");
                return None;
            }
        };

        let mut user_proxies = Vec::new();
        for user_path in user_paths {
            let builder = match UserProxy::builder(&dbus_system_conn)
                .path(user_path)
            {
                Ok(b) => b,
                Err(err) => {
                    debug!("Failed to set user proxy path: {err}");
                    continue;
                }
            };
            match builder.build().await {
                Ok(proxy) => user_proxies.push(proxy),
                Err(err) => {
                    debug!("Failed to build user proxy: {err}");
                }
            }
        }

        let mut usernames = HashMap::new();
        for user_proxy in &user_proxies {
            let real_name = user_proxy.real_name().await.unwrap_or_default();
            let user_name = user_proxy.user_name().await.unwrap_or_default();
            if !user_name.is_empty() {
                let display_name = if real_name.is_empty() {
                    user_name.clone()
                } else {
                    real_name
                };
                usernames.insert(display_name, user_name);
            }
        }

        let mut shells = HashMap::new();
        for user_proxy in &user_proxies {
            let user_name = user_proxy.user_name().await.unwrap_or_default();
            let shell = user_proxy.shell().await.unwrap_or_default();
            if !user_name.is_empty() {
                shells.insert(user_name, shell);
            }
        }

        Some((usernames, shells))
    }

    /// Get available X11 and Wayland sessions.
    ///
    /// These are defined as either X11 or Wayland session desktop files stored in specific
    /// directories.
    async fn init_sessions(config: &Config) -> Result<SessionMap, Box<dyn Error>> {
        let mut found_session_names = HashSet::new();
        let mut sessions = HashMap::new();

        // Use the XDG spec if available, else use the one that's compiled.
        // The XDG env var can change after compilation in some distros like NixOS.
        let session_dirs = if let Ok(sess_parent_dirs) = env::var(XDG_DIR_ENV_VAR) {
            debug!("Found XDG env var {XDG_DIR_ENV_VAR}: {sess_parent_dirs}");
            match sess_parent_dirs
                .split(':')
                .map(|parent_dir| format!("{parent_dir}/xsessions:{parent_dir}/wayland-sessions"))
                .reduce(|a, b| a + ":" + &b)
            {
                None => SESSION_DIRS.to_string(),
                Some(dirs) => dirs,
            }
        } else {
            SESSION_DIRS.to_string()
        };

        for sess_dir in session_dirs.split(':') {
            let sess_dir_path = Path::new(sess_dir);
            let sess_parent_dir = if let Some(sess_parent_dir) = sess_dir_path.parent() {
                sess_parent_dir
            } else {
                warn!("Session directory does not have a parent: {sess_dir}");
                continue;
            };

            let is_x11 = if let Some(name) = sess_dir_path.file_name() {
                name == "xsessions"
            } else {
                false
            };
            let cmd_prefix = if is_x11 {
                Some(&config.get_sys_commands().x11_prefix)
            } else {
                None
            };

            debug!("Checking session directory: {sess_dir}");
            // Iterate over all '.desktop' files.
            for glob_path in glob(&format!("{sess_dir}/*.desktop"))
                .expect("Invalid glob pattern for session desktop files")
            {
                let path = match glob_path {
                    Ok(path) => path,
                    Err(err) => {
                        warn!("Error when globbing: {err}");
                        continue;
                    }
                };
                info!("Now scanning session file: {}", path.display());

                let fname_and_type = match path.strip_prefix(sess_parent_dir) {
                    Ok(fname_and_type) => fname_and_type.to_owned(),
                    Err(err) => {
                        warn!("Error with file name: {err}");
                        continue;
                    }
                };

                if found_session_names.contains(&fname_and_type) {
                    debug!(
                        "{fname_and_type:?} was already found elsewhere, skipping {}",
                        path.display()
                    );
                    continue;
                };

                let entry = Entry::parse(tokio::fs::read(&path).await?)?;
                let section = if let Some(section) = entry.section("Desktop Entry") {
                    section
                } else {
                    warn!("Session file {} is not a desktop entry", path.display());
                    continue;
                };

                let hidden = section
                    .attr("Hidden")
                    .first()
                    .is_some_and(|s| s.parse().unwrap_or(false));
                let no_display = section
                    .attr("NoDisplay")
                    .first()
                    .is_some_and(|s| s.parse().unwrap_or(false));

                if hidden | no_display {
                    found_session_names.insert(fname_and_type);
                    continue;
                };

                // Parse the desktop file to get the session command.
                let cmd = if let Some(cmd_str) = section.attr("Exec").first() {
                    let mut cmd = if let Some(prefix) = cmd_prefix {
                        prefix.clone()
                    } else {
                        Vec::new()
                    };
                    let prefix_len = cmd.len();
                    cmd.extend(Shlex::new(cmd_str.as_str()));
                    if cmd.len() > prefix_len {
                        cmd
                    } else {
                        warn!(
                            "Couldn't split command of '{}' into arguments: {}",
                            path.display(),
                            cmd_str.as_str()
                        );
                        // Skip the desktop file, since a missing command means that we can't
                        // use it.
                        continue;
                    }
                } else {
                    warn!("No command found for session: {}", path.display());
                    // Skip the desktop file, since a missing command means that we can't use it.
                    continue;
                };

                // Get the full name of this session.
                let name = if let Some(name) = section.attr("Name").first() {
                    debug!(
                        "Found name '{}' for session '{}' with command '{:?}'",
                        name.as_str(),
                        path.display(),
                        cmd
                    );
                    name.as_str()
                } else if let Some(stem) = path.file_stem() {
                    // Get the stem of the filename of this desktop file.
                    // This is used as backup, in case the file name doesn't exist.
                    if let Some(stem) = stem.to_str() {
                        debug!(
                            "Using file stem '{stem}', since no name was found for session: {}",
                            path.display()
                        );
                        stem
                    } else {
                        warn!("Non-UTF-8 file stem in session file: {}", path.display());
                        // No way to display this session name, so just skip it.
                        continue;
                    }
                } else {
                    warn!("No file stem found for session: {}", path.display());
                    // No file stem implies no file name, which shouldn't happen.
                    // Since there's no full name nor file stem, just skip this anomalous
                    // session.
                    continue;
                };
                found_session_names.insert(fname_and_type);
                sessions.insert(
                    name.to_string(),
                    SessionInfo {
                        command: cmd,
                        sess_type: if is_x11 {
                            SessionType::X11
                        } else {
                            SessionType::Wayland
                        },
                    },
                );
            }
        }

        Ok(sessions)
    }

    /// Get the mapping of a user's full name to their system username.
    ///
    /// If the full name is not available, their system username is used.
    pub fn get_users(&self) -> &UserMap {
        &self.users
    }

    /// Get the mapping of a system username to their shell.
    pub fn get_shells(&self) -> &ShellMap {
        &self.shells
    }

    /// Get the mapping of a session's full name to its command.
    ///
    /// If the full name is not available, the filename stem is used.
    pub fn get_sessions(&self) -> &SessionMap {
        &self.sessions
    }
}
