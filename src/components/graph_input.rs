use egui::Context;

use crate::GraphEditorApp;

/// グラフのエンコードを表示する
pub fn draw_graph_input(app: &mut GraphEditorApp, ctx: &Context) {
    if !app.hovered_on_input_window {
        app.input_text = app.graph.encode(app.zero_indexed)
    }

    // テキストの表示
    egui::Window::new("Graph Input")
        .collapsible(true)
        .title_bar(true)
        .default_width(20.0)
        .show(ctx, |ui| {
            // カーソルがあるか判定
            app.hovered_on_input_window = ui.rect_contains_pointer(ui.max_rect());

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
                            && app
                                .graph
                                .apply_input(
                                    app.config.visualize_method.as_ref(),
                                    &app.input_text,
                                    app.zero_indexed,
                                    ctx.used_size(),
                                )
                                .is_ok()
                        {
                            app.is_animated = true;
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
