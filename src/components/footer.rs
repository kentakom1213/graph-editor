use egui::Context;

use crate::{config::APP_VERSION, mode::EditMode, GraphEditorApp};

/// フッターを描画する
pub fn draw_footer(app: &mut GraphEditorApp, ctx: &Context) {
    let vertex_count = app
        .state
        .graph
        .vertices
        .iter()
        .filter(|vertex| !vertex.is_deleted)
        .count();
    let edge_count = app
        .state
        .graph
        .edges()
        .iter()
        .filter(|edge| !edge.is_deleted)
        .count();
    let indexing_label = if app.state.zero_indexed {
        "0-indexed"
    } else {
        "1-indexed"
    };
    let direction_label = if app.state.graph.is_directed {
        "Directed"
    } else {
        "Undirected"
    };

    egui::TopBottomPanel::bottom("footer_panel").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(
                egui::RichText::new(format!(
                    "Mode: {} | {} | {} | vertices: {} | edges: {}",
                    edit_mode_label(&app.state.edit_mode),
                    indexing_label,
                    direction_label,
                    vertex_count,
                    edge_count
                ))
                .size(app.config.footer_font_size()),
            );

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.hyperlink_to(
                    egui::RichText::new("GitHub").size(app.config.footer_font_size()),
                    "https://github.com/kentakom1213/graph-editor",
                );
                ui.label(
                    egui::RichText::new(format!("Graph Editor v{APP_VERSION}"))
                        .size(app.config.footer_font_size()),
                );
            });
        });
    });
}

fn edit_mode_label(mode: &EditMode) -> &'static str {
    match mode {
        EditMode::Normal => "Normal",
        EditMode::AddVertex => "Add Vertex",
        EditMode::AddEdge { .. } => "Add Edge",
        EditMode::Colorize => "Colorize",
        EditMode::Delete => "Delete",
    }
}
