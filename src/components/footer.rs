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
        app.ui
            .cursor_hover
            .set_footer_panel(ui.rect_contains_pointer(ui.max_rect()));

        ui.horizontal(|ui| {
            if ui
                .button(egui::RichText::new("⚙").size(app.config.footer_font_size()))
                .on_hover_text("Toggle settings")
                .clicked()
            {
                app.ui.show_settings = !app.ui.show_settings;
            }
            ui.separator();
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
                ui.separator();
                let animate = ui
                    .selectable_label(app.state.is_animated, "Animate")
                    .on_hover_text("Toggle force-directed layout animation");
                if animate.clicked() {
                    app.set_animation_enabled(!app.state.is_animated);
                }
                let show_numbers = ui
                    .selectable_label(app.state.show_number, "Number")
                    .on_hover_text("Toggle vertex number labels");
                if show_numbers.clicked() {
                    app.state.show_number = !app.state.show_number;
                }
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
