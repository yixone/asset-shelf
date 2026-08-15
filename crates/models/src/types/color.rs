#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
pub struct Color(pub i32);

impl Color {
    pub fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        Color(((r as i32) << 16) | ((g as i32) << 8) | b as i32)
    }

    pub fn rgb(self) -> (u8, u8, u8) {
        (
            ((self.0 >> 16) & 0xFF) as u8,
            ((self.0 >> 8) & 0xFF) as u8,
            ((self.0 & 0xFF) as u8),
        )
    }

    pub fn hex(self) -> String {
        let (r, g, b) = self.rgb();
        format!("#{r:02x}{g:02x}{b:02x}")
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Color::from_rgb(r, g, b)
    }
}

impl From<i32> for Color {
    fn from(value: i32) -> Self {
        Color(value)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::Color;

    #[test]
    fn color_from_rgb() {
        let color = (255, 15, 67);
        let int_color = Color::from_rgb(color.0, color.1, color.2);
        assert_eq!(int_color.rgb(), color);
    }

    #[test]
    fn color_hex() {
        let int_color = Color::from_rgb(255, 255, 255);
        assert_eq!(int_color.hex(), "#ffffff");
    }
}
