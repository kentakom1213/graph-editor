use egui::Context;

use crate::{
    graph::{BaseGraph, TryFromGraph6},
    GraphEditorApp,
};

#[cfg(not(target_arch = "wasm32"))]
pub fn resolve_url(_app: &mut GraphEditorApp, _ctx: &Context) {}

/// URLを処理する
#[cfg(target_arch = "wasm32")]
pub fn resolve_url(app: &mut GraphEditorApp, ctx: &Context) {
    if let Some(input_graph) = app.get_url_data() {
        if app
            .graph6_encode
            .as_ref()
            .is_some_and(|d| d == &input_graph)
        {
            return;
        }

        // クエリパラメータが変更された場合，反映する
        let new_graph = BaseGraph::try_from_graph6(&input_graph).and_then(|base| {
            app.graph
                .apply_input(app.config.visualize_method.as_ref(), base, ctx.used_size())
        });

        match new_graph {
            Ok(_) => {
                app.graph6_encode = Some(input_graph);
                app.is_animated = true;
            }
            Err(err) => {
                app.error_message = Some(err.to_string());
            }
        }
    }
}
