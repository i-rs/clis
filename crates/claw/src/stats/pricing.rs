use std::collections::HashMap;

/// Per-model pricing (USD per 1M tokens).
#[derive(Debug, Clone)]
pub struct ModelPricing {
    pub input_per_m: f64,
    pub output_per_m: f64,
}

impl ModelPricing {
    pub fn estimate_cost(&self, prompt_tokens: u32, completion_tokens: u32) -> f64 {
        (prompt_tokens as f64 / 1_000_000.0) * self.input_per_m
            + (completion_tokens as f64 / 1_000_000.0) * self.output_per_m
    }
}

/// Built-in pricing table with known model costs.
/// Unknown models default to $0.00.
#[derive(Debug, Clone)]
pub struct ModelPricingTable {
    inner: HashMap<String, ModelPricing>,
}

impl ModelPricingTable {
    pub fn new() -> Self {
        let mut inner = HashMap::new();
        inner.insert("gpt-4o-mini".to_string(), ModelPricing { input_per_m: 0.150, output_per_m: 0.600 });
        inner.insert("gpt-4o".to_string(), ModelPricing { input_per_m: 2.500, output_per_m: 10.000 });
        inner.insert("gpt-4".to_string(), ModelPricing { input_per_m: 30.00, output_per_m: 60.00 });
        inner.insert("gpt-4-turbo".to_string(), ModelPricing { input_per_m: 10.00, output_per_m: 30.00 });
        inner.insert("claude-sonnet-4-20250514".to_string(), ModelPricing { input_per_m: 3.000, output_per_m: 15.000 });
        inner.insert("claude-sonnet-4".to_string(), ModelPricing { input_per_m: 3.000, output_per_m: 15.000 });
        inner.insert("claude-3-5-sonnet".to_string(), ModelPricing { input_per_m: 3.000, output_per_m: 15.000 });
        inner.insert("claude-haiku-4-20250514".to_string(), ModelPricing { input_per_m: 0.800, output_per_m: 4.000 });
        inner.insert("claude-haiku-4".to_string(), ModelPricing { input_per_m: 0.800, output_per_m: 4.000 });
        inner.insert("claude-3-haiku".to_string(), ModelPricing { input_per_m: 0.250, output_per_m: 1.250 });
        inner.insert("deepseek-chat".to_string(), ModelPricing { input_per_m: 0.500, output_per_m: 2.000 });
        inner.insert("deepseek-reasoner".to_string(), ModelPricing { input_per_m: 0.500, output_per_m: 2.000 });
        Self { inner }
    }

    /// Override pricing for a specific model.
    pub fn set(&mut self, model: &str, pricing: ModelPricing) {
        self.inner.insert(model.to_string(), pricing);
    }

    /// Estimate cost for a model. Returns 0.0 for unknown models.
    pub fn estimate(&self, model: &str, prompt_tokens: u32, completion_tokens: u32) -> f64 {
        match self.inner.get(model) {
            Some(p) => p.estimate_cost(prompt_tokens, completion_tokens),
            None => {
                let mut candidates: Vec<_> = self.inner.iter()
                    .filter(|(key, _)| model.starts_with(key.as_str()))
                    .collect();
                candidates.sort_by(|a, b| b.0.len().cmp(&a.0.len()));
                match candidates.into_iter().next() {
                    Some((_, p)) => p.estimate_cost(prompt_tokens, completion_tokens),
                    None => 0.0,
                }
            }
        }
    }

    /// Apply custom pricing overrides from config.
    pub fn apply_overrides(&mut self, overrides: &HashMap<String, super::StatsPricingConfig>) {
        for (model, cfg) in overrides {
            self.set(model, ModelPricing {
                input_per_m: cfg.input,
                output_per_m: cfg.output,
            });
        }
    }
}

impl Default for ModelPricingTable {
    fn default() -> Self {
        Self::new()
    }
}
