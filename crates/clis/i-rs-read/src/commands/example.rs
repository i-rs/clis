use anyhow::Result;
use clap::Args;

#[derive(Args, Debug)]
pub struct ExampleArgs {}

pub fn example(_args: ExampleArgs) -> Result<()> {
    println!(
        r#"
📚 i-rs-read Examples

1. Add a new book:
   i-rs-read add "The Rust Programming Language" "Steve Klabnik" 500

2. Add a book with tags:
   i-rs-read add "Programming Rust" "Jim Blandy" 400 --tags rust,programming

3. Update reading progress:
   i-rs-read update "The Rust Programming Language" --current-page 250

4. Mark book as completed:
   i-rs-read update "The Rust Programming Language" --status completed --rating 5

5. Add rating and review:
   i-rs-read update "The Rust Programming Language" --rating 4.5 --review "Excellent book!"

6. List all books:
   i-rs-read list

7. List books by tag:
   i-rs-read list --tag rust

8. List books by status:
   i-rs-read list --status reading

9. Get book details:
   i-rs-read get "The Rust Programming Language"

10. Delete a book:
    i-rs-read delete "Old Book Title"

11. View statistics:
    i-rs-read stats

12. Add remarks:
    i-rs-read update "Book Title" --add-remark "Chapter 1 complete"

13. Remove remark:
    i-rs-read update "Book Title" --remove-remark 1

14. Use JSON output:
    i-rs-read list --json
    i-rs-read get "Book Title" --json

"#
    );

    Ok(())
}
