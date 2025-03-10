use egui::Context;

use crate::{mode::EditMode, GraphEditorApp};

/// 編集メニューを表示する
pub fn draw_edit_menu(app: &mut GraphEditorApp, ctx: &Context) {
    egui::Window::new("Edit Mode")
        .fixed_size(egui::vec2(200.0, 150.0))
        .collapsible(false)
        .show(ctx, |ui| {
            egui::Frame::new()
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        ui.radio_value(
                            &mut app.edit_mode,
                            EditMode::default_normal(),
                            egui::RichText::new("Normal [Esc]").size(app.config.menu_font_size),
                        );
                        ui.radio_value(
                            &mut app.edit_mode,
                            EditMode::default_add_vertex(),
                            egui::RichText::new("Add Vertex [V]").size(app.config.menu_font_size),
                        );
                        ui.radio_value(
                            &mut app.edit_mode,
                            EditMode::default_add_edge(),
                            egui::RichText::new("Add Edge [E]").size(app.config.menu_font_size),
                        );
                        ui.radio_value(
                            &mut app.edit_mode,
                            EditMode::default_delete_edge(),
                            egui::RichText::new("Delete Edge [D]").size(app.config.menu_font_size),
                        );

                        ui.separator();

                        // グラフのクリア
                        if ui
                            .button(
                                egui::RichText::new("Clear All").size(app.config.menu_font_size),
                            )
                            .clicked()
                        {
                            app.graph.clear();
                            app.next_z_index = 0;
                        }
                    });
                });
        });
}
