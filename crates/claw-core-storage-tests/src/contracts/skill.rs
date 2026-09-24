use i_rs_claw_core::storage::ClawStorage;
use std::sync::Arc;

pub async fn run(storage: &Arc<ClawStorage>) -> anyhow::Result<()> {
    let agent_id = "skill-test-agent";
    let skill_name = "test-skill";
    let skill_content = "# Test Skill\n\nDo something useful.";

    storage
        .skills
        .install(agent_id, skill_name, skill_content)
        .await?;

    let list = storage.skills.list(agent_id).await?;
    assert!(
        list.iter().any(|s| s.name == skill_name),
        "skill should appear in list"
    );

    let loaded = storage.skills.get(agent_id, skill_name).await?;
    assert!(loaded.is_some(), "skill should be retrievable by name");

    storage.skills.remove(agent_id, skill_name).await?;

    let after = storage.skills.get(agent_id, skill_name).await?;
    assert!(after.is_none(), "skill should be gone after remove");

    // Test a skill with parameters — should appear in executable list
    let exec_name = "exec-skill";
    let exec_content = "---\ndescription = \"Executable test\"\nparameters = { type = \"object\", properties = {} }\n---\n\nExecute this.";
    storage
        .skills
        .install(agent_id, exec_name, exec_content)
        .await?;

    let executable = storage.skills.list_executable(agent_id).await?;
    assert!(
        executable.iter().any(|s| s.name == exec_name),
        "skill with parameters should appear in executable list"
    );

    storage.skills.remove(agent_id, exec_name).await?;

    Ok(())
}
