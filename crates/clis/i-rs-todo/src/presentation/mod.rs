i_rs_core::presentation!(TodoRow, "todos");

pub fn print_todo_count(pending: usize, done: usize) {
    println!(
        "\n{} {} pending, {} {} done",
        "Total:".dimmed(),
        pending.to_string().cyan(),
        done.to_string().green(),
        "done".dimmed()
    );
}
