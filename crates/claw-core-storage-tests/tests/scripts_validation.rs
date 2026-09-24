use claw_core_storage_tests::conversations::runner::loader;
use std::path::Path;

/// Validate all conversation scripts in the scenarios directory.
/// Every script must be valid JSON with at least 1 step.
#[test]
fn test_all_scripts_load_successfully() {
    let scenarios = Path::new("conversations/scenarios");
    let scripts = loader::discover_scripts(scenarios).unwrap();

    assert!(
        !scripts.is_empty(),
        "expected at least 1 script in {}/",
        scenarios.display()
    );

    for script_path in &scripts {
        let script = loader::load_script(script_path)
            .unwrap_or_else(|e| panic!("failed to load {:?}: {}", script_path, e));

        assert!(!script.steps.is_empty(), "{:?} has 0 steps", script_path);

        // Verify step numbering is sequential starting from 1
        for (i, step) in script.steps.iter().enumerate() {
            assert_eq!(
                step.step as usize,
                i + 1,
                "{:?} step {} has wrong step number (expected {})",
                script_path,
                step.title,
                i + 1
            );
        }

        println!("  ✅ {} — {} steps", script.meta.name, script.steps.len());
    }

    println!("\nTotal: {} scripts validated", scripts.len());
}
