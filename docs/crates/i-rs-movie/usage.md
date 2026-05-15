# i-rs-movie Usage

## Global Flags

- `--json` — Output in JSON format

## Installation

```bash
npm install -g @i-rs/i-rs-movie
# or
brew install i-rs/homebrew-tap/i-rs-movie
```

## Commands

### add

Add a new movie to the library.

```bash
i-rs-movie add <NAME> [OPTIONS]
```

Arguments:
- `NAME` - Movie name (required)

Options:
- `-y, --year <YEAR>` - Release year
- `-d, --director <DIRECTOR>` - Director name
- `-w, --watched` - Mark as already watched
- `-r, --rating <RATING>` - Rating (0.0-10.0)
- `--review <REVIEW>` - Review text (can be repeated)
- `--release-date <DATE>` - Release date (YYYY-MM-DD)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `--remark <REMARK>` - Remarks (can be repeated)

### list

List movies with optional filters.

```bash
i-rs-movie list [OPTIONS]
```

Options:
- `--watched` - Show only watched movies
- `--unwatched` - Show only unwatched movies
- `-t, --tag <TAG>` - Filter by tag

### get

Get details of a specific movie.

```bash
i-rs-movie get <NAME>
```

### watch

Mark a movie as watched with optional rating and review.

```bash
i-rs-movie watch <NAME> [OPTIONS]
```

Arguments:
- `NAME` - Movie name (required)

Options:
- `-r, --rating <RATING>` - Rating (0.0-10.0)
- `--review <REVIEW>` - Review text (can be repeated)

### update

Update movie information.

```bash
i-rs-movie update <NAME> [OPTIONS]
```

Arguments:
- `NAME` - Movie name (required)

Options:
- `-y, --year <YEAR>` - Release year
- `-d, --director <DIRECTOR>` - Director name
- `-r, --rating <RATING>` - Rating (0.0-10.0)
- `--review <REVIEW>` - Review text (can be repeated)
- `--release-date <DATE>` - Release date (YYYY-MM-DD)
- `-t, --tag <TAG>` - Tags (can be repeated)
- `--remark <REMARK>` - Remarks (can be repeated)

### delete

Remove a movie from the library.

```bash
i-rs-movie delete <NAME>
```

### stats

Show movie collection statistics.

```bash
i-rs-movie stats
```

Displays:
- Total movies
- Watched count
- Unwatched count
- Average rating (of rated movies)

## Rating System

Movies are rated on a scale of 0.0 to 10.0:
- 9.0-10.0: Masterpiece
- 7.0-8.9: Excellent
- 5.0-6.9: Good
- 3.0-4.9: Average
- 0.0-2.9: Poor

## Tags

Use tags to categorize movies:

```bash
# Add movies with tags
i-rs-movie add "Inception" --tag sci-fi --tag thriller --tag mind-bending

# Filter by tag
i-rs-movie list --tag sci-fi
```

Common tags:
- Genre: `sci-fi`, `action`, `comedy`, `drama`, `horror`, `romance`, `thriller`
- Mood: `feel-good`, `dark`, `fun`, `thought-provoking`
- Source: `book-adaptation`, `based-on-true-story`, `original`
- Language: `japanese`, `korean`, `chinese`, `foreign`

### data

Manage data (export, import, clear).

```bash
i-rs-movie data export
i-rs-movie data import [FILE]
i-rs-movie data clear
```

Subcommands:
- `export` - Export all data as JSON to stdout
- `import [FILE]` - Import data from JSON file or stdin
- `clear` - Clear all data
### example

Show usage examples.

```bash
i-rs-movie example
```
### skill

Show skill information.

```bash
i-rs-movie skill [summary|content|raw]
```

## Data Storage

- macOS: `~/.config/i-rs/movies.json`
- Linux: `~/.config/i-rs/movies.json`
- Windows: `~\AppData\Roaming\i-rs\movie.json`

## Environment Variables

- `CONFIG_DIR` - Override config directory path

```bash
CONFIG_DIR=/tmp i-rs-movie list
```

## JSON Output

All commands support `--json` flag for JSON output:

```bash
# List in JSON format
i-rs-movie list --json

# Get movie details in JSON
i-rs-movie get "Inception" --json

# Get statistics in JSON
i-rs-movie stats --json
```
