use eframe::egui;

use crate::components::{Colors, CursorHoverState, InspectorTab};
use crate::graph::Graph;
use crate::mode::EditMode;
use crate::view_state::GraphViewState;

pub struct AppState {
    pub graph: Graph,
    pub graph_view: GraphViewState,
    pub is_animated: bool,
    pub last_mouse_pos: Option<egui::Pos2>,
    pub next_z_index: u32,
    pub edit_mode: EditMode,
    pub selected_color: Colors,
    pub zero_indexed: bool,
    pub show_number: bool,
}

pub struct UiState {
    pub cursor_hover: CursorHoverState,
    pub input_text: String,
    pub input_has_focus: bool,
    pub error_message: Option<String>,
    pub confirm_clear_all: bool,
    pub inspector_tab: InspectorTab,
}
