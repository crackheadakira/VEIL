use std::path::Path;

use gpui::{Global, Rgba};
use serde::Deserialize;

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Default,
    Disabled,
    Hovered,
    Active,
}

#[derive(Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Primary,
    Secondary,
    Tertiary,
}

#[derive(Clone, Deserialize)]
pub struct ColorSet {
    pub default: Rgba,
    pub disabled: Rgba,
    pub hovered: Rgba,
    pub active: Rgba,
}

impl ColorSet {
    /// Get a color dynamically by state
    pub fn get(&self, state: State) -> Rgba {
        match state {
            State::Default => self.default,
            State::Disabled => self.disabled,
            State::Hovered => self.hovered,
            State::Active => self.active,
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct ColorRole {
    pub primary: ColorSet,
    pub secondary: ColorSet,
    pub tertiary: ColorSet,
}

impl ColorRole {
    /// Get a color dynamically by state and level
    pub fn get(&self, level: Level, state: State) -> Rgba {
        match level {
            Level::Primary => self.primary.get(state),
            Level::Secondary => self.secondary.get(state),
            Level::Tertiary => self.tertiary.get(state),
        }
    }
}

#[derive(Clone, Deserialize)]
pub struct Theme {
    pub text: ColorRole,

    pub background: ColorRole,

    pub border: ColorRole,
}

impl Global for Theme {}

impl Default for Theme {
    fn default() -> Self {
        Self {
            text: ColorRole {
                primary: ColorSet {
                    default: rgb(160, 159, 164),
                    disabled: rgb(123, 122, 129),
                    hovered: rgb(179, 178, 182),
                    active: rgb(198, 198, 201),
                },

                secondary: ColorSet {
                    default: rgb(104, 105, 112),
                    disabled: rgb(76, 77, 83),
                    hovered: rgb(133, 134, 140),
                    active: rgb(164, 165, 169),
                },

                tertiary: ColorSet {
                    default: rgb(71, 71, 73),
                    disabled: rgb(48, 48, 50),
                    hovered: rgb(95, 95, 97),
                    active: rgb(120, 120, 122),
                },
            },

            background: ColorRole {
                primary: ColorSet {
                    default: rgb(10, 10, 13),
                    disabled: rgb(5, 5, 7),
                    hovered: rgb(13, 14, 17),
                    active: rgb(18, 18, 23),
                },

                secondary: ColorSet {
                    default: rgb(11, 11, 14),
                    disabled: rgb(6, 6, 8),
                    hovered: rgb(17, 18, 22),
                    active: rgb(24, 24, 30),
                },

                tertiary: ColorSet {
                    default: rgb(13, 13, 16),
                    disabled: rgb(7, 7, 9),
                    hovered: rgb(19, 19, 24),
                    active: rgb(26, 26, 33),
                },
            },

            border: ColorRole {
                primary: ColorSet {
                    default: rgb(33, 31, 39),
                    disabled: rgb(20, 20, 26),
                    hovered: rgb(45, 43, 52),
                    active: rgb(57, 54, 66),
                },

                secondary: ColorSet {
                    default: rgb(38, 35, 46),
                    disabled: rgb(25, 23, 31),
                    hovered: rgb(52, 48, 62),
                    active: rgb(65, 61, 78),
                },

                tertiary: ColorSet {
                    default: rgb(45, 42, 53),
                    disabled: rgb(30, 28, 36),
                    hovered: rgb(61, 57, 72),
                    active: rgb(79, 71, 100),
                },
            },
        }
    }
}

const fn rgb(r: u8, g: u8, b: u8) -> Rgba {
    Rgba {
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    }
}

/*impl Theme {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let data = std::fs::read_to_string(path)?;
        let theme: Theme = serde_json::from_str(&data)?;
        Ok(theme)
    }
}*/
