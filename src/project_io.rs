use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    fmt,
    rc::Rc,
};

use num_traits::One;

use crate::{
    components::Colors,
    graph::{visualize_methods, Edge, Graph, Vertex, Visualizer},
    math::affine::Affine2D,
    view_state::GraphViewState,
};

const GRAPH_FILE_FORMAT: &str = "graph-editor";
const GRAPH_FILE_VERSION: u32 = 1;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphFile {
    pub format: String,
    pub version: u32,
    pub graph: GraphData,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GraphData {
    pub directed: bool,
    pub index_origin: u8,
    #[serde(default)]
    pub features: GraphFeatures,
    pub vertices: Vec<VertexData>,
    pub edges: Vec<EdgeData>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct GraphFeatures {
    pub vertex_position: bool,
    pub vertex_style: bool,
    pub edge_style: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VertexData {
    pub id: usize,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<PositionData>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<VertexStyleData>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EdgeData {
    pub id: usize,
    pub from: usize,
    pub to: usize,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<EdgeStyleData>,
}

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct PositionData {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VertexStyleData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<f32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f32>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EdgeStyleData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub stroke_width: Option<f32>,
}

#[derive(Debug, Clone, Copy)]
pub struct SaveOptions {
    pub include_vertex_position: bool,
    pub include_vertex_style: bool,
    pub include_edge_style: bool,
}

impl Default for SaveOptions {
    fn default() -> Self {
        Self {
            include_vertex_position: true,
            include_vertex_style: true,
            include_edge_style: true,
        }
    }
}

#[derive(Debug)]
pub enum ImportError {
    InvalidJson(String),
    InvalidFormat(String),
    UnsupportedVersion(u32),
    InvalidIndexOrigin(u8),
    DuplicateVertexId(usize),
    DuplicateEdgeId(usize),
    MissingVertex { edge_id: usize, vertex_id: usize },
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(err) => write!(f, "Invalid JSON: {err}"),
            Self::InvalidFormat(format) => write!(f, "Invalid graph format: {format}"),
            Self::UnsupportedVersion(version) => {
                write!(f, "Unsupported graph file version: {version}")
            }
            Self::InvalidIndexOrigin(origin) => write!(f, "Invalid index origin: {origin}"),
            Self::DuplicateVertexId(id) => write!(f, "Duplicate vertex id: {id}"),
            Self::DuplicateEdgeId(id) => write!(f, "Duplicate edge id: {id}"),
            Self::MissingVertex { edge_id, vertex_id } => {
                write!(f, "Edge {edge_id} references missing vertex {vertex_id}")
            }
        }
    }
}

impl std::error::Error for ImportError {}

#[derive(Debug)]
pub enum ExportError {
    SerializeFailed(String),
}

impl fmt::Display for ExportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SerializeFailed(err) => write!(f, "Failed to serialize graph JSON: {err}"),
        }
    }
}

impl std::error::Error for ExportError {}

#[derive(Debug)]
pub struct ImportedGraph {
    pub graph: Graph,
    pub view: GraphViewState,
    pub zero_indexed: bool,
    pub used_generated_positions: bool,
}

pub fn export_graph_to_file(
    graph: &Graph,
    view: &GraphViewState,
    zero_indexed: bool,
    options: SaveOptions,
) -> GraphFile {
    let active_vertices: Vec<_> = graph
        .vertices
        .iter()
        .filter(|vertex| !vertex.is_deleted)
        .collect();
    let mut vertex_id_map = HashMap::new();
    for (index, vertex) in active_vertices.iter().enumerate() {
        vertex_id_map.insert(vertex.id, index);
    }

    let vertices = active_vertices
        .into_iter()
        .enumerate()
        .map(|(index, vertex)| VertexData {
            id: index,
            label: Some(
                view.vertices
                    .get(vertex.id)
                    .and_then(|state| state.label.clone())
                    .unwrap_or_else(|| display_vertex_id(index, zero_indexed).to_string()),
            ),
            position: options.include_vertex_position.then(|| PositionData {
                x: vertex.get_position().x,
                y: vertex.get_position().y,
            }),
            style: options.include_vertex_style.then(|| {
                let defaults = crate::config::AppConfig::default();
                let vertex_state = view.vertices.get(vertex.id);
                let color = vertex_state.map(|state| state.color).unwrap_or_default();
                VertexStyleData {
                    fill: Some(color_to_hex(color.vertex())),
                    stroke: Some(color_to_hex(color.vertex())),
                    text: Some(color_to_hex(
                        vertex_state
                            .and_then(|state| state.text_color)
                            .unwrap_or(egui::Color32::BLACK),
                    )),
                    radius: vertex_state
                        .and_then(|state| state.radius)
                        .filter(|radius| (*radius - defaults.vertex_radius).abs() > f32::EPSILON),
                    stroke_width: vertex_state
                        .and_then(|state| state.stroke_width)
                        .filter(|width| (*width - defaults.vertex_stroke).abs() > f32::EPSILON),
                }
            }),
        })
        .collect();

    let edges = graph
        .edges
        .iter()
        .enumerate()
        .filter(|(_, edge)| !edge.is_deleted)
        .filter_map(|(edge_index, edge)| {
            let from = *vertex_id_map.get(&edge.from)?;
            let to = *vertex_id_map.get(&edge.to)?;
            Some(EdgeData {
                id: edge_index,
                from,
                to,
                label: Some(String::new()),
                style: options.include_edge_style.then(|| {
                    let defaults = crate::config::AppConfig::default();
                    let color = view
                        .edges
                        .get(edge_index)
                        .map(|state| state.color)
                        .unwrap_or_default();
                    EdgeStyleData {
                        stroke: Some(color_to_hex(color.edge())),
                        text: Some(color_to_hex(color.edge())),
                        stroke_width: view
                            .edges
                            .get(edge_index)
                            .and_then(|state| state.stroke_width)
                            .filter(|width| (*width - defaults.edge_stroke).abs() > f32::EPSILON),
                    }
                }),
            })
        })
        .collect();

    GraphFile {
        format: GRAPH_FILE_FORMAT.to_string(),
        version: GRAPH_FILE_VERSION,
        graph: GraphData {
            directed: graph.is_directed,
            index_origin: if zero_indexed { 0 } else { 1 },
            features: GraphFeatures {
                vertex_position: options.include_vertex_position,
                vertex_style: options.include_vertex_style,
                edge_style: options.include_edge_style,
            },
            vertices,
            edges,
        },
    }
}

pub fn export_graph_to_json(
    graph: &Graph,
    view: &GraphViewState,
    zero_indexed: bool,
    options: SaveOptions,
) -> Result<String, ExportError> {
    let file = export_graph_to_file(graph, view, zero_indexed, options);
    serde_json::to_string_pretty(&file).map_err(|err| ExportError::SerializeFailed(err.to_string()))
}

pub fn import_graph_from_file(file: GraphFile) -> Result<ImportedGraph, ImportError> {
    if file.format != GRAPH_FILE_FORMAT {
        return Err(ImportError::InvalidFormat(file.format));
    }
    if file.version != GRAPH_FILE_VERSION {
        return Err(ImportError::UnsupportedVersion(file.version));
    }
    if file.graph.index_origin != 0 && file.graph.index_origin != 1 {
        return Err(ImportError::InvalidIndexOrigin(file.graph.index_origin));
    }

    let mut vertices = file.graph.vertices;
    let mut seen_vertex_ids = HashSet::new();
    for vertex in &vertices {
        if !seen_vertex_ids.insert(vertex.id) {
            return Err(ImportError::DuplicateVertexId(vertex.id));
        }
    }
    vertices.sort_by_key(|vertex| vertex.id);

    let mut vertex_id_map = HashMap::new();
    for (index, vertex) in vertices.iter().enumerate() {
        vertex_id_map.insert(vertex.id, index);
    }

    let mut edges = file.graph.edges;
    let mut seen_edge_ids = HashSet::new();
    for edge in &edges {
        if !seen_edge_ids.insert(edge.id) {
            return Err(ImportError::DuplicateEdgeId(edge.id));
        }
        if !vertex_id_map.contains_key(&edge.from) {
            return Err(ImportError::MissingVertex {
                edge_id: edge.id,
                vertex_id: edge.from,
            });
        }
        if !vertex_id_map.contains_key(&edge.to) {
            return Err(ImportError::MissingVertex {
                edge_id: edge.id,
                vertex_id: edge.to,
            });
        }
    }
    edges.sort_by_key(|edge| edge.id);

    let edge_pairs = edges
        .iter()
        .map(|edge| {
            (
                *vertex_id_map.get(&edge.from).expect("validated above"),
                *vertex_id_map.get(&edge.to).expect("validated above"),
            )
        })
        .collect::<Vec<_>>();

    let default_positions = default_vertex_positions(vertices.len(), &edge_pairs);
    let affine = Rc::new(RefCell::new(Affine2D::one()));
    let mut used_generated_positions = false;

    let graph_vertices = vertices
        .iter()
        .enumerate()
        .map(|(index, vertex)| {
            let position = match vertex.position {
                Some(position) => egui::pos2(position.x, position.y),
                None => {
                    used_generated_positions = true;
                    default_positions[index]
                }
            };
            Vertex {
                id: index,
                position,
                velocity: egui::Vec2::ZERO,
                is_deleted: false,
                affine: affine.clone(),
            }
        })
        .collect::<Vec<_>>();

    let graph_edges = edge_pairs
        .iter()
        .map(|&(from, to)| Edge::new(from, to))
        .collect::<Vec<_>>();

    let graph = Graph {
        is_directed: file.graph.directed,
        affine,
        vertices: graph_vertices,
        edges: graph_edges,
    };

    let mut view = GraphViewState::new_for_graph(&graph);
    for (index, vertex) in vertices.iter().enumerate() {
        view.vertices[index].label = vertex.label.clone();
        if let Some(style) = &vertex.style {
            view.vertices[index].color = color_from_vertex_style(style);
            view.vertices[index].text_color = style.text.as_deref().and_then(parse_hex_color);
            view.vertices[index].radius = style.radius;
            view.vertices[index].stroke_width = style.stroke_width;
        }
    }
    for (index, edge) in edges.iter().enumerate() {
        if let Some(style) = &edge.style {
            view.edges[index].color = color_from_edge_style(style);
            view.edges[index].stroke_width = style.stroke_width;
        }
    }

    Ok(ImportedGraph {
        graph,
        view,
        zero_indexed: file.graph.index_origin == 0,
        used_generated_positions,
    })
}

pub fn import_graph_from_json(json: &str) -> Result<ImportedGraph, ImportError> {
    let file = serde_json::from_str::<GraphFile>(json)
        .map_err(|err| ImportError::InvalidJson(err.to_string()))?;
    import_graph_from_file(file)
}

fn display_vertex_id(id: usize, zero_indexed: bool) -> usize {
    if zero_indexed {
        id
    } else {
        id + 1
    }
}

fn default_vertex_positions(n: usize, edges: &[(usize, usize)]) -> Vec<egui::Pos2> {
    let size = egui::vec2(720.0, 720.0);
    let margin = size * 0.1;
    let base = egui::pos2(0.0, 0.0);
    let positions = visualize_methods::Spectral.resolve_vertex_position(n, edges);
    positions
        .into_iter()
        .map(|pos| base + margin + pos * size * 0.8)
        .collect()
}

fn color_to_hex(color: egui::Color32) -> String {
    format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b())
}

fn parse_hex_color(hex: &str) -> Option<egui::Color32> {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some(egui::Color32::from_rgb(r, g, b))
}

fn color_from_vertex_style(style: &VertexStyleData) -> Colors {
    let color = style
        .fill
        .as_deref()
        .or(style.stroke.as_deref())
        .or(style.text.as_deref())
        .and_then(parse_hex_color);
    match_color(color, true)
}

fn color_from_edge_style(style: &EdgeStyleData) -> Colors {
    let color = style
        .stroke
        .as_deref()
        .or(style.text.as_deref())
        .and_then(parse_hex_color);
    match_color(color, false)
}

fn match_color(color: Option<egui::Color32>, vertex: bool) -> Colors {
    let Some(color) = color else {
        return Colors::Default;
    };

    for candidate in [
        Colors::Default,
        Colors::Red,
        Colors::Green,
        Colors::Blue,
        Colors::Yellow,
        Colors::Orange,
        Colors::Violet,
        Colors::Pink,
        Colors::Brown,
        Colors::Cyan,
        Colors::Indigo,
        Colors::Gray,
    ] {
        let expected = if vertex {
            candidate.vertex()
        } else {
            candidate.edge()
        };
        if expected == color {
            return candidate;
        }
    }

    Colors::Default
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use num_traits::One;

    use crate::{
        components::Colors,
        graph::{Edge, Graph, Vertex},
        math::affine::Affine2D,
        view_state::GraphViewState,
    };

    use super::{
        export_graph_to_json, import_graph_from_json, GraphData, GraphFeatures, GraphFile,
        ImportError, SaveOptions, VertexData,
    };

    fn sample_graph() -> (Graph, GraphViewState) {
        let affine = Rc::new(RefCell::new(Affine2D::one()));
        let graph = Graph {
            is_directed: true,
            affine: affine.clone(),
            vertices: vec![
                Vertex {
                    id: 0,
                    position: egui::pos2(120.0, 80.0),
                    velocity: egui::Vec2::ZERO,
                    is_deleted: false,
                    affine: affine.clone(),
                },
                Vertex {
                    id: 1,
                    position: egui::pos2(260.0, 140.0),
                    velocity: egui::Vec2::ZERO,
                    is_deleted: false,
                    affine: affine.clone(),
                },
            ],
            edges: vec![Edge::new(0, 1)],
        };
        let mut view = GraphViewState::new_for_graph(&graph);
        view.vertices[0].color = Colors::Red;
        view.edges[0].color = Colors::Blue;
        (graph, view)
    }

    #[test]
    fn exports_structure_only_without_position_and_style_fields() {
        let (graph, view) = sample_graph();
        let json = export_graph_to_json(
            &graph,
            &view,
            true,
            SaveOptions {
                include_vertex_position: false,
                include_vertex_style: false,
                include_edge_style: false,
            },
        )
        .unwrap();

        assert!(!json.contains("\"position\""));
        assert!(!json.contains("\"style\""));
    }

    #[test]
    fn exports_position_and_style_fields_when_enabled() {
        let (graph, view) = sample_graph();
        let json = export_graph_to_json(&graph, &view, true, SaveOptions::default()).unwrap();

        assert!(json.contains("\"position\""));
        assert!(json.contains("\"style\""));
        assert!(!json.contains("\"radius\""));
        assert!(!json.contains("\"stroke_width\""));
    }

    #[test]
    fn imports_preserve_edge_connections() {
        let (graph, view) = sample_graph();
        let json = export_graph_to_json(&graph, &view, true, SaveOptions::default()).unwrap();
        let imported = import_graph_from_json(&json).unwrap();

        assert_eq!(imported.graph.edges[0].from, 0);
        assert_eq!(imported.graph.edges[0].to, 1);
        assert!(imported.zero_indexed);
    }

    #[test]
    fn rejects_edges_that_reference_missing_vertices() {
        let file = GraphFile {
            format: "graph-editor".to_string(),
            version: 1,
            graph: GraphData {
                directed: false,
                index_origin: 0,
                features: GraphFeatures::default(),
                vertices: vec![VertexData {
                    id: 0,
                    label: None,
                    position: None,
                    style: None,
                }],
                edges: vec![super::EdgeData {
                    id: 0,
                    from: 0,
                    to: 1,
                    label: None,
                    style: None,
                }],
            },
        };

        let json = serde_json::to_string(&file).unwrap();
        let err = import_graph_from_json(&json).unwrap_err();
        assert!(matches!(
            err,
            ImportError::MissingVertex {
                edge_id: 0,
                vertex_id: 1
            }
        ));
    }

    #[test]
    fn rejects_duplicate_vertex_ids() {
        let file = GraphFile {
            format: "graph-editor".to_string(),
            version: 1,
            graph: GraphData {
                directed: false,
                index_origin: 0,
                features: GraphFeatures::default(),
                vertices: vec![
                    VertexData {
                        id: 0,
                        label: None,
                        position: None,
                        style: None,
                    },
                    VertexData {
                        id: 0,
                        label: None,
                        position: None,
                        style: None,
                    },
                ],
                edges: vec![],
            },
        };

        let json = serde_json::to_string(&file).unwrap();
        let err = import_graph_from_json(&json).unwrap_err();
        assert!(matches!(err, ImportError::DuplicateVertexId(0)));
    }

    #[test]
    fn rejects_unsupported_version() {
        let file = GraphFile {
            format: "graph-editor".to_string(),
            version: 999,
            graph: GraphData {
                directed: false,
                index_origin: 0,
                features: GraphFeatures::default(),
                vertices: vec![],
                edges: vec![],
            },
        };

        let json = serde_json::to_string(&file).unwrap();
        let err = import_graph_from_json(&json).unwrap_err();
        assert!(matches!(err, ImportError::UnsupportedVersion(999)));
    }

    #[test]
    fn rejects_invalid_index_origin() {
        let file = GraphFile {
            format: "graph-editor".to_string(),
            version: 1,
            graph: GraphData {
                directed: false,
                index_origin: 2,
                features: GraphFeatures::default(),
                vertices: vec![],
                edges: vec![],
            },
        };

        let json = serde_json::to_string(&file).unwrap();
        let err = import_graph_from_json(&json).unwrap_err();
        assert!(matches!(err, ImportError::InvalidIndexOrigin(2)));
    }
}
