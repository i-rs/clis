use std::time::Instant;

/// Run all contracts against a backend, generate a report.
/// This is a demo — run with: cargo test -p claw-core-storage-tests --test report_demo
#[tokio::test]
async fn generate_file_report() -> anyhow::Result<()> {
    let start = Instant::now();

    let fs = claw_core_storage_tests::backends::helpers::file_storage().await;
    let mut contracts = Vec::new();

    // Session
    let t = Instant::now();
    let result = claw_core_storage_tests::contracts::session::run(&fs.storage).await;
    contracts.push(make_result("session", result, t.elapsed()));

    // MessageLog
    let t = Instant::now();
    let result = claw_core_storage_tests::contracts::message_log::run(&fs.storage).await;
    contracts.push(make_result("message_log", result, t.elapsed()));

    // Memory
    let t = Instant::now();
    let result = claw_core_storage_tests::contracts::memory::run(&fs.storage).await;
    contracts.push(make_result("memory", result, t.elapsed()));

    // Stats
    let t = Instant::now();
    let result = claw_core_storage_tests::contracts::stats::run(&fs.storage).await;
    contracts.push(make_result("stats", result, t.elapsed()));

    // ToolCache
    let t = Instant::now();
    let result = claw_core_storage_tests::contracts::tool_cache::run(&fs.storage).await;
    contracts.push(make_result("tool_cache", result, t.elapsed()));

    // Skill
    let t = Instant::now();
    let result = claw_core_storage_tests::contracts::skill::run(&fs.storage).await;
    contracts.push(make_result("skill", result, t.elapsed()));

    let backend_report = claw_core_storage_tests::reporter::BackendReport {
        name: "File".into(),
        contracts,
        duration_ms: start.elapsed().as_millis() as u64,
    };

    let mut report = claw_core_storage_tests::reporter::TestRunReport::new();
    report.add_backend(backend_report);

    let reports_dir = std::path::Path::new("reports");
    report.save(reports_dir)?;

    println!("{}", report.to_markdown());

    Ok(())
}

fn make_result(
    name: &str,
    result: anyhow::Result<()>,
    duration: std::time::Duration,
) -> claw_core_storage_tests::reporter::ContractResult {
    match result {
        Ok(()) => claw_core_storage_tests::reporter::ContractResult {
            name: name.into(),
            passed: true,
            error: None,
            duration_ms: duration.as_millis() as u64,
        },
        Err(e) => claw_core_storage_tests::reporter::ContractResult {
            name: name.into(),
            passed: false,
            error: Some(e.to_string()),
            duration_ms: duration.as_millis() as u64,
        },
    }
}
