# i-rs-movie

Movie tracking CLI tool for managing your personal film library.

## Features

- Track movies with name, year, director, and release date
- Mark movies as watched/unwatched with rating and review
- Support for tags and remarks
- View movie statistics (total, watched, unwatched, average rating)
- Filter movies by watch status or tags
- JSON output support for all commands

## Install

```bash
npm install -g @i-rs/i-rs-movie
# or
brew install i-rs/homebrew-tap/i-rs-movie
```

## Quick Start

```bash
# Add a movie
i-rs-movie add "Inception" --year 2010 --director "Christopher Nolan"

# List all movies
i-rs-movie list

# Mark as watched with rating
i-rs-movie watch "Inception" --rating 9.0

# View statistics
i-rs-movie stats
```

## Commands

| Command | Description |
|---------|-------------|
| `add` | Add a new movie to the library |
| `list` | List all movies (filter by watched/unwatched or tag) |
| `get` | Get details of a specific movie |
| `watch` | Mark a movie as watched (with optional rating/review) |
| `update` | Update movie information |
| `delete` | Remove a movie from the library |
| `stats` | Show movie collection statistics |
| `example` | Show usage examples |
| `skill` | View AI skill documentation |

## Data Storage

Configuration is stored locally at:
- macOS: `~/.config/i-rs/movies.json`
- Linux: `~/.config/i-rs/movies.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## License

MIT OR Apache-2.0
