# Liquid Glass Login Card — Design Document

**Date:** 2026-06-12
**Status:** Approved
**Scope:** Visual enhancement — no code changes required

## Overview

Add a liquid glass (frosted glass with breathing animation) effect to the ReGreet login card. Implemented entirely via user-space CSS — no changes to ReGreet's Rust code.

## Visual Design

- **Frosted glass panel** with background blur sitting on the black background
- **Subtle dark tint** (`rgba(255, 255, 255, 0.07)`) — visible against black without overpowering the white kanji
- **Soft border** with low-opacity white for glass edge definition
- **Outer shadow** for depth + **inset highlight** for top-edge light catch
- **Breathing animation** — gentle 4-second opacity oscillation between 100% and 88%

## CSS Implementation

Users add the following to their `regreet.css` (or custom CSS file via `--style`):

```css
/* Liquid glass login card */
.glass-login {
    background-color: rgba(255, 255, 255, 0.07);
    backdrop-filter: blur(39px);
    -webkit-backdrop-filter: blur(39px);
    border-radius: 20px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    box-shadow:
        0 8px 32px rgba(0, 0, 0, 0.3),
        inset 0 1px 0 rgba(255, 255, 255, 0.25);
    animation: glass-breathe 4s ease-in-out infinite;
}

@keyframes glass-breathe {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.88; }
}
```

## Applying the CSS Class

The login card `gtk::Frame` in `src/gui/templates.rs` already uses `add_css_class: "background"`. Users who want the glass effect should:

1. Add the CSS above to their `regreet.css`
2. Either replace `"background"` with `"glass-login"` in a local patch, OR add `"glass-login"` alongside `"background"` if the template is modified

### Note on `::before`/`::after`

The original reference CSS used `::before` and `::after` pseudo-elements for highlight edges. GTK4 CSS does not support these pseudo-elements, so the design omits them. The inset `box-shadow` provides sufficient edge definition.

## GTK Version Requirements

| Feature | Minimum GTK |
|---|---|
| `backdrop-filter: blur()` | GTK 4.22+ (GNOME 50) |
| `@keyframes` animation | GTK 4.0+ |
| `box-shadow` with `inset` | GTK 4.0+ |

On systems with GTK < 4.22, the glass card still renders with the semi-transparent background, border, and animation — just without the backdrop blur. The effect degrades gracefully.

## Compatibility

- **No code changes** — this is a pure CSS customization
- **No new dependencies** — uses existing CSS loading mechanism (`--style` flag / `regreet.css`)
- **No config changes** — no new TOML options
- **Existing custom CSS** — users with existing `regreet.css` files can add the glass classes alongside their current styles

## Files Affected

None. This is a documentation-only change. Optionally, a sample CSS file can be shipped:

- `regreet-glass.sample.css` — example CSS file users can copy from

## Precedents

- ReGreet already supports custom CSS via `regreet.css` (see `src/constants.rs:CSS_PATH`)
- The `"background"` CSS class is already applied to the login frame and clock frame in `src/gui/templates.rs`
- GTK 4.22's `backdrop-filter` support was confirmed in the [GNOME GTK NEWS](https://github.com/GNOME/gtk/blob/main/NEWS) and demonstrated in the wild (March 2026)
