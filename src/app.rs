use eframe::egui;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::components::{
    draw_central_panel, draw_clear_all_modal, draw_color_settings, draw_edit_menu,
    draw_error_modal, draw_footer, draw_graph_io, draw_top_panel, Colors, CursorHoverState,
    PanelTabState,
};
use crate::config::AppConfig;
use crate::graph::Graph;
use crate::mode::EditMode;
use crate::update::request_repaint;

pub struct GraphEditorApp {
    pub graph: Graph,
    pub is_animated: bool,
    pub last_mouse_pos: Option<egui::Pos2>,
    pub next_z_index: u32,
    pub edit_mode: EditMode,
    pub selected_color: Colors,
    pub zero_indexed: bool,
    pub cursor_hover: CursorHoverState,
    pub config: AppConfig,
    pub input_text: String,
    pub error_message: Option<String>,
    pub confirm_clear_all: bool,
    pub panel_tab: PanelTabState,
    pub export_format: ExportFormat,
    pub export_in_progress: bool,
    pub export_request: Option<ExportRequest>,
}

const UI_STATE_STORAGE_KEY: &str = "graph-editor:ui-state";

#[derive(Debug, Clone, Serialize, Deserialize)]
struct UiState {
    version: u32,
    zero_indexed: bool,
    is_directed: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            version: 1,
            zero_indexed: false,
            is_directed: false,
        }
    }
}

impl GraphEditorApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut app = Self::default();
        let state: UiState = cc
            .storage
            .and_then(|storage| eframe::get_value(storage, UI_STATE_STORAGE_KEY))
            .unwrap_or_default();
        app.zero_indexed = state.zero_indexed;
        app.graph.is_directed = state.is_directed;
        app
    }

    pub fn deselect_all_vertices_edges(&mut self) {
        for vertex in self.graph.vertices_mut() {
            vertex.is_pressed = false;
            vertex.is_selected = false;
        }
        for edge in self.graph.edges_mut() {
            edge.is_pressed = false;
        }
    }

    pub fn switch_normal_mode(&mut self) {
        self.deselect_all_vertices_edges();
        self.edit_mode = EditMode::default_normal();
    }

    pub fn switch_add_vertex_mode(&mut self) {
        self.deselect_all_vertices_edges();
        self.edit_mode = EditMode::default_add_vertex();
    }

    pub fn switch_add_edge_mode(&mut self) {
        self.deselect_all_vertices_edges();
        self.edit_mode = EditMode::default_add_edge();
    }

    pub fn switch_colorize_mode(&mut self) {
        self.deselect_all_vertices_edges();
        self.edit_mode = EditMode::default_colorize();
    }

    pub fn switch_delete_mode(&mut self) {
        self.deselect_all_vertices_edges();
        self.edit_mode = EditMode::default_delete();
    }

    pub fn request_export_image(&mut self, ctx: &egui::Context) {
        if self.export_in_progress {
            return;
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let default_name = format!("graph.{}", self.export_format.extension());
            let dialog = rfd::FileDialog::new()
                .add_filter("PNG Image", &["png"])
                .add_filter("SVG Image", &["svg"])
                .set_file_name(&default_name);

            let Some(mut path) = dialog.save_file() else {
                return;
            };

            let format = match path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_ascii_lowercase())
                .as_deref()
            {
                Some("svg") => ExportFormat::Svg,
                Some("png") => ExportFormat::Png,
                _ => {
                    path.set_extension(self.export_format.extension());
                    self.export_format
                }
            };

            self.export_in_progress = true;
            self.export_request = Some(ExportRequest {
                format,
                file_name: path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(&default_name)
                    .to_string(),
                path: Some(path),
            });
        }

        #[cfg(target_arch = "wasm32")]
        {
            self.export_in_progress = true;
            self.export_request = Some(ExportRequest {
                format: self.export_format,
                file_name: format!("graph.{}", self.export_format.extension()),
                path: None,
            });
        }

        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
            "graph-export",
        )));
        ctx.request_repaint();
    }

    pub fn handle_export_events(&mut self, ctx: &egui::Context) {
        if self.export_request.is_none() {
            return;
        }

        let screenshot = ctx.input(|i| {
            i.raw.events.iter().find_map(|event| {
                if let egui::Event::Screenshot { image, .. } = event {
                    Some(image.clone())
                } else {
                    None
                }
            })
        });

        let Some(screenshot) = screenshot else {
            return;
        };

        let export_request = self.export_request.take();
        self.export_in_progress = false;

        let Some(export_request) = export_request else {
            return;
        };

        let pixels_per_point = ctx.pixels_per_point();
        let Some(mut region) = graph_bounds_rect(self) else {
            self.error_message = Some("Export failed: no vertices to capture.".to_string());
            return;
        };
        region = region.intersect(ctx.screen_rect());

        if region.width() <= 0.0 || region.height() <= 0.0 {
            self.error_message = Some("Export failed: invalid capture region.".to_string());
            return;
        }

        let mut color_image = screenshot.region(&region, Some(pixels_per_point));

        if color_image.width() == 0 || color_image.height() == 0 {
            self.error_message = Some("Export failed: empty capture region.".to_string());
            return;
        }

        let result = export_color_image(&mut color_image, &export_request);

        if let Err(err) = result {
            self.error_message = Some(err.to_string());
        }
    }
}

impl Default for GraphEditorApp {
    fn default() -> Self {
        Self {
            graph: Graph::default(),
            is_animated: true,
            last_mouse_pos: None,
            next_z_index: 2,
            edit_mode: EditMode::default_normal(),
            selected_color: Colors::Default,
            zero_indexed: false,
            cursor_hover: CursorHoverState::default(),
            config: AppConfig::default(),
            input_text: String::new(),
            error_message: None,
            confirm_clear_all: false,
            panel_tab: PanelTabState::default(),
            export_format: ExportFormat::Png,
            export_in_progress: false,
            export_request: None,
        }
    }
}

impl eframe::App for GraphEditorApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        let state = UiState {
            version: 1,
            zero_indexed: self.zero_indexed,
            is_directed: self.graph.is_directed,
        };
        eframe::set_value(storage, UI_STATE_STORAGE_KEY, &state);
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // トップパネル（タブバー）を描画
        draw_top_panel(self, ctx);

        // メイン領域を描画
        draw_central_panel(self, ctx);

        // 現在選択されているタブに応じてサイドパネルの内容を切り替える
        if self.panel_tab.edit_menu {
            // 編集メニューを描画
            draw_edit_menu(self, ctx);
        }
        if self.panel_tab.color_settings {
            // 色の設定を描画
            draw_color_settings(self, ctx);
        }
        if self.panel_tab.graph_io {
            // グラフの入力を描画
            draw_graph_io(self, ctx);
        }

        // フッターを描画
        draw_footer(self, ctx);

        // エラーメッセージを描画
        draw_error_modal(self, ctx);
        draw_clear_all_modal(self, ctx);

        self.handle_export_events(ctx);

        // 再描画
        request_repaint(self, ctx);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    Png,
    Svg,
}

impl ExportFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Png => "png",
            Self::Svg => "svg",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExportRequest {
    pub format: ExportFormat,
    pub file_name: String,
    pub path: Option<PathBuf>,
}

fn export_color_image(
    color_image: &mut egui::ColorImage,
    export_request: &ExportRequest,
) -> anyhow::Result<()> {
    use anyhow::Context;
    use base64::Engine as _;
    use image::ImageEncoder as _;

    let width = color_image.width() as u32;
    let height = color_image.height() as u32;

    let mut rgba = Vec::with_capacity((width * height) as usize * 4);
    for color in &color_image.pixels {
        rgba.extend_from_slice(&color.to_srgba_unmultiplied());
    }

    let rgba_image =
        image::RgbaImage::from_raw(width, height, rgba).context("failed to build RGBA image")?;

    let (w, h) = (rgba_image.width(), rgba_image.height());

    let png_bytes = {
        let mut bytes = Vec::new();
        let encoder = image::codecs::png::PngEncoder::new(&mut bytes);
        encoder
            .write_image(&rgba_image, w, h, image::ColorType::Rgba8.into())
            .context("failed to encode PNG")?;
        bytes
    };

    let bytes = match export_request.format {
        ExportFormat::Png => png_bytes,
        ExportFormat::Svg => {
            let encoded = base64::engine::general_purpose::STANDARD.encode(png_bytes);
            let svg = format!(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\">\n\
  <image href=\"data:image/png;base64,{encoded}\" width=\"{w}\" height=\"{h}\" />\n\
</svg>\n"
            );
            svg.into_bytes()
        }
    };

    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = export_request
            .path
            .as_ref()
            .context("missing export path")?;
        std::fs::write(path, bytes)
            .with_context(|| format!("failed to save image: {:?}", path))?;
    }

    #[cfg(target_arch = "wasm32")]
    {
        let file_name = export_request.file_name.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let handle = rfd::AsyncFileDialog::new()
                .set_file_name(&file_name)
                .add_filter("PNG Image", &["png"])
                .add_filter("SVG Image", &["svg"])
                .save_file()
                .await;
            if let Some(handle) = handle {
                if let Err(err) = handle.write(&bytes).await {
                    log::error!("failed to save image: {err}");
                }
            }
        });
    }

    Ok(())
}

fn graph_bounds_rect(app: &GraphEditorApp) -> Option<egui::Rect> {
    let mut iter = app.graph.vertices.iter();
    let first = iter.next()?;
    let mut min = first.get_position();
    let mut max = first.get_position();

    for vertex in iter {
        let pos = vertex.get_position();
        min.x = min.x.min(pos.x);
        min.y = min.y.min(pos.y);
        max.x = max.x.max(pos.x);
        max.y = max.y.max(pos.y);
    }

    let padding = app.config.vertex_radius
        + app.config.edge_stroke
        + app
            .config
            .edge_arrow_length
            .max(app.config.edge_arrow_width);

    min.x -= padding;
    min.y -= padding;
    max.x += padding;
    max.y += padding;

    Some(egui::Rect::from_min_max(min, max))
}
