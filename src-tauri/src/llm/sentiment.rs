use super::client::{self, ChatMessage};
use crate::persona::Persona;

/// Whether an incoming message aligned with what the active persona likes,
/// dislikes, or neither — drives the love meter and the reply bubble's
/// styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sentiment {
    Liked,
    Disliked,
    Neutral,
}

impl Sentiment {
    /// How much this turn moves the (unbounded, per-persona) love meter.
    pub fn delta(self) -> i32 {
        match self {
            Sentiment::Liked => 1,
            Sentiment::Disliked => -1,
            Sentiment::Neutral => 0,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Sentiment::Liked => "liked",
            Sentiment::Disliked => "disliked",
            Sentiment::Neutral => "neutral",
        }
    }
}

/// Asks the model whether `message` aligns with what `persona` likes or
/// dislikes (both free-text, configured in the Persona editor). Skips the
/// LLM call entirely — always Neutral — if neither field is configured, so
/// personas without a love-meter setup pay no extra latency for this.
pub async fn classify_message(
    port: u16,
    persona: &Persona,
    message: &str,
) -> anyhow::Result<Sentiment> {
    let likes = persona.likes.trim();
    let dislikes = persona.dislikes.trim();
    if likes.is_empty() && dislikes.is_empty() {
        return Ok(Sentiment::Neutral);
    }

    let mut instructions = format!(
        "You are judging whether a single message aligns with what {} likes or dislikes, \
         for an affection meter in a chat app. ",
        persona.name
    );
    if !likes.is_empty() {
        instructions.push_str(&format!(
            "{} responds positively to: {}. ",
            persona.name, likes
        ));
    }
    if !dislikes.is_empty() {
        instructions.push_str(&format!(
            "{} responds negatively to: {}. ",
            persona.name, dislikes
        ));
    }
    instructions.push_str(
        "Read the user's message below and decide: does it align with what they like, what \
         they dislike, or neither? Respond with exactly one word: LIKE, DISLIKE, or NEUTRAL \
         — nothing else.",
    );

    let messages = vec![
        ChatMessage::system(instructions),
        ChatMessage::user(message.to_string()),
    ];
    let reply = client::stream_chat(port, &messages, |_| {}).await?;
    let upper = reply.to_uppercase();

    // Check DISLIKE first since it contains "LIKE" as a substring.
    Ok(if upper.contains("DISLIKE") {
        Sentiment::Disliked
    } else if upper.contains("LIKE") {
        Sentiment::Liked
    } else {
        Sentiment::Neutral
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Settings;
    use crate::llm::process::LlmProcess;
    use crate::persona::schema::PersonaTraits;
    use std::path::PathBuf;

    fn test_persona(likes: &str, dislikes: &str) -> Persona {
        Persona {
            id: "test".to_string(),
            name: "TestBot".to_string(),
            description: String::new(),
            system_prompt: "You are TestBot.".to_string(),
            traits: PersonaTraits::default(),
            sprite_sheet: None,
            is_builtin: false,
            likes: likes.to_string(),
            dislikes: dislikes.to_string(),
            love: 0,
        }
    }

    #[test]
    fn skips_llm_call_when_unconfigured() {
        // No async runtime needed at all if this really short-circuits —
        // proven by the fact this plain #[test] (not #[tokio::test]) can
        // still .await it via a minimal blocking runtime.
        let persona = test_persona("", "");
        let result = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap()
            .block_on(classify_message(0, &persona, "anything at all"));
        assert_eq!(result.unwrap(), Sentiment::Neutral);
    }

    /// Not a CI test — exercises real classification against a real
    /// llama-server + model.
    /// Run with: cargo test --lib -- --ignored --nocapture classifies_liked_and_disliked_messages
    #[ignore]
    #[tokio::test]
    async fn classifies_liked_and_disliked_messages() {
        let settings = Settings {
            llama_server_path: Some(PathBuf::from(
                "/home/erik/llama.cpp/build/bin/llama-server",
            )),
            model_path: Some(PathBuf::from(
                "/home/erik/models/qwen2.5-1.5b-instruct-q4_k_m.gguf",
            )),
            port: 8094,
            context_size: 4096,
            active_persona_id: None,
        };

        let mut llm = LlmProcess::default();
        let port = llm
            .ensure_running(&settings)
            .await
            .expect("llama-server should start");

        let persona = test_persona(
            "compliments, kindness, and being told they're a good assistant",
            "insults, rudeness, and being told they're useless",
        );

        let liked = classify_message(port, &persona, "You're such a great assistant, thank you!")
            .await
            .expect("classification should succeed");
        println!("liked-leaning message classified as: {liked:?}");

        let disliked = classify_message(port, &persona, "You're useless and I hate talking to you.")
            .await
            .expect("classification should succeed");
        println!("disliked-leaning message classified as: {disliked:?}");

        llm.shutdown().await;
    }
}
