use std::collections::HashMap;

use crate::{
    entities::{Color, ColorType, Colors},
    use_cases::ports::Colorer,
};

#[derive(Debug)]
pub struct ColorProvider {
    background: HashMap<String, Colors>,
}

impl ColorProvider {
    pub fn new(color_config: HashMap<String, Colors>) -> Self {
        Self {
            background: color_config,
        }
    }
}

impl Colorer for ColorProvider {
    fn background_colors(&self) -> &HashMap<String, Colors> {
        &self.background
    }

    fn color_text(&self, text: &str, color_type: ColorType) -> Option<Color> {
        self.background.get(text).map(|config| match color_type {
            ColorType::Background => config.background,
        })
    }

    fn update_palette(&mut self, text: String, colors: Colors) {
        self.background.insert(text, colors);
    }
}
