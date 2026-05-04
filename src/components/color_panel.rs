#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum Colors {
    #[default]
    Default,
    Red,
    Green,
    Blue,
    Yellow,
    Orange,
    Violet,
    Pink,
    Brown,
    Cyan,
    Indigo,
    Gray,
}

pub const COLOR_SLOTS: [Colors; 12] = [
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
];

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum PaletteTheme {
    #[default]
    Vivid,
    Viridis,
    Plasma,
}

impl PaletteTheme {
    pub fn all() -> [PaletteTheme; 3] {
        [Self::Vivid, Self::Viridis, Self::Plasma]
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Vivid => "Vivid",
            Self::Viridis => "Viridis",
            Self::Plasma => "Plasma",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Vivid => "Higher-chroma palette for stronger categorical separation",
            Self::Viridis => "Perceptually uniform blue-green-yellow scientific palette",
            Self::Plasma => "Perceptually uniform purple-magenta-yellow scientific palette",
        }
    }

    pub fn storage_key(self) -> &'static str {
        match self {
            Self::Vivid => "vivid",
            Self::Viridis => "viridis",
            Self::Plasma => "plasma",
        }
    }

    pub fn from_storage_key(value: &str) -> Self {
        match value {
            "viridis" => Self::Viridis,
            "plasma" => Self::Plasma,
            _ => Self::Vivid,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Default)]
pub enum VertexPattern {
    #[default]
    None,
    Diagonal,
    Dots,
    Cross,
}

pub const VERTEX_PATTERNS: [VertexPattern; 4] = [
    VertexPattern::None,
    VertexPattern::Diagonal,
    VertexPattern::Dots,
    VertexPattern::Cross,
];

impl VertexPattern {
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Diagonal => "Diagonal",
            Self::Dots => "Dots",
            Self::Cross => "Cross",
        }
    }

    pub fn storage_key(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Diagonal => "diagonal",
            Self::Dots => "dots",
            Self::Cross => "cross",
        }
    }

    pub fn from_storage_key(value: &str) -> Self {
        match value {
            "diagonal" => Self::Diagonal,
            "dots" => Self::Dots,
            "cross" => Self::Cross,
            _ => Self::None,
        }
    }
}

impl Colors {
    fn to_egui_color(self, theme: PaletteTheme) -> Option<egui::Color32> {
        match self {
            Colors::Default => None,
            Colors::Red => Some(match theme {
                PaletteTheme::Vivid => egui::Color32::from_rgb(220, 72, 68),
                PaletteTheme::Viridis => egui::Color32::from_rgb(68, 5, 88),
                PaletteTheme::Plasma => egui::Color32::from_rgb(13, 8, 135),
            }),
            Colors::Green => Some(match theme {
                PaletteTheme::Vivid => egui::Color32::from_rgb(52, 166, 98),
                PaletteTheme::Viridis => egui::Color32::from_rgb(72, 35, 116),
                PaletteTheme::Plasma => egui::Color32::from_rgb(63, 3, 156),
            }),
            Colors::Blue => Some(match theme {
                PaletteTheme::Vivid => egui::Color32::from_rgb(52, 116, 210),
                PaletteTheme::Viridis => egui::Color32::from_rgb(64, 67, 135),
                PaletteTheme::Plasma => egui::Color32::from_rgb(106, 0, 167),
            }),
            Colors::Yellow => Some(match theme {
                PaletteTheme::Vivid => egui::Color32::from_rgb(210, 166, 40),
                PaletteTheme::Viridis => egui::Color32::from_rgb(52, 94, 141),
                PaletteTheme::Plasma => egui::Color32::from_rgb(140, 10, 164),
            }),
            Colors::Orange => Some(match theme {
                PaletteTheme::Vivid => egui::Color32::from_rgb(224, 122, 32),
                PaletteTheme::Viridis => egui::Color32::from_rgb(41, 120, 142),
                PaletteTheme::Plasma => egui::Color32::from_rgb(170, 35, 149),
            }),
            Colors::Violet => Some(match theme {
                PaletteTheme::Vivid => egui::Color32::from_rgb(136, 92, 214),
                PaletteTheme::Viridis => egui::Color32::from_rgb(32, 144, 140),
                PaletteTheme::Plasma => egui::Color32::from_rgb(196, 62, 124),
            }),
            Colors::Pink => Some(match theme {
                PaletteTheme::Vivid => egui::Color32::from_rgb(216, 82, 156),
                PaletteTheme::Viridis => egui::Color32::from_rgb(34, 167, 132),
                PaletteTheme::Plasma => egui::Color32::from_rgb(220, 95, 102),
            }),
            Colors::Brown => Some(match theme {
                PaletteTheme::Vivid => egui::Color32::from_rgb(145, 101, 60),
                PaletteTheme::Viridis => egui::Color32::from_rgb(68, 190, 112),
                PaletteTheme::Plasma => egui::Color32::from_rgb(241, 131, 76),
            }),
            Colors::Cyan => Some(match theme {
                PaletteTheme::Vivid => egui::Color32::from_rgb(32, 166, 174),
                PaletteTheme::Viridis => egui::Color32::from_rgb(121, 209, 81),
                PaletteTheme::Plasma => egui::Color32::from_rgb(252, 166, 54),
            }),
            Colors::Indigo => Some(match theme {
                PaletteTheme::Vivid => egui::Color32::from_rgb(76, 88, 194),
                PaletteTheme::Viridis => egui::Color32::from_rgb(189, 223, 38),
                PaletteTheme::Plasma => egui::Color32::from_rgb(252, 206, 37),
            }),
            Colors::Gray => Some(match theme {
                PaletteTheme::Vivid => egui::Color32::from_rgb(112, 122, 134),
                PaletteTheme::Viridis => egui::Color32::from_rgb(253, 231, 37),
                PaletteTheme::Plasma => egui::Color32::from_rgb(240, 249, 33),
            }),
        }
    }

    pub fn vertex(&self, theme: PaletteTheme) -> egui::Color32 {
        self.to_egui_color(theme).unwrap_or(egui::Color32::WHITE)
    }

    pub fn edge(&self, theme: PaletteTheme) -> egui::Color32 {
        self.to_egui_color(theme)
            .unwrap_or(egui::Color32::from_rgb(100, 100, 100))
    }

    pub fn label(self) -> &'static str {
        match self {
            Colors::Default => "Default",
            Colors::Red => "Red",
            Colors::Green => "Green",
            Colors::Blue => "Blue",
            Colors::Yellow => "Yellow",
            Colors::Orange => "Orange",
            Colors::Violet => "Violet",
            Colors::Pink => "Pink",
            Colors::Brown => "Brown",
            Colors::Cyan => "Cyan",
            Colors::Indigo => "Indigo",
            Colors::Gray => "Gray",
        }
    }
}

pub fn default_vertex_text_color(fill: egui::Color32) -> egui::Color32 {
    let luma = 0.2126 * fill.r() as f32 + 0.7152 * fill.g() as f32 + 0.0722 * fill.b() as f32;
    if luma < 140.0 {
        egui::Color32::WHITE
    } else {
        egui::Color32::BLACK
    }
}

pub fn pattern_color(fill: egui::Color32) -> egui::Color32 {
    let luma = 0.2126 * fill.r() as f32 + 0.7152 * fill.g() as f32 + 0.0722 * fill.b() as f32;
    if luma < 140.0 {
        egui::Color32::from_rgba_unmultiplied(255, 255, 255, 180)
    } else {
        egui::Color32::from_rgba_unmultiplied(20, 24, 28, 140)
    }
}
