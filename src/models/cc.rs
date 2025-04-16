use std::time::Duration;
use serde::{Serialize, Deserialize};

/// Types of transition curves for CC value changes
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum TransitionCurve {
    Linear,
    Exponential,
    Logarithmic,
    SCurve,
}

impl Default for TransitionCurve {
    fn default() -> Self {
        TransitionCurve::Linear
    }
}

/// A single MIDI CC value with optional transition information
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CCValue {
    /// MIDI channel (0-15)
    pub channel: u8,

    /// CC number (0-127)
    pub cc_number: u8,

    /// Current/target value (0-127)
    pub value: u8,

    /// Optional friendly name for this CC
    #[serde(default)]
    pub name: Option<String>,

    /// Whether this CC should transition to its value
    #[serde(default)]
    pub transition: bool,

    /// Transition duration in beats (only used if transition is true)
    #[serde(default)]
    pub transition_beats: Option<f32>,

    /// Transition duration in milliseconds (only used if transition is true and transition_beats is None)
    #[serde(default)]
    pub transition_ms: Option<u32>,

    /// Curve for the transition
    #[serde(default)]
    pub curve: TransitionCurve,

    /// Optional description for AI-assisted generation
    #[serde(default)]
    pub description: Option<String>,
}

impl CCValue {
    /// Create a new CC value with default settings
    pub fn new(channel: u8, cc_number: u8, value: u8) -> Self {
        CCValue {
            channel,
            cc_number,
            value,
            name: None,
            transition: false,
            transition_beats: None,
            transition_ms: None,
            curve: TransitionCurve::default(),
            description: None,
        }
    }

    /// Get a copy of this CC value with a specific value
    pub fn with_value(&self, value: u8) -> Self {
        let mut copy = self.clone();
        copy.value = value;
        copy
    }

    /// Get the transition duration in milliseconds given a tempo
    pub fn get_transition_duration_ms(&self, tempo: f64) -> Option<u32> {
        if !self.transition {
            return None;
        }

        if let Some(ms) = self.transition_ms {
            Some(ms)
        } else if let Some(beats) = self.transition_beats {
            // Convert beats to milliseconds using the tempo
            let beat_duration_ms = (60.0 / tempo) * 1000.0;
            Some((beats as f64 * beat_duration_ms) as u32)
        } else {
            None
        }
    }

    /// Set transition in beats
    pub fn with_transition_beats(mut self, beats: f32, curve: TransitionCurve) -> Self {
        self.transition = true;
        self.transition_beats = Some(beats);
        self.transition_ms = None;
        self.curve = curve;
        self
    }

    /// Set transition in milliseconds
    pub fn with_transition_ms(mut self, ms: u32, curve: TransitionCurve) -> Self {
        self.transition = true;
        self.transition_ms = Some(ms);
        self.transition_beats = None;
        self.curve = curve;
        self
    }

    /// Add a name and description
    pub fn with_metadata(mut self, name: &str, description: Option<&str>) -> Self {
        self.name = Some(name.to_string());
        if let Some(desc) = description {
            self.description = Some(desc.to_string());
        }
        self
    }
}

/// Tests for CCValue
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cc_value_creation() {
        let cc = CCValue::new(0, 1, 64);
        assert_eq!(cc.channel, 0);
        assert_eq!(cc.cc_number, 1);
        assert_eq!(cc.value, 64);
        assert_eq!(cc.transition, false);
    }

    #[test]
    fn test_cc_transition_duration() {
        let mut cc = CCValue::new(0, 1, 64);
        assert_eq!(cc.get_transition_duration_ms(120.0), None);

        cc.transition = true;
        cc.transition_ms = Some(1000);
        assert_eq!(cc.get_transition_duration_ms(120.0), Some(1000));

        cc.transition_ms = None;
        cc.transition_beats = Some(4.0);
        assert_eq!(cc.get_transition_duration_ms(120.0), Some(2000));

        // Different tempo
        assert_eq!(cc.get_transition_duration_ms(60.0), Some(4000));
    }
}