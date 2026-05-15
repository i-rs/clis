---
name: "i-rs-movie"
description: "Tracks movie collection (add/list/get/watch/update/delete/stats). Invoke when user needs to manage personal movie library, track watched/unwatched movies, rate films, or view movie statistics."
---

# i-rs-movie

Movie tracking CLI tool for managing your personal film library.

## Storage

- Config: `~/.config/i-rs/movies.json`

## Global Flags

- `--json` — Output in JSON format
## Commands

### add

Add a new movie.

```bash
i-rs-movie add <NAME> [OPTIONS]
```

Options:
- `-y, --year <YEAR>` - Release year
- `-d, --director <DIRECTOR>` - Director name
- `-w, --watched` - Mark as watched
- `-r, --rating <RATING>` - Rating (0.0-10.0)
- `--review <REVIEW>` - Review text
- `--release-date <DATE>` - Release date (YYYY-MM-DD)
- `-t, --tag <TAG>` - Tags
- `--remark <REMARK>` - Remarks

### list

List movies with optional filters.

```bash
i-rs-movie list [OPTIONS]
```

Options:
- `--watched` - Show only watched
- `--unwatched` - Show only unwatched
- `-t, --tag <TAG>` - Filter by tag

### get

Get movie details.

```bash
i-rs-movie get <NAME>
```

### watch

Mark movie as watched.

```bash
i-rs-movie watch <NAME> [OPTIONS]
```

Options:
- `-r, --rating <RATING>` - Rating (0.0-10.0)
- `--review <REVIEW>` - Review text

### update

Update movie information.

```bash
i-rs-movie update <NAME> [OPTIONS]
```

Options:
- `-y, --year <YEAR>` - Release year
- `-d, --director <DIRECTOR>` - Director name
- `-r, --rating <RATING>` - Rating (0.0-10.0)
- `--review <REVIEW>` - Review text
- `--release-date <DATE>` - Release date
- `-t, --tag <TAG>` - Tags
- `--remark <REMARK>` - Remarks

### delete

Delete a movie.

```bash
i-rs-movie delete <NAME>
```

### stats

Show collection statistics.

```bash
i-rs-movie stats
```

### data

Manage data (export, import, clear).

```bash
i-rs-movie data export
i-rs-movie data import [FILE]
i-rs-movie data clear
```

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

## Examples

```bash
# Add a movie
i-rs-movie add "Inception" --year 2010 --director "Christopher Nolan" [OPTIONS]

# Add as watched with rating
i-rs-movie add "The Matrix" --watched --rating 9.0 [OPTIONS]

# List all movies
i-rs-movie list [OPTIONS]

# List unwatched movies
i-rs-movie list --unwatched

# List by tag
i-rs-movie list --tag sci-fi

# Mark as watched
i-rs-movie watch "Inception" --rating 9.5 --review "Mind-bending"

# View statistics
i-rs-movie stats

# Get details
i-rs-movie get "Inception"

# Update movie
i-rs-movie update "Inception" --tag masterpiece

# Delete movie
i-rs-movie delete "Inception"
```
