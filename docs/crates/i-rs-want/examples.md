# i-rs-want Examples

## Basic Usage

### Adding Items

```bash
# Simple items
i-rs-want add "New Headphones"
i-rs-want add "Mechanical Keyboard"

# With price
i-rs-want add "New Headphones" --price 299.99
i-rs-want add "Book: Rust Programming" --price 49.99
i-rs-want add "Monitor" --price 499.99

# With URL
i-rs-want add "Keyboard" --url "https://example.com/keyboard" --price 149.99
```

### Setting Priority

```bash
# Priority levels
i-rs-want add "Essential Item" --price 50 --priority high
i-rs-want add "Nice to Have" --price 100 --priority medium
i-rs-want add "Someday Maybe" --price 500 --priority low
```

## Viewing Items

```bash
# List all
i-rs-want list

# List pending only
i-rs-want list --pending

# List completed
i-rs-want list --done

# Get details
i-rs-want get "New Headphones"
```

## Managing Items

```bash
# Mark as done (purchased)
i-rs-want update "New Headphones" --done

# Mark as pending again
i-rs-want update "Keyboard" --undone

# Update priority
i-rs-want update "Monitor" --priority high

# Delete item
i-rs-want delete "Old Item"
```

## Organized Wish Lists

```bash
# Tech gadgets
i-rs-want add "Wireless Earbuds" --price 199.99 --priority high --tag tech --tag audio
i-rs-want add "Smart Watch" --price 399.99 --priority medium --tag tech --tag wearable
i-rs-want add "Portable SSD" --price 120 --priority medium --tag tech

# Books
i-rs-want add "Clean Code" --price 49.99 --priority high --tag books --tag programming
i-rs-want add "The Pragmatic Programmer" --price 44.99 --priority high --tag books --tag programming
i-rs-want add "Science Fiction Novel" --price 15.99 --priority low --tag books

# Home
i-rs-want add "Plant Stand" --price 35 --priority low --tag home --tag decor
i-rs-want add "Desk Lamp" --price 80 --priority medium --tag home --tag office

# Gifts
i-rs-want add "Gift for Mom" --price 100 --priority high --tag gift
i-rs-want add "Birthday Present" --price 50 --priority medium --tag gift
```

## Birthday/Holiday Planning

```bash
# Things I want
i-rs-want add "New Headphones" --price 299.99 --priority high --tag birthday --tag wishlist
i-rs-want add "Mechanical Keyboard" --price 149.99 --priority high --tag birthday --tag wishlist
i-rs-want add "Books" --price 100 --priority medium --tag birthday

# Gifts for others
i-rs-want add "Gift for Dad" --price 150 --priority high --tag gift
i-rs-want add "Gift for Sister" --price 75 --priority medium --tag gift
```