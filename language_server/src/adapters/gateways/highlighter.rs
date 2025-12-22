use std::collections::HashMap;

use crate::{
    entities::{Color, ColorType, Colors},
    use_cases::ports::Colorer,
};

#[derive(Debug, Default)]
pub struct Highlighter {
    color_config: HashMap<String, Colors>,
}

impl Highlighter {
    pub fn new(color_config: HashMap<String, Colors>) -> Self {
        Self { color_config }
    }
}

impl Colorer for Highlighter {
    fn color_text(&self, text: &str, color_type: ColorType) -> Option<Color> {
        self.color_config.get(text).map(|config| match color_type {
            ColorType::Background => config.background,
        })
    }

    fn update_palette(&mut self, text: String, colors: Colors) {
        self.color_config.insert(text, colors);
    }
}
