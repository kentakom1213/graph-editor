use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Context;
use eframe::egui;

use crate::config::AppConfig;
use crate::graph::Graph;
use crate::math::bezier::{calc_bezier_control_point, calc_intersection_of_bezier_and_circle};
use crate::view_state::GraphViewState;

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
    pub _file_name: String,
    pub _path: Option<PathBuf>,
}

pub struct ExportContext<'a> {
    pub graph: &'a Graph,
    pub view: &'a GraphViewState,
    pub config: &'a AppConfig,
    pub show_number: bool,
    pub zero_indexed: bool,
}

pub fn build_export_request(format: ExportFormat) -> Option<ExportRequest> {
    let default_name = format!("graph.{}", format.extension());

    #[cfg(not(target_arch = "wasm32"))]
    {
        let dialog = rfd::FileDialog::new()
            .add_filter("PNG Image", &["png"])
            .add_filter("SVG Image", &["svg"])
            .set_file_name(&default_name);

        let mut path = dialog.save_file()?;

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

        Some(ExportRequest {
            format: resolved_format,
            _file_name: path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(&default_name)
                .to_string(),
            _path: Some(path),
        })
    }

    #[cfg(target_arch = "wasm32")]
    {
        Some(ExportRequest {
            format,
            _file_name: default_name,
            _path: None,
        })
    }
}

pub fn save_export_bytes(export_request: &ExportRequest, bytes: Vec<u8>) -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = export_request
            ._path
            .as_ref()
            .context("missing export path")?;
        std::fs::write(path, bytes).with_context(|| format!("failed to save image: {path:?}"))?;
        Ok(())
    }

    #[cfg(target_arch = "wasm32")]
    {
        let file_name = export_request._file_name.clone();
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

pub fn export_color_image(color_image: &mut egui::ColorImage) -> anyhow::Result<Vec<u8>> {
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

    let mut bytes = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut bytes);
    encoder
        .write_image(&rgba_image, w, h, image::ColorType::Rgba8.into())
        .context("failed to encode PNG")?;

    Ok(bytes)
}

pub fn export_svg_bytes(ctx: &ExportContext<'_>) -> anyhow::Result<Vec<u8>> {
    let active_vertex_count = ctx.graph.vertices.iter().filter(|v| !v.is_deleted).count();
    let default_vertex_radius = ctx.config.effective_vertex_radius(active_vertex_count);
    let vertex_font_size = ctx.config.effective_vertex_font_size(active_vertex_count);
    let bounds = graph_bounds_rect(ctx).context("missing graph bounds")?;
    let width = bounds.width().max(1.0);
    let height = bounds.height().max(1.0);

    let mut svg = String::new();
    svg.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{width}\" height=\"{height}\" viewBox=\"0 0 {width} {height}\">\n"
    ));

    let (bg_hex, bg_alpha) = color_to_svg(ctx.config.bg_color);
    if let Some(alpha) = bg_alpha {
        svg.push_str(&format!(
            "  <rect width=\"100%\" height=\"100%\" fill=\"{bg_hex}\" fill-opacity=\"{alpha}\" />\n"
        ));
    } else {
        svg.push_str(&format!(
            "  <rect width=\"100%\" height=\"100%\" fill=\"{bg_hex}\" />\n"
        ));
    }

    let snapshot = ctx.view.snapshot(ctx.graph);
    let mut vertices = snapshot.vertices.clone();
    let mut edges = snapshot.edges.clone();

    let vertex_map: HashMap<usize, _> = vertices.iter().map(|v| (v.id, v.clone())).collect();

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

        let from_pos = from_vertex.position;
        let to_pos = to_vertex.position;
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

        let stroke_width = edge.stroke_width.unwrap_or(ctx.config.edge_stroke);
        let target_radius = vertex_map
            .get(&edge.to)
            .and_then(|vertex| vertex.radius)
            .unwrap_or(default_vertex_radius);
        if snapshot.is_directed {
            if edge_count.get(&(edge.from, edge.to)) == Some(&1) {
                let dir = (to_pos - from_pos).normalized();
                let arrowhead = to_pos - dir * target_radius;
                let endpoint = arrowhead - dir * ctx.config.edge_arrow_length;
                let arrow_dir = dir * ctx.config.edge_arrow_length;
                let left = egui::Pos2::new(
                    arrowhead.x
                        - arrow_dir.x
                        - arrow_dir.y
                            * (ctx.config.edge_arrow_width / ctx.config.edge_arrow_length),
                    arrowhead.y - arrow_dir.y
                        + arrow_dir.x
                            * (ctx.config.edge_arrow_width / ctx.config.edge_arrow_length),
                );
                let right = egui::Pos2::new(
                    arrowhead.x - arrow_dir.x
                        + arrow_dir.y
                            * (ctx.config.edge_arrow_width / ctx.config.edge_arrow_length),
                    arrowhead.y
                        - arrow_dir.y
                        - arrow_dir.x
                            * (ctx.config.edge_arrow_width / ctx.config.edge_arrow_length),
                );

                let end_x = endpoint.x - bounds.min.x;
                let end_y = endpoint.y - bounds.min.y;
                svg.push_str(&format!(
                    "  <line x1=\"{from_x}\" y1=\"{from_y}\" x2=\"{end_x}\" y2=\"{end_y}\" {stroke_style} stroke-width=\"{}\" fill=\"none\" />\n",
                    stroke_width
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
                    ctx.config.edge_bezier_distance,
                    false,
                );
                if let Some((arrowhead, dir)) = calc_intersection_of_bezier_and_circle(
                    from_pos,
                    control,
                    to_pos,
                    to_pos,
                    target_radius,
                ) {
                    let control_x = control.x - bounds.min.x;
                    let control_y = control.y - bounds.min.y;
                    svg.push_str(&format!(
                        "  <path d=\"M {from_x} {from_y} Q {control_x} {control_y} {to_x} {to_y}\" {stroke_style} stroke-width=\"{}\" fill=\"none\" />\n",
                        stroke_width
                    ));

                    let mask_start =
                        arrowhead - dir.normalized() * ctx.config.edge_arrow_length / 2.0;
                    let (mask_hex, mask_alpha) = color_to_svg(ctx.config.bg_color);
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
                        stroke_width
                    ));

                    let arrow_dir = dir.normalized() * ctx.config.edge_arrow_length;
                    let left = egui::Pos2::new(
                        arrowhead.x
                            - arrow_dir.x
                            - arrow_dir.y
                                * (ctx.config.edge_arrow_width / ctx.config.edge_arrow_length),
                        arrowhead.y - arrow_dir.y
                            + arrow_dir.x
                                * (ctx.config.edge_arrow_width / ctx.config.edge_arrow_length),
                    );
                    let right = egui::Pos2::new(
                        arrowhead.x - arrow_dir.x
                            + arrow_dir.y
                                * (ctx.config.edge_arrow_width / ctx.config.edge_arrow_length),
                        arrowhead.y
                            - arrow_dir.y
                            - arrow_dir.x
                                * (ctx.config.edge_arrow_width / ctx.config.edge_arrow_length),
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
                stroke_width
            ));
        }
    }

    vertices.sort_by_key(|v| v.z_index);
    for vertex in vertices {
        let pos = vertex.position;
        let x = pos.x - bounds.min.x;
        let y = pos.y - bounds.min.y;
        let vertex_radius = vertex.radius.unwrap_or(default_vertex_radius);
        let vertex_stroke = vertex.stroke_width.unwrap_or(ctx.config.vertex_stroke);
        let fill_color = vertex.color.vertex();
        let (fill_hex, fill_alpha) = color_to_svg(fill_color);
        if let Some(alpha) = fill_alpha {
            svg.push_str(&format!(
                "  <circle cx=\"{x}\" cy=\"{y}\" r=\"{vertex_radius}\" fill=\"{fill_hex}\" fill-opacity=\"{alpha}\" />\n",
            ));
        } else {
            svg.push_str(&format!(
                "  <circle cx=\"{x}\" cy=\"{y}\" r=\"{vertex_radius}\" fill=\"{fill_hex}\" />\n",
            ));
        }

        let (stroke_hex, stroke_alpha) = color_to_svg(ctx.config.vertex_color_outline);
        if let Some(alpha) = stroke_alpha {
            svg.push_str(&format!(
                "  <circle cx=\"{x}\" cy=\"{y}\" r=\"{vertex_radius}\" fill=\"none\" stroke=\"{stroke_hex}\" stroke-opacity=\"{alpha}\" stroke-width=\"{}\" />\n",
                vertex_stroke
            ));
        } else {
            svg.push_str(&format!(
                "  <circle cx=\"{x}\" cy=\"{y}\" r=\"{vertex_radius}\" fill=\"none\" stroke=\"{stroke_hex}\" stroke-width=\"{}\" />\n",
                vertex_stroke
            ));
        }

        if ctx.show_number {
            let vertex_show_id = vertex.label.clone().unwrap_or_else(|| {
                if ctx.zero_indexed {
                    vertex.id
                } else {
                    vertex.id + 1
                }
                .to_string()
            });
            let (text_hex, text_alpha) =
                color_to_svg(vertex.text_color.unwrap_or(ctx.config.vertex_font_color));
            let text_adjust_y = y + 4.5;
            if let Some(alpha) = text_alpha {
                svg.push_str(&format!(
                    "  <text x=\"{x}\" y=\"{text_adjust_y}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-size=\"{vertex_font_size}\" fill=\"{text_hex}\" fill-opacity=\"{alpha}\">{vertex_show_id}</text>\n",
                ));
            } else {
                svg.push_str(&format!(
                    "  <text x=\"{x}\" y=\"{text_adjust_y}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-size=\"{vertex_font_size}\" fill=\"{text_hex}\">{vertex_show_id}</text>\n",
                ));
            }
        }
    }

    svg.push_str("</svg>\n");
    Ok(svg.into_bytes())
}

pub fn graph_bounds_rect(ctx: &ExportContext<'_>) -> Option<egui::Rect> {
    let snapshot = ctx.view.snapshot(ctx.graph);
    let active_vertex_count = snapshot.vertices.len();
    let default_vertex_radius = ctx.config.effective_vertex_radius(active_vertex_count);
    let max_vertex_radius = snapshot
        .vertices
        .iter()
        .map(|vertex| vertex.radius.unwrap_or(default_vertex_radius))
        .fold(default_vertex_radius, f32::max);
    let max_edge_stroke = snapshot
        .edges
        .iter()
        .map(|edge| edge.stroke_width.unwrap_or(ctx.config.edge_stroke))
        .fold(ctx.config.edge_stroke, f32::max);
    let mut iter = ctx.graph.vertices.iter().filter(|v| !v.is_deleted);
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

    let padding = max_vertex_radius
        + max_edge_stroke
        + ctx
            .config
            .edge_arrow_length
            .max(ctx.config.edge_arrow_width);

    min.x -= padding;
    min.y -= padding;
    max.x += padding;
    max.y += padding;

    Some(egui::Rect::from_min_max(min, max))
}

fn svg_point(pos: egui::Pos2, bounds: egui::Rect) -> String {
    format!("{} {}", pos.x - bounds.min.x, pos.y - bounds.min.y)
}

fn color_to_svg(color: egui::Color32) -> (String, Option<f32>) {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    let hex = format!("#{r:02X}{g:02X}{b:02X}");
    let alpha = if a < 255 {
        Some(a as f32 / 255.0)
    } else {
        None
    };
    (hex, alpha)
}
