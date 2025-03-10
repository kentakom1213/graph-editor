use egui::Context;

use crate::GraphEditorApp;

/// グラフのエンコードを表示する
pub fn draw_graph_input(app: &mut GraphEditorApp, ctx: &Context) {
    // テキストの表示
    egui::Window::new("Graph Input")
        .collapsible(true)
        .title_bar(true)
        .show(ctx, |ui| {
            // グラフのコード形式
            let graph_encoded = app.graph.encode(app.zero_indexed);

            egui::Frame::default()
                .inner_margin(egui::Margin::same(10))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        if ui.button(egui::RichText::new("Copy").size(20.0)).clicked() {
                            ctx.copy_text(graph_encoded.clone());
                        }
                        ui.label(" ");

                        // 0-indexed / 1-indexed の選択
                        ui.radio_value(
                            &mut app.zero_indexed,
                            true,
                            egui::RichText::new("0-indexed").size(20.0),
                        );
                        ui.radio_value(
                            &mut app.zero_indexed,
                            false,
                            egui::RichText::new("1-indexed").size(20.0),
                        );
                    });
                    ui.separator();

                    // コード形式で表示
                    ui.label(egui::RichText::new(graph_encoded).monospace().size(20.0));
                });
        });
}
