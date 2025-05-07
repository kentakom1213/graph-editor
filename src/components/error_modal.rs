use egui::Context;

use crate::GraphEditorApp;

/// エラー表示を行うモーダル画面
pub fn draw_error_modal(app: &mut GraphEditorApp, ctx: &Context) {
    if let Some(message) = app.error_message.to_owned() {
        // 背景を暗くする
        let screen_rect = ctx.screen_rect();
        let dark_color = egui::Color32::from_black_alpha(160);
        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Background,
            egui::Id::new("modal_bg"),
        ));
        painter.rect_filled(screen_rect, 0.0, dark_color);

        if ctx.input(|i| {
            i.key_pressed(egui::Key::Escape)
                || i.pointer.any_released() && !app.hovered_on_input_window
        }) {
            app.error_message = None;
            return;
        }

        // エラーモーダル本体
        egui::Window::new(
            egui::RichText::new("Error")
                .strong()
                .size(app.config.menu_font_size_normal)
                .color(egui::Color32::from_rgb(255, 100, 80)),
        )
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .frame(egui::Frame::popup(ctx.style().as_ref()).inner_margin(10.0))
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(message).size(app.config.menu_font_size_normal));
        });
    }
}
