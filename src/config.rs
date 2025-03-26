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
    pub edge_arrow_length: f32,
    pub edge_arrow_width: f32,
    pub edge_bezier_distance: f32,
    pub edge_stroke: f32,
    pub menu_font_size_normal: f32,
    pub menu_font_size_mini: f32,
    pub footer_font_size: f32,
    pub graph_input_font_size: f32,
    /// クーロン定数
    pub simulate_c: f32,
    /// ばね定数
    pub simulate_k: f32,
    /// ばねの自然長
    pub simulate_l: f32,
    /// 減衰定数
    pub simulate_h: f32,
    /// 頂点の重さ
    pub simulate_m: f32,
    /// 微小時間
    pub simulate_time_delta: f32,
    pub simulate_dist_eps: f32,
    pub repaint_duration: std::time::Duration,
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
            edge_arrow_length: 18.0,
            edge_arrow_width: 9.0,
            edge_bezier_distance: 50.0,
            menu_font_size_normal: 20.0,
            menu_font_size_mini: 15.0,
            footer_font_size: 13.0,
            graph_input_font_size: 20.0,
            simulate_c: 5e5,
            simulate_k: 2.0,
            simulate_l: 250.0,
            simulate_h: 0.9,
            simulate_m: 10.0,
            simulate_time_delta: 0.1,
            simulate_dist_eps: 10.0,
            repaint_duration: std::time::Duration::from_millis(5),
        }
    }
}
