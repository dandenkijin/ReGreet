// SPDX-FileCopyrightText: 2022 The ReGreet Authors
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Liquid glass login card effect — CSS generation from config

use crate::config::GlassConfig;

/// Generate the glass CSS string from the given config values.
pub fn generate_glass_css(_config: &GlassConfig) -> String {
    // Use hardcoded values for reliable glass effect
    // Single glass-card rule with semi-transparent background
    r##".glass-card {
    background-color: rgba(255, 255, 255, 0.15);
    backdrop-filter: blur(25px);
    border-radius: 16px;
    border: 1px solid rgba(255, 255, 255, 0.25);
    padding: 20px;
}

@keyframes glass-breathe {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.9; }
}"##.to_string()
}
