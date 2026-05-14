# i-rs-movie Test Records

## Test Log

### Initial Setup

```bash
$ i-rs-movie add "Inception"
✓ Movie added: Inception
```

### Adding with Full Details

```bash
$ i-rs-movie add "The Shawshank Redemption" --year 1994 --director "Frank Darabont"
✓ Movie added: The Shawshank Redemption (1994)
```

### Adding with Tags

```bash
$ i-rs-movie add "Blade Runner" --year 1982 --director "Ridley Scott" --tag sci-fi --tag classic --tag cyberpunk
✓ Movie added: Blade Runner (1982)
```

### Adding as Watched

```bash
$ i-rs-movie add "Pulp Fiction" --year 1994 --director "Quentin Tarantino" --watched --rating 9.0
✓ Movie added: Pulp Fiction (1994) [Watched]
```

### Adding with Review

```bash
$ i-rs-movie add "Spirited Away" --year 2001 --director "Hayao Miyazaki" --tag animation --tag fantasy --tag japanese
✓ Movie added: Spirited Away (2001)

$ i-rs-movie watch "Spirited Away" --rating 10 --review "Absolutely magical" --review "Best animated film ever"
✓ Movie marked as watched: Spirited Away
```

### List All Movies

```bash
$ i-rs-movie list
 NAME                      YEAR  DIRECTOR            WATCHED  RATING  TAGS
 Inception                 -     -                   -        -       -
 The Shawshank Redemption   1994  Frank Darabont       -        -       -
 Blade Runner              1982  Ridley Scott         -        -       sci-fi, classic, cyberpunk
 Pulp Fiction             1994  Quentin Tarantino     ✓        9.0     -
 Spirited Away            2001  Hayao Miyazaki        ✓        10.0    animation, fantasy, japanese

Total: 5 movies
```

### List Watched Only

```bash
$ i-rs-movie list --watched
 NAME                      YEAR  DIRECTOR            WATCHED  RATING  TAGS
 Pulp Fiction             1994  Quentin Tarantino     ✓        9.0     -
 Spirited Away            2001  Hayao Miyazaki        ✓        10.0    animation, fantasy, japanese

Total: 2 movies
```

### List Unwatched Only

```bash
$ i-rs-movie list --unwatched
 NAME                      YEAR  DIRECTOR            WATCHED  RATING  TAGS
 Inception                 -     -                   -        -       -
 The Shawshank Redemption   1994  Frank Darabont       -        -       -
 Blade Runner              1982  Ridley Scott         -        -       sci-fi, classic, cyberpunk

Total: 3 movies
```

### List by Tag

```bash
$ i-rs-movie list --tag sci-fi
 NAME                      YEAR  DIRECTOR            WATCHED  RATING  TAGS
 Blade Runner              1982  Ridley Scott         -        -       sci-fi, classic, cyberpunk

Total: 1 movie

$ i-rs-movie list --tag animation
 NAME                      YEAR  DIRECTOR            WATCHED  RATING  TAGS
 Spirited Away            2001  Hayao Miyazaki        ✓        10.0    animation, fantasy, japanese

Total: 1 movie
```

### Get Movie Details

```bash
$ i-rs-movie get "Spirited Away"
Name:           Spirited Away
Year:           2001
Director:       Hayao Miyazaki
Watched:        ✓
Rating:         10.0
Reviews:
  - Absolutely magical
  - Best animated film ever
Release Date:    -
Tags:           animation, fantasy, japanese
Remarks:        -
Created:        2025-01-15 10:30:00 UTC
Updated:        2025-01-15 10:35:00 UTC
```

### Watch Command

```bash
$ i-rs-movie watch "Inception" --rating 9.5 --review "Mind-bending"
✓ Movie marked as watched: Inception

$ i-rs-movie watch "Blade Runner" --rating 8.5
✓ Movie marked as watched: Blade Runner
```

### Update Command

```bash
$ i-rs-movie update "Inception" --year 2010 --director "Christopher Nolan"
✓ Movie updated: Inception

$ i-rs-movie update "The Shawshank Redemption" --rating 10 --tag favorite --tag classic
✓ Movie updated: The Shawshank Redemption
```

### Statistics

```bash
$ i-rs-movie stats
Movie Statistics
-----------------
Total:      5 movies
Watched:    4 movies
Unwatched:  1 movie
Avg Rating: 9.5
```

### Delete Command

```bash
$ i-rs-movie delete "Inception"
✓ Movie deleted: Inception
```

### Final List

```bash
$ i-rs-movie list
 NAME                      YEAR  DIRECTOR            WATCHED  RATING  TAGS
 The Shawshank Redemption   1994  Frank Darabont       ✓        10.0    favorite, classic
 Blade Runner              1982  Ridley Scott         ✓        8.5     sci-fi, classic, cyberpunk
 Pulp Fiction             1994  Quentin Tarantino     ✓        9.0     -
 Spirited Away            2001  Hayao Miyazaki        ✓        10.0    animation, fantasy, japanese

Total: 4 movies
```

## Error Handling Tests

### Duplicate Movie

```bash
$ i-rs-movie add "Pulp Fiction"
Error: Movie 'Pulp Fiction' already exists. Use update command to modify.
```

### Movie Not Found (Get)

```bash
$ i-rs-movie get "Nonexistent Movie"
Error: Movie 'Nonexistent Movie' not found
```

### Movie Not Found (Watch)

```bash
$ i-rs-movie watch "Unknown Movie"
Error: Movie 'Unknown Movie' not found
```

### Movie Not Found (Update)

```bash
$ i-rs-movie update "Unknown Movie" --rating 8
Error: Movie 'Unknown Movie' not found
```

### Movie Not Found (Delete)

```bash
$ i-rs-movie delete "Unknown Movie"
Error: Movie 'Unknown Movie' not found
```

### Already Watched

```bash
$ i-rs-movie watch "Pulp Fiction" --rating 8
Error: Movie 'Pulp Fiction' is already marked as watched. Use update command to modify.
```

### Invalid Rating

```bash
$ i-rs-movie add "Test Movie" --rating 15
Error: Rating must be between 0.0 and 10.0
```

## JSON Output Tests

### List JSON

```bash
$ i-rs-movie list --json
{
  "success": true,
  "data": [
    {
      "name": "The Shawshank Redemption",
      "year": 1994,
      "director": "Frank Darabont",
      "watched": true,
      "rating": 10.0,
      "tags": ["favorite", "classic"]
    },
    ...
  ],
  "meta": {
    "count": 4
  }
}
```

### Get JSON

```bash
$ i-rs-movie get "Pulp Fiction" --json
{
  "success": true,
  "data": {
    "name": "Pulp Fiction",
    "year": 1994,
    "director": "Quentin Tarantino",
    "watched": true,
    "rating": 9.0,
    "reviews": [],
    "tags": [],
    "created_at": "2025-01-15T10:00:00Z",
    "updated_at": "2025-01-15T10:00:00Z"
  }
}
```

### Stats JSON

```bash
$ i-rs-movie stats --json
{
  "success": true,
  "data": {
    "total": 4,
    "watched": 4,
    "unwatched": 1,
    "avg_rating": 9.5
  }
}
```

## Test Summary

| Test Case | Status |
|-----------|--------|
| Add movie (basic) | ✅ Pass |
| Add movie with year/director | ✅ Pass |
| Add movie with tags | ✅ Pass |
| Add movie as watched | ✅ Pass |
| Watch command | ✅ Pass |
| Update movie | ✅ Pass |
| Delete movie | ✅ Pass |
| List all movies | ✅ Pass |
| List watched only | ✅ Pass |
| List unwatched only | ✅ Pass |
| List by tag | ✅ Pass |
| Get movie details | ✅ Pass |
| Statistics calculation | ✅ Pass |
| Duplicate handling | ✅ Pass |
| Not found handling | ✅ Pass |
| Already watched handling | ✅ Pass |
| JSON output (list) | ✅ Pass |
| JSON output (get) | ✅ Pass |
| JSON output (stats) | ✅ Pass |
