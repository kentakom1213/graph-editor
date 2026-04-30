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
        .resizable(false)
        .exact_width(196.0)
        .show(ctx, |ui| {
            app.ui
                .cursor_hover
                .set_inspector_panel(ui.rect_contains_pointer(ui.max_rect()));

            ui.horizontal(|ui| {
                draw_tab_button(
                    ui,
                    app.ui.inspector_tab == InspectorTab::Graph,
                    "Graph",
                    app.config.tab_font_size(),
                    || app.ui.inspector_tab = InspectorTab::Graph,
                );
                draw_tab_button(
                    ui,
                    app.ui.inspector_tab == InspectorTab::View,
                    "View",
                    app.config.tab_font_size(),
                    || app.ui.inspector_tab = InspectorTab::View,
                );
                draw_tab_button(
                    ui,
                    app.ui.inspector_tab == InspectorTab::Io,
                    "I/O",
                    app.config.tab_font_size(),
                    || app.ui.inspector_tab = InspectorTab::Io,
                );
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
    ui.label(egui::RichText::new("Indexing").size(app.config.section_font_size()));
    draw_toggle_button(
        ui,
        app.state.zero_indexed,
        "0-indexed",
        app.config.button_font_size(),
        || app.state.zero_indexed = true,
    );
    draw_toggle_button(
        ui,
        !app.state.zero_indexed,
        "1-indexed",
        app.config.button_font_size(),
        || app.state.zero_indexed = false,
    );

    ui.separator();
    ui.label(egui::RichText::new("Direction").size(app.config.section_font_size()));
    draw_toggle_button(
        ui,
        !app.state.graph.is_directed,
        "Undirected",
        app.config.button_font_size(),
        || app.state.graph.is_directed = false,
    );
    draw_toggle_button(
        ui,
        app.state.graph.is_directed,
        "Directed",
        app.config.button_font_size(),
        || app.state.graph.is_directed = true,
    );

    ui.separator();
    ui.label(egui::RichText::new("Operations").size(app.config.section_font_size()));

    if ui
        .add_enabled(
            !app.state.graph.is_directed,
            egui::Button::new(
                egui::RichText::new("Complement").size(app.config.button_font_size()),
            ),
        )
        .clicked()
    {
        app.rebuild_from_base_graph(ctx, app.state.graph.calc_complement());
    }

    if ui
        .add_enabled(
            app.state.graph.is_directed,
            egui::Button::new(
                egui::RichText::new("Revert Edge").size(app.config.button_font_size()),
            ),
        )
        .clicked()
    {
        app.rebuild_from_base_graph(ctx, app.state.graph.calc_reverted());
    }

    if ui
        .button(egui::RichText::new("Reset Colors").size(app.config.button_font_size()))
        .clicked()
    {
        app.state.graph_view.reset_colors();
    }

    ui.separator();
    ui.label(egui::RichText::new("Danger Zone").size(app.config.section_font_size()));
    if ui
        .button(egui::RichText::new("Clear All").size(app.config.button_font_size()))
        .clicked()
    {
        app.ui.confirm_clear_all = true;
    }
}

fn draw_view_tab(app: &mut GraphEditorApp, ui: &mut egui::Ui) {
    ui.label(egui::RichText::new("Display").size(app.config.section_font_size()));
    ui.checkbox(
        &mut app.state.show_number,
        egui::RichText::new("Show Numbers").size(app.config.body_font_size()),
    );

    ui.separator();
    ui.label(egui::RichText::new("Simulation").size(app.config.section_font_size()));
    let mut is_animated = app.state.is_animated;
    if ui
        .checkbox(
            &mut is_animated,
            egui::RichText::new("Animate").size(app.config.body_font_size()),
        )
        .changed()
    {
        app.set_animation_enabled(is_animated);
    }
}

fn draw_io_tab(app: &mut GraphEditorApp, ctx: &Context, ui: &mut egui::Ui) {
    const GRAPH_TEXT_EDITOR_HEIGHT: f32 = 280.0;

    if !app.ui.input_has_focus && !app.ui.input_is_dirty {
        app.sync_input_text_from_graph();
    }

    ui.label(egui::RichText::new("Graph Text").size(app.config.section_font_size()));
    ui.horizontal(|ui| {
        if ui
            .button(egui::RichText::new("Copy").size(app.config.button_font_size()))
            .clicked()
        {
            ctx.copy_text(app.ui.input_text.clone());
        }

        if ui
            .button(egui::RichText::new("Apply").size(app.config.button_font_size()))
            .clicked()
        {
            let new_graph = BaseGraph::parse(&app.ui.input_text, app.state.zero_indexed);
            match new_graph {
                Ok(base_graph) => app.rebuild_from_base_graph(ctx, base_graph),
                Err(err) => app.ui.error_message = Some(err.to_string()),
            }
        }
    });

    ui.separator();

    let editor = egui::TextEdit::multiline(&mut app.ui.input_text)
        .font(egui::FontId::monospace(app.config.input_font_size()))
        .desired_rows(10)
        .desired_width(f32::INFINITY);
    let response = egui::ScrollArea::vertical()
        .id_salt("graph_text_editor_scroll")
        .max_height(GRAPH_TEXT_EDITOR_HEIGHT)
        .show(ui, |ui| {
            ui.add_sized([ui.available_width(), GRAPH_TEXT_EDITOR_HEIGHT], editor)
        })
        .inner;
    app.ui.input_has_focus = response.has_focus();
    app.ui.input_is_dirty = app.ui.input_text != app.ui.input_synced_text;

    ui.separator();
    ui.label(egui::RichText::new("Export Image").size(app.config.section_font_size()));
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Format").size(app.config.body_font_size()));
        let mut format = app.export.format();
        egui::ComboBox::from_id_salt("inspector_export_format")
            .selected_text(match format {
                crate::export::ExportFormat::Png => {
                    egui::RichText::new("PNG").size(app.config.body_font_size())
                }
                crate::export::ExportFormat::Svg => {
                    egui::RichText::new("SVG").size(app.config.body_font_size())
                }
            })
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut format,
                    crate::export::ExportFormat::Png,
                    egui::RichText::new("PNG").size(app.config.body_font_size()),
                );
                ui.selectable_value(
                    &mut format,
                    crate::export::ExportFormat::Svg,
                    egui::RichText::new("SVG").size(app.config.body_font_size()),
                );
            });
        app.export.set_format(format);
    });

    if ui
        .add_enabled(
            !app.export.is_busy(),
            egui::Button::new(egui::RichText::new("Export").size(app.config.button_font_size())),
        )
        .clicked()
    {
        app.request_export_image(ctx);
    }
}

fn draw_toggle_button(
    ui: &mut egui::Ui,
    selected: bool,
    label: &str,
    font_size: f32,
    on_click: impl FnOnce(),
) {
    if ui
        .add_sized(
            [170.0, 32.0],
            egui::SelectableLabel::new(selected, egui::RichText::new(label).size(font_size)),
        )
        .clicked()
    {
        on_click();
    }
}

fn draw_tab_button(
    ui: &mut egui::Ui,
    selected: bool,
    label: &str,
    font_size: f32,
    on_click: impl FnOnce(),
) {
    if ui
        .add_sized(
            [56.0, 30.0],
            egui::SelectableLabel::new(selected, egui::RichText::new(label).size(font_size)),
        )
        .clicked()
    {
        on_click();
    }
}
