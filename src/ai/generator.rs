use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use uuid::Uuid;

use crate::models::project::Project;
use crate::models::scene::Scene;
use crate::models::cc::CCValue;
use crate::models::cc::TransitionCurve;

/// Parameters for AI scene generation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenerationParams {
    /// Description of the desired scene
    pub description: String,

    /// Base CC definitions to use
    pub cc_definitions: Vec<CCDefinitionRef>,

    /// Optional previous scene to reference
    pub previous_scene: Option<SceneRef>,

    /// Whether to add transitions
    pub use_transitions: bool,

    /// Whether to generate random variation
    pub add_randomness: bool,

    /// Tags to influence generation
    pub tags: Vec<String>,
}

/// Reference to a CC definition
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CCDefinitionRef {
    pub channel: u8,
    pub cc_number: u8,
    pub name: String,
    pub description: Option<String>,
}

/// Reference to a scene
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SceneRef {
    pub id: String,
    pub name: String,
    pub cc_values: HashMap<String, CCValue>,
}

/// AI-generated scene with metadata
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GeneratedScene {
    /// The generated scene
    pub scene: Scene,

    /// Explanation of the generation choices
    pub explanation: String,
}

/// AI scene generator
pub struct SceneGenerator {
    // In a real implementation, this would likely hold LLM client
    // or other AI components
    api_key: Option<String>,
    model: String,
}

impl SceneGenerator {
    /// Create a new scene generator
    pub fn new(api_key: Option<String>) -> Self {
        SceneGenerator {
            api_key,
            model: "internal".to_string(), // Placeholder for real model
        }
    }

    /// Set the model to use for generation
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    /// Generate a scene based on a description and available CC definitions
    pub fn generate(&self, params: GenerationParams) -> Result<GeneratedScene, String> {
        // In a real implementation, this would call out to an LLM API
        // For now, we'll use a simple rule-based approach

        let mut scene = Scene::new(
            &Uuid::new_v4().to_string(),
            &format!("Generated: {}", self.get_name_from_description(&params.description)),
        );

        scene.description = Some(params.description.clone());
        scene.tags = params.tags.clone();

        // Generate CC values based on the description
        for cc_def in &params.cc_definitions {
            // For demo purposes, generate values using pseudo-AI rules
            if let Some(value) = self.generate_cc_value_for_def(cc_def, &params) {
                scene.add_cc(value);
            }
        }

        // Add transitions if requested
        if params.use_transitions {
            self.add_transitions(&mut scene, &params);
        }

        // Generate an explanation
        let explanation = self.generate_explanation(&scene, &params);

        Ok(GeneratedScene {
            scene,
            explanation,
        })
    }

    /// Extract a short name from a description
    fn get_name_from_description(&self, description: &str) -> String {
        let words: Vec<&str> = description.split_whitespace().collect();

        if words.len() <= 3 {
            description.to_string()
        } else {
            // Extract a subset of significant words
            let mut name_words = Vec::new();

            // Extract adjectives and nouns (simplified approach)
            for word in words.iter().take(8) {
                if word.len() >= 4 {
                    name_words.push(*word);
                    if name_words.len() >= 3 {
                        break;
                    }
                }
            }

            if name_words.is_empty() {
                // Fall back to first few words if no significant words found
                words.iter().take(3).map(|w| *w).collect::<Vec<&str>>().join(" ")
            } else {
                name_words.join(" ")
            }
        }
    }

    /// Generate a CC value based on a definition and params
    fn generate_cc_value_for_def(&self, cc_def: &CCDefinitionRef, params: &GenerationParams) -> Option<CCValue> {
        let mut value = 0;
        let description = params.description.to_lowercase();

        // Simple keyword-based rules - in a real implementation this would
        // use an LLM to interpret the description more intelligently

        // For volume (CC 7)
        if cc_def.cc_number == 7 {
            if cc_def.name.to_lowercase().contains("volume") {
                if description.contains("loud") || description.contains("intense") {
                    value = 100;
                } else if description.contains("quiet") || description.contains("soft") {
                    value = 60;
                } else if description.contains("silent") || description.contains("mute") {
                    value = 0;
                } else {
                    value = 80; // Default volume
                }
            }
        }

        // For pan (CC 10)
        else if cc_def.cc_number == 10 {
            if cc_def.name.to_lowercase().contains("pan") {
                if description.contains("left") {
                    value = 30;
                } else if description.contains("right") {
                    value = 90;
                } else if description.contains("center") || description.contains("middle") {
                    value = 64;
                } else if description.contains("wide") || description.contains("stereo") {
                    // For "wide" concepts, we'll just use center
                    value = 64;
                } else {
                    value = 64; // Default center
                }
            }
        }

        // For filter cutoff (CC 74)
        else if cc_def.cc_number == 74 {
            if cc_def.name.to_lowercase().contains("filter") ||
                cc_def.name.to_lowercase().contains("cutoff") {
                if description.contains("bright") || description.contains("open") ||
                    description.contains("clear") {
                    value = 120;
                } else if description.contains("dark") || description.contains("muffled") ||
                    description.contains("filtered") {
                    value = 40;
                } else if description.contains("warm") {
                    value = 80;
                } else {
                    value = 100; // Default
                }
            }
        }

        // For resonance (CC 71)
        else if cc_def.cc_number == 71 {
            if cc_def.name.to_lowercase().contains("res") ||
                cc_def.name.to_lowercase().contains("resonance") {
                if description.contains("resonant") || description.contains("harsh") ||
                    description.contains("squelch") {
                    value = 100;
                } else if description.contains("smooth") || description.contains("soft") {
                    value = 20;
                } else {
                    value = 40; // Default
                }
            }
        }

        // For other CCs, use a more generic approach
        else {
            // Map the description sentiment to a value
            let sentiment_value = self.analyze_sentiment_for_cc(&description, cc_def);

            // Add randomness if requested
            if params.add_randomness {
                let random_factor = (std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .subsec_nanos() % 20) as i32 - 10; // -10 to +10

                value = ((sentiment_value as i32 + random_factor).max(0).min(127)) as u8;
            } else {
                value = sentiment_value;
            }
        }

        // Create the CC value
        let mut cc = CCValue::new(cc_def.channel, cc_def.cc_number, value);
        cc.name = Some(cc_def.name.clone());
        cc.description = cc_def.description.clone();

        Some(cc)
    }

    /// Simple sentiment analysis for CC values
    fn analyze_sentiment_for_cc(&self, description: &str, cc_def: &CCDefinitionRef) -> u8 {
        // This is a very simplistic approach - a real implementation would use
        // more sophisticated NLP or an LLM

        // Positive words increase the value
        let positive_words = [
            "high", "open", "bright", "intense", "maximum", "loud", "strong",
            "fast", "quick", "energetic", "vibrant", "rich", "full"
        ];

        // Negative words decrease the value
        let negative_words = [
            "low", "closed", "dark", "subtle", "minimum", "quiet", "weak",
            "slow", "gentle", "calm", "soft", "thin", "empty"
        ];

        // Calculate a base value
        let mut sentiment_score = 0;

        for word in positive_words.iter() {
            if description.contains(word) {
                sentiment_score += 1;
            }
        }

        for word in negative_words.iter() {
            if description.contains(word) {
                sentiment_score -= 1;
            }
        }

        // Convert to a value between 0-127
        let base_value = 64; // Neutral middle point
        let sentiment_range = 40; // How much the sentiment can shift the value

        let value = (base_value as i32 + sentiment_score * sentiment_range / 2)
            .max(0)
            .min(127) as u8;

        value
    }

    /// Add transitions to a scene based on description
    fn add_transitions(&self, scene: &mut Scene, params: &GenerationParams) {
        if !params.use_transitions {
            return;
        }

        let description = params.description.to_lowercase();

        // Choose appropriate transition curves based on description
        let curve = if description.contains("sudden") || description.contains("abrupt") {
            TransitionCurve::Exponential
        } else if description.contains("gentle") || description.contains("smooth") {
            TransitionCurve::SCurve
        } else if description.contains("gradual") || description.contains("slow") {
            TransitionCurve::Logarithmic
        } else {
            TransitionCurve::Linear
        };

        // Apply transitions to CC values that typically benefit from them
        for cc_value in scene.cc_values.values_mut() {
            let name = cc_value.name.as_deref().unwrap_or("").to_lowercase();

            // Common parameters that benefit from transitions
            if name.contains("filter") || name.contains("volume") ||
                name.contains("pan") || name.contains("pitch") {
                cc_value.transition = true;
                cc_value.curve = curve;

                // Set transition time based on description
                if description.contains("fast") {
                    cc_value.transition_beats = Some(0.5);
                } else if description.contains("slow") {
                    cc_value.transition_beats = Some(4.0);
                } else {
                    cc_value.transition_beats = Some(1.0);
                }
            }
        }
    }

    /// Generate an explanation for the scene
    fn generate_explanation(&self, scene: &Scene, params: &GenerationParams) -> String {
        // In a real implementation, this would use an LLM to generate
        // a more natural and detailed explanation

        let mut explanation = format!(
            "I created a scene based on the description: \"{}\".\n\n",
            params.description
        );

        explanation.push_str("Here's what I did:\n");

        // Explain CC value choices
        for cc in scene.cc_values_vec() {
            let name = cc.name.as_deref().unwrap_or("unnamed");

            explanation.push_str(&format!(
                "- Set {} to {} because ",
                name,
                cc.value
            ));

            // Simple explanation based on value range
            if cc.value > 100 {
                explanation.push_str("it needed to be maximized for this scene.\n");
            } else if cc.value > 80 {
                explanation.push_str("a high value seemed appropriate.\n");
            } else if cc.value > 60 {
                explanation.push_str("a moderate-high value worked best.\n");
            } else if cc.value > 40 {
                explanation.push_str("a moderate value was most fitting.\n");
            } else if cc.value > 20 {
                explanation.push_str("a lower value matched the description.\n");
            } else {
                explanation.push_str("a minimal value was appropriate.\n");
            }

            // Add transition explanation if applicable
            if cc.transition {
                explanation.push_str(&format!(
                    "  - Added a {} transition over {} beats.\n",
                    match cc.curve {
                        TransitionCurve::Linear => "linear",
                        TransitionCurve::Exponential => "exponential",
                        TransitionCurve::Logarithmic => "logarithmic",
                        TransitionCurve::SCurve => "smooth S-curve",
                    },
                    cc.transition_beats.unwrap_or(1.0)
                ));
            }
        }

        explanation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name_extraction() {
        let generator = SceneGenerator::new(None);

        assert_eq!(
            generator.get_name_from_description("Bright and airy atmosphere"),
            "Bright airy atmosphere"
        );

        assert_eq!(
            generator.get_name_from_description("Test"),
            "Test"
        );
    }

    #[test]
    fn test_generate_basic_scene() {
        let generator = SceneGenerator::new(None);

        let cc_defs = vec![
            CCDefinitionRef {
                channel: 0,
                cc_number: 7,
                name: "Volume".to_string(),
                description: None,
            },
            CCDefinitionRef {
                channel: 0,
                cc_number: 10,
                name: "Pan".to_string(),
                description: None,
            },
        ];

        let params = GenerationParams {
            description: "Loud sound panned to the left".to_string(),
            cc_definitions: cc_defs,
            previous_scene: None,
            use_transitions: false,
            add_randomness: false,
            tags: vec![],
        };

        let result = generator.generate(params).unwrap();

        // Check for expected values based on the description
        let volume = result.scene.get_cc(0, 7).unwrap();
        let pan = result.scene.get_cc(0, 10).unwrap();

        assert!(volume.value > 90, "Volume should be high for 'loud'");
        assert!(pan.value < 50, "Pan should be to the left");
    }
}