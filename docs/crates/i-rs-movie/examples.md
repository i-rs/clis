# i-rs-movie Examples

## Basic Usage

### Adding Movies

```bash
# Add a movie with basic info
i-rs-movie add "Inception"

# Add with year and director
i-rs-movie add "Inception" --year 2010 --director "Christopher Nolan"

# Add and mark as already watched with rating
i-rs-movie add "The Matrix" --year 1999 --director "Wachowskis" --watched --rating 9.0
```

### Listing Movies

```bash
# List all movies
i-rs-movie list

# List only watched movies
i-rs-movie list --watched

# List only unwatched movies (to-watch list)
i-rs-movie list --unwatched
```

### Watching Movies

```bash
# Mark as watched
i-rs-movie watch "Inception"

# Mark with rating
i-rs-movie watch "Inception" --rating 9.0

# Mark with rating and review
i-rs-movie watch "Inception" --rating 9.0 --review "Mind-bending masterpiece" --review "Visual effects are incredible"
```

## With Tags

### Single Tag

```bash
i-rs-movie add "Blade Runner" --year 1982 --director "Ridley Scott" --tag sci-fi
i-rs-movie add "The Terminator" --year 1984 --director "James Cameron" --tag sci-fi
i-rs-movie add "Groundhog Day" --year 1993 --director "Harold Ramis" --tag comedy
```

### Multiple Tags

```bash
i-rs-movie add "Pulp Fiction" --year 1994 --director "Quentin Tarantino" --tag crime --tag dark-comedy --tag cult-classic

i-rs-movie add "Spirited Away" --year 2001 --director "Hayao Miyazaki" --tag animation --tag fantasy --tag japanese --tag masterpiece
```

### Filtering by Tags

```bash
# Show all sci-fi movies
i-rs-movie list --tag sci-fi

# Show all Japanese movies
i-rs-movie list --tag japanese

# Show all animation
i-rs-movie list --tag animation
```

## With Reviews

### Single Review

```bash
i-rs-movie add "The Godfather" --year 1972 --director "Francis Ford Coppola" --watched --rating 10 --review "The greatest film ever made"
```

### Multiple Reviews

```bash
i-rs-movie watch "2001: A Space Odyssey" \
  --rating 9.5 \
  --review "Groundbreaking visual effects" \
  --review "A bit slow at times" \
  --review "But the ending is worth it"
```

## Release Date

```bash
# Add with release date
i-rs-movie add "Dune" --year 2021 --director "Denis Villeneuve" --release-date 2021-10-22

# Add upcoming movie
i-rs-movie add "Dune: Part Two" --director "Denis Villeneuve" --release-date 2024-03-15
```

## Combining Options

### Complete Movie Entry

```bash
i-rs-movie add "Parasite" \
  --year 2019 \
  --director "Bong Joon-ho" \
  --watched \
  --rating 9.5 \
  --review "Brilliant social commentary" \
  --review "Perfect blend of genres" \
  --tag thriller \
  --tag dark-comedy \
  --tag korean \
  --tag masterpiece \
  --release-date 2019-05-30
```

### Building Your Watchlist

```bash
# Add movies you want to watch
i-rs-movie add "Citizen Kane" --year 1941 --director "Orson Welles" --tag classic
i-rs-movie add "Seven Samurai" --year 1954 --director "Akira Kurosawa" --tag classic --tag japanese
i-rs-movie add "12 Angry Men" --year 1957 --director "Sidney Lumet" --tag drama --tag classic
i-rs-movie add "The Dark Knight" --year 2008 --director "Christopher Nolan" --tag superhero --tag action

# Mark some as watched later
i-rs-movie watch "12 Angry Men" --rating 9.0 --review "Riveting from start to finish"
```

## Updating Movies

```bash
# Update rating after rewatching
i-rs-movie update "Inception" --rating 10

# Add more tags
i-rs-movie update "Pulp Fiction" --tag nonlinear

# Add review to existing movie
i-rs-movie update "The Matrix" --review "Changed my view on reality"

# Change director info
i-rs-movie update "The Matrix" --director "Lana Wachowski, Lilly Wachowski"
```

## Statistics

```bash
# View your movie collection stats
i-rs-movie stats
```

Output:
```
Movie Statistics
-----------------
Total: 25 movies
Watched: 18 movies
Unwatched: 7 movies
Average Rating: 8.2
```

## Building a Collection

### By Genre

```bash
# Sci-Fi Collection
i-rs-movie add "Blade Runner 2049" --year 2017 --tag sci-fi
i-rs-movie add "Arrival" --year 2016 --tag sci-fi
i-rs-movie add "Ex Machina" --year 2014 --tag sci-fi
i-rs-movie add "Interstellar" --year 2014 --tag sci-fi

# Action Collection
i-rs-movie add "Mad Max: Fury Road" --year 2015 --tag action
i-rs-movie add "John Wick" --year 2014 --tag action

# Animation Collection
i-rs-movie add "Your Name" --year 2016 --tag animation --tag japanese
i-rs-movie add "Spider-Man: Into the Spider-Verse" --year 2018 --tag animation
i-rs-movie add "Wall-E" --year 2008 --tag animation
```

### By Director

```bash
# Christopher Nolan
i-rs-movie add "Memento" --year 2000 --tag thriller
i-rs-movie add "The Dark Knight" --year 2008 --tag superhero
i-rs-movie add "Inception" --year 2010 --tag sci-fi
i-rs-movie add "Dunkirk" --year 2017 --tag war

# Quentin Tarantino
i-rs-movie add "Reservoir Dogs" --year 1992 --tag crime
i-rs-movie add "Pulp Fiction" --year 1994 --tag crime
i-rs-movie add "Kill Bill" --year 2003 --tag action
i-rs-movie add "Django Unchained" --year 2012 --tag western
```

## Workflow Examples

### Weekly Movie Night

```bash
# Add upcoming movie
i-rs-movie add "New Movie" --tag to-watch

# After watching
i-rs-movie watch "New Movie" --rating 8 --review "Great movie night choice"
```

### Monthly Review

```bash
# Check what you watched this month
i-rs-movie list --watched

# Check to-watch list
i-rs-movie list --unwatched

# Get overall stats
i-rs-movie stats
```

## Advanced Usage

### JSON Integration

```bash
# Export movie list
i-rs-movie list --json > movies.json

# Get specific movie details
i-rs-movie get "Inception" --json

# Get stats in JSON
i-rs-movie stats --json
```

### Custom Tags for Organization

```bash
# Personal ratings
i-rs-movie add "Movie A" --tag 5-stars
i-rs-movie add "Movie B" --tag 4-stars

# Time period
i-rs-movie add "Movie C" --tag 90s
i-rs-movie add "Movie D" --tag 2020s

# rewatch
i-rs-movie add "Movie E" --tag rewatch
i-rs-movie watch "Movie E" --rating 10 --review "Second viewing, even better"

# Gift from someone
i-rs-movie add "Movie F" --tag gift --remark "From John"
```
