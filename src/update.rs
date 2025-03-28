use crate::GraphEditorApp;

pub fn request_repaint(app: &mut GraphEditorApp, ctx: &egui::Context) {
    if app.is_animated {
        ctx.request_repaint();
    }
}
