use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-pig Examples".bold().cyan());
    println!();

    println!("{}", "Record a craving:".bold().green());
    println!("  i-rs-pig add \"奶茶\" --tag drinks");
    println!("  i-rs-pig add \"炸鸡\" --description \"KFC crispy\" --tag fastfood");
    println!();

    println!("{}", "List records:".bold().green());
    println!("  i-rs-pig list");
    println!("  i-rs-pig list --tag drinks");
    println!();

    println!("{}", "Get details:".bold().green());
    println!("  i-rs-pig get abc12345");
    println!();

    println!("{}", "Delete record:".bold().green());
    println!("  i-rs-pig delete abc12345");
    println!();
}
