# i-rs-gift Examples

## Basic Usage

### Add a Gift You Gave

```bash
i-rs-gift add "Birthday Watch" sent "Mom" birthday 500 2024-12-25 --tag family
```

### Add a Gift You Received

```bash
i-rs-gift add "AirPods Pro" received "Boss" christmas 1200 2024-12-25 --tag work
```

### Add Gift with Remarks

```bash
i-rs-gift add "Rose Bouquet" sent "Wife" anniversary 300 2024-02-14 --tag romance --remark "Red roses"
```

## Listing Gifts

### List All Gifts

```bash
i-rs-gift list
```

### List Only Sent Gifts

```bash
i-rs-gift list --type sent
```

### List Only Received Gifts

```bash
i-rs-gift list --type received
```

### List Gifts by Tag

```bash
i-rs-gift list --tag family
i-rs-gift list --tag work
```

## Viewing Details

### Get Gift Details

```bash
i-rs-gift get "Birthday Watch"
```

### Get Gift Details in JSON

```bash
i-rs-gift get "Birthday Watch" --json
```

## Statistics

### View Gift Statistics

```bash
i-rs-gift stats
```

Output shows:
- Total gifts sent/received
- Total and average values
- Balance (received - sent)
- Top occasions
- Top recipients

## Managing Gifts

### Delete a Gift

```bash
i-rs-gift delete "Birthday Watch"
```

## JSON Output

All commands support `--json` flag:

```bash
i-rs-gift list --json
i-rs-gift stats --json
```
