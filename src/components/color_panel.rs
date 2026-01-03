use egui::Context;

use crate::GraphEditorApp;

use crate::graph::BaseGraph;

/// グラフのエンコードを表示する
pub fn draw_color_settings(app: &mut GraphEditorApp, ctx: &Context) {
    // テキストの表示
    egui::Window::new("Color")
        .collapsible(false)
        .default_width(150.0)
        .show(ctx, |ui| {
            // カーソルがあるか判定
            app.cursor_hover
                .set_color_window(ui.rect_contains_pointer(ui.max_rect()));

            egui::Frame::default()
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                egui::RichText::new("Copy").size(app.config.menu_font_size_normal),
                            )
                            .clicked()
                        {
                            ctx.copy_text(app.input_text.clone());
                        }

                        if ui
                            .button(
                                egui::RichText::new("Apply").size(app.config.menu_font_size_normal),
                            )
                            .clicked()
                        {
                            let new_graph = BaseGraph::parse(&app.input_text, app.zero_indexed)
                                .and_then(|base| {
                                    app.graph.rebuild_from_basegraph(
                                        app.config.visualizer.as_ref(),
                                        base,
                                        ctx.used_size(),
                                    )
                                });

                            match new_graph {
                                Ok(_) => {
                                    app.is_animated = true;
                                }
                                Err(err) => {
                                    app.error_message = Some(err.to_string());
                                }
                            }
                        }
                    });
                    ui.separator();

                    // コード形式で表示
                    app.cursor_hover
                        .set_input_window(ui.code_editor(&mut app.input_text).has_focus());
                });
        });
}
