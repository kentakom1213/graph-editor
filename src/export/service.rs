use eframe::egui;

use crate::export::{
    build_export_request, export_color_image, export_svg_bytes, graph_bounds_rect,
    save_export_bytes, ExportContext, ExportFormat, ExportRequest,
};

pub struct ExportService {
    format: ExportFormat,
    in_progress: bool,
    request: Option<ExportRequest>,
}

impl Default for ExportService {
    fn default() -> Self {
        Self {
            format: ExportFormat::Png,
            in_progress: false,
            request: None,
        }
    }
}

impl ExportService {
    pub fn format(&self) -> ExportFormat {
        self.format
    }

    pub fn set_format(&mut self, format: ExportFormat) {
        self.format = format;
    }

    pub fn is_busy(&self) -> bool {
        self.in_progress
    }

    pub fn request_export(
        &mut self,
        ctx: &egui::Context,
        export_ctx: &ExportContext<'_>,
    ) -> Option<String> {
        if self.in_progress {
            return None;
        }

        let export_request = build_export_request(self.format)?;

        if export_request.format == ExportFormat::Svg {
            self.in_progress = true;
            let result = export_svg_bytes(export_ctx)
                .and_then(|bytes| save_export_bytes(&export_request, bytes));
            self.in_progress = false;
            if let Err(err) = result {
                return Some(err.to_string());
            }
            return None;
        }

        self.in_progress = true;
        self.request = Some(export_request);
        ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::new(
            "graph-export",
        )));
        ctx.request_repaint();
        None
    }

    pub fn handle_events(
        &mut self,
        ctx: &egui::Context,
        export_ctx: &ExportContext<'_>,
    ) -> Option<String> {
        self.request.as_ref()?;

        let screenshot = ctx.input(|i| {
            i.raw.events.iter().find_map(|event| {
                if let egui::Event::Screenshot { image, .. } = event {
                    Some(image.clone())
                } else {
                    None
                }
            })
        });

        let screenshot = screenshot?;

        let export_request = self.request.take();
        self.in_progress = false;

        let export_request = export_request?;

        let pixels_per_point = ctx.pixels_per_point();
        let Some(mut region) = graph_bounds_rect(export_ctx) else {
            return Some("Export failed: no vertices to capture.".to_string());
        };
        region = region.intersect(ctx.screen_rect());

        if region.width() <= 0.0 || region.height() <= 0.0 {
            return Some("Export failed: invalid capture region.".to_string());
        }

        let mut color_image = screenshot.region(&region, Some(pixels_per_point));

        if color_image.width() == 0 || color_image.height() == 0 {
            return Some("Export failed: empty capture region.".to_string());
        }

        let result = export_color_image(&mut color_image)
            .and_then(|bytes| save_export_bytes(&export_request, bytes));

        if let Err(err) = result {
            return Some(err.to_string());
        }

        None
    }
}
