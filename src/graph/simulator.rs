use crate::graph::Graph;

/// シミュレーションを行う
pub trait Simulator {
    /// 1ステップ分シミュレートする
    fn simulate_step(&self, graph: &mut Graph);
}

pub mod simulation_methods {
    use crate::{
        config::SimulateConfig,
        graph::{Graph, Simulator},
    };

    const DISTANCE_EPS: f32 = 1e-5;

    /// 力学モデル
    pub struct ForceDirectedModel {
        pub config: SimulateConfig,
    }

    impl Simulator for ForceDirectedModel {
        fn simulate_step(&self, graph: &mut Graph) {
            let &SimulateConfig {
                c,
                k,
                l,
                h,
                m,
                max_v,
                dt,
            } = &self.config;

            // ドラッグ差分を解消
            graph
                .vertices_mut()
                .iter_mut()
                .for_each(|v| v.solve_drag_offset());

            let n = graph.vertices.len();

            for i in 0..n {
                let v = graph.vertices[i].clone();

                // vからxへ向かう単位ベクトル
                let r = |x: egui::Pos2| -> egui::Vec2 { (x - v.position).normalized() };

                // 頂点vに働く力
                let fv = graph
                    .vertices
                    .iter()
                    .filter(|w| w.position.distance(v.position) > DISTANCE_EPS)
                    // 頂点間の斥力
                    .map(|w| -r(w.position) * c / v.position.distance_sq(w.position))
                    // 辺による引力
                    .chain(
                        graph
                            .neighbor_vertices(v.id)
                            .map(|w| r(w.position) * (v.position.distance(w.position) - l) * k),
                    )
                    .fold(egui::Vec2::ZERO, |acc, f| acc + f);

                // 速度を更新
                let mut next_velocity = (v.velocity + fv * dt / m) * h;

                if next_velocity.length() > max_v {
                    next_velocity = next_velocity.normalized() * max_v;
                }

                // 位置を更新（半陰的オイラー）
                let next_position = v.position + next_velocity * dt;

                graph.vertices[i].velocity = next_velocity;
                graph.vertices[i].position = next_position;
            }
        }
    }
}
