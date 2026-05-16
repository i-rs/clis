use owo_colors::OwoColorize;

pub fn handle_example() {
    println!();
    println!("{}", "i-rs-movie Examples".bold().cyan());
    println!();

    println!("{}", "Add Movie:".bold().green());
    println!("  i-rs-movie add \"The Matrix\" --year 1999 --director \"Wachowski\"");
    println!("  i-rs-movie add \"Inception\" --year 2010 --tag scifi --tag thriller");
    println!();

    println!("{}", "Add Watched Movie with Rating:".bold().green());
    println!(
        "  i-rs-movie add \"Interstellar\" --year 2014 --director \"Nolan\" --watched --rating 9.5"
    );
    println!();

    println!("{}", "List Movies:".bold().green());
    println!("  i-rs-movie list");
    println!("  i-rs-movie list --watched");
    println!("  i-rs-movie list --unwatched");
    println!("  i-rs-movie list --tag scifi");
    println!();

    println!("{}", "Get Movie Details:".bold().green());
    println!("  i-rs-movie get \"The Matrix\"");
    println!();

    println!("{}", "Watch a Movie:".bold().green());
    println!("  i-rs-movie watch \"The Matrix\" --rating 9.0");
    println!("  i-rs-movie watch \"Inception\" --rating 8.5 --review \"Amazing plot\"");
    println!();

    println!("{}", "Update Movie:".bold().green());
    println!("  i-rs-movie update \"The Matrix\" --rating 9.5 --tag classic");
    println!("  i-rs-movie update \"Inception\" --director \"Christopher Nolan\"");
    println!();

    println!("{}", "Delete Movie:".bold().green());
    println!("  i-rs-movie delete \"The Matrix\"");
    println!();

    println!("{}", "Statistics:".bold().green());
    println!("  i-rs-movie stats");
    println!();

    println!("{}", "JSON Output:".bold().green());
    println!("  i-rs-movie list --json");
    println!("  i-rs-movie get \"The Matrix\" --json");
    println!();

    println!("{}", "Options:".bold().yellow());
    println!("  --year, -y:        Release year");
    println!("  --director, -d:    Director name");
    println!("  --watched, -w:     Mark as watched");
    println!("  --rating, -r:      Rating (0-10)");
    println!("  --review:         Review lines (repeatable)");
    println!("  --release-date:   Release date (YYYY-MM-DD)");
    println!("  --tag, -t:        Tags (repeatable)");
    println!("  --remark:         Remark lines (repeatable)");
    println!();
}
