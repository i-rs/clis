# i-rs CLI Tools

A collection of 70+ cross-platform CLI tools built with Rust for personal data management.

## Tools

<details>
<summary><b>⚙️ Core & System</b></summary>

| Tool | Description |
|------|-------------|
| [i-rs-server](crates/i-rs-server) | Server management |
| [i-rs-password](crates/i-rs-password) | Password management |
| [i-rs-keys](crates/i-rs-keys) | API key management |
| [i-rs-kv](crates/i-rs-kv) | Key-value storage |
| [i-rs-deploy](crates/i-rs-deploy) | Deployment tracking |
</details>

<details>
<summary><b>📝 Notes & Bookmarks</b></summary>

| Tool | Description |
|------|-------------|
| [i-rs-note](crates/i-rs-note) | Note management |
| [i-rs-bookmark](crates/i-rs-bookmark) | Bookmark management |
| [i-rs-article](crates/i-rs-article) | Article reading tracker |
| [i-rs-read](crates/i-rs-read) | Reading tracker |
| [i-rs-quote](crates/i-rs-quote) | Quote collection |
| [i-rs-spark](crates/i-rs-spark) | Inspiration capture |
| [i-rs-snippet](crates/i-rs-snippet) | Code snippet manager |
| [i-rs-vocab](crates/i-rs-vocab) | Vocabulary learning |
</details>

<details>
<summary><b>❤️ Health & Fitness</b></summary>

| Tool | Description |
|------|-------------|
| [i-rs-weight](crates/i-rs-weight) | Weight tracking |
| [i-rs-height](crates/i-rs-height) | Height tracking |
| [i-rs-mood](crates/i-rs-mood) | Mood tracking |
| [i-rs-sleep](crates/i-rs-sleep) | Sleep tracking |
| [i-rs-step](crates/i-rs-step) | Step counting |
| [i-rs-water](crates/i-rs-water) | Water intake tracking |
| [i-rs-cal](crates/i-rs-cal) | Calorie estimation |
| [i-rs-fast](crates/i-rs-fast) | Fasting tracking |
| [i-rs-exercise](crates/i-rs-exercise) | Exercise tracking |
| [i-rs-run](crates/i-rs-run) | Running records |
| [i-rs-cycling](crates/i-rs-cycling) | Cycling tracking |
| [i-rs-dose](crates/i-rs-dose) | Medicine dosage |
| [i-rs-allergy](crates/i-rs-allergy) | Allergy tracking |
| [i-rs-sit](crates/i-rs-sit) | Sedentary reminder |
| [i-rs-vision](crates/i-rs-vision) | Vision tracking |
</details>

<details>
<summary><b>💰 Finance</b></summary>

| Tool | Description |
|------|-------------|
| [i-rs-ledger](crates/i-rs-ledger) | Accounting ledger |
| [i-rs-budget](crates/i-rs-budget) | Budget management |
| [i-rs-recur](crates/i-rs-recur) | Recurring expenses |
| [i-rs-sub](crates/i-rs-sub) | Subscription tracking |
| [i-rs-invest](crates/i-rs-invest) | Investment tracking |
| [i-rs-debt](crates/i-rs-debt) | Debt management |
| [i-rs-invoice](crates/i-rs-invoice) | Invoice management |
| [i-rs-tax](crates/i-rs-tax) | Tax records |
| [i-rs-goal](crates/i-rs-goal) | Savings goals |
</details>

<details>
<summary><b>📅 Reminders & Expiry</b></summary>

| Tool | Description |
|------|-------------|
| [i-rs-remind](crates/i-rs-remind) | Event reminders |
| [i-rs-domain](crates/i-rs-domain) | Domain expiry tracking |
| [i-rs-bestby](crates/i-rs-bestby) | Best-by date tracking |
| [i-rs-tick](crates/i-rs-tick) | Duration tracking |
| [i-rs-time](crates/i-rs-time) | Time tracking |
| [i-rs-event](crates/i-rs-event) | Event management |
| [i-rs-birthday](crates/i-rs-birthday) | Birthday tracking |
</details>

<details>
<summary><b>🏠 Home & Care</b></summary>

| Tool | Description |
|------|-------------|
| [i-rs-sheet](crates/i-rs-sheet) | Bedsheet replacement |
| [i-rs-toothbrush](crates/i-rs-toothbrush) | Toothbrush replacement |
| [i-rs-towel](crates/i-rs-towel) | Towel replacement |
| [i-rs-bed](crates/i-rs-bed) | Mattress/pillow replacement |
| [i-rs-ac](crates/i-rs-ac) | AC cleaning |
| [i-rs-filter](crates/i-rs-filter) | Filter cleaning |
| [i-rs-purify](crates/i-rs-purify) | Water purifier filter |
| [i-rs-appliance](crates/i-rs-appliance) | Appliance management |
| [i-rs-plant](crates/i-rs-plant) | Plant care |
</details>

<details>
<summary><b>🐾 Pets</b></summary>

| Tool | Description |
|------|-------------|
| [i-rs-feedpet](crates/i-rs-feedpet) | Pet feeding |
| [i-rs-petbath](crates/i-rs-petbath) | Pet bathing |
| [i-rs-walkdog](crates/i-rs-walkdog) | Dog walking |
| [i-rs-aqua](crates/i-rs-aqua) | Aquarium maintenance |
</details>

<details>
<summary><b>📋 Productivity & Tracking</b></summary>

| Tool | Description |
|------|-------------|
| [i-rs-todo](crates/i-rs-todo) | Todo tracking |
| [i-rs-habit](crates/i-rs-habit) | Habit tracking |
| [i-rs-project](crates/i-rs-project) | Project management |
| [i-rs-want](crates/i-rs-want) | Wish list |
| [i-rs-gift](crates/i-rs-gift) | Gift planning |
| [i-rs-car](crates/i-rs-car) | Vehicle management |
| [i-rs-meal](crates/i-rs-meal) | Meal tracking |
| [i-rs-pig](crates/i-rs-pig) | Craving tracking |
| [i-rs-grocery](crates/i-rs-grocery) | Grocery list |
| [i-rs-contact](crates/i-rs-contact) | Contact management |
| [i-rs-movie](crates/i-rs-movie) | Movie tracking |
| [i-rs-podcast](crates/i-rs-podcast) | Podcast tracking |
| [i-rs-cycle](crates/i-rs-cycle) | Menstrual cycle |
</details>

## Quick Install

```bash
# Install individual tools
cargo install --path crates/i-rs-{name}
# e.g. cargo install --path crates/i-rs-todo

# Or build all
cargo build
```

npm and Homebrew packages are published on release.

## Development

```bash
# Build all
cargo build

# Run specific tool
cargo run -p i-rs-todo -- --help

# Test specific crate
cargo test -p i-rs-todo

# Check for warnings
cargo check
```

## Release

```bash
# Update version in root Cargo.toml, then:
git tag v0.0.x
git push origin v0.0.x
```

CI (cargo-dist) auto-builds and publishes to GitHub Releases, npm, and Homebrew.

## Documentation

Full documentation at [docs](docs/) (VitePress).

Each crate has a README with usage examples.

## License

MIT OR Apache-2.0
