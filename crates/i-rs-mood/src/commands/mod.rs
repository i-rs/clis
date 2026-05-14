pub mod add;
pub mod delete;
pub mod example;
pub mod list;
pub mod skill;
pub mod update;

pub use add::handle_add;
pub use delete::handle_delete;
pub use example::handle_example;
pub use list::handle_list;
pub use skill::{handle_skill, SkillCommand};
pub use update::handle_update;

pub mod data;
