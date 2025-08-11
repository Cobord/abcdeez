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
    pub fn background_color(&self) -> Color {
        match self.mode {
            ThemeMode::Light => Color::from_rgb8(255, 255, 255),
            ThemeMode::Dark => Color::from_rgb8(30, 30, 30),
            ThemeMode::Auto => Color::from_rgb8(255, 255, 255), // Default to light
        }
    }

    pub fn text_color(&self) -> Color {
        match self.mode {
            ThemeMode::Light => Color::from_rgb8(0, 0, 0),
            ThemeMode::Dark => Color::from_rgb8(255, 255, 255),
            ThemeMode::Auto => Color::from_rgb8(0, 0, 0),
        }
    }

    pub fn primary_color(&self) -> Color {
        Color::from_rgb8(0, 122, 255) // iOS blue
    }

    pub fn success_color(&self) -> Color {
        Color::from_rgb8(52, 199, 89) // Green
    }

    pub fn error_color(&self) -> Color {
        Color::from_rgb8(255, 59, 48) // Red
    }

    pub fn warning_color(&self) -> Color {
        Color::from_rgb8(255, 149, 0) // Orange
    }

    pub fn surface_color(&self) -> Color {
        match self.mode {
            ThemeMode::Light => Color::from_rgb8(245, 245, 250),
            ThemeMode::Dark => Color::from_rgb8(45, 45, 50),
            ThemeMode::Auto => Color::from_rgb8(245, 245, 250),
        }
    }
}