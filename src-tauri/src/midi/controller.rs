use std::sync::Arc;
use std::thread;
use std::time::Duration;

use midir::{MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};

use crate::midi::devices::MidiDevice;

/// Color representation using RGB
#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };
    pub const YELLOW: Color = Color {
        r: 255,
        g: 255,
        b: 0,
    };
    pub const MAGENTA: Color = Color {
        r: 255,
        g: 0,
        b: 255,
    };
    pub const CYAN: Color = Color {
        r: 0,
        g: 255,
        b: 255,
    };
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
    };
}

/// Event from a grid controller
#[derive(Clone, Debug)]
pub enum ControllerEvent {
    /// Pad pressed (grid_id, velocity)
    PadPressed(u8, u8),
    /// Pad released (grid_id)
    PadReleased(u8),
    /// Button pressed (id)
    ButtonPressed(u8),
    /// Button released (id)
    ButtonReleased(u8),
}

/// Trait defining functionality for grid controllers
pub trait GridController: Send + Sync {
    /// Connect to the controller
    fn connect(&mut self) -> Result<(), String>;

    /// Disconnect from the controller
    fn disconnect(&mut self) -> Result<(), String>;

    /// Set the color of a pad
    fn set_pad_color(&mut self, grid_id: u8, color: Color) -> Result<(), String>;

    /// Set the color of a button
    fn set_button_color(&mut self, button_id: u8, color: Color) -> Result<(), String>;

    /// Clear all pad colors
    fn clear(&mut self) -> Result<(), String>;

    /// Map from application grid id (0-63) to controller-specific id
    fn map_grid_id(&self, app_grid_id: u8) -> u8;

    /// Map from controller-specific id to application grid id (0-63)
    fn map_to_app_grid_id(&self, controller_id: u8) -> Option<u8>;

    /// Register a callback for controller events
    fn set_event_callback(&mut self, callback: Arc<dyn Fn(ControllerEvent) + Send + Sync>);

    /// Create a clone of this controller
    fn clone_box(&self) -> Box<dyn GridController>;
}

/// Implementation for Novation Launchpad MK2
pub struct LaunchpadMk2 {
    device: MidiDevice,
    input_connection: Option<MidiInputConnection<()>>,
    output_connection: Option<MidiOutputConnection>,
    event_callback: Option<Arc<dyn Fn(ControllerEvent) + Send + Sync>>,
}

impl LaunchpadMk2 {
    pub fn new(device: MidiDevice) -> Self {
        LaunchpadMk2 {
            device,
            input_connection: None,
            output_connection: None,
            event_callback: None,
        }
    }

    /// Convert RGB color to Launchpad's color format
    fn rgb_to_launchpad_color(&self, color: Color) -> u8 {
        // Simple conversion - more accurate would use a lookup table
        let r = color.r / 85; // 0-3
        let g = color.g / 85; // 0-3
        let b = color.b / 85; // 0-3

        // Launchpad MK2 uses a 4x4x4 color space, encoded as:
        // 16 * r + 4 * g + b
        16 * r + 4 * g + b
    }
}

impl Clone for LaunchpadMk2 {
    fn clone(&self) -> Self {
        LaunchpadMk2 {
            device: self.device.clone(),
            input_connection: None,  // Connections can't be cloned
            output_connection: None, // Connections can't be cloned
            event_callback: self.event_callback.clone(),
        }
    }
}

impl GridController for LaunchpadMk2 {
    fn clone_box(&self) -> Box<dyn GridController> {
        Box::new(self.clone())
    }

    fn connect(&mut self) -> Result<(), String> {
        // Connect to MIDI input
        let midi_in = MidiInput::new("snap-blaster").map_err(|e| e.to_string())?;

        // Find the port
        let in_ports = midi_in.ports();
        let in_port = in_ports
            .iter()
            .find(|p| {
                midi_in
                    .port_name(p)
                    .map(|name| name == self.device.name)
                    .unwrap_or(false)
            })
            .ok_or_else(|| format!("Could not find MIDI input device: {}", self.device.name))?;

        // Create a callback to handle incoming MIDI messages
        let callback = {
            let event_callback = self.event_callback.clone();

            move |_timestamp, message: &[u8], _: &mut ()| {
                if message.len() < 3 {
                    return;
                }

                // Parse MIDI message based on Launchpad MK2 protocol
                let status = message[0];
                let note = message[1];
                let velocity = message[2];

                let event = match (status, velocity) {
                    (0x90, 0) => Some(ControllerEvent::PadReleased(note)),
                    (0x90, v) => Some(ControllerEvent::PadPressed(note, v)),
                    (0xB0, 0) => Some(ControllerEvent::ButtonReleased(note)),
                    (0xB0, v) => Some(ControllerEvent::ButtonPressed(note)),
                    _ => None,
                };

                if let Some(event) = event {
                    if let Some(ref callback) = event_callback {
                        callback(event);
                    }
                }
            }
        };

        let input_conn = midi_in
            .connect(in_port, "launchpad-input", callback, ())
            .map_err(|e| e.to_string())?;
        self.input_connection = Some(input_conn);

        // Connect to MIDI output
        let midi_out = MidiOutput::new("snap-blaster").map_err(|e| e.to_string())?;

        // Find the port
        let out_ports = midi_out.ports();
        let out_port = out_ports
            .iter()
            .find(|p| {
                midi_out
                    .port_name(p)
                    .map(|name| name == self.device.name)
                    .unwrap_or(false)
            })
            .ok_or_else(|| format!("Could not find MIDI output device: {}", self.device.name))?;

        let output_conn = midi_out
            .connect(out_port, "launchpad-output")
            .map_err(|e| e.to_string())?;
        self.output_connection = Some(output_conn);

        // Set to programmer mode (for RGB control)
        if let Some(ref mut conn) = self.output_connection {
            conn.send(&[0xF0, 0x00, 0x20, 0x29, 0x02, 0x18, 0x22, 0x00, 0xF7])
                .map_err(|e| e.to_string())?;

            // Wait a moment for the device to process
            thread::sleep(Duration::from_millis(100));
        }

        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), String> {
        // Reset the Launchpad
        if let Some(ref mut conn) = self.output_connection {
            conn.send(&[0xF0, 0x00, 0x20, 0x29, 0x02, 0x18, 0x0E, 0x00, 0xF7])
                .map_err(|e| e.to_string())?;
        }

        self.input_connection = None;
        self.output_connection = None;

        Ok(())
    }

    fn set_pad_color(&mut self, grid_id: u8, color: Color) -> Result<(), String> {
        let launchpad_id = self.map_grid_id(grid_id);
        let color_value = self.rgb_to_launchpad_color(color);

        if let Some(ref mut conn) = self.output_connection {
            // Launchpad MK2 uses Note On messages for setting pad colors
            conn.send(&[0x90, launchpad_id, color_value])
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    fn set_button_color(&mut self, button_id: u8, color: Color) -> Result<(), String> {
        let color_value = self.rgb_to_launchpad_color(color);

        if let Some(ref mut conn) = self.output_connection {
            // Launchpad MK2 uses CC messages for setting button colors
            conn.send(&[0xB0, button_id, color_value])
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    fn clear(&mut self) -> Result<(), String> {
        // Set all pads to black
        for i in 0..64 {
            self.set_pad_color(i, Color::BLACK)?;
        }

        // Set all buttons to black
        for i in 104..112 {
            self.set_button_color(i, Color::BLACK)?;
        }

        Ok(())
    }

    fn map_grid_id(&self, app_grid_id: u8) -> u8 {
        if app_grid_id >= 64 {
            return 0; // Invalid ID
        }

        // Map from linear 0-63 to Launchpad's 9x9 grid
        // Launchpad MK2 has notes laid out as:
        // 11 12 13 14 15 16 17 18 19
        // 21 22 23 24 25 26 27 28 29
        // ...
        // 81 82 83 84 85 86 87 88 89

        let row = app_grid_id / 8;
        let col = app_grid_id % 8;

        // Convert to Launchpad-specific format
        // Adding 1 to both row and col because Launchpad starts at 1
        ((row + 1) * 10) + (col + 1)
    }

    fn map_to_app_grid_id(&self, controller_id: u8) -> Option<u8> {
        // Extract row and column from Launchpad ID
        let row = controller_id / 10;
        let col = controller_id % 10;

        // Validate the ID is within the grid
        if row < 1 || row > 8 || col < 1 || col > 8 {
            return None;
        }

        // Convert to app grid ID
        // Subtracting 1 from both row and col because app grid starts at 0
        Some(((row - 1) * 8) + (col - 1))
    }

    fn set_event_callback(&mut self, callback: Arc<dyn Fn(ControllerEvent) + Send + Sync>) {
        self.event_callback = Some(callback);
    }
}

/// Factory to create controllers based on device type
pub struct ControllerFactory;

impl ControllerFactory {
    pub fn create_controller(device: MidiDevice) -> Result<Box<dyn GridController>, String> {
        if !device.is_controller {
            return Err(format!(
                "Device '{}' is not a supported controller",
                device.name
            ));
        }

        // Determine controller type based on device name
        if device.name.to_lowercase().contains("launchpad") {
            if device.name.to_lowercase().contains("mk2") {
                return Ok(Box::new(LaunchpadMk2::new(device)));
            } else {
                // Future: implement other Launchpad variants
                return Err(format!("Unsupported Launchpad variant: {}", device.name));
            }
        }

        Err(format!("Unsupported controller type: {}", device.name))
    }
}


/// Implementation for Novation Launchpad X
pub struct LaunchpadX {
    device: MidiDevice,
    input_connection: Option<MidiInputConnection<()>>,
    output_connection: Option<MidiOutputConnection>,
    event_callback: Option<Arc<dyn Fn(ControllerEvent) + Send + Sync>>,
}

impl LaunchpadX {
    pub fn new(device: MidiDevice) -> Self {
        LaunchpadX {
            device,
            input_connection: None,
            output_connection: None,
            event_callback: None,
        }
    }

    /// Convert RGB color to Launchpad X's color format
    fn rgb_to_launchpad_color(&self, color: Color) -> u8 {
        // Simple conversion - more accurate would use a lookup table
        let r = color.r / 85; // 0-3
        let g = color.g / 85; // 0-3
        let b = color.b / 85; // 0-3

        // Launchpad X uses a 4x4x4 color space, encoded as:
        // 16 * r + 4 * g + b
        16 * r + 4 * g + b
    }
}

impl Clone for LaunchpadX {
    fn clone(&self) -> Self {
        LaunchpadX {
            device: self.device.clone(),
            input_connection: None,  // Connections can't be cloned
            output_connection: None, // Connections can't be cloned
            event_callback: self.event_callback.clone(),
        }
    }
}

impl GridController for LaunchpadX {
    fn clone_box(&self) -> Box<dyn GridController> {
        Box::new(self.clone())
    }

    fn connect(&mut self) -> Result<(), String> {
        // Connect to MIDI input
        let midi_in = MidiInput::new("snap-blaster").map_err(|e| e.to_string())?;

        // Find the port
        let in_ports = midi_in.ports();
        let in_port = in_ports
            .iter()
            .find(|p| {
                midi_in
                    .port_name(p)
                    .map(|name| name == self.device.name)
                    .unwrap_or(false)
            })
            .ok_or_else(|| format!("Could not find MIDI input device: {}", self.device.name))?;

        // Create a callback to handle incoming MIDI messages
        let callback = {
            let event_callback = self.event_callback.clone();

            move |_timestamp, message: &[u8], _: &mut ()| {
                if message.len() < 3 {
                    return;
                }

                // Parse MIDI message based on Launchpad X protocol
                let status = message[0];
                let note = message[1];
                let velocity = message[2];

                let event = match (status, velocity) {
                    (0x90, 0) => Some(ControllerEvent::PadReleased(note)),
                    (0x90, v) => Some(ControllerEvent::PadPressed(note, v)),
                    (0xB0, 0) => Some(ControllerEvent::ButtonReleased(note)),
                    (0xB0, v) => Some(ControllerEvent::ButtonPressed(note)),
                    _ => None,
                };

                if let Some(event) = event {
                    if let Some(ref callback) = event_callback {
                        callback(event);
                    }
                }
            }
        };

        let input_conn = midi_in
            .connect(in_port, "launchpad-input", callback, ())
            .map_err(|e| e.to_string())?;
        self.input_connection = Some(input_conn);

        // Connect to MIDI output
        let midi_out = MidiOutput::new("snap-blaster").map_err(|e| e.to_string())?;

        // Find the port
        let out_ports = midi_out.ports();
        let out_port = out_ports
            .iter()
            .find(|p| {
                midi_out
                    .port_name(p)
                    .map(|name| name == self.device.name)
                    .unwrap_or(false)
            })
            .ok_or_else(|| format!("Could not find MIDI output device: {}", self.device.name))?;

        let output_conn = midi_out
            .connect(out_port, "launchpad-output")
            .map_err(|e| e.to_string())?;
        self.output_connection = Some(output_conn);

        // Set to programmer mode (for RGB control) - Launchpad X specific SysEx command
        if let Some(ref mut conn) = self.output_connection {
            conn.send(&[0xF0, 0x00, 0x20, 0x29, 0x02, 0x0D, 0x0E, 0x01, 0xF7])
                .map_err(|e| e.to_string())?;

            // Wait a moment for the device to process
            thread::sleep(Duration::from_millis(100));
        }

        Ok(())
    }

    fn disconnect(&mut self) -> Result<(), String> {
        // Reset the Launchpad X
        if let Some(ref mut conn) = self.output_connection {
            conn.send(&[0xF0, 0x00, 0x20, 0x29, 0x02, 0x0D, 0x0E, 0x00, 0xF7])
                .map_err(|e| e.to_string())?;
        }

        self.input_connection = None;
        self.output_connection = None;

        Ok(())
    }

    fn set_pad_color(&mut self, grid_id: u8, color: Color) -> Result<(), String> {
        let launchpad_id = self.map_grid_id(grid_id);
        let color_value = self.rgb_to_launchpad_color(color);

        if let Some(ref mut conn) = self.output_connection {
            // Launchpad X uses Note On messages for setting pad colors
            conn.send(&[0x90, launchpad_id, color_value])
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    fn set_button_color(&mut self, button_id: u8, color: Color) -> Result<(), String> {
        let color_value = self.rgb_to_launchpad_color(color);

        if let Some(ref mut conn) = self.output_connection {
            // Launchpad X uses CC messages for setting button colors
            conn.send(&[0xB0, button_id, color_value])
                .map_err(|e| e.to_string())?;
        }

        Ok(())
    }

    fn clear(&mut self) -> Result<(), String> {
        // Set all pads to black
        for i in 0..64 {
            self.set_pad_color(i, Color::BLACK)?;
        }

        // Set all buttons to black
        for i in 104..112 {
            self.set_button_color(i, Color::BLACK)?;
        }

        Ok(())
    }

    fn map_grid_id(&self, app_grid_id: u8) -> u8 {
        if app_grid_id >= 64 {
            return 0; // Invalid ID
        }

        // Map from linear 0-63 to Launchpad X's 9x9 grid
        // Launchpad X has a similar layout to the MK2:
        // 11 12 13 14 15 16 17 18 19
        // 21 22 23 24 25 26 27 28 29
        // ...
        // 81 82 83 84 85 86 87 88 89

        let row = app_grid_id / 8;
        let col = app_grid_id % 8;

        // Convert to Launchpad-specific format
        // Adding 1 to both row and col because Launchpad starts at 1
        ((row + 1) * 10) + (col + 1)
    }

    fn map_to_app_grid_id(&self, controller_id: u8) -> Option<u8> {
        // Extract row and column from Launchpad ID
        let row = controller_id / 10;
        let col = controller_id % 10;

        // Validate the ID is within the grid
        if row < 1 || row > 8 || col < 1 || col > 8 {
            return None;
        }

        // Convert to app grid ID
        // Subtracting 1 from both row and col because app grid starts at 0
        Some(((row - 1) * 8) + (col - 1))
    }

    fn set_event_callback(&mut self, callback: Arc<dyn Fn(ControllerEvent) + Send + Sync>) {
        self.event_callback = Some(callback);
    }
}