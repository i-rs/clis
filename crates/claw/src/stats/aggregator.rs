use super::pricing::ModelPricingTable;
use super::{StatsPeriod, TokenRecord, TokenStats};

/// Aggregate a set of TokenRecords into TokenStats.
pub fn aggregate(records: &[TokenRecord], pricing: &ModelPricingTable) -> TokenStats {
    if records.is_empty() {
        return TokenStats {
            period: StatsPeriod::All,
            total_requests: 0,
            total_tokens: 0,
            total_prompt_tokens: 0,
            total_completion_tokens: 0,
            total_cost_usd: 0.0,
            avg_tokens_per_request: 0.0,
            avg_latency_ms: 0.0,
            success_rate: 100.0,
            by_model: Vec::new(),
            by_agent: Vec::new(),
            daily_series: Vec::new(),
        };
    }

    let total_requests = records.len() as u32;
    let total_prompt_tokens: u64 = records.iter().map(|r| r.prompt_tokens as u64).sum();
    let total_completion_tokens: u64 = records.iter().map(|r| r.completion_tokens as u64).sum();
    let total_tokens = total_prompt_tokens + total_completion_tokens;
    let total_cost_usd: f64 = records.iter().map(|r| r.estimated_cost_usd).sum();
    let avg_tokens_per_request = if total_requests > 0 {
        total_tokens as f64 / total_requests as f64
    } else {
        0.0
    };
    let total_latency: u64 = records.iter().map(|r| r.latency_ms).sum();
    let avg_latency_ms = if total_requests > 0 {
        total_latency as f64 / total_requests as f64
    } else {
        0.0
    };
    let success_count = records.iter().filter(|r| r.success).count();
    let success_rate = if total_requests > 0 {
        (success_count as f64 / total_requests as f64) * 100.0
    } else {
        100.0
    };

    TokenStats {
        period: StatsPeriod::All,
        total_requests,
        total_tokens,
        total_prompt_tokens,
        total_completion_tokens,
        total_cost_usd,
        avg_tokens_per_request,
        avg_latency_ms,
        success_rate,
        by_model: group_by_model(records, pricing),
        by_agent: group_by_agent(records, pricing),
        daily_series: group_by_day(records, pricing),
    }
}

/// Filter records by time period.
#[allow(dead_code)]
pub fn filter_by_period<'a>(records: &'a [TokenRecord], period: &StatsPeriod) -> Vec<&'a TokenRecord> {
    let now = chrono::Local::now().naive_local();
    let today_start = now.date().and_hms_opt(0, 0, 0).map(|dt| dt.and_utc().timestamp()).unwrap_or(0);

    let from_ts = match period {
        StatsPeriod::Today => today_start,
        StatsPeriod::Last7Days => today_start - 7 * 86400,
        StatsPeriod::Last30Days => today_start - 30 * 86400,
        StatsPeriod::All => i64::MIN,
        StatsPeriod::Custom { from, .. } => *from,
    };

    let to_ts = match period {
        StatsPeriod::Custom { to, .. } => *to,
        _ => i64::MAX,
    };

    records.iter().filter(|r| r.timestamp >= from_ts && r.timestamp <= to_ts).collect()
}

pub fn group_by_model(records: &[TokenRecord], _pricing: &ModelPricingTable) -> Vec<super::ModelStats> {
    let mut map: std::collections::HashMap<String, super::ModelStats> = std::collections::HashMap::new();

    for r in records {
        let entry = map.entry(r.model.clone()).or_insert(super::ModelStats {
            model: r.model.clone(),
            request_count: 0,
            total_tokens: 0,
            total_cost_usd: 0.0,
            avg_latency_ms: 0.0,
        });
        entry.request_count += 1;
        entry.total_tokens += r.total_tokens as u64;
        entry.total_cost_usd += r.estimated_cost_usd;
        entry.avg_latency_ms = (entry.avg_latency_ms * (entry.request_count - 1) as f64 + r.latency_ms as f64)
            / entry.request_count as f64;
    }

    let mut result: Vec<_> = map.into_values().collect();
    result.sort_by_key(|b| std::cmp::Reverse(b.total_tokens));
    result
}

pub fn group_by_agent(records: &[TokenRecord], _pricing: &ModelPricingTable) -> Vec<super::AgentStats> {
    let mut map: std::collections::HashMap<String, super::AgentStats> = std::collections::HashMap::new();

    for r in records {
        let entry = map.entry(r.agent_id.clone()).or_insert(super::AgentStats {
            agent_id: r.agent_id.clone(),
            request_count: 0,
            total_tokens: 0,
            total_cost_usd: 0.0,
        });
        entry.request_count += 1;
        entry.total_tokens += r.total_tokens as u64;
        entry.total_cost_usd += r.estimated_cost_usd;
    }

    let mut result: Vec<_> = map.into_values().collect();
    result.sort_by_key(|b| std::cmp::Reverse(b.total_tokens));
    result
}

pub fn group_by_day(records: &[TokenRecord], _pricing: &ModelPricingTable) -> Vec<super::DailyStats> {
    let mut map: std::collections::BTreeMap<String, super::DailyStats> = std::collections::BTreeMap::new();

    for r in records {
        // Convert timestamp to date string
        let seconds = r.timestamp.max(0);
        let naive = chrono::DateTime::from_timestamp(seconds, 0)
            .map(|dt| dt.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let entry = map.entry(naive.clone()).or_insert(super::DailyStats {
            date: naive,
            request_count: 0,
            total_tokens: 0,
            total_cost_usd: 0.0,
        });
        entry.request_count += 1;
        entry.total_tokens += r.total_tokens as u64;
        entry.total_cost_usd += r.estimated_cost_usd;
    }

    map.into_values().collect()
}

/// Get today's aggregated summary.
pub fn today_summary(records: &[TokenRecord], tz_offset: chrono::FixedOffset) -> super::TodaySummary {
    let today_start = crate::utils::now_in_tz(tz_offset)
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .map(|dt| dt.and_utc().timestamp())
        .unwrap_or(0);

    let today_records: Vec<_> = records.iter().filter(|r| r.timestamp >= today_start).collect();

    super::TodaySummary {
        requests: today_records.len() as u32,
        tokens: today_records.iter().map(|r| r.total_tokens as u64).sum(),
        cost_usd: today_records.iter().map(|r| r.estimated_cost_usd).sum(),
    }
}
