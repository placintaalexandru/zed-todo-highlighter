use typed_builder::TypedBuilder;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum ColorType {
    Background,
}

#[derive(Debug, Clone, Copy)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl Color {
    pub fn into_components(self) -> (u8, u8, u8, u8) {
        let Self { r, g, b, a } = self;
        (r, g, b, a)
    }
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Debug, TypedBuilder)]
pub struct Colors {
    pub background: Color,
}

impl Colors {
    pub fn new(background: Color) -> Self {
        Self { background }
    }
}
