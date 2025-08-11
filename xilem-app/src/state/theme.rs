// Theme and styling management

use xilem::Color;

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
        match self.mode {
            ThemeMode::Light => Color::from_rgb8(255, 255, 255), // Pure white
            ThemeMode::Dark => Color::from_rgb8(18, 18, 18),     // Near-black, not pure black for eye comfort
            ThemeMode::Auto => Color::from_rgb8(255, 255, 255),  // Default to light
        }
    }

    /// Returns a high-contrast text color.
    pub fn text_color(&self) -> Color {
        match self.mode {
            ThemeMode::Light => Color::from_rgb8(0, 0, 0),       // Pure black
            ThemeMode::Dark => Color::from_rgb8(255, 255, 255),  // Pure white
            ThemeMode::Auto => Color::from_rgb8(0, 0, 0),
        }
    }

    /// Returns a highly visible primary color.
    pub fn primary_color(&self) -> Color {
        match self.mode {
            ThemeMode::Light => Color::from_rgb8(0, 92, 197),    // Strong blue, WCAG AA on white
            ThemeMode::Dark => Color::from_rgb8(51, 153, 255),   // Lighter blue for dark bg, WCAG AA on dark
            ThemeMode::Auto => Color::from_rgb8(0, 92, 197),
        }
    }

    /// Returns a high-contrast success color.
    pub fn success_color(&self) -> Color {
        match self.mode {
            ThemeMode::Light => Color::from_rgb8(0, 128, 0),     // Strong green
            ThemeMode::Dark => Color::from_rgb8(80, 220, 100),   // Lighter green for dark bg
            ThemeMode::Auto => Color::from_rgb8(0, 128, 0),
        }
    }

    /// Returns a high-contrast error color.
    pub fn error_color(&self) -> Color {
        match self.mode {
            ThemeMode::Light => Color::from_rgb8(200, 0, 0),     // Strong red
            ThemeMode::Dark => Color::from_rgb8(255, 85, 85),    // Lighter red for dark bg
            ThemeMode::Auto => Color::from_rgb8(200, 0, 0),
        }
    }

    /// Returns a high-contrast warning color.
    pub fn warning_color(&self) -> Color {
        match self.mode {
            ThemeMode::Light => Color::from_rgb8(204, 102, 0),   // Strong orange
            ThemeMode::Dark => Color::from_rgb8(255, 204, 77),   // Lighter orange for dark bg
            ThemeMode::Auto => Color::from_rgb8(204, 102, 0),
        }
    }

    /// Returns a high-contrast surface color for cards/panels.
    pub fn surface_color(&self) -> Color {
        match self.mode {
            ThemeMode::Light => Color::from_rgb8(240, 240, 240), // Light gray, high contrast with text
            ThemeMode::Dark => Color::from_rgb8(28, 28, 30),     // Slightly lighter than bg for separation
            ThemeMode::Auto => Color::from_rgb8(240, 240, 240),
        }
    }
}