use egui::Context;

use crate::GraphEditorApp;

/// グラフのエンコードを表示する
pub fn draw_graph_input(app: &mut GraphEditorApp, ctx: &Context) {
    // テキストの表示
    egui::Window::new("Graph Input")
        .collapsible(true)
        .title_bar(true)
        .show(ctx, |ui| {
            // カーソルがあるか判定
            app.hovered_on_input_window = ui.rect_contains_pointer(ui.max_rect());

            // グラフのコード形式
            let graph_encoded = app.graph.encode(app.zero_indexed);

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
                            ctx.copy_text(graph_encoded.clone());
                        }
                    });
                    ui.separator();

                    // コード形式で表示
                    ui.label(
                        egui::RichText::new(graph_encoded)
                            .monospace()
                            .size(app.config.graph_input_font_size),
                    );
                });
        });
}
