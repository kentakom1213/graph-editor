use egui::Context;

use crate::config::APP_VERSION;

/// フッターを描画する
pub fn draw_footer(ctx: &Context) {
    // 画面下部にフッターを追加
    let screen_rect = ctx.screen_rect();
    let footer_height = 30.0;
    let footer_pos = egui::pos2(20.0, screen_rect.max.y - footer_height);

    egui::Area::new("Footer".into())
        .fixed_pos(footer_pos)
        .show(ctx, |ui| {
            egui::Frame::default().show(ui, |ui| {
                ui.horizontal_centered(|ui| {
                    ui.label(format!("Graph Editor v{APP_VERSION} © 2025 kentakom1213"));

                    if ui
                        .hyperlink_to("GitHub", "https://github.com/kentakom1213/graph-editor")
                        .clicked()
                    {}
                });
            });
        });
}
