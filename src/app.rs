use eframe::egui;
use serde::{Deserialize, Serialize};

use crate::components::{
    draw_central_panel, draw_clear_all_modal, draw_error_modal, draw_footer, draw_inspector_panel,
    draw_tool_bar, draw_top_panel, Colors, CursorHoverState, InspectorTab,
};
use crate::config::{AppConfig, SimulatorKind};
use crate::export::{ExportFormat, ExportService};
use crate::graph::{simulation_methods, BaseGraph, Simulator};
use crate::math::affine::Affine2D;
use crate::mode::EditMode;
use crate::state::{AppState, UiState};
use crate::update::request_repaint;
use crate::view_state::GraphViewState;

pub struct GraphEditorApp {
    pub state: AppState,
    pub ui: UiState,
    pub export: ExportService,
    pub config: AppConfig,
}

const UI_STATE_STORAGE_KEY: &str = "graph-editor:ui-state";
const GRAPH_LAYOUT_SETTLE_STEPS: usize = 120;
const AUTO_FIT_DIAMETER_THRESHOLD: usize = 12;
const EDGE_LENGTH_SHRINK_DIAMETER_THRESHOLD: usize = 10;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct StoredUiState {
    version: u32,
    zero_indexed: bool,
    show_number: bool,
    is_animated: bool,
    is_directed: bool,
    export_format: String,
    title_font_size: f32,
    ui_font_size: f32,
    vertex_font_size: f32,
    vertex_radius: f32,
    vertex_stroke: f32,
    edge_stroke: f32,
    edge_bezier_distance: f32,
    scale_min: f32,
    scale_max: f32,
    scale_delta: f32,
}

impl Default for StoredUiState {
    fn default() -> Self {
        let defaults = AppConfig::default();
        Self {
            version: 2,
            zero_indexed: false,
            show_number: true,
            is_animated: true,
            is_directed: false,
            export_format: ExportFormat::Png.extension().to_string(),
            title_font_size: defaults.title_font_size,
            ui_font_size: defaults.ui_font_size,
            vertex_font_size: defaults.vertex_font_size,
            vertex_radius: defaults.vertex_radius,
            vertex_stroke: defaults.vertex_stroke,
            edge_stroke: defaults.edge_stroke,
            edge_bezier_distance: defaults.edge_bezier_distance,
            scale_min: defaults.scale_min,
            scale_max: defaults.scale_max,
            scale_delta: defaults.scale_delta,
        }
    }
}

impl GraphEditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();
        let state: StoredUiState = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, UI_STATE_STORAGE_KEY))
            .unwrap_or_default();
        app.state.zero_indexed = state.zero_indexed;
        app.state.show_number = state.show_number;
        app.state.is_animated = state.is_animated;
        app.state.graph.is_directed = state.is_directed;
        app.config.title_font_size = state.title_font_size;
        app.config.ui_font_size = state.ui_font_size;
        app.config.vertex_font_size = state.vertex_font_size;
        app.config.vertex_radius = state.vertex_radius;
        app.config.vertex_stroke = state.vertex_stroke;
        app.config.edge_stroke = state.edge_stroke;
        app.config.edge_bezier_distance = state.edge_bezier_distance;
        app.config.scale_min = state.scale_min;
        app.config.scale_max = state.scale_max;
        app.config.scale_delta = state.scale_delta;
        app.state.simulation_edge_length = app.config.simulator_config.l;
        let format = match state.export_format.as_str() {
            "svg" => ExportFormat::Svg,
            _ => ExportFormat::Png,
        };
        app.export.set_format(format);
        app.sync_input_text_from_graph();
        app
    }

    pub fn deselect_all_vertices_edges(&mut self) {
        for vertex in &mut self.state.graph_view.vertices {
            vertex.is_pressed = false;
            vertex.is_selected = false;
        }
        for edge in &mut self.state.graph_view.edges {
            edge.is_pressed = false;
        }
    }

    pub fn switch_normal_mode(&mut self) {
        self.deselect_all_vertices_edges();
        self.state.edit_mode = EditMode::default_normal();
    }

    pub fn switch_add_vertex_mode(&mut self) {
        self.deselect_all_vertices_edges();
        self.state.edit_mode = EditMode::default_add_vertex();
    }

    pub fn switch_add_edge_mode(&mut self) {
        self.deselect_all_vertices_edges();
        self.state.edit_mode = EditMode::default_add_edge();
    }

    pub fn switch_colorize_mode(&mut self) {
        self.deselect_all_vertices_edges();
        self.state.edit_mode = EditMode::default_colorize();
    }

    pub fn switch_delete_mode(&mut self) {
        self.deselect_all_vertices_edges();
        self.state.edit_mode = EditMode::default_delete();
    }

    pub fn request_export_image(&mut self, ctx: &egui::Context) {
        let export_ctx = crate::export::ExportContext {
            graph: &self.state.graph,
            view: &self.state.graph_view,
            config: &self.config,
            show_number: self.state.show_number,
            zero_indexed: self.state.zero_indexed,
        };
        if let Some(err) = self.export.request_export(ctx, &export_ctx) {
            self.ui.error_message = Some(err);
        }
    }

    pub fn handle_export_events(&mut self, ctx: &egui::Context) {
        let export_ctx = crate::export::ExportContext {
            graph: &self.state.graph,
            view: &self.state.graph_view,
            config: &self.config,
            show_number: self.state.show_number,
            zero_indexed: self.state.zero_indexed,
        };
        if let Some(err) = self.export.handle_events(ctx, &export_ctx) {
            self.ui.error_message = Some(err);
        }
    }

    pub fn sync_input_text_from_graph(&mut self) {
        let encoded = self.state.graph.encode(self.state.zero_indexed);
        self.ui.input_text = encoded.clone();
        self.ui.input_synced_text = encoded;
        self.ui.input_is_dirty = false;
    }

    fn effective_layout_edge_length(&self) -> f32 {
        let diameter = self.state.graph.approx_diameter_lower_bound();
        if diameter < EDGE_LENGTH_SHRINK_DIAMETER_THRESHOLD {
            return self.config.simulator_config.l;
        }

        let shrink = 1.0 + (diameter as f32).sqrt() * 0.35;
        (self.config.simulator_config.l / shrink).max(self.config.vertex_radius * 2.4)
    }

    fn simulator_with_edge_length(&self, edge_length: f32) -> Box<dyn Simulator> {
        match self.config.simulator_kind {
            SimulatorKind::ForceDirected => {
                let mut config = self.config.simulator_config;
                config.l = edge_length;
                Box::new(simulation_methods::ForceDirectedModel { config })
            }
        }
    }

    pub fn current_simulator(&self) -> Box<dyn Simulator> {
        self.simulator_with_edge_length(self.state.simulation_edge_length)
    }

    fn settle_graph_layout(&mut self, edge_length: f32) {
        self.state.simulation_edge_length = edge_length;
        let simulator = self.simulator_with_edge_length(edge_length);
        for _ in 0..GRAPH_LAYOUT_SETTLE_STEPS {
            simulator.simulate_step(&mut self.state.graph);
        }

        for vertex in &mut self.state.graph.vertices {
            vertex.velocity = egui::Vec2::ZERO;
        }
    }

    fn auto_fit_graph_to_canvas(&mut self, canvas_rect: egui::Rect) {
        let should_fit =
            self.state.graph.approx_diameter_lower_bound() >= AUTO_FIT_DIAMETER_THRESHOLD;
        if !should_fit || self.state.graph.vertices.is_empty() {
            return;
        }

        let mut min = self.state.graph.vertices[0].position;
        let mut max = self.state.graph.vertices[0].position;
        for vertex in &self.state.graph.vertices[1..] {
            min.x = min.x.min(vertex.position.x);
            min.y = min.y.min(vertex.position.y);
            max.x = max.x.max(vertex.position.x);
            max.y = max.y.max(vertex.position.y);
        }

        let graph_rect = egui::Rect::from_min_max(min, max);
        let target_rect = canvas_rect.shrink2(canvas_rect.size() * 0.08);
        let graph_size = graph_rect.size();
        let target_size = target_rect.size();

        let scale_x = if graph_size.x <= f32::EPSILON {
            1.0
        } else {
            target_size.x / graph_size.x
        };
        let scale_y = if graph_size.y <= f32::EPSILON {
            1.0
        } else {
            target_size.y / graph_size.y
        };
        let scale = scale_x.min(scale_y).min(1.0);
        let graph_center = graph_rect.center().to_vec2();
        let target_center = target_rect.center().to_vec2();
        let translation = target_center - graph_center * scale;

        *self.state.graph.affine.borrow_mut() = Affine2D([
            [scale, 0.0, translation.x],
            [0.0, scale, translation.y],
            [0.0, 0.0, 1.0],
        ]);
    }

    pub fn rebuild_from_base_graph(&mut self, ctx: &egui::Context, base_graph: BaseGraph) {
        let canvas_rect = self.ui.canvas_rect.unwrap_or_else(|| ctx.available_rect());
        let visualizer = self.config.visualizer();
        let new_graph_result = self.state.graph.rebuild_from_basegraph(
            visualizer.as_ref(),
            self.config.density_threshold,
            base_graph,
            canvas_rect,
        );
        match new_graph_result {
            Ok(_) => {
                let edge_length = self.effective_layout_edge_length();
                self.settle_graph_layout(edge_length);
                self.auto_fit_graph_to_canvas(canvas_rect);
                self.state.graph_view.reset_for_graph(&self.state.graph);
                self.state.next_z_index = self.state.graph.vertices.len() as u32;
                self.state.is_animated = false;
                self.sync_input_text_from_graph();
            }
            Err(err) => {
                self.ui.error_message = Some(err.to_string());
            }
        }
    }
}

impl Default for GraphEditorApp {
    fn default() -> Self {
        let config = AppConfig::default();
        let graph = crate::graph::Graph::default();
        Self {
            state: AppState {
                graph_view: GraphViewState::new_for_graph(&graph),
                graph,
                is_animated: true,
                simulation_edge_length: config.simulator_config.l,
                last_mouse_pos: None,
                next_z_index: 2,
                edit_mode: EditMode::default_normal(),
                selected_color: Colors::Default,
                zero_indexed: false,
                show_number: true,
            },
            ui: UiState {
                cursor_hover: CursorHoverState::default(),
                canvas_rect: None,
                input_text: String::new(),
                input_synced_text: String::new(),
                input_has_focus: false,
                input_is_dirty: false,
                show_settings: false,
                error_message: None,
                confirm_clear_all: false,
                inspector_tab: InspectorTab::default(),
            },
            export: ExportService::default(),
            config,
        }
    }
}

impl eframe::App for GraphEditorApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let state = StoredUiState {
            version: 2,
            zero_indexed: self.state.zero_indexed,
            show_number: self.state.show_number,
            is_animated: self.state.is_animated,
            is_directed: self.state.graph.is_directed,
            export_format: self.export.format().extension().to_string(),
            title_font_size: self.config.title_font_size,
            ui_font_size: self.config.ui_font_size,
            vertex_font_size: self.config.vertex_font_size,
            vertex_radius: self.config.vertex_radius,
            vertex_stroke: self.config.vertex_stroke,
            edge_stroke: self.config.edge_stroke,
            edge_bezier_distance: self.config.edge_bezier_distance,
            scale_min: self.config.scale_min,
            scale_max: self.config.scale_max,
            scale_delta: self.config.scale_delta,
        };
        eframe::set_value(storage, UI_STATE_STORAGE_KEY, &state);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.state.graph_view.apply_deletions(&self.state.graph);
        self.state.graph.apply_deletions();

        draw_top_panel(self, ctx);
        draw_tool_bar(self, ctx);
        draw_inspector_panel(self, ctx);
        draw_central_panel(self, ctx);
        draw_footer(self, ctx);
        draw_error_modal(self, ctx);
        draw_clear_all_modal(self, ctx);

        self.handle_export_events(ctx);

        // 再描画
        request_repaint(self, ctx);
    }
}
