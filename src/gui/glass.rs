// SPDX-FileCopyrightText: 2022 The ReGreet Authors
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Liquid glass login card effect — CSS generation from config

use crate::config::GlassConfig;

/// Generate the glass CSS string from the given config values.
pub fn generate_glass_css(config: &GlassConfig) -> String {
    format!(
        r#".glass-login {{
    background-color: rgba(255, 255, 255, {opacity});
    backdrop-filter: blur({blur}px);
    -webkit-backdrop-filter: blur({blur}px);
    border-radius: {border_radius}px;
    border: 1px solid {border_color};
    box-shadow:
        0 8px 32px {shadow_color},
        inset 0 1px 0 {highlight_color};
    animation: glass-breathe {duration} ease-in-out infinite;
}}

@keyframes glass-breathe {{
    0%, 100% {{ opacity: 1; }}
    50% {{ opacity: {opacity_min}; }}
}}"#,
        opacity = config.opacity,
        blur = config.blur,
        border_radius = config.border_radius,
        border_color = config.border_color,
        shadow_color = config.shadow_color,
        highlight_color = config.highlight_color,
        duration = config.duration,
        opacity_min = config.opacity_min,
    )
}
