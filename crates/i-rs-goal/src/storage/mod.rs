use crate::models::{Milestone, SavingsGoal, Store};
use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;


i_rs_core::create_store!(Store, "goal");


pub fn find_goal<'a>(store: &'a mut Store, name: &str) -> Option<&'a mut SavingsGoal> {
    store.goals.iter_mut().find(|g| g.name == name)
}

pub fn add_goal(
    store: &mut Store,
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
    
    store.goals.push(goal.clone());
    save_store(store)?;
    
    Ok(goal)
}

pub fn delete_goal(store: &mut Store, name: &str) -> Result<bool> {
    let initial_len = store.goals.len();
    store.goals.retain(|g| g.name != name);
    
    if store.goals.len() < initial_len {
        save_store(store)?;
        Ok(true)
    } else {
        Ok(false)
    }
}

pub fn deposit_to_goal(store: &mut Store, name: &str, amount: f64) -> Result<SavingsGoal> {
    let goal = find_goal(store, name)
        .ok_or_else(|| anyhow::anyhow!("Goal '{}' not found", name))?;
    
    goal.current_amount += amount;
    goal.updated_at = Utc::now();
    goal.check_milestones();
    
    let updated_goal = goal.clone();
    save_store(store)?;
    
    Ok(updated_goal)
}

pub fn add_milestone(store: &mut Store, goal_name: &str, name: String, amount: f64) -> Result<SavingsGoal> {
    let goal = find_goal(store, goal_name)
        .ok_or_else(|| anyhow::anyhow!("Goal '{}' not found", goal_name))?;
    
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

pub fn remove_milestone(store: &mut Store, goal_name: &str, milestone_id: &str) -> Result<SavingsGoal> {
    let goal = find_goal(store, goal_name)
        .ok_or_else(|| anyhow::anyhow!("Goal '{}' not found", goal_name))?;
    
    let initial_len = goal.milestones.len();
    goal.milestones.retain(|m| m.id != milestone_id);
    
    if goal.milestones.len() < initial_len {
        goal.updated_at = Utc::now();
        let updated_goal = goal.clone();
        save_store(store)?;
        Ok(updated_goal)
    } else {
        Err(anyhow::anyhow!("Milestone '{}' not found", milestone_id))
    }
}
