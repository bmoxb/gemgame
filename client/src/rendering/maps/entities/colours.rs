use macroquad::color::Color;
use shared::maps::entities::{ClothingColour, HairColour, SkinColour};

pub fn clothing(col: ClothingColour) -> Color {
    match col {
        ClothingColour::Grey => Color::from_rgba(220, 220, 220, 255),
        ClothingColour::White => Color::from_rgba(245, 245, 245, 255),
        ClothingColour::Red => Color::from_rgba(255, 80, 80, 255),
        ClothingColour::Green => Color::from_rgba(120, 255, 120, 255),
        ClothingColour::Blue => Color::from_rgba(160, 160, 255, 255)
    }
}

pub fn skin(col: SkinColour) -> Color {
    match col {
        SkinColour::Black => Color::from_rgba(62, 39, 35, 255),
        SkinColour::Brown => Color::from_rgba(165, 125, 108, 255),
        SkinColour::Pale => Color::from_rgba(215, 178, 160, 255),
        SkinColour::White => Color::from_rgba(238, 210, 200, 255)
    }
}

pub fn hair(col: HairColour) -> Color {
    match col {
        HairColour::Black => Color::from_rgba(35, 18, 18, 255),
        HairColour::Brown => Color::from_rgba(90, 56, 37, 255),
        HairColour::Blonde => Color::from_rgba(210, 155, 105, 255),
        HairColour::White => Color::from_rgba(220, 220, 200, 255),
        HairColour::Red => Color::from_rgba(240, 10, 10, 255),
        HairColour::Green => Color::from_rgba(10, 240, 10, 255),
        HairColour::Blue => Color::from_rgba(10, 10, 240, 255)
    }
}
