use egui::Context;

use crate::{graph::BaseGraph, GraphEditorApp};

/// グラフのエンコードを表示する
pub fn draw_graph_io(app: &mut GraphEditorApp, ctx: &Context) {
    if !app.ui.cursor_hover.get_input_window() {
        app.ui.input_text = app.state.graph.encode(app.state.zero_indexed)
    }

    // テキストの表示
    egui::Window::new("Graph Input")
        .collapsible(false)
        .default_width(150.0)
        .show(ctx, |ui| {
            // カーソルがあるか判定
            app.ui.cursor_hover
                .set_input_window(ui.rect_contains_pointer(ui.max_rect()));

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
                            ctx.copy_text(app.ui.input_text.clone());
                        }

                        if ui
                            .button(
                                egui::RichText::new("Apply").size(app.config.menu_font_size_normal),
                            )
                            .clicked()
                        {
                            let new_graph =
                                BaseGraph::parse(&app.ui.input_text, app.state.zero_indexed)
                                .and_then(|base| {
                                    app.state.graph.rebuild_from_basegraph(
                                        app.config.visualizer.as_ref(),
                                        app.config.density_threshold,
                                        base,
                                        ctx.used_size(),
                                    )
                                });

                            match new_graph {
                                Ok(_) => {
                                    app.state.graph_view.reset_for_graph(&app.state.graph);
                                    app.state.next_z_index = app.state.graph.vertices.len() as u32;
                                    app.state.is_animated = true;
                                }
                                Err(err) => {
                                    app.ui.error_message = Some(err.to_string());
                                }
                            }
                        }
                    });
                    ui.separator();

                    // コード形式で表示
                    if ui.code_editor(&mut app.ui.input_text).has_focus() {
                        app.ui.cursor_hover.set_input_window(true);
                    }
                });
        });
}
