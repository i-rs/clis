# i-rs-api

REST API server for i-rs CLI tools.

## Features

- RESTful API for all i-rs tools
- Directly calls existing CLI tools via subprocess
- Returns JSON output from CLI tools
- Easy to extend for any new CLI tool

## Install

```bash
cargo build -p i-rs-api
# or
cargo install --path crates/i-rs-api
```

## Run

```bash
# Start the server
./target/debug/i-rs-api
# or
cargo run -p i-rs-api

# Server starts on http://127.0.0.1:8080
```

## API Endpoints

### Health Check

```bash
GET /
```

### Todo

```bash
GET    /api/todo              # List all todos
POST   /api/todo              # Add a todo
GET    /api/todo/:name        # Get a todo
PUT    /api/todo/:name        # Update a todo
POST   /api/todo/:name/done   # Mark todo as done
DELETE /api/todo/:name        # Delete a todo
```

### Weight

```bash
GET    /api/weight            # List all weight records
POST   /api/weight            # Add a weight record
GET    /api/weight/:date      # Get weight for date
GET    /api/weight/stats      # Get weight statistics
```

### Habit

```bash
GET    /api/habit             # List all habits
POST   /api/habit             # Add a habit
GET    /api/habit/:name      # Get a habit
POST   /api/habit/:name/checkin  # Checkin a habit
GET    /api/habit/:name/stats    # Get habit statistics
```

### Note

```bash
GET    /api/note              # List all notes
POST   /api/note              # Add a note
GET    /api/note/:name        # Get a note
DELETE /api/note/:name        # Delete a note
```

### Bookmark

```bash
GET    /api/bookmark              # List all bookmarks
POST   /api/bookmark              # Add a bookmark
GET    /api/bookmark/:name        # Get a bookmark
DELETE /api/bookmark/:name        # Delete a bookmark
```

### Mood

```bash
GET    /api/mood              # List all moods
POST   /api/mood              # Add a mood
GET    /api/mood/:date        # Get mood for date
DELETE /api/mood/:date        # Delete mood
GET    /api/mood/stats        # Get mood statistics
```

## Usage Examples

### List todos

```bash
curl http://localhost:8080/api/todo
```

### Add a todo

```bash
curl -X POST http://localhost:8080/api/todo \
  -H "Content-Type: application/json" \
  -d '{"name": "buy-milk", "title": "Buy Milk", "priority": "high"}'
```

### Get a specific todo

```bash
curl http://localhost:8080/api/todo/buy-milk
```

### Add weight

```bash
curl -X POST http://localhost:8080/api/weight \
  -H "Content-Type: application/json" \
  -d '{"weight": 70.5}'
```

### List habits

```bash
curl http://localhost:8080/api/habit
```

### Checkin a habit

```bash
curl -X POST http://localhost:8080/api/habit/exercise/checkin
```

## Architecture

The API server acts as a thin wrapper around the existing i-rs CLI tools:

```
HTTP Request → API Handler → CLI Command → JSON Output → HTTP Response
```

Benefits:
- Zero code duplication
- Data consistency with CLI tools
- Easy to add new endpoints

## License

MIT OR Apache-2.0
