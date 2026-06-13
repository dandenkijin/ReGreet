// SPDX-FileCopyrightText: 2022 The ReGreet Authors
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Liquid glass login card effect — CSS generation from config

use crate::config::GlassConfig;

/// Generate the glass CSS string from the given config values.
pub fn generate_glass_css(config: &GlassConfig) -> String {
    // Use config values for customizable glass effect
    // Ensure frame, grid, and all children are transparent so backdrop-filter works
    // Format opacity as rgba color for background
    let background_color = format!("rgba(255, 255, 255, {})", config.opacity);
    format!(
        r##".glass-card {{
    background-color: {};
    backdrop-filter: blur({}px);
    border-radius: {}px;
    border: 1px solid {};
    padding: 20px;
}}

/* Force frame and all children transparent so backdrop-filter can see through */
.glass-card,
.glass-card > *,
.glass-card > * > * {{
    background-color: transparent;
}}"##,
        background_color,
        config.blur,
        config.border_radius,
        config.border_color
    )
}
