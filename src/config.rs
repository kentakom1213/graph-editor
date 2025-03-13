use egui::Color32;

/// バージョン情報
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 全体の設定
pub struct AppConfig {
    pub bg_color: Color32,
    pub vertex_radius: f32,
    pub vertex_stroke: f32,
    pub vertex_color_outline: Color32,
    pub vertex_color_normal: Color32,
    pub vertex_color_dragged: Color32,
    pub vertex_color_selected: Color32,
    pub vertex_font_size: f32,
    pub vertex_font_color: Color32,
    pub edge_color_normal: Color32,
    pub edge_color_hover: Color32,
    pub edge_stroke: f32,
    pub menu_font_size_normal: f32,
    pub menu_font_size_mini: f32,
    pub graph_input_font_size: f32,
    pub footer_font_size: f32,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            bg_color: Color32::from_rgb(230, 230, 230),
            vertex_radius: 50.0,
            vertex_stroke: 3.0,
            vertex_color_outline: Color32::from_rgb(150, 150, 150),
            vertex_color_normal: Color32::WHITE,
            vertex_color_dragged: Color32::from_rgb(200, 100, 100),
            vertex_color_selected: Color32::from_rgb(100, 200, 100),
            vertex_font_size: 50.0,
            vertex_font_color: Color32::BLACK,
            edge_color_normal: Color32::from_rgb(100, 100, 100),
            edge_color_hover: Color32::from_rgb(200, 100, 100),
            edge_stroke: 6.0,
            menu_font_size_normal: 20.0,
            menu_font_size_mini: 14.0,
            graph_input_font_size: 20.0,
            footer_font_size: 14.0,
        }
    }
}
