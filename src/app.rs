use eframe::egui;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::components::{
    draw_central_panel, draw_clear_all_modal, draw_color_settings, draw_edit_menu,
    draw_error_modal, draw_footer, draw_graph_io, draw_top_panel, Colors, CursorHoverState,
    PanelTabState,
};
use crate::config::AppConfig;
use crate::graph::Graph;
use crate::math::bezier::{calc_bezier_control_point, calc_intersection_of_bezier_and_circle};
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

        let Some(export_request) = build_export_request(self.export_format) else {
            return;
        };

        if export_request.format == ExportFormat::Svg {
            self.export_in_progress = true;
            let result = export_svg_bytes(self)
                .and_then(|bytes| save_export_bytes(&export_request, bytes));
            self.export_in_progress = false;
            if let Err(err) = result {
                self.error_message = Some(err.to_string());
            }
            return;
        }

        self.export_in_progress = true;
        self.export_request = Some(export_request);
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

        let result = export_color_image(&mut color_image)
            .and_then(|bytes| save_export_bytes(&export_request, bytes));

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

fn export_color_image(color_image: &mut egui::ColorImage) -> anyhow::Result<Vec<u8>> {
    use anyhow::Context;
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

    Ok(png_bytes)
}

fn graph_bounds_rect(app: &GraphEditorApp) -> Option<egui::Rect> {
    let mut iter = app.graph.vertices.iter().filter(|v| !v.is_deleted);
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

fn build_export_request(format: ExportFormat) -> Option<ExportRequest> {
    let default_name = format!("graph.{}", format.extension());

    #[cfg(not(target_arch = "wasm32"))]
    {
        let dialog = rfd::FileDialog::new()
            .add_filter("PNG Image", &["png"])
            .add_filter("SVG Image", &["svg"])
            .set_file_name(&default_name);

        let Some(mut path) = dialog.save_file() else {
            return None;
        };

        let resolved_format = match path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_ascii_lowercase())
            .as_deref()
        {
            Some("svg") => ExportFormat::Svg,
            Some("png") => ExportFormat::Png,
            _ => {
                path.set_extension(format.extension());
                format
            }
        };

        return Some(ExportRequest {
            format: resolved_format,
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
        Some(ExportRequest {
            format,
            file_name: default_name,
            path: None,
        })
    }
}

fn save_export_bytes(export_request: &ExportRequest, bytes: Vec<u8>) -> anyhow::Result<()> {
    use anyhow::Context;

    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = export_request
            .path
            .as_ref()
            .context("missing export path")?;
        std::fs::write(path, bytes)
            .with_context(|| format!("failed to save image: {:?}", path))?;
        return Ok(());
    }

    #[cfg(target_arch = "wasm32")]
    {
        let file_name = export_request.file_name.clone();
        let filter_label = match export_request.format {
            ExportFormat::Png => "PNG Image",
            ExportFormat::Svg => "SVG Image",
        };
        let filter_ext = match export_request.format {
            ExportFormat::Png => "png",
            ExportFormat::Svg => "svg",
        };
        wasm_bindgen_futures::spawn_local(async move {
            let handle = rfd::AsyncFileDialog::new()
                .set_file_name(&file_name)
                .add_filter(filter_label, &[filter_ext])
                .save_file()
                .await;
            if let Some(handle) = handle {
                if let Err(err) = handle.write(&bytes).await {
                    log::error!("failed to save image: {err}");
                }
            }
        });
        Ok(())
    }
}

fn export_svg_bytes(app: &GraphEditorApp) -> anyhow::Result<Vec<u8>> {
    use anyhow::Context;

    let bounds = graph_bounds_rect(app).context("missing graph bounds")?;
    let width = bounds.width().max(1.0);
    let height = bounds.height().max(1.0);

    let mut svg = String::new();
    svg.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\">\n"
    ));

    let (bg_hex, bg_alpha) = color_to_svg(app.config.bg_color);
    if let Some(alpha) = bg_alpha {
        svg.push_str(&format!(
            "  <rect width=\"100%\" height=\"100%\" fill=\"{bg_hex}\" fill-opacity=\"{alpha}\" />\n"
        ));
    } else {
        svg.push_str(&format!(
            "  <rect width=\"100%\" height=\"100%\" fill=\"{bg_hex}\" />\n"
        ));
    }

    let mut vertices: Vec<_> = app.graph.vertices.iter().filter(|v| !v.is_deleted).collect();
    let mut edges: Vec<_> = app.graph.edges.iter().filter(|e| !e.is_deleted).collect();

    let vertex_map: HashMap<usize, _> = vertices.iter().map(|v| (v.id, *v)).collect();

    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();
    for edge in edges.iter() {
        *edge_count.entry((edge.from, edge.to)).or_insert(0) += 1;
        *edge_count.entry((edge.to, edge.from)).or_insert(0) += 1;
    }

    for edge in edges.drain(..) {
        let Some(from_vertex) = vertex_map.get(&edge.from) else {
            continue;
        };
        let Some(to_vertex) = vertex_map.get(&edge.to) else {
            continue;
        };

        let from_pos = from_vertex.get_position();
        let to_pos = to_vertex.get_position();
        let edge_color = edge.color.edge();
        let (stroke_hex, stroke_alpha) = color_to_svg(edge_color);
        let stroke_style = if let Some(alpha) = stroke_alpha {
            format!("stroke=\"{stroke_hex}\" stroke-opacity=\"{alpha}\"")
        } else {
            format!("stroke=\"{stroke_hex}\"")
        };

        let from_x = from_pos.x - bounds.min.x;
        let from_y = from_pos.y - bounds.min.y;
        let to_x = to_pos.x - bounds.min.x;
        let to_y = to_pos.y - bounds.min.y;

        if app.graph.is_directed {
            if edge_count.get(&(edge.from, edge.to)) == Some(&1) {
                let dir = (to_pos - from_pos).normalized();
                let arrowhead = to_pos - dir * app.config.vertex_radius;
                let endpoint = arrowhead - dir * app.config.edge_arrow_length;
                let arrow_dir = dir * app.config.edge_arrow_length;
                let left = egui::Pos2::new(
                    arrowhead.x - arrow_dir.x
                        - arrow_dir.y
                            * (app.config.edge_arrow_width / app.config.edge_arrow_length),
                    arrowhead.y - arrow_dir.y
                        + arrow_dir.x
                            * (app.config.edge_arrow_width / app.config.edge_arrow_length),
                );
                let right = egui::Pos2::new(
                    arrowhead.x - arrow_dir.x
                        + arrow_dir.y
                            * (app.config.edge_arrow_width / app.config.edge_arrow_length),
                    arrowhead.y - arrow_dir.y
                        - arrow_dir.x
                            * (app.config.edge_arrow_width / app.config.edge_arrow_length),
                );

                let end_x = endpoint.x - bounds.min.x;
                let end_y = endpoint.y - bounds.min.y;
                svg.push_str(&format!(
                    "  <line x1=\"{from_x}\" y1=\"{from_y}\" x2=\"{end_x}\" y2=\"{end_y}\" {stroke_style} stroke-width=\"{}\" fill=\"none\" />\n",
                    app.config.edge_stroke
                ));

                let (fill_hex, fill_alpha) = color_to_svg(edge_color);
                if let Some(alpha) = fill_alpha {
                    svg.push_str(&format!(
                        "  <polygon points=\"{} {} {}\" fill=\"{fill_hex}\" fill-opacity=\"{alpha}\" />\n",
                        svg_point(arrowhead, bounds),
                        svg_point(left, bounds),
                        svg_point(right, bounds)
                    ));
                } else {
                    svg.push_str(&format!(
                        "  <polygon points=\"{} {} {}\" fill=\"{fill_hex}\" />\n",
                        svg_point(arrowhead, bounds),
                        svg_point(left, bounds),
                        svg_point(right, bounds)
                    ));
                }
            } else {
                let control = calc_bezier_control_point(
                    from_pos,
                    to_pos,
                    app.config.edge_bezier_distance,
                    false,
                );
                if let Some((arrowhead, dir)) = calc_intersection_of_bezier_and_circle(
                    from_pos,
                    control,
                    to_pos,
                    to_pos,
                    app.config.vertex_radius,
                ) {
                    let control_x = control.x - bounds.min.x;
                    let control_y = control.y - bounds.min.y;
                    svg.push_str(&format!(
                        "  <path d=\"M {from_x} {from_y} Q {control_x} {control_y} {to_x} {to_y}\" {stroke_style} stroke-width=\"{}\" fill=\"none\" />\n",
                        app.config.edge_stroke
                    ));

                    let mask_start =
                        arrowhead - dir.normalized() * app.config.edge_arrow_length / 2.0;
                    let (mask_hex, mask_alpha) = color_to_svg(app.config.bg_color);
                    let mask_style = if let Some(alpha) = mask_alpha {
                        format!("stroke=\"{mask_hex}\" stroke-opacity=\"{alpha}\"")
                    } else {
                        format!("stroke=\"{mask_hex}\"")
                    };
                    svg.push_str(&format!(
                        "  <line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" {mask_style} stroke-width=\"{}\" fill=\"none\" />\n",
                        mask_start.x - bounds.min.x,
                        mask_start.y - bounds.min.y,
                        arrowhead.x - bounds.min.x,
                        arrowhead.y - bounds.min.y,
                        app.config.edge_stroke
                    ));

                    let arrow_dir = dir.normalized() * app.config.edge_arrow_length;
                    let left = egui::Pos2::new(
                        arrowhead.x - arrow_dir.x
                            - arrow_dir.y
                                * (app.config.edge_arrow_width / app.config.edge_arrow_length),
                        arrowhead.y - arrow_dir.y
                            + arrow_dir.x
                                * (app.config.edge_arrow_width / app.config.edge_arrow_length),
                    );
                    let right = egui::Pos2::new(
                        arrowhead.x - arrow_dir.x
                            + arrow_dir.y
                                * (app.config.edge_arrow_width / app.config.edge_arrow_length),
                        arrowhead.y - arrow_dir.y
                            - arrow_dir.x
                                * (app.config.edge_arrow_width / app.config.edge_arrow_length),
                    );
                    let (fill_hex, fill_alpha) = color_to_svg(edge_color);
                    if let Some(alpha) = fill_alpha {
                        svg.push_str(&format!(
                            "  <polygon points=\"{} {} {}\" fill=\"{fill_hex}\" fill-opacity=\"{alpha}\" />\n",
                            svg_point(arrowhead, bounds),
                            svg_point(left, bounds),
                            svg_point(right, bounds)
                        ));
                    } else {
                        svg.push_str(&format!(
                            "  <polygon points=\"{} {} {}\" fill=\"{fill_hex}\" />\n",
                            svg_point(arrowhead, bounds),
                            svg_point(left, bounds),
                            svg_point(right, bounds)
                        ));
                    }
                }
            }
        } else {
            svg.push_str(&format!(
                "  <line x1=\"{from_x}\" y1=\"{from_y}\" x2=\"{to_x}\" y2=\"{to_y}\" {stroke_style} stroke-width=\"{}\" fill=\"none\" />\n",
                app.config.edge_stroke
            ));
        }
    }

    vertices.sort_by_key(|v| v.z_index);
    for vertex in vertices {
        let pos = vertex.get_position();
        let x = pos.x - bounds.min.x;
        let y = pos.y - bounds.min.y;
        let fill_color = vertex.color.vertex();
        let (fill_hex, fill_alpha) = color_to_svg(fill_color);
        if let Some(alpha) = fill_alpha {
            svg.push_str(&format!(
                "  <circle cx=\"{x}\" cy=\"{y}\" r=\"{}\" fill=\"{fill_hex}\" fill-opacity=\"{alpha}\" />\n",
                app.config.vertex_radius
            ));
        } else {
            svg.push_str(&format!(
                "  <circle cx=\"{x}\" cy=\"{y}\" r=\"{}\" fill=\"{fill_hex}\" />\n",
                app.config.vertex_radius
            ));
        }

        let (stroke_hex, stroke_alpha) = color_to_svg(app.config.vertex_color_outline);
        if let Some(alpha) = stroke_alpha {
            svg.push_str(&format!(
                "  <circle cx=\"{x}\" cy=\"{y}\" r=\"{}\" fill=\"none\" stroke=\"{stroke_hex}\" stroke-opacity=\"{alpha}\" stroke-width=\"{}\" />\n",
                app.config.vertex_radius,
                app.config.vertex_stroke
            ));
        } else {
            svg.push_str(&format!(
                "  <circle cx=\"{x}\" cy=\"{y}\" r=\"{}\" fill=\"none\" stroke=\"{stroke_hex}\" stroke-width=\"{}\" />\n",
                app.config.vertex_radius,
                app.config.vertex_stroke
            ));
        }

        let vertex_show_id = if app.zero_indexed {
            vertex.id
        } else {
            vertex.id + 1
        };
        let (text_hex, text_alpha) = color_to_svg(app.config.vertex_font_color);
        if let Some(alpha) = text_alpha {
            svg.push_str(&format!(
                "  <text x=\"{x}\" y=\"{y}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-size=\"{}\" fill=\"{text_hex}\" fill-opacity=\"{alpha}\">{}</text>\n",
                app.config.vertex_font_size,
                vertex_show_id
            ));
        } else {
            svg.push_str(&format!(
                "  <text x=\"{x}\" y=\"{y}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-size=\"{}\" fill=\"{text_hex}\">{}</text>\n",
                app.config.vertex_font_size,
                vertex_show_id
            ));
        }
    }

    svg.push_str("</svg>\n");
    Ok(svg.into_bytes())
}

fn svg_point(pos: egui::Pos2, bounds: egui::Rect) -> String {
    format!(
        "{} {}",
        pos.x - bounds.min.x,
        pos.y - bounds.min.y
    )
}

fn color_to_svg(color: egui::Color32) -> (String, Option<f32>) {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);
    let alpha = if a < 255 { Some(a as f32 / 255.0) } else { None };
    (hex, alpha)
}
