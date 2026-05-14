pub mod add;
pub mod delete;
pub mod example;
pub mod get;
pub mod list;
pub mod set;
pub mod skill;

pub use add::handle_add;
pub use delete::handle_delete;
pub use example::handle_example;
pub use get::handle_get;
pub use list::handle_list;
pub use set::{handle_set, handle_target};
pub use skill::{handle_skill, SkillCommand};

pub mod data;
