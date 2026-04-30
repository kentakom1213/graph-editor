use egui::Color32;

use crate::graph::{simulation_methods, visualize_methods, Simulator, Visualizer};

/// バージョン情報
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// 全体の設定
pub struct AppConfig {
    pub bg_color: Color32,
    pub vertex_radius: f32,
    pub vertex_stroke: f32,
    pub vertex_color_outline: Color32,
    pub vertex_color_dragged: Color32,
    pub vertex_color_selected: Color32,
    pub vertex_font_size: f32,
    pub vertex_font_color: Color32,
    pub edge_color_hover: Color32,
    pub edge_arrow_length: f32,
    pub edge_arrow_width: f32,
    pub edge_bezier_distance: f32,
    pub edge_stroke: f32,
    pub menu_font_size_normal: f32,
    pub section_font_size: f32,
    pub tab_font_size: f32,
    pub button_font_size: f32,
    pub body_font_size: f32,
    pub input_font_size: f32,
    pub footer_font_size: f32,
    /// 最大倍率
    pub scale_max: f32,
    /// 最小倍率
    pub scale_min: f32,
    /// 倍率の刻み
    pub scale_delta: f32,
    /// 回転の刻み（ラジアン）
    pub rotate_delta: f32,
    /// 初期配置を省略する密度のしきい値
    pub density_threshold: f32,
    /// 可視化アルゴリズム種別
    pub visualizer_kind: VisualizerKind,
    /// 可視化アルゴリズム設定
    pub visualizer_config: VisualizerConfig,
    /// シミュレーションアルゴリズム種別
    pub simulator_kind: SimulatorKind,
    /// シミュレーション設定
    pub simulator_config: SimulateConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            bg_color: Color32::from_rgb(230, 230, 230),
            vertex_radius: 36.0,
            vertex_stroke: 3.0,
            vertex_color_outline: Color32::from_rgb(150, 150, 150),
            vertex_color_dragged: Color32::from_rgb(200, 100, 100),
            vertex_color_selected: Color32::from_rgb(100, 200, 100),
            vertex_font_size: 40.0,
            vertex_font_color: Color32::BLACK,
            edge_color_hover: Color32::from_rgb(200, 100, 100),
            edge_stroke: 6.0,
            edge_arrow_length: 18.0,
            edge_arrow_width: 9.0,
            edge_bezier_distance: 50.0,
            menu_font_size_normal: 20.0,
            section_font_size: 15.0,
            tab_font_size: 18.0,
            button_font_size: 18.0,
            body_font_size: 16.0,
            input_font_size: 16.0,
            footer_font_size: 13.0,
            scale_max: 3.0,
            scale_min: 0.1,
            scale_delta: 0.002,
            rotate_delta: 0.03,
            density_threshold: 0.2,
            visualizer_kind: VisualizerKind::Spectral,
            visualizer_config: VisualizerConfig::default(),
            simulator_kind: SimulatorKind::ForceDirected,
            simulator_config: SimulateConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn visualizer(&self) -> Box<dyn Visualizer> {
        match self.visualizer_kind {
            VisualizerKind::Naive => Box::new(visualize_methods::Naive),
            VisualizerKind::Spectral => Box::new(visualize_methods::Spectral),
            VisualizerKind::HillClimbing => Box::new(visualize_methods::HillClimbing(
                self.visualizer_config.hill_climbing_iter,
            )),
            VisualizerKind::SimulatedAnnealing => Box::new(visualize_methods::SimulatedAnnealing {
                max_iter: self.visualizer_config.simulated_annealing_max_iter,
                initial_temp: self.visualizer_config.simulated_annealing_initial_temp,
                cooling_rate: self.visualizer_config.simulated_annealing_cooling_rate,
            }),
        }
    }

    pub fn simulator(&self) -> Box<dyn Simulator> {
        match self.simulator_kind {
            SimulatorKind::ForceDirected => Box::new(simulation_methods::ForceDirectedModel {
                config: self.simulator_config,
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualizerKind {
    Naive,
    Spectral,
    HillClimbing,
    SimulatedAnnealing,
}

#[derive(Debug, Clone, Copy)]
pub struct VisualizerConfig {
    pub hill_climbing_iter: usize,
    pub simulated_annealing_max_iter: usize,
    pub simulated_annealing_initial_temp: f32,
    pub simulated_annealing_cooling_rate: f32,
}

impl Default for VisualizerConfig {
    fn default() -> Self {
        Self {
            hill_climbing_iter: 2000,
            simulated_annealing_max_iter: 1000,
            simulated_annealing_initial_temp: 10.0,
            simulated_annealing_cooling_rate: 0.995,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimulatorKind {
    ForceDirected,
}

/// シミュレーションの設定
#[derive(Debug, Clone, Copy)]
pub struct SimulateConfig {
    /// クーロン定数
    pub c: f32,
    /// ばね定数
    pub k: f32,
    /// ばねの自然長
    pub l: f32,
    /// 減衰定数
    pub h: f32,
    /// 頂点の重さ
    pub m: f32,
    /// 最大速度
    pub max_v: f32,
    /// 微小時間
    pub dt: f32,
}

impl Default for SimulateConfig {
    fn default() -> Self {
        Self {
            c: 2e5,
            k: 7.0,
            l: 180.0,
            h: 0.73,
            m: 10.0,
            max_v: 100.0,
            dt: 0.2,
        }
    }
}
