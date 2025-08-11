// Theme and styling management

// Use xilem's Color for native, define our own for web
#[cfg(any(
    all(feature = "xilem-native", not(feature = "xilem-web")),
    all(feature = "xilem-native", feature = "xilem-web")
))]
pub use xilem::Color;

// Define our own Color struct only for web
#[cfg(all(feature = "xilem-web", not(feature = "xilem-native")))]
#[derive(Debug, Clone, Copy)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[cfg(all(feature = "xilem-web", not(feature = "xilem-native")))]
impl Color {
    pub fn from_rgb8(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b, a: 255 }
    }
    
    pub fn from_rgba8(r: u8, g: u8, b: u8, a: u8) -> Self {
        Color { r, g, b, a }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ThemeMode {
    Light,
    Dark,
    Auto,
}

#[derive(Debug, Clone)]
pub struct Theme {
    pub mode: ThemeMode,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            mode: ThemeMode::Light,
        }
    }
}

impl Theme {
    /// Returns a high-contrast background color.
    pub fn background_color(&self) -> Color {
        #[cfg(any(
            all(feature = "xilem-native", not(feature = "xilem-web")),
            all(feature = "xilem-native", feature = "xilem-web")
        ))]
        {
            match self.mode {
                ThemeMode::Light => Color::WHITE,
                ThemeMode::Dark => Color::from_rgb8(18, 18, 18),
                ThemeMode::Auto => Color::WHITE,
            }
        }
        #[cfg(all(feature = "xilem-web", not(feature = "xilem-native")))]
        {
            match self.mode {
                ThemeMode::Light => Color::from_rgb8(255, 255, 255),
                ThemeMode::Dark => Color::from_rgb8(18, 18, 18),
                ThemeMode::Auto => Color::from_rgb8(255, 255, 255),
            }
        }
    }

    /// Returns a high-contrast text color.
    pub fn text_color(&self) -> Color {
        #[cfg(any(
            all(feature = "xilem-native", not(feature = "xilem-web")),
            all(feature = "xilem-native", feature = "xilem-web")
        ))]
        {
            match self.mode {
                ThemeMode::Light => Color::BLACK,
                ThemeMode::Dark => Color::WHITE,
                ThemeMode::Auto => Color::BLACK,
            }
        }
        #[cfg(all(feature = "xilem-web", not(feature = "xilem-native")))]
        {
            match self.mode {
                ThemeMode::Light => Color::from_rgb8(0, 0, 0),
                ThemeMode::Dark => Color::from_rgb8(255, 255, 255),
                ThemeMode::Auto => Color::from_rgb8(0, 0, 0),
            }
        }
    }

    /// Returns a highly visible primary color.
    pub fn primary_color(&self) -> Color {
        #[cfg(any(
            all(feature = "xilem-native", not(feature = "xilem-web")),
            all(feature = "xilem-native", feature = "xilem-web")
        ))]
        {
            match self.mode {
                ThemeMode::Light => Color::from_rgb8(0, 92, 197),
                ThemeMode::Dark => Color::from_rgb8(51, 153, 255),
                ThemeMode::Auto => Color::from_rgb8(0, 92, 197),
            }
        }
        #[cfg(all(feature = "xilem-web", not(feature = "xilem-native")))]
        {
            match self.mode {
                ThemeMode::Light => Color::from_rgb8(0, 92, 197),
                ThemeMode::Dark => Color::from_rgb8(51, 153, 255),
                ThemeMode::Auto => Color::from_rgb8(0, 92, 197),
            }
        }
    }

    /// Returns a high-contrast success color.
    pub fn success_color(&self) -> Color {
        #[cfg(any(
            all(feature = "xilem-native", not(feature = "xilem-web")),
            all(feature = "xilem-native", feature = "xilem-web")
        ))]
        {
            match self.mode {
                ThemeMode::Light => Color::from_rgb8(0, 128, 0),
                ThemeMode::Dark => Color::from_rgb8(80, 220, 100),
                ThemeMode::Auto => Color::from_rgb8(0, 128, 0),
            }
        }
        #[cfg(all(feature = "xilem-web", not(feature = "xilem-native")))]
        {
            match self.mode {
                ThemeMode::Light => Color::from_rgb8(0, 128, 0),
                ThemeMode::Dark => Color::from_rgb8(80, 220, 100),
                ThemeMode::Auto => Color::from_rgb8(0, 128, 0),
            }
        }
    }

    /// Returns a high-contrast error color.
    pub fn error_color(&self) -> Color {
        #[cfg(any(
            all(feature = "xilem-native", not(feature = "xilem-web")),
            all(feature = "xilem-native", feature = "xilem-web")
        ))]
        {
            match self.mode {
                ThemeMode::Light => Color::from_rgb8(200, 0, 0),
                ThemeMode::Dark => Color::from_rgb8(255, 85, 85),
                ThemeMode::Auto => Color::from_rgb8(200, 0, 0),
            }
        }
        #[cfg(all(feature = "xilem-web", not(feature = "xilem-native")))]
        {
            match self.mode {
                ThemeMode::Light => Color::from_rgb8(200, 0, 0),
                ThemeMode::Dark => Color::from_rgb8(255, 85, 85),
                ThemeMode::Auto => Color::from_rgb8(200, 0, 0),
            }
        }
    }

    /// Returns a high-contrast warning color.
    pub fn warning_color(&self) -> Color {
        #[cfg(any(
            all(feature = "xilem-native", not(feature = "xilem-web")),
            all(feature = "xilem-native", feature = "xilem-web")
        ))]
        {
            match self.mode {
                ThemeMode::Light => Color::from_rgb8(204, 102, 0),
                ThemeMode::Dark => Color::from_rgb8(255, 204, 77),
                ThemeMode::Auto => Color::from_rgb8(204, 102, 0),
            }
        }
        #[cfg(all(feature = "xilem-web", not(feature = "xilem-native")))]
        {
            match self.mode {
                ThemeMode::Light => Color::from_rgb8(204, 102, 0),
                ThemeMode::Dark => Color::from_rgb8(255, 204, 77),
                ThemeMode::Auto => Color::from_rgb8(204, 102, 0),
            }
        }
    }

    /// Returns a high-contrast surface color for cards/panels.
    pub fn surface_color(&self) -> Color {
        #[cfg(any(
            all(feature = "xilem-native", not(feature = "xilem-web")),
            all(feature = "xilem-native", feature = "xilem-web")
        ))]
        {
            match self.mode {
                ThemeMode::Light => Color::from_rgb8(240, 240, 240),
                ThemeMode::Dark => Color::from_rgb8(28, 28, 30),
                ThemeMode::Auto => Color::from_rgb8(240, 240, 240),
            }
        }
        #[cfg(all(feature = "xilem-web", not(feature = "xilem-native")))]
        {
            match self.mode {
                ThemeMode::Light => Color::from_rgb8(240, 240, 240),
                ThemeMode::Dark => Color::from_rgb8(28, 28, 30),
                ThemeMode::Auto => Color::from_rgb8(240, 240, 240),
            }
        }
    }
}