use serde::{Deserialize, Serialize};

use crate::color::Rgb;

/// One parameter of an effect, described well enough for an interface that does not know the effect.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct ControlDefinition {
    pub id: String,
    pub label: String,
    pub description: String,
    pub kind: ControlKind,
}

/// A control without a default is required.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ControlKind {
    Color {
        default: Option<Rgb>,
    },
    Palette {
        min_colors: u32,
        max_colors: u32,
        default: Vec<Rgb>,
    },
    Speed {
        range: Bounds,
        default: u32,
        unit: SpeedUnit,
    },
    Direction {
        default: Direction,
    },
    /// Percent of the effect's own colours; 0 is black.
    Brightness {
        range: Bounds,
        default: u32,
    },
    Number {
        range: Bounds,
        step: u32,
        default: u32,
        unit: NumberUnit,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub struct Bounds {
    pub min: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum SpeedUnit {
    CyclesPerMinute,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum NumberUnit {
    Percent,
    Count,
}

/// Direction along the LED order of the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum Direction {
    Forward,
    Backward,
}

impl Direction {
    pub(crate) fn orient(self, position: f64) -> f64 {
        match self {
            Self::Forward => position,
            Self::Backward => 1.0 - position,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ControlValue {
    Color(Rgb),
    Palette(Vec<Rgb>),
    Number(u32),
    Direction(Direction),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ValueType {
    Color,
    Palette,
    Number,
    Direction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
pub enum ValueProblem {
    UnknownControl,
    Missing,
    WrongType { expected: ValueType },
    OutOfRange { min: u32, max: u32 },
    OffStep { step: u32 },
    PaletteLength { min: u32, max: u32 },
}

impl ControlKind {
    pub(crate) fn default_value(&self) -> Option<ControlValue> {
        match self {
            Self::Color { default } => default.map(ControlValue::Color),
            Self::Palette { default, .. } => Some(ControlValue::Palette(default.clone())),
            Self::Speed { default, .. }
            | Self::Brightness { default, .. }
            | Self::Number { default, .. } => Some(ControlValue::Number(*default)),
            Self::Direction { default } => Some(ControlValue::Direction(*default)),
        }
    }

    pub(crate) fn check(&self, value: &ControlValue) -> Result<(), ValueProblem> {
        match (self, value) {
            (Self::Color { .. }, ControlValue::Color(_))
            | (Self::Direction { .. }, ControlValue::Direction(_)) => Ok(()),
            (
                Self::Palette {
                    min_colors,
                    max_colors,
                    ..
                },
                ControlValue::Palette(colors),
            ) => {
                let count = colors.len();
                if (*min_colors as usize..=*max_colors as usize).contains(&count) {
                    Ok(())
                } else {
                    Err(ValueProblem::PaletteLength {
                        min: *min_colors,
                        max: *max_colors,
                    })
                }
            }
            (
                Self::Speed { range, .. } | Self::Brightness { range, .. },
                ControlValue::Number(value),
            ) => range.check(*value),
            (Self::Number { range, step, .. }, ControlValue::Number(value)) => {
                range.check(*value)?;
                if (value - range.min) % (*step).max(1) == 0 {
                    Ok(())
                } else {
                    Err(ValueProblem::OffStep { step: *step })
                }
            }
            _ => Err(ValueProblem::WrongType {
                expected: self.value_type(),
            }),
        }
    }

    fn value_type(&self) -> ValueType {
        match self {
            Self::Color { .. } => ValueType::Color,
            Self::Palette { .. } => ValueType::Palette,
            Self::Speed { .. } | Self::Brightness { .. } | Self::Number { .. } => ValueType::Number,
            Self::Direction { .. } => ValueType::Direction,
        }
    }
}

impl Bounds {
    pub(crate) fn clamp(self, value: u32) -> u32 {
        value.clamp(self.min, self.max)
    }

    fn check(self, value: u32) -> Result<(), ValueProblem> {
        if (self.min..=self.max).contains(&value) {
            Ok(())
        } else {
            Err(ValueProblem::OutOfRange {
                min: self.min,
                max: self.max,
            })
        }
    }
}
