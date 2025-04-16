use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use midir::{MidiOutput, MidiOutputPort, MidiOutputConnection};

use crate::models::cc::CCValue;
use crate::models::scene::Scene;
use crate::midi::devices::MidiDevice;

/// Types of commands that can be sent to the MIDI engine
pub enum MidiCommand {
    /// Send a CC message immediately
    SendCC {
        channel: u8,
        cc_number: u8,
        value: u8,
    },
    /// Transition a CC value over time
    Transition {
        channel: u8,
        cc_number: u8,
        start_value: u8,
        end_value: u8,
        duration_ms: u32,
        curve: TransitionCurve,
    },
    /// Activate a scene (sends all CC values in the scene)
    ActivateScene {
        scene: Scene,
        quantize_beats: Option<u8>,
    },
    /// Morph between two scenes over time
    MorphScenes {
        start_scene: Scene,
        end_scene: Scene,
        duration_ms: u32,
        curve: TransitionCurve,
    },
    /// Stop all ongoing transitions
    StopTransitions,
    /// Set the engine's tempo in BPM
    SetTempo(f64),
    /// Request the engine to shut down
    Shutdown,
}

/// Types of curves for transitions between values
#[derive(Clone, Copy)]
pub enum TransitionCurve {
    Linear,
    Exponential,
    Logarithmic,
    SCurve,
}

impl TransitionCurve {
    /// Applies the curve function to a normalized (0.0-1.0) position
    fn apply(&self, position: f64) -> f64 {
        match self {
            TransitionCurve::Linear => position,
            TransitionCurve::Exponential => position * position,
            TransitionCurve::Logarithmic => position.sqrt(),
            TransitionCurve::SCurve => {
                // Simple S-curve: y = 3x² - 2x³ (smoother transitions at endpoints)
                let x = position;
                3.0 * x * x - 2.0 * x * x * x
            }
        }
    }
}

/// A transition in progress
struct ActiveTransition {
    channel: u8,
    cc_number: u8,
    start_value: u8,
    end_value: u8,
    start_time: Instant,
    duration: Duration,
    curve: TransitionCurve,
}

/// Main MIDI engine that processes and sends MIDI commands
pub struct MidiEngine {
    command_queue: Arc<Mutex<VecDeque<MidiCommand>>>,
    connections: Vec<MidiOutputConnection>,
    transitions: Vec<ActiveTransition>,
    tempo: f64, // BPM
    running: Arc<Mutex<bool>>,
    thread_handle: Option<thread::JoinHandle<()>>,
}

impl MidiEngine {
    /// Create a new MIDI engine
    pub fn new() -> Result<Self, String> {
        let engine = MidiEngine {
            command_queue: Arc::new(Mutex::new(VecDeque::new())),
            connections: Vec::new(),
            transitions: Vec::new(),
            tempo: 120.0,
            running: Arc::new(Mutex::new(true)),
            thread_handle: None,
        };

        Ok(engine)
    }

    /// Start the MIDI engine processing thread
    pub fn start(&mut self) -> Result<(), String> {
        if self.thread_handle.is_some() {
            return Err("Engine already running".to_string());
        }

        let command_queue = Arc::clone(&self.command_queue);
        let running = Arc::clone(&self.running);

        let handle = thread::spawn(move || {
            let mut last_process = Instant::now();

            while *running.lock().unwrap() {
                // Process any pending commands
                let mut commands = command_queue.lock().unwrap();

                // Process up to 10 commands per iteration to prevent blocking
                for _ in 0..10 {
                    if let Some(cmd) = commands.pop_front() {
                        drop(commands); // Release the lock while processing

                        // Process command (implementation will be expanded)
                        // Placeholder for command processing

                        // Re-acquire the lock for the next iteration
                        commands = command_queue.lock().unwrap();
                    } else {
                        break;
                    }
                }

                drop(commands); // Release the lock

                // Sleep for a short duration to prevent CPU hogging
                // 1ms gives us approximately 1000Hz processing rate
                thread::sleep(Duration::from_millis(1));
            }
        });

        self.thread_handle = Some(handle);
        Ok(())
    }

    /// Add a MIDI output device connection
    pub fn add_output(&mut self, device: &MidiDevice) -> Result<(), String> {
        let midi_out = MidiOutput::new("snap-blaster")
            .map_err(|e| format!("Failed to create MIDI output: {}", e))?;

        // Find the port by name
        let ports = midi_out.ports();
        let port = ports.iter()
            .find(|p| {
                midi_out.port_name(p).map(|name| name == device.name).unwrap_or(false)
            })
            .ok_or_else(|| format!("Could not find MIDI device: {}", device.name))?;

        // Connect to the port
        let connection = midi_out.connect(port, "midi-connection")
            .map_err(|e| format!("Failed to connect to MIDI port: {}", e))?;
        self.connections.push(connection);

        Ok(())
    }

    /// Send a command to the MIDI engine
    pub fn send_command(&self, command: MidiCommand) -> Result<(), String> {
        let mut queue = self.command_queue.lock().unwrap();
        queue.push_back(command);
        Ok(())
    }

    /// Process a MIDI CC message
    fn send_cc(&mut self, channel: u8, cc: u8, value: u8) {
        // MIDI CC message format: 0xB0 + channel, cc number, value
        let status_byte = 0xB0 + (channel & 0x0F);

        for connection in &mut self.connections {
            let _ = connection.send(&[status_byte, cc, value]);
        }
    }

    /// Shutdown the MIDI engine
    pub fn shutdown(&mut self) -> Result<(), String> {
        // Set running flag to false
        {
            let mut running = self.running.lock().unwrap();
            *running = false;
        }

        // Wait for thread to finish
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }

        Ok(())
    }
}

impl Drop for MidiEngine {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}