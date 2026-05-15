use crate::models::{GoalStore, Milestone, SavingsGoal};
use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;


i_rs_core::create_store!(GoalStore, "goal");


pub fn add_entry(
    store: &mut GoalStore,
    name: String,
    target_amount: f64,
    deadline: chrono::DateTime<Utc>,
    tags: Vec<String>,
    remark: Vec<String>,
    milestones: Vec<(String, f64)>,
) -> Result<SavingsGoal> {
    let now = Utc::now();

    let milestone_list: Vec<Milestone> = milestones
        .into_iter()
        .map(|(name, amount)| Milestone {
            id: Uuid::new_v4().to_string(),
            name,
            amount,
            reached: false,
            reached_at: None,
        })
        .collect();

    let goal = SavingsGoal {
        id: Uuid::new_v4().to_string(),
        name,
        target_amount,
        current_amount: 0.0,
        deadline,
        milestones: milestone_list,
        tags,
        remark,
        created_at: now,
        updated_at: now,
    };

    store.goals.insert(goal.name.clone(), goal.clone());
    save_store(store)?;

    Ok(goal)
}

pub fn deposit_to_goal(store: &mut GoalStore, name: &str, amount: f64) -> Result<SavingsGoal> {
    let goal = store.goals.get_mut(name)
        .ok_or_else(|| anyhow::anyhow!("Goal '{name}' not found"))?;

    goal.current_amount += amount;
    goal.updated_at = Utc::now();
    goal.check_milestones();

    let updated_goal = goal.clone();
    save_store(store)?;

    Ok(updated_goal)
}

pub fn add_milestone(store: &mut GoalStore, goal_name: &str, name: String, amount: f64) -> Result<SavingsGoal> {
    let goal = store.goals.get_mut(goal_name)
        .ok_or_else(|| anyhow::anyhow!("Goal '{goal_name}' not found"))?;

    let milestone = Milestone {
        id: Uuid::new_v4().to_string(),
        name,
        amount,
        reached: goal.current_amount >= amount,
        reached_at: if goal.current_amount >= amount { Some(Utc::now().format("%Y-%m-%d %H:%M").to_string()) } else { None },
    };

    goal.milestones.push(milestone);
    goal.updated_at = Utc::now();

    let updated_goal = goal.clone();
    save_store(store)?;

    Ok(updated_goal)
}

pub fn remove_milestone(store: &mut GoalStore, goal_name: &str, milestone_id: &str) -> Result<SavingsGoal> {
    let goal = store.goals.get_mut(goal_name)
        .ok_or_else(|| anyhow::anyhow!("Goal '{goal_name}' not found"))?;

    let initial_len = goal.milestones.len();
    goal.milestones.retain(|m| m.id != milestone_id);

    if goal.milestones.len() < initial_len {
        goal.updated_at = Utc::now();
        let updated_goal = goal.clone();
        save_store(store)?;
        Ok(updated_goal)
    } else {
        Err(anyhow::anyhow!("Milestone '{milestone_id}' not found"))
    }
}
