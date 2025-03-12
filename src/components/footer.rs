use egui::Context;

use crate::{config::APP_VERSION, GraphEditorApp};

/// フッターを描画する
pub fn draw_footer(app: &mut GraphEditorApp, ctx: &Context) {
    // 画面下部にフッターを追加
    let screen_rect = ctx.screen_rect();
    let footer_height = 30.0;
    let footer_pos = egui::pos2(20.0, screen_rect.max.y - footer_height);

    egui::Area::new("Footer".into())
        .fixed_pos(footer_pos)
        .show(ctx, |ui| {
            egui::Frame::default().show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "Graph Editor v{APP_VERSION} © 2025 kentakom1213"
                        ))
                        .size(app.config.menu_font_size_mini),
                    );

                    if ui
                        .hyperlink_to(
                            egui::RichText::new("GitHub").size(app.config.menu_font_size_mini),
                            "https://github.com/kentakom1213/graph-editor",
                        )
                        .clicked()
                    {}
                });
            });
        });
}
