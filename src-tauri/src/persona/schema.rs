use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    pub id: String,
    pub name: String,
    pub description: String,
    pub system_prompt: String,
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
    /// The love meter — unbounded in both directions, +1/-1 per classified
    /// message (liked/disliked), unchanged on neutral. Persisted per-persona
    /// so each character keeps its own running affection score.
    #[serde(default)]
    pub love: i32,
    /// A short sample of how this persona actually talks — shown to the
    /// model as a style reference, and also the source `llm::sentiment::
    /// classify_message` reasons from to judge each incoming message for
    /// the love meter. A good example should show the character reacting
    /// to something they like or dislike, not just neutral small talk.
    /// Empty means "don't judge" for the love meter (see that module).
    #[serde(default)]
    pub example_dialogue: String,
    /// What the persona says first, before the user sends anything —
    /// seeded directly into the conversation rather than generated.
    #[serde(default)]
    pub first_message: String,
}

impl Persona {
    /// Builds the actual system prompt sent to the model: the persona's own
    /// prompt, plus its example dialogue (if any) as a style reference.
    pub fn compose_system_prompt(&self) -> String {
        let mut prompt = self.system_prompt.trim().to_string();

        let example = self.example_dialogue.trim();
        if !example.is_empty() {
            prompt.push_str("\n\nHere's an example of how you talk:\n");
            prompt.push_str(example);
            prompt.push_str("\n\nMatch this voice and style.");
        }

        prompt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_persona(system_prompt: &str, example_dialogue: &str) -> Persona {
        Persona {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: String::new(),
            system_prompt: system_prompt.to_string(),
            sprite_sheet: None,
            is_builtin: false,
            love: 0,
            example_dialogue: example_dialogue.to_string(),
            first_message: String::new(),
        }
    }

    #[test]
    fn no_example_dialogue_is_just_the_base_prompt() {
        let persona = test_persona("You are Test.", "");
        assert_eq!(persona.compose_system_prompt(), "You are Test.");
    }

    #[test]
    fn example_dialogue_gets_folded_in_with_instruction() {
        let persona = test_persona("You are Test.", "User: hi\nTest: heya!");
        let prompt = persona.compose_system_prompt();

        assert!(prompt.contains("You are Test."));
        assert!(prompt.contains("User: hi\nTest: heya!"));
        assert!(prompt.contains("Match this voice and style"));
    }

    #[test]
    fn different_example_dialogue_produces_different_prompts() {
        let a = test_persona("You are Test.", "Test: sup.");
        let b = test_persona("You are Test.", "Test: Good day to you, friend.");
        assert_ne!(a.compose_system_prompt(), b.compose_system_prompt());
    }
}
