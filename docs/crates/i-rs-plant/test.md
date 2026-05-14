# Test Records

## Test Plant Data

Use these test commands to verify i-rs-plant installation:

### Test 1: Add plants

```bash
i-rs-plant add --name "Monstera" --species "Monstera deliciosa" --location "Living Room" --interval 7
i-rs-plant add --name "Snake Plant" --species "Sansevieria trifasciata" --location "Office" --interval 14
i-rs-plant add --name "Fiddle Leaf Fig" --species "Ficus lyrata" --location "Bedroom" --interval 10
```

### Test 2: List plants

```bash
i-rs-plant list
```

Expected: 3 plants displayed in table format

### Test 3: Get plant details

```bash
i-rs-plant get Monstera
```

Expected: Detailed info about Monstera

### Test 4: Water a plant

```bash
i-rs-plant water Monstera
```

### Test 5: View statistics

```bash
i-rs-plant stats
```

### Test 6: JSON output

```bash
i-rs-plant list --json
i-rs-plant get Snake Plant --json
```

### Test 7: Update plant

```bash
i-rs-plant update Monstera --location "Hallway" --interval 5
```

### Test 8: Delete plants

```bash
i-rs-plant delete Fiddle Leaf Fig
i-rs-plant delete Snake Plant
i-rs-plant delete Monstera
```

### Test 9: Verify empty state

```bash
i-rs-plant list
```

Expected: "No plants found."

## Verification Checklist

- [ ] `i-rs-plant --help` works
- [ ] `i-rs-plant add` creates a plant
- [ ] `i-rs-plant list` displays plants
- [ ] `i-rs-plant get` shows details
- [ ] `i-rs-plant water` updates last watered
- [ ] `i-rs-plant stats` shows statistics
- [ ] `i-rs-plant --json` outputs JSON
- [ ] `i-rs-plant update` modifies plants
- [ ] `i-rs-plant delete` removes plants
- [ ] Data persists after restart
