mod editor;
mod settings;

use crate::settings::Settings;
use dslcad_storage::protocol::Render;
use std::error::Error;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::mpsc::{channel, Receiver, Sender};

enum PreviewEvent {
    Rendering,
    Render(Render),
    Error(String),
}

#[derive(Clone)]
pub struct PreviewHandle {
    tx: Sender<PreviewEvent>,
}

impl PreviewHandle {
    pub fn show_rendering(&self) {
        self.tx.send(PreviewEvent::Rendering).unwrap()
    }

    pub fn show_render(&self, render: Render) {
        self.tx.send(PreviewEvent::Render(render)).unwrap()
    }

    pub fn show_error(&self, error: String) {
        self.tx.send(PreviewEvent::Error(error)).unwrap()
    }
}

pub struct Preview {
    rx: Receiver<PreviewEvent>,
}

/// Camera rotations around the part axes, in degrees. Unset axes keep their
/// default value.
///
/// - `x` tilts the camera from the top of the part (`0` is a top view, `90` is
///   a side view, defaults to the preview's isometric tilt)
/// - `y` rotates the camera around the vertical axis (`0` looks from the x-axis,
///   defaults to 45)
/// - `z` rolls the camera around the view direction (defaults to 0)
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AxisAngles {
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub z: Option<f32>,
}

impl FromStr for AxisAngles {
    type Err = String;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let input = input.trim().to_ascii_lowercase();
        if input.is_empty() {
            return Err("expected an angle like 'x90y45'".to_string());
        }

        // A bare number is shorthand for a rotation around the vertical axis
        if input.starts_with(|c: char| c.is_ascii_digit() || matches!(c, '-' | '+' | '.')) {
            return Ok(Self {
                y: Some(
                    input
                        .parse()
                        .map_err(|_| format!("invalid angle '{}'", input))?,
                ),
                ..Default::default()
            });
        }

        let mut angles = Self::default();
        let bytes = input.as_bytes();
        let mut index = 0;
        while index < bytes.len() {
            let axis = bytes[index] as char;
            index += 1;

            let start = index;
            while index < bytes.len() && !bytes[index].is_ascii_alphabetic() {
                index += 1;
            }
            let value = &input[start..index];
            let value: f32 = value
                .parse()
                .map_err(|_| format!("invalid angle for axis '{}' in '{}'", axis, input))?;

            match axis {
                'x' => angles.x = Some(value),
                'y' => angles.y = Some(value),
                'z' => angles.z = Some(value),
                _ => return Err(format!("unknown axis '{}' in '{}'", axis, input)),
            }
        }

        Ok(angles)
    }
}

/// Options for rendering a single view of a part to an image.
#[derive(Clone, Debug)]
pub struct ScreenshotOptions {
    /// Path the png is written to.
    pub path: PathBuf,
    /// Rotations of the camera around the part axes.
    pub angle: AxisAngles,
    /// Zoom factor applied to the fit-to-view distance. Larger values move
    /// the camera closer. Defaults to 1.
    pub zoom: Option<f32>,
}

impl Preview {
    pub fn new() -> (Self, PreviewHandle) {
        let (tx, rx) = channel();
        (Self { rx }, PreviewHandle { tx })
    }

    pub fn open(self, cheetsheet: String) {
        editor::main(cheetsheet, self.rx, Settings::default()).unwrap();
    }

    /// Render a single view to the image described by the options and exit.
    pub fn screenshot(self, options: ScreenshotOptions) -> Result<(), Box<dyn Error>> {
        editor::screenshot(self.rx, options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_axis_angles() {
        assert_eq!(
            "x90y45".parse::<AxisAngles>().unwrap(),
            AxisAngles {
                x: Some(90.0),
                y: Some(45.0),
                z: None
            }
        );
        assert_eq!(
            "y45X90".parse::<AxisAngles>().unwrap(),
            AxisAngles {
                x: Some(90.0),
                y: Some(45.0),
                z: None
            }
        );
        assert_eq!(
            "x-90y45.5z10".parse::<AxisAngles>().unwrap(),
            AxisAngles {
                x: Some(-90.0),
                y: Some(45.5),
                z: Some(10.0)
            }
        );
        assert_eq!(
            "-45".parse::<AxisAngles>().unwrap(),
            AxisAngles {
                x: None,
                y: Some(-45.0),
                z: None
            }
        );
    }

    #[test]
    fn rejects_invalid_axis_angles() {
        assert!("".parse::<AxisAngles>().is_err());
        assert!("a90".parse::<AxisAngles>().is_err());
        assert!("x".parse::<AxisAngles>().is_err());
        assert!("90y45".parse::<AxisAngles>().is_err());
        assert!("x90foo".parse::<AxisAngles>().is_err());
    }
}
