use midir::{MidiInput, MidiOutput};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Structure to represent a MIDI device
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MidiDevice {
    pub id: String,
    pub name: String,
    pub is_input: bool,
    pub is_controller: bool,
}

/// Registry of available MIDI devices
pub struct DeviceRegistry {
    devices: Arc<Mutex<HashMap<String, MidiDevice>>>,
}

impl DeviceRegistry {
    /// Create a new device registry
    pub fn new() -> Self {
        DeviceRegistry {
            devices: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Scan for available MIDI devices
    pub fn scan_devices(&self) -> Result<Vec<MidiDevice>, String> {
        let mut all_devices = Vec::new();

        // Scan for MIDI input devices
        if let Ok(midi_in) = MidiInput::new("snap-blaster-scanner") {
            let in_ports = midi_in.ports();

            for port in in_ports {
                if let Ok(port_name) = midi_in.port_name(&port) {
                    let id = format!("in:{}", port_name);

                    // Detect if this might be a controller device
                    // This is a heuristic based on common controller names
                    let is_controller = port_name.to_lowercase().contains("launchpad")
                        || port_name.to_lowercase().contains("apc")
                        || port_name.to_lowercase().contains("push")
                        || port_name.to_lowercase().contains("fire")
                        || port_name.to_lowercase().contains("grid");

                    let device = MidiDevice {
                        id: id.clone(),
                        name: port_name,
                        is_input: true,
                        is_controller,
                    };

                    all_devices.push(device.clone());

                    let mut devices = self.devices.lock().unwrap();
                    devices.insert(id, device);
                }
            }
        }

        // Scan for MIDI output devices
        if let Ok(midi_out) = MidiOutput::new("snap-blaster-scanner") {
            let out_ports = midi_out.ports();

            for port in out_ports {
                if let Ok(port_name) = midi_out.port_name(&port) {
                    let id = format!("out:{}", port_name);

                    // Skip if this is already registered as an input
                    if self.devices.lock().unwrap().values().any(|d| d.name == port_name && d.is_input) {
                        continue;
                    }

                    let device = MidiDevice {
                        id: id.clone(),
                        name: port_name,
                        is_input: false,
                        is_controller: false, // Output devices are not controllers
                    };

                    all_devices.push(device.clone());

                    let mut devices = self.devices.lock().unwrap();
                    devices.insert(id, device);
                }
            }
        }

        Ok(all_devices)
    }

    /// Get a list of all devices
    pub fn get_all_devices(&self) -> Vec<MidiDevice> {
        let devices = self.devices.lock().unwrap();
        devices.values().cloned().collect()
    }

    /// Get devices that can be used as controllers
    pub fn get_controller_devices(&self) -> Vec<MidiDevice> {
        let devices = self.devices.lock().unwrap();
        devices.values()
            .filter(|d| d.is_controller)
            .cloned()
            .collect()
    }

    /// Get devices that can be used as MIDI outputs
    pub fn get_output_devices(&self) -> Vec<MidiDevice> {
        let devices = self.devices.lock().unwrap();
        devices.values()
            .filter(|d| !d.is_input)
            .cloned()
            .collect()
    }

    /// Get a specific device by ID
    pub fn get_device(&self, id: &str) -> Option<MidiDevice> {
        let devices = self.devices.lock().unwrap();
        devices.get(id).cloned()
    }
}

/// Factory to create the device registry
pub struct DeviceRegistryFactory;

impl DeviceRegistryFactory {
    /// Create and initialize the device registry
    pub fn create() -> Result<DeviceRegistry, String> {
        let registry = DeviceRegistry::new();
        registry.scan_devices()?;
        Ok(registry)
    }
}