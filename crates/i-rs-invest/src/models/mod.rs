use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tabled::Tabled;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AssetType {
    Stock,
    Fund,
    Crypto,
}

impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssetType::Stock => write!(f, "stock"),
            AssetType::Fund => write!(f, "fund"),
            AssetType::Crypto => write!(f, "crypto"),
        }
    }
}

impl std::str::FromStr for AssetType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "stock" => Ok(AssetType::Stock),
            "fund" => Ok(AssetType::Fund),
            "crypto" => Ok(AssetType::Crypto),
            _ => Err(format!("Invalid asset type: {}. Use stock, fund, or crypto", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Investment {
    pub name: String,
    pub symbol: String,
    pub asset_type: AssetType,
    pub quantity: f64,
    pub buy_price: f64,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub buy_date: DateTime<Utc>,
    #[serde(default)]
    pub current_price: Option<f64>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub remark: Vec<String>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_seconds")]
    pub updated_at: DateTime<Utc>,
}

impl Investment {
    pub fn total_cost(&self) -> f64 {
        self.quantity * self.buy_price
    }

    pub fn current_value(&self) -> Option<f64> {
        self.current_price.map(|price| self.quantity * price)
    }

    pub fn profit_loss(&self) -> Option<f64> {
        self.current_price.map(|price| (price - self.buy_price) * self.quantity)
    }

    pub fn profit_loss_percentage(&self) -> Option<f64> {
        self.current_price.map(|price| ((price - self.buy_price) / self.buy_price) * 100.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvestmentStore {
    pub investments: HashMap<String, Investment>,
}

impl Default for InvestmentStore {
    fn default() -> Self {
        Self {
            investments: HashMap::new(),
        }
    }
}

#[derive(Tabled)]
pub struct InvestmentRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "SYMBOL")]
    symbol: String,
    #[tabled(rename = "TYPE")]
    asset_type: String,
    #[tabled(rename = "QTY")]
    quantity: String,
    #[tabled(rename = "BUY_PRICE")]
    buy_price: String,
    #[tabled(rename = "CURRENT")]
    current_price: String,
    #[tabled(rename = "P/L%")]
    profit_loss_pct: String,
    #[tabled(rename = "VALUE")]
    value: String,
    #[tabled(rename = "TAGS")]
    tags: String,
}

impl InvestmentRow {
    pub fn from_investment(investment: &Investment) -> Self {
        let profit_loss_pct = investment
            .profit_loss_percentage()
            .map(|p| format!("{:+.2}%", p))
            .unwrap_or_else(|| "N/A".to_string());

        let current_price_str = investment
            .current_price
            .map(|p| format!("{:.2}", p))
            .unwrap_or_else(|| "N/A".to_string());

        let value_str = investment
            .current_value()
            .map(|v| format!("{:.2}", v))
            .unwrap_or_else(|| "N/A".to_string());

        Self {
            name: investment.name.clone(),
            symbol: investment.symbol.clone(),
            asset_type: investment.asset_type.to_string().to_uppercase(),
            quantity: format!("{:.4}", investment.quantity),
            buy_price: format!("{:.2}", investment.buy_price),
            current_price: current_price_str,
            profit_loss_pct,
            value: value_str,
            tags: if investment.tags.is_empty() {
                "-".to_string()
            } else {
                investment.tags.join(", ")
            },
        }
    }
}
