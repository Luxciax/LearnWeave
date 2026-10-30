use serde::{Deserialize, Serialize};

use crate::domain::provider::GenerationOptions;

use super::ProviderAdapter;

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ModelCapabilities {
    pub system_prompt: bool,
    pub temperature: bool,
    pub top_p: bool,
    pub max_output_tokens: bool,
    pub reasoning: bool,
    pub json_mode: bool,
    pub json_schema: bool,
    pub streaming: bool,
    pub vision: bool,
    pub context_window: Option<u32>,
}

impl ModelCapabilities {
    const fn all(reasoning: bool) -> Self {
        Self {
            system_prompt: true,
            temperature: true,
            top_p: true,
            max_output_tokens: true,
            reasoning,
            json_mode: true,
            json_schema: false,
            streaming: true,
            vision: false,
            context_window: None,
        }
    }
}

pub fn infer(adapter: ProviderAdapter, model: &str) -> ModelCapabilities {
    let model = model.trim().to_ascii_lowercase();
    match adapter {
        ProviderAdapter::OpenAiResponses => {
            let reasoning = is_openai_reasoning_model(&model);
            ModelCapabilities {
                system_prompt: true,
                temperature: !reasoning,
                top_p: !reasoning,
                max_output_tokens: true,
                reasoning,
                json_mode: true,
                json_schema: true,
                streaming: true,
                vision: model.contains("vision") || model.contains("gpt-4o") || model.contains("gpt-5"),
                context_window: None,
            }
        }
        ProviderAdapter::OpenAiCompatible => {
            let mut caps = ModelCapabilities::all(is_openai_reasoning_model(&model));
            caps.vision = model.contains("vision") || model.contains("vl") || model.contains("gpt-4o");
            caps
        }
        ProviderAdapter::Anthropic => {
            let mut caps = ModelCapabilities::all(is_anthropic_reasoning_model(&model));
            caps.json_mode = false;
            caps.json_schema = false;
            caps.vision = model.contains("claude");
            caps
        }
        ProviderAdapter::Gemini => {
            let mut caps = ModelCapabilities::all(is_gemini_reasoning_model(&model));
            caps.json_schema = true;
            caps.vision = true;
            caps
        },
    }
}

pub fn constrain_options(
    options: &GenerationOptions,
    capabilities: ModelCapabilities,
) -> GenerationOptions {
    GenerationOptions {
        temperature: capabilities.temperature.then_some(options.temperature).flatten(),
        max_output_tokens: capabilities
            .max_output_tokens
            .then_some(options.max_output_tokens)
            .flatten(),
        top_p: capabilities.top_p.then_some(options.top_p).flatten(),
        reasoning_mode: capabilities
            .reasoning
            .then_some(options.reasoning_mode.clone())
            .flatten(),
    }
}

fn is_openai_reasoning_model(model: &str) -> bool {
    model.starts_with("gpt-5")
        || model.starts_with("o1")
        || model.starts_with("o3")
        || model.starts_with("o4")
}

fn is_anthropic_reasoning_model(model: &str) -> bool {
    model.contains("claude-4")
        || model.contains("claude-opus-4")
        || model.contains("claude-sonnet-4")
        || model.contains("claude-haiku-4")
}

fn is_gemini_reasoning_model(model: &str) -> bool {
    model.contains("gemini-2.5") || model.contains("gemini-3")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_reasoning_models_disable_sampling_on_responses() {
        let caps = infer(ProviderAdapter::OpenAiResponses, "gpt-5.6-sol");
        assert!(caps.reasoning);
        assert!(!caps.temperature);
        assert!(!caps.top_p);
        assert!(caps.max_output_tokens);
    }

    #[test]
    fn unsupported_options_are_removed_before_request_building() {
        let options = GenerationOptions {
            temperature: Some(0.7),
            top_p: Some(0.8),
            max_output_tokens: Some(2048),
            reasoning_mode: Some("high".into()),
        };
        let constrained = constrain_options(
            &options,
            ModelCapabilities {
                system_prompt: true,
                temperature: false,
                top_p: false,
                max_output_tokens: true,
                reasoning: true,
                json_mode: true,
                json_schema: false,
                streaming: true,
                vision: false,
                context_window: None,
            },
        );
        assert_eq!(constrained.temperature, None);
        assert_eq!(constrained.top_p, None);
        assert_eq!(constrained.max_output_tokens, Some(2048));
        assert_eq!(constrained.reasoning_mode.as_deref(), Some("high"));
    }
}
