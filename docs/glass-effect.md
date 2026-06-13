# Liquid Glass Effect — Wayland Compositor Compatibility

## Overview

The liquid glass effect uses GTK 4's CSS `backdrop-filter: blur()` property to create a frosted glass login card. This requires both GTK 4.22+ and compositor support for the Wayland `wlr-layer-shell` protocol with blur capabilities.

## GTK Version Requirement

- **Minimum**: GTK 4.22 (released March 2024)
- **Your system**: GTK 4.22.4 ✅

## Compositor Support Matrix

| Compositor | Blur Support | Notes |
|------------|--------------|-------|
| **Hyprland** | ✅ Native | Built-in blur/shader pipeline; best experience |
| **KWin (KDE Plasma)** | ✅ Native | Excellent GTK integration |
| **Mutter (GNOME)** | ✅ Native | Reference implementation |
| **Wayfire** | ✅ Good | Supports wlr-layer-shell + blur shaders |
| **Niri** | ✅ Good | Modern compositor; good layer-shell support |
| **Sway (vanilla)** | ❌ No blur | Layer-shell works; blur requires patch |
| **SwayFX** | ✅ Full | Sway fork with blur/shader support |
| **River** | ⚠️ Depends | Needs wlroots ≥ 0.17 + blur protocol support |
| **Labwc** | ⚠️ Depends | Similar to Sway; depends on wlroots version |
| **Cage/Kiosk** | ❌ No | Minimal compositors; no layer-shell/blur |

## What Happens Without Blur Support

If the compositor doesn't support `backdrop-filter: blur()`:

- The **semi-transparent background** (white with alpha) still renders
- The **border** and **border-radius** still render
- The **blur effect is silently ignored** by GTK
- Result: A translucent card instead of frosted glass

This is by design — GTK gracefully degrades unsupported CSS.

## Configuration

Enable in `/etc/greetd/regreet.toml`:

```toml
[glass]
enabled = true
blur = 25              # Blur radius in pixels (GTK 4.22+)
opacity = 0.15         # Background opacity (0.0–1.0)
border_radius = 16     # Card corner radius
border_color = "rgba(255, 255, 255, 0.25)"
```

Or use the FloGreet-compatible alias:

```toml
[appearance]
enable_glass_effect = true
```

## Verifying Support

Run in demo mode to test on your compositor:

```bash
cargo run --release -- --demo
```

Check logs for:
```
INFO regreet::gui::component: Applying liquid glass effect (blur=25px)
```

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| No blur, transparent card | Compositor lacks blur support | Use SwayFX, Hyprland, or enable blur in compositor config |
| Card invisible | Opacity too low | Increase `opacity` (try 0.2–0.3) |
| GTK warnings | Unsupported CSS | Expected on older GTK; upgrade to 4.22+ |
| Flickering | Driver/compositor issue | Update GPU drivers; try different compositor |

## Technical Details

- Uses `gtk::CssProvider` at `STYLE_PROVIDER_PRIORITY_USER`
- Targets `.glass-card` CSS class on the login `gtk::Frame`
- Forces child transparency so `backdrop-filter` sees through to background
- Requires `wlr-layer-shell` for proper fullscreen positioning on Wayland

## References

- [GTK CSS backdrop-filter docs](https://docs.gtk.org/gtk4/css-overview.html#backdrop-filter)
- [wlr-layer-shell protocol](https://github.com/wlr-layer-shell/wlr-layer-shell)
- [Hyprland blur documentation](https://wiki.hyprland.org/Configuring/Variables/#blur)