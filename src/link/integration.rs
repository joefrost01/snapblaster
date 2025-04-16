use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use rusty_link::SessionState;

/// Ableton Link integration
///
/// This provides synchronization with other music applications
/// using the Ableton Link protocol
pub struct LinkIntegration {
    // The underlying Ableton Link instance
    link: rusty_link::AblLink,
    // Whether Link is currently enabled
    enabled: Arc<Mutex<bool>>,
    // Current tempo in BPM
    tempo: Arc<Mutex<f64>>,
    // Current beat position
    beat_position: Arc<Mutex<f64>>,
    // Callback to invoke on beat events
    beat_callback: Arc<Mutex<Option<Box<dyn Fn(u32) + Send>>>>,
    // Callback to invoke on bar events (assuming 4/4 time signature)
    bar_callback: Arc<Mutex<Option<Box<dyn Fn(u32) + Send>>>>,
    // Thread handle for the timing thread
    thread_handle: Arc<Mutex<Option<thread::JoinHandle<()>>>>,
}

impl LinkIntegration {
    /// Create a new Link integration
    pub fn new(initial_tempo: f64) -> Self {
        // Create the Link instance with initial tempo
        let link = rusty_link::AblLink::new(initial_tempo);

        // Start with Link disabled
        link.enable(false);

        LinkIntegration {
            link,
            enabled: Arc::new(Mutex::new(false)),
            tempo: Arc::new(Mutex::new(initial_tempo)),
            beat_position: Arc::new(Mutex::new(0.0)),
            beat_callback: Arc::new(Mutex::new(None)),
            bar_callback: Arc::new(Mutex::new(None)),
            thread_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// Enable or disable Link synchronization
    pub fn enable(&mut self, enable: bool) {
        let mut enabled = self.enabled.lock().unwrap();
        *enabled = enable;
        self.link.enable(enable);
    }

    /// Check if Link is enabled
    pub fn is_enabled(&self) -> bool {
        *self.enabled.lock().unwrap()
    }

    /// Get the current tempo
    pub fn get_tempo(&self) -> f64 {
        if self.is_enabled() {
            let mut session_state = SessionState::new();
            self.link.capture_app_session_state(&mut session_state);
            let tempo = session_state.tempo();
            *self.tempo.lock().unwrap() = tempo;
            tempo
        } else {
            *self.tempo.lock().unwrap()
        }
    }

    /// Set the tempo (only effective if Link is not connected to peers)
    pub fn set_tempo(&mut self, tempo: f64) {
        let mut session_state = SessionState::new();
        self.link.capture_app_session_state(&mut session_state);
        session_state.set_tempo(tempo, self.link.clock_micros());
        self.link.commit_app_session_state(&session_state);

        if !self.link.num_peers() > 0 {
            let mut current_tempo = self.tempo.lock().unwrap();
            *current_tempo = tempo;
        }
    }

    /// Get the current beat position
    pub fn get_beat_position(&self) -> f64 {
        if self.is_enabled() {
            let mut session_state = SessionState::new();
            self.link.capture_app_session_state(&mut session_state);
            let beat = session_state.beat_at_time(self.link.clock_micros(), 4.0);
            *self.beat_position.lock().unwrap() = beat;
            beat
        } else {
            *self.beat_position.lock().unwrap()
        }
    }

    /// Get the phase position within the current bar (0.0 to 1.0)
    pub fn get_phase(&self) -> f64 {
        let beat = self.get_beat_position();

        // Assuming 4/4 time signature
        let phase = (beat % 4.0) / 4.0;
        phase
    }

    /// Start the Link timing thread
    // In link/integration.rs, modify the start method to not clone AblLink directly
    pub fn start(&mut self) {
        // Stop any existing thread
        self.stop();

        let enabled = Arc::clone(&self.enabled);
        let beat_position = Arc::clone(&self.beat_position);
        let beat_callback = Arc::clone(&self.beat_callback);
        let bar_callback = Arc::clone(&self.bar_callback);

        // Instead of cloning the Link instance, create an Arc to share it
        use std::sync::Mutex;
        let link = Arc::new(Mutex::new(rusty_link::AblLink::new(self.get_tempo())));
        let link_clone = Arc::clone(&link);

        let handle = thread::spawn(move || {
            let mut last_beat = 0;
            let mut last_bar = 0;

            while *enabled.lock().unwrap() {
                if link_clone.lock().unwrap().num_peers() > 0 {
                    let mut session_state = SessionState::new();
                    link_clone.lock().unwrap().capture_app_session_state(&mut session_state);

                    // Get the current beat position
                    let beat = session_state.beat_at_time(link_clone.lock().unwrap().clock_micros(), 4.0);

                    // Rest of the thread function remains the same,
                    // just replace any reference to 'link' with 'link_clone'

                    // Update the shared beat position
                    *beat_position.lock().unwrap() = beat;

                    // Calculate the current beat and bar
                    let current_beat = beat.floor() as u32;
                    let current_bar = (current_beat / 4) as u32;

                    // If we've moved to a new beat, trigger the beat callback
                    if current_beat != last_beat {
                        if let Some(callback) = &*beat_callback.lock().unwrap() {
                            callback(current_beat % 4);
                        }

                        last_beat = current_beat;
                    }

                    // If we've moved to a new bar, trigger the bar callback
                    if current_bar != last_bar {
                        if let Some(callback) = &*bar_callback.lock().unwrap() {
                            callback(current_bar);
                        }

                        last_bar = current_bar;
                    }
                }

                // Sleep for a short time to avoid using too much CPU
                thread::sleep(Duration::from_millis(10));
            }
        });

        let mut thread_handle = self.thread_handle.lock().unwrap();
        *thread_handle = Some(handle);
    }

    /// Stop the Link timing thread
    pub fn stop(&mut self) {
        let mut thread_handle = self.thread_handle.lock().unwrap();

        if let Some(handle) = thread_handle.take() {
            // Set enabled to false to stop the thread
            {
                let mut enabled = self.enabled.lock().unwrap();
                *enabled = false;
            }

            // Wait for the thread to finish
            if handle.join().is_err() {
                // Handle error if thread panicked
                println!("Link timing thread panicked");
            }
        }
    }

    /// Set the callback to invoke on beat boundaries
    pub fn set_beat_callback<F>(&mut self, callback: F)
    where
        F: Fn(u32) + Send + 'static,
    {
        let mut beat_callback = self.beat_callback.lock().unwrap();
        *beat_callback = Some(Box::new(callback));
    }

    /// Set the callback to invoke on bar boundaries
    pub fn set_bar_callback<F>(&mut self, callback: F)
    where
        F: Fn(u32) + Send + 'static,
    {
        let mut bar_callback = self.bar_callback.lock().unwrap();
        *bar_callback = Some(Box::new(callback));
    }

    /// Forcefully reset the beat position to a specific time
    pub fn reset_beat_position(&mut self, beat: f64) {
        let mut session_state = SessionState::new();
        self.link.capture_app_session_state(&mut session_state);
        let time = self.link.clock_micros();
        session_state.request_beat_at_time(beat, time, 4.0);
        self.link.commit_app_session_state(&session_state);

        let mut beat_position = self.beat_position.lock().unwrap();
        *beat_position = beat;
    }

    /// Get the number of peers connected
    pub fn num_peers(&self) -> u32 {
        self.link.num_peers() as u32
    }

    /// Check if connected to any peers
    pub fn is_connected_to_peers(&self) -> bool {
        self.link.num_peers() > 0
    }

    /// Start playback at the next quantized boundary
    pub fn start_at_next_quantized_boundary(&mut self, quantum: f64) {
        let mut session_state = SessionState::new();
        self.link.capture_app_session_state(&mut session_state);

        // Get the current beat position
        let now_micros = self.link.clock_micros();
        let current_beat = session_state.beat_at_time(now_micros, quantum);

        // Calculate the next quantum boundary
        let next_boundary = (current_beat / quantum).ceil() * quantum;

        // Set the next boundary as the new beat position
        session_state.request_beat_at_time(next_boundary, now_micros, quantum);
        self.link.commit_app_session_state(&session_state);
    }
}

impl Drop for LinkIntegration {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests depend on the rusty_link crate
    // and may require a real Link session to fully test

    #[test]
    fn test_link_creation() {
        let link = LinkIntegration::new(120.0);

        assert_eq!(link.get_tempo(), 120.0);
        assert_eq!(link.is_enabled(), false);
    }

    #[test]
    fn test_enable_disable() {
        let mut link = LinkIntegration::new(120.0);

        link.enable(true);
        assert_eq!(link.is_enabled(), true);

        link.enable(false);
        assert_eq!(link.is_enabled(), false);
    }

    #[test]
    fn test_tempo_changes() {
        let mut link = LinkIntegration::new(120.0);

        link.set_tempo(140.0);
        assert_eq!(link.get_tempo(), 140.0);
    }
}