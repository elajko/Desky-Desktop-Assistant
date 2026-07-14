use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PersonaTraits {
    pub formality: f32,
    pub humor: f32,
    pub verbosity: f32,
    pub proactivity: f32,
}

impl Default for PersonaTraits {
    fn default() -> Self {
        Self {
            formality: 0.5,
            humor: 0.5,
            verbosity: 0.5,
            proactivity: 0.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    pub id: String,
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub traits: PersonaTraits,
    /// Which avatar sprite sheet this persona uses. The avatar system
    /// doesn't exist yet — nothing reads this field today — but personas
    /// are meant to own their look once it does, so the schema carries it
    /// now rather than bolting it on later.
    #[serde(default)]
    pub sprite_sheet: Option<String>,
    /// True for personas that shipped as bundled presets (shows a "reset to
    /// default" option in the UI). Built-ins are still fully editable and
    /// deletable like any other persona — this only gates that one action.
    pub is_builtin: bool,
}

// Applies regardless of persona customization: the model can look things up
// and preview a file-organizing plan, but it can never delete files, and
// file moves only happen after the user explicitly confirms in the app UI.
const SAFETY_FOOTER: &str = "You can call tools to inspect the system and preview file \
    organization, but you can never delete files, and file moves only happen after the \
    user explicitly confirms in the app UI — you cannot apply them yourself.";

impl Persona {
    /// Builds the actual system prompt sent to the model: the persona's own
    /// prompt, plus natural-language modifiers derived from its trait
    /// sliders, plus the fixed safety footer every persona carries.
    pub fn compose_system_prompt(&self) -> String {
        let mut modifiers: Vec<&str> = Vec::new();

        if self.traits.formality > 0.65 {
            modifiers.push("Speak formally and professionally.");
        } else if self.traits.formality < 0.35 {
            modifiers.push("Speak casually, like a friend.");
        }

        if self.traits.humor > 0.65 {
            modifiers.push("Feel free to use humor and playful phrasing.");
        } else if self.traits.humor < 0.35 {
            modifiers.push("Keep a serious, matter-of-fact tone.");
        }

        if self.traits.verbosity < 0.35 {
            modifiers.push("Keep responses brief — a sentence or two unless asked for detail.");
        } else if self.traits.verbosity > 0.65 {
            modifiers.push("Feel free to elaborate and give thorough, detailed answers.");
        }

        if self.traits.proactivity > 0.65 {
            modifiers.push("Proactively suggest relevant follow-up actions or information.");
        } else if self.traits.proactivity < 0.35 {
            modifiers.push("Answer only what's asked — don't volunteer extra suggestions.");
        }

        let mut prompt = self.system_prompt.trim().to_string();
        if !modifiers.is_empty() {
            prompt.push(' ');
            prompt.push_str(&modifiers.join(" "));
        }
        prompt.push(' ');
        prompt.push_str(SAFETY_FOOTER);
        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn persona_with_traits(traits: PersonaTraits) -> Persona {
        Persona {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: String::new(),
            system_prompt: "You are Desky.".to_string(),
            traits,
            sprite_sheet: None,
            is_builtin: false,
        }
    }

    #[test]
    fn high_humor_low_verbosity_adds_expected_modifiers() {
        let persona = persona_with_traits(PersonaTraits {
            formality: 0.5,
            humor: 0.9,
            verbosity: 0.1,
            proactivity: 0.5,
        });
        let prompt = persona.compose_system_prompt();

        assert!(prompt.contains("humor"), "expected a humor modifier: {prompt}");
        assert!(prompt.contains("brief"), "expected a brevity modifier: {prompt}");
        assert!(
            prompt.contains("never delete files"),
            "expected the safety footer: {prompt}"
        );
    }

    #[test]
    fn different_traits_produce_different_prompts() {
        let concise = persona_with_traits(PersonaTraits {
            formality: 0.5,
            humor: 0.1,
            verbosity: 0.1,
            proactivity: 0.3,
        });
        let snarky = persona_with_traits(PersonaTraits {
            formality: 0.1,
            humor: 0.9,
            verbosity: 0.4,
            proactivity: 0.4,
        });

        assert_ne!(concise.compose_system_prompt(), snarky.compose_system_prompt());
    }

    #[test]
    fn neutral_traits_add_no_modifiers() {
        let persona = persona_with_traits(PersonaTraits::default());
        let prompt = persona.compose_system_prompt();
        // Just the base prompt + safety footer, no trait modifier sentences.
        assert_eq!(prompt, format!("{} {}", persona.system_prompt, SAFETY_FOOTER));
    }
}
