use egui::Context;

use crate::{graph::BaseGraph, GraphEditorApp};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InspectorTab {
    #[default]
    Graph,
    View,
    Io,
}

pub fn draw_inspector_panel(app: &mut GraphEditorApp, ctx: &Context) {
    egui::SidePanel::right("inspector_panel")
        .resizable(true)
        .default_width(260.0)
        .min_width(220.0)
        .show(ctx, |ui| {
            app.ui
                .cursor_hover
                .set_inspector_panel(ui.rect_contains_pointer(ui.max_rect()));

            ui.horizontal(|ui| {
                ui.selectable_value(&mut app.ui.inspector_tab, InspectorTab::Graph, "Graph");
                ui.selectable_value(&mut app.ui.inspector_tab, InspectorTab::View, "View");
                ui.selectable_value(&mut app.ui.inspector_tab, InspectorTab::Io, "I/O");
            });
            ui.separator();

            match app.ui.inspector_tab {
                InspectorTab::Graph => draw_graph_tab(app, ctx, ui),
                InspectorTab::View => draw_view_tab(app, ui),
                InspectorTab::Io => draw_io_tab(app, ctx, ui),
            }
        });
}

fn draw_graph_tab(app: &mut GraphEditorApp, ctx: &Context, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("Indexing").size(app.config.menu_font_size_mini));
    ui.radio_value(&mut app.state.zero_indexed, true, "0-indexed");
    ui.radio_value(&mut app.state.zero_indexed, false, "1-indexed");

    ui.separator();
    ui.label(egui::RichText::new("Direction").size(app.config.menu_font_size_mini));
    ui.radio_value(&mut app.state.graph.is_directed, false, "Undirected");
    ui.radio_value(&mut app.state.graph.is_directed, true, "Directed");

    ui.separator();
    ui.label(egui::RichText::new("Operations").size(app.config.menu_font_size_mini));

    if ui
        .add_enabled(
            !app.state.graph.is_directed,
            egui::Button::new("Complement"),
        )
        .clicked()
    {
        app.rebuild_from_base_graph(ctx, app.state.graph.calc_complement());
    }

    if ui
        .add_enabled(
            app.state.graph.is_directed,
            egui::Button::new("Revert Edge"),
        )
        .clicked()
    {
        app.rebuild_from_base_graph(ctx, app.state.graph.calc_reverted());
    }

    if ui.button("Reset Colors").clicked() {
        app.state.graph_view.reset_colors();
    }

    ui.separator();
    ui.label(egui::RichText::new("Danger Zone").size(app.config.menu_font_size_mini));
    if ui.button("Clear All").clicked() {
        app.ui.confirm_clear_all = true;
    }
}

fn draw_view_tab(app: &mut GraphEditorApp, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("Display").size(app.config.menu_font_size_mini));
    ui.checkbox(&mut app.state.show_number, "Show Numbers");

    ui.separator();
    ui.label(egui::RichText::new("Simulation").size(app.config.menu_font_size_mini));
    ui.checkbox(&mut app.state.is_animated, "Animate");
}

fn draw_io_tab(app: &mut GraphEditorApp, ctx: &Context, ui: &mut egui::Ui) {
    if !app.ui.input_has_focus {
        app.ui.input_text = app.state.graph.encode(app.state.zero_indexed);
    }

    ui.label(egui::RichText::new("Graph Text").size(app.config.menu_font_size_mini));
    ui.horizontal(|ui| {
        if ui.button("Copy").clicked() {
            ctx.copy_text(app.ui.input_text.clone());
        }

        if ui.button("Apply").clicked() {
            let new_graph = BaseGraph::parse(&app.ui.input_text, app.state.zero_indexed);
            match new_graph {
                Ok(base_graph) => app.rebuild_from_base_graph(ctx, base_graph),
                Err(err) => app.ui.error_message = Some(err.to_string()),
            }
        }
    });
    ui.separator();

    let editor = egui::TextEdit::multiline(&mut app.ui.input_text)
        .font(egui::TextStyle::Monospace)
        .desired_rows(12)
        .desired_width(f32::INFINITY);
    let response = ui.add(editor);
    app.ui.input_has_focus = response.has_focus();

    ui.separator();
    ui.label(egui::RichText::new("Export Image").size(app.config.menu_font_size_mini));
    ui.horizontal(|ui| {
        ui.label("Format");
        let mut format = app.export.format();
        egui::ComboBox::from_id_salt("inspector_export_format")
            .selected_text(match format {
                crate::export::ExportFormat::Png => "PNG",
                crate::export::ExportFormat::Svg => "SVG",
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut format, crate::export::ExportFormat::Png, "PNG");
                ui.selectable_value(&mut format, crate::export::ExportFormat::Svg, "SVG");
            });
        app.export.set_format(format);
    });

    if ui
        .add_enabled(!app.export.is_busy(), egui::Button::new("Export"))
        .clicked()
    {
        app.request_export_image(ctx);
    }
}
