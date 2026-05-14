# i-rs-movie

Movie tracking CLI tool for managing your personal film library.

## Overview

i-rs-movie helps you build and manage your personal movie collection. Track movies you want to watch, record ones you've seen with ratings and reviews, and get statistics about your viewing habits.

## Quick Start

```bash
# Add a movie to your watchlist
i-rs-movie add "The Shawshank Redemption" --year 1994 --director "Frank Darabont"

# Add a movie you've already watched
i-rs-movie add "Inception" --year 2010 --director "Christopher Nolan" --watched --rating 9.0

# List all movies
i-rs-movie list

# Mark a movie as watched
i-rs-movie watch "The Shawshank Redemption" --rating 10

# View your statistics
i-rs-movie stats

# List only unwatched movies
i-rs-movie list --unwatched

# List movies by tag
i-rs-movie list --tag sci-fi
```

## Installation

```bash
# npm
npm install -g @i-rs/i-rs-movie

# or Homebrew
brew install i-rs/homebrew-tap/i-rs-movie
```

## Data Storage

- macOS: `~/.config/i-rs/movies.json`
- Linux: `~/.config/i-rs/movies.json`
- Windows: `~\AppData\Roaming\i-rs\config.json`

## Features

- **Movie Tracking**: Add movies with name, year, director, and release date
- **Watch Status**: Mark movies as watched/unwatched
- **Ratings & Reviews**: Rate watched movies and write reviews
- **Tags & Remarks**: Organize movies with custom tags and notes
- **Statistics**: View total, watched, unwatched counts and average rating
- **Filtering**: Filter by watch status or tags

## Commands

- [Usage](./usage.md) - Detailed command reference
- [Examples](./examples.md) - Extensive usage examples
- [Test](./test.md) - Test records
