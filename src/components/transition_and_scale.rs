use crate::{
    math::affine::{Affine2D, ApplyAffine},
    GraphEditorApp,
};

pub fn drag_central_panel(app: &mut GraphEditorApp, ui: &mut egui::Ui) {
    let response = ui.allocate_response(ui.available_size(), egui::Sense::drag());

    // マウス入力の処理
    if response.dragged_by(egui::PointerButton::Primary) {
        if let Some(mouse_pos) = ui.input(|i| i.pointer.hover_pos()) {
            if let Some(last_pos) = app.state.last_mouse_pos {
                let cur_affine = app.state.graph.affine.borrow().to_owned();
                if let Some(inv) = cur_affine.inverse() {
                    let cur_local = mouse_pos.applied(&inv);
                    let last_local = last_pos.applied(&inv);
                    let delta = cur_local - last_local;
                    *app.state.graph.affine.borrow_mut() *= Affine2D::from_transition(delta);
                }
            }
            app.state.last_mouse_pos = Some(mouse_pos);
        }
    } else {
        app.state.last_mouse_pos = None;
    }
}

/// グラフのスケールを行う
pub fn scale_central_panel(app: &mut GraphEditorApp, ui: &mut egui::Ui) {
    if app.ui.cursor_hover.any() {
        return;
    }

    let input = ui.input(|i| i.clone());

    // スクロールに対応
    if let Some(pos) = input.pointer.hover_pos() {
        let scroll_delta = input.smooth_scroll_delta.y;

        // 現在のscaleの逆数倍で変化させる
        let cur_affine = app.state.graph.affine.borrow().to_owned();
        let cur_scale = cur_affine.scale();

        if let Some(inv) = cur_affine.inverse() {
            // 中心の調整
            let center = pos.applied(&inv);

            let scale = 1.0 + app.config.scale_delta * scroll_delta / cur_scale;
            let affine = Affine2D::from_center_and_scale(center, scale);

            if let Some(res) =
                cur_affine.try_compose(&affine, app.config.scale_min, app.config.scale_max)
            {
                *app.state.graph.affine.borrow_mut() = res;
            }
        }
    }

    // キー入力による回転（{, }）
    let rotate_dir = if input.key_down(egui::Key::OpenBracket) {
        -1.0
    } else if input.key_down(egui::Key::CloseBracket) {
        1.0
    } else {
        0.0
    };

    if rotate_dir != 0.0 {
        let cur_affine = app.state.graph.affine.borrow().to_owned();
        if let Some(inv) = cur_affine.inverse() {
            let center = input
                .pointer
                .hover_pos()
                .unwrap_or_else(|| ui.max_rect().center())
                .applied(&inv);
            let rad = app.config.rotate_delta * rotate_dir;
            let affine = Affine2D::from_center_and_rotation(center, rad);
            if let Some(res) = cur_affine.try_compose(&affine, f32::MIN, f32::MAX) {
                *app.state.graph.affine.borrow_mut() = res;
            }
        }
    }

    // 2本指ジェスチャーに対応
    if let Some(multitouch) = input.multi_touch() {
        let scale = multitouch.zoom_delta;

        let cur_affine = app.state.graph.affine.borrow().to_owned();

        if let Some(inv) = cur_affine.inverse() {
            // 中心の調整
            let center = multitouch.center_pos.applied(&inv);

            // アフィン変換の生成
            let affine = Affine2D::from_center_and_scale(center, scale);

            if let Some(res) =
                cur_affine.try_compose(&affine, app.config.scale_min, app.config.scale_max)
            {
                *app.state.graph.affine.borrow_mut() = res;
            }
        }
    }
}
