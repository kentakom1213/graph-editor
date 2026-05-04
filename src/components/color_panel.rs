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

impl Colors {
    fn to_egui_color(self) -> Option<egui::Color32> {
        match self {
            Colors::Default => None,
            Colors::Red => Some(egui::Color32::from_rgb(255, 70, 70)),
            Colors::Green => Some(egui::Color32::from_rgb(70, 255, 70)),
            Colors::Blue => Some(egui::Color32::from_rgb(70, 70, 255)),
            Colors::Yellow => Some(egui::Color32::from_rgb(255, 255, 0)),
            Colors::Orange => Some(egui::Color32::from_rgb(255, 165, 0)),
            Colors::Violet => Some(egui::Color32::from_rgb(238, 130, 238)),
            Colors::Pink => Some(egui::Color32::from_rgb(255, 192, 203)),
            Colors::Brown => Some(egui::Color32::from_rgb(181, 101, 29)),
            Colors::Cyan => Some(egui::Color32::from_rgb(64, 200, 224)),
            Colors::Indigo => Some(egui::Color32::from_rgb(92, 72, 186)),
            Colors::Gray => Some(egui::Color32::from_rgb(130, 130, 130)),
        }
    }

    pub fn vertex(&self) -> egui::Color32 {
        self.to_egui_color().unwrap_or(egui::Color32::WHITE)
    }

    pub fn edge(&self) -> egui::Color32 {
        self.to_egui_color()
            .unwrap_or(egui::Color32::from_rgb(100, 100, 100))
    }
}
