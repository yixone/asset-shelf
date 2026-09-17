// TODO: Move to `media-core` crate

const RED_SHIFT: usize = 16;
const GREEN_SHIFT: usize = 8;

/// RGB color stored as a 32-bit integer (8 bits for each color channel)
///
/// ### Usage
/// ```
/// use schema::types::Color;
///
/// let red = Color::new(255, 0, 0);
/// assert_eq!(red.hex(), "#ff0000");
/// ```
#[cfg_attr(feature = "sqlx", derive(sqlx::Type), sqlx(transparent))]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color(pub(crate) i32);

impl Color {
    /// White color (`#FFFFFF`)
    pub const WHITE: Color = Color(0xFFFFFF);

    /// Black color (`#000000`)
    pub const BLACK: Color = Color(0x0);

    /// Creates a new [`Color`]
    pub const fn new(red: u8, green: u8, blue: u8) -> Self {
        Color(((red as i32) << RED_SHIFT) | ((green as i32) << GREEN_SHIFT) | (blue as i32))
    }

    /// Returns the color as a RGB channels tuple
    pub const fn rgb(self) -> (u8, u8, u8) {
        (
            ((self.0 >> RED_SHIFT) & 0xFF) as u8,
            ((self.0 >> GREEN_SHIFT) & 0xFF) as u8,
            ((self.0) & 0xFF) as u8,
        )
    }

    /// Returns the color as a hex string in the format `#ffffff`
    pub fn hex(self) -> String {
        let (r, g, b) = self.rgb();
        format!("#{r:02x}{g:02x}{b:02x}")
    }
}
