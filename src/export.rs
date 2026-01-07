use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Context;
use eframe::egui;

use crate::app::GraphEditorApp;
use crate::math::bezier::{calc_bezier_control_point, calc_intersection_of_bezier_and_circle};

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

pub fn build_export_request(format: ExportFormat) -> Option<ExportRequest> {
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

pub fn save_export_bytes(export_request: &ExportRequest, bytes: Vec<u8>) -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        let path = export_request
            .path
            .as_ref()
            .context("missing export path")?;
        std::fs::write(path, bytes).with_context(|| format!("failed to save image: {:?}", path))?;
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

pub fn export_svg_bytes(app: &GraphEditorApp) -> anyhow::Result<Vec<u8>> {
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

    let mut vertices: Vec<_> = app
        .graph
        .vertices
        .iter()
        .filter(|v| !v.is_deleted)
        .collect();
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
                    arrowhead.x
                        - arrow_dir.x
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
                    arrowhead.y
                        - arrow_dir.y
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
                        arrowhead.x
                            - arrow_dir.x
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
                        arrowhead.y
                            - arrow_dir.y
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
        let text_adjust_y = y + 4.5;
        if let Some(alpha) = text_alpha {
            svg.push_str(&format!(
                "  <text x=\"{x}\" y=\"{text_adjust_y}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-size=\"{}\" fill=\"{text_hex}\" fill-opacity=\"{alpha}\">{}</text>\n",
                app.config.vertex_font_size,
                vertex_show_id
            ));
        } else {
            svg.push_str(&format!(
                "  <text x=\"{x}\" y=\"{text_adjust_y}\" text-anchor=\"middle\" dominant-baseline=\"middle\" font-size=\"{}\" fill=\"{text_hex}\">{}</text>\n",
                app.config.vertex_font_size,
                vertex_show_id
            ));
        }
    }

    svg.push_str("</svg>\n");
    Ok(svg.into_bytes())
}

pub fn graph_bounds_rect(app: &GraphEditorApp) -> Option<egui::Rect> {
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

fn svg_point(pos: egui::Pos2, bounds: egui::Rect) -> String {
    format!("{} {}", pos.x - bounds.min.x, pos.y - bounds.min.y)
}

fn color_to_svg(color: egui::Color32) -> (String, Option<f32>) {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    let hex = format!("#{:02X}{:02X}{:02X}", r, g, b);
    let alpha = if a < 255 {
        Some(a as f32 / 255.0)
    } else {
        None
    };
    (hex, alpha)
}
