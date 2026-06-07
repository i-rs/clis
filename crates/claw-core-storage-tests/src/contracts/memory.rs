use std::sync::Arc;
use i_rs_claw_core::storage::ClawStorage;
use i_rs_claw_core::memory::CrossSessionMemory;

pub async fn run(storage: &Arc<ClawStorage>) -> anyhow::Result<()> {
    let agent = "memory-test-agent";

    // Create and populate memory
    let mut mem = CrossSessionMemory::for_agent_with_storage(storage, agent);
    mem.record_tool_use("i-rs-weight");
    mem.record_tool_use("i-rs-weight");
    mem.record_tool_use("i-rs-todo");
    mem.set_user_name("测试用户");
    mem.flush();

    // Load fresh and verify
    let loaded = CrossSessionMemory::for_agent_with_storage(storage, agent);
    let formatted = loaded.format_user_memory();
    assert!(
        formatted.contains("i-rs-weight"),
        "memory should contain tool frequency, got: {}",
        formatted
    );

    // Update and re-save
    {
        let mut mem2 = CrossSessionMemory::for_agent_with_storage(storage, agent);
        mem2.record_tool_use("i-rs-weight");
        mem2.flush();
    }

    let reloaded = CrossSessionMemory::for_agent_with_storage(storage, agent);
    let memory_text = reloaded.format_user_memory();
    assert!(
        memory_text.contains("i-rs-weight"),
        "memory should contain i-rs-weight tool, got: {}",
        memory_text
    );

    // Cleanup
    let mut empty = CrossSessionMemory::for_agent_with_storage(storage, agent);
    empty.flush();

    Ok(())
}
