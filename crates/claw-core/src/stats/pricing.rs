use std::collections::HashMap;
use std::sync::LazyLock;

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

static MODEL_PRICING_TABLE: LazyLock<HashMap<String, (f64, f64)>> = LazyLock::new(|| {
    let raw: toml::Value =
        toml::from_str(include_str!("../../data/pricing.toml")).expect("failed to parse pricing.toml");
    let mut m = HashMap::new();
    if let Some(models) = raw.get("models").and_then(|v| v.as_table()) {
        for (name, val) in models {
            let input = val.get("input").and_then(|v| v.as_float()).unwrap_or(0.0);
            let output = val.get("output").and_then(|v| v.as_float()).unwrap_or(0.0);
            m.insert(name.clone(), (input, output));
        }
    }
    m
});

/// Built-in pricing table with known model costs.
/// Unknown models default to $0.00.
#[derive(Debug, Clone)]
pub struct ModelPricingTable {
    inner: HashMap<String, ModelPricing>,
}

impl ModelPricingTable {
    pub fn new() -> Self {
        let mut inner = HashMap::new();
        for (name, (input, output)) in MODEL_PRICING_TABLE.iter() {
            inner.insert(
                name.clone(),
                ModelPricing {
                    input_per_m: *input,
                    output_per_m: *output,
                },
            );
        }
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
                let mut candidates: Vec<_> = self
                    .inner
                    .iter()
                    .filter(|(key, _)| model.starts_with(key.as_str()))
                    .collect();
                candidates.sort_by_key(|b| std::cmp::Reverse(b.0.len()));
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
            self.set(
                model,
                ModelPricing {
                    input_per_m: cfg.input,
                    output_per_m: cfg.output,
                },
            );
        }
    }
}

impl Default for ModelPricingTable {
    fn default() -> Self {
        Self::new()
    }
}
