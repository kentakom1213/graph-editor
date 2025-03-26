use crate::GraphEditorApp;

pub fn update_paint(app: &mut GraphEditorApp, ctx: &egui::Context) {
    if app.is_animated {
        let now = std::time::Instant::now();
        let elapsed = now - app.last_update;

        if elapsed >= app.config.repaint_duration {
            app.last_update = now;
            ctx.request_repaint();
        }
    }
}
