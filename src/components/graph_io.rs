use egui::Context;

use crate::{graph::BaseGraph, GraphEditorApp};

/// グラフのエンコードを表示する
pub fn draw_graph_io(app: &mut GraphEditorApp, ctx: &Context) {
    if !app.hovered_on_input_window {
        app.input_text = app.graph.encode(app.zero_indexed)
    }

    // テキストの表示
    egui::SidePanel::right("Graph I/O").show(ctx, |ui| {
        // カーソルがあるか判定
        app.hovered_on_input_window = ui.rect_contains_pointer(ui.max_rect());

        egui::Frame::default()
            .inner_margin(egui::Margin::same(10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .button(egui::RichText::new("Copy").size(app.config.menu_font_size_normal))
                        .clicked()
                    {
                        ctx.copy_text(app.input_text.clone());
                    }

                    if ui
                        .button(egui::RichText::new("Apply").size(app.config.menu_font_size_normal))
                        .clicked()
                    {
                        let new_graph = BaseGraph::parse(&app.input_text, app.zero_indexed)
                            .and_then(|base| {
                                app.graph.apply_input(
                                    app.config.visualize_method.as_ref(),
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
                if ui.code_editor(&mut app.input_text).has_focus() {
                    app.hovered_on_input_window = true;
                }
            });
    });
}
