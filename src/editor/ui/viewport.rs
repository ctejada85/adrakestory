//! Viewport controls and status display.

use bevy_egui::egui;

/// Render viewport controls overlay
pub fn render_viewport_controls(ctx: &egui::Context) {
    // Render a small overlay in the bottom-left of the viewport
    // Positioned above the status bar (which is ~30px tall)
    egui::Window::new("Viewport Controls")
        .anchor(egui::Align2::LEFT_BOTTOM, [10.0, -40.0])
        .title_bar(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Camera Controls:");
            ui.label("• Right-click drag: Orbit");
            ui.label("• Middle-click drag: Pan");
            ui.label("• Scroll: Zoom");
            ui.label("• Home: Reset");
        });
}
