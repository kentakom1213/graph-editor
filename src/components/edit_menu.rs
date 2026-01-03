use egui::Context;

use crate::{mode::EditMode, GraphEditorApp};

/// 編集メニューを表示する
pub fn draw_edit_menu(app: &mut GraphEditorApp, ctx: &Context) {
    egui::SidePanel::left("Menu")
        .min_width(200.0)
        .show(ctx, |ui| {
            // カーソルがあるか判定
            app.cursor_hover
                .set_menu_window(ui.rect_contains_pointer(ui.max_rect()));

            egui::Frame::new()
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    ui.vertical(|ui| {
                        // モード切替
                        ui.label(
                            egui::RichText::new("Edit Mode").size(app.config.menu_font_size_mini),
                        );

                        ui.radio_value(
                            &mut app.edit_mode,
                            EditMode::default_normal(),
                            egui::RichText::new("Normal [Esc]")
                                .size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.edit_mode,
                            EditMode::default_add_vertex(),
                            egui::RichText::new("Add Vertex [V]")
                                .size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.edit_mode,
                            EditMode::default_add_edge(),
                            egui::RichText::new("Add Edge [E]")
                                .size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.edit_mode,
                            EditMode::default_colorize(),
                            egui::RichText::new("Colorize [C]")
                                .size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.edit_mode,
                            EditMode::default_delete(),
                            egui::RichText::new("Delete [D]")
                                .size(app.config.menu_font_size_normal),
                        );

                        ui.separator();

                        // 0-indexed / 1-indexed の選択
                        ui.label(
                            egui::RichText::new("Indexing [1]")
                                .size(app.config.menu_font_size_mini),
                        );

                        ui.radio_value(
                            &mut app.zero_indexed,
                            true,
                            egui::RichText::new("0-indexed").size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.zero_indexed,
                            false,
                            egui::RichText::new("1-indexed").size(app.config.menu_font_size_normal),
                        );

                        ui.separator();

                        ui.label(
                            egui::RichText::new("Direction [Shift + D]")
                                .size(app.config.menu_font_size_mini),
                        );
                        ui.radio_value(
                            &mut app.graph.is_directed,
                            false,
                            egui::RichText::new("Undirected")
                                .size(app.config.menu_font_size_normal),
                        );
                        ui.radio_value(
                            &mut app.graph.is_directed,
                            true,
                            egui::RichText::new("Directed").size(app.config.menu_font_size_normal),
                        );

                        ui.separator();

                        ui.checkbox(
                            &mut app.is_animated,
                            egui::RichText::new("Animate [A]")
                                .size(app.config.menu_font_size_normal),
                        );

                        ui.separator();

                        // 補グラフを取る
                        let complement_button = egui::Button::new(
                            egui::RichText::new("Complement")
                                .size(app.config.menu_font_size_normal),
                        );
                        let complement_response =
                            ui.add_enabled(!app.graph.is_directed, complement_button);

                        if complement_response.clicked() {
                            let complement = app.graph.calc_complement();
                            let new_graph_result = app.graph.rebuild_from_basegraph(
                                app.config.visualizer.as_ref(),
                                complement,
                                ctx.used_size(),
                            );
                            match new_graph_result {
                                Ok(_) => {
                                    app.is_animated = true;
                                }
                                Err(err) => {
                                    app.error_message = Some(err.to_string());
                                }
                            }
                        }

                        // 逆辺を張る
                        let revert_button = egui::Button::new(
                            egui::RichText::new("Revert Edge")
                                .size(app.config.menu_font_size_normal),
                        );
                        let revert_response = ui.add_enabled(app.graph.is_directed, revert_button);

                        if revert_response.clicked() {
                            let reverted = app.graph.calc_reverted();
                            let new_graph_result = app.graph.rebuild_from_basegraph(
                                app.config.visualizer.as_ref(),
                                reverted,
                                ctx.used_size(),
                            );
                            match new_graph_result {
                                Ok(_) => {
                                    app.is_animated = true;
                                }
                                Err(err) => {
                                    app.error_message = Some(err.to_string());
                                }
                            }
                        }

                        // 色のリセット
                        let reset_color_button = egui::Button::new(
                            egui::RichText::new("Reset Colors")
                                .size(app.config.menu_font_size_normal),
                        );
                        if ui.add(reset_color_button).clicked() {
                            app.graph.reset_colors();
                        }

                        ui.separator();

                        // グラフのクリア
                        ui.label(
                            egui::RichText::new("Clear All").size(app.config.menu_font_size_mini),
                        );

                        if ui
                            .button(
                                egui::RichText::new("Clear All")
                                    .size(app.config.menu_font_size_normal),
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
