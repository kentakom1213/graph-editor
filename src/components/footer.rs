use egui::Context;

use crate::{config::APP_VERSION, GraphEditorApp};

/// フッターを描画する
pub fn draw_footer(app: &mut GraphEditorApp, ctx: &Context) {
    // 画面下部にフッターを追加
    egui::Area::new("Footer".into())
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-20.0, -10.0))
        .show(ctx, |ui| {
            egui::Frame::default().show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "Graph Editor v{APP_VERSION} © 2025 kentakom1213"
                        ))
                        .size(app.config.footer_font_size),
                    );

                    if ui
                        .hyperlink_to(
                            egui::RichText::new("GitHub").size(app.config.footer_font_size),
                            "https://github.com/kentakom1213/graph-editor",
                        )
                        .clicked()
                    {}
                });
            });
        });
}
