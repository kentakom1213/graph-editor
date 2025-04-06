use crate::GraphEditorApp;

#[cfg(not(target_arch = "wasm32"))]
pub fn resolve_url(_app: &mut GraphEditorApp) {}

/// URLを処理する
#[cfg(target_arch = "wasm32")]
pub fn resolve_url(app: &mut GraphEditorApp) {
    let data = app.get_url_data();

    web_sys::console::log_1(&format!("{data:?}").into());
}
