use crate::{
    math::affine::{Affine2D, ApplyAffine},
    GraphEditorApp,
};

/// 右クリックでドラッグを行う
pub fn drag_central_panel(app: &mut GraphEditorApp, ui: &mut egui::Ui) {
    let response = ui.allocate_response(ui.available_size(), egui::Sense::drag());

    // マウス入力の処理
    if response.dragged_by(egui::PointerButton::Secondary) {
        if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
            if let Some(last_pos) = app.last_mouse_pos {
                let cur_scale = app.graph.affine.borrow().scale_x();
                let delta = mouse_pos - last_pos;
                *app.graph.affine.borrow_mut() *= Affine2D::from_transition(delta / cur_scale);
            }
            app.last_mouse_pos = Some(mouse_pos);
        }
    } else {
        app.last_mouse_pos = None;
    }

    // 2本指ジェスチャーに対応
    if let Some(multitouch) = ui.input(|i| i.multi_touch()) {
        let cur_scale = app.graph.affine.borrow().scale_x();
        *app.graph.affine.borrow_mut() *=
            Affine2D::from_transition(multitouch.translation_delta / cur_scale);
    }
}

/// グラフのスケールを行う
pub fn scale_central_panel(app: &mut GraphEditorApp, ui: &mut egui::Ui) {
    let input = ui.input(|i| i.clone());

    // スクロールに対応
    if let Some(pos) = input.pointer.hover_pos() {
        let scroll_delta = input.smooth_scroll_delta.y;

        // 現在のscaleの逆数倍で変化させる
        let cur_affine = app.graph.affine.borrow().to_owned();

        let cur_scale = cur_affine.scale_x();
        let scale = 1.0 + 0.001 * scroll_delta / cur_scale;

        if let Some(inv) = cur_affine.inverse() {
            // 中心の調整
            let center = pos.applied(&inv);

            // アフィン変換の生成
            let affine = Affine2D::from_center_and_scale(center, scale);

            if let Some(res) = cur_affine.try_compose(&affine, 0.1, 3.0) {
                *app.graph.affine.borrow_mut() = res;
            }
        }
    }

    // 2本指ジェスチャーに対応
    if let Some(multitouch) = input.multi_touch() {
        let scale = multitouch.zoom_delta;

        let cur_affine = app.graph.affine.borrow().to_owned();

        if let Some(inv) = cur_affine.inverse() {
            // 中心の調整
            let center = multitouch.center_pos.applied(&inv);

            // アフィン変換の生成
            let affine = Affine2D::from_center_and_scale(center, scale);

            if let Some(res) = cur_affine.try_compose(&affine, 0.1, 3.0) {
                *app.graph.affine.borrow_mut() = res;
            }
        }
    }
}
