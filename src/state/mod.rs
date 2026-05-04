use eframe::egui;

use crate::components::{Colors, CursorHoverState, InspectorTab, PaletteTheme, VertexPattern};
use crate::graph::Graph;
use crate::mode::EditMode;
use crate::view_state::GraphViewState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IoFormat {
    #[default]
    EdgeList,
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditTarget {
    Vertex(usize),
    Edge(usize),
}

pub struct AppState {
    pub graph: Graph,
    pub graph_view: GraphViewState,
    pub is_animated: bool,
    pub simulation_edge_length: f32,
    pub last_mouse_pos: Option<egui::Pos2>,
    pub next_z_index: u32,
    pub edit_mode: EditMode,
    pub selected_color: Colors,
    pub selected_pattern: VertexPattern,
    pub palette_theme: PaletteTheme,
    pub zero_indexed: bool,
    pub show_number: bool,
}

pub struct UiState {
    pub cursor_hover: CursorHoverState,
    pub canvas_rect: Option<egui::Rect>,
    pub input_text: String,
    pub input_synced_text: String,
    pub io_format: IoFormat,
    pub json_text: String,
    pub json_synced_text: String,
    pub input_has_focus: bool,
    pub input_is_dirty: bool,
    pub json_is_dirty: bool,
    pub save_vertex_position: bool,
    pub save_vertex_style: bool,
    pub save_edge_style: bool,
    pub show_settings: bool,
    pub error_message: Option<String>,
    pub confirm_clear_all: bool,
    pub inspector_tab: InspectorTab,
    pub edit_target: Option<EditTarget>,
    pub edit_window_pos: Option<egui::Pos2>,
}
