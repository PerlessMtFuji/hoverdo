//! OS-level window effects (Mica/Acrylic on Windows).
//!
//! Applied to the main (library) window only: it renders against a Mica
//! surface on Windows 11+, with Acrylic as the Windows 10 fallback. Other
//! targets are supported only for development convenience and get no native
//! effect.
//!
//! Widget windows deliberately skip this. They are transparent, borderless
//! and rounded; a Mica/Acrylic backdrop would paint an opaque square material
//! behind the rounded card, exposing the corners and turning the opacity
//! slider into a dimmer instead of real see-through transparency.

#[cfg(target_os = "windows")]
pub fn apply_window_effects(window: &tauri::WebviewWindow) {
    use window_vibrancy::{apply_acrylic, apply_mica};

    // Prefer Mica (Windows 11+). On older builds fall back to Acrylic so the
    // window is still translucent rather than opaque.
    if apply_mica(window, None).is_err() {
        let _ = apply_acrylic(window, Some((18, 18, 18, 125)));
    }
}

#[cfg(not(target_os = "windows"))]
pub fn apply_window_effects(_window: &tauri::WebviewWindow) {
    // No-op on non-Windows targets - we still want `cargo check` and `tauri dev`
    // to work on Linux/macOS development hosts.
}
