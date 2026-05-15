---
name: "find-i-rs"
description: "Recommends the correct i-rs CLI tool for a given task. Read this to determine which i-rs tool to invoke."
---

# find-i-rs

Analyze the user's request and recommend the appropriate i-rs CLI tool skill.

## How to Use

When you receive a task that might involve an i-rs CLI tool, follow this decision flow:

1. **Identify keywords** in the user's request
2. **Match against the routing table** below
3. **Load the recommended skill** via the `skill` tool
4. **Follow that skill's instructions** to implement the solution

## Routing Table (41 Tools)

### Core Tools
| User wants to... | Keywords | Tool | Skill |
|---|---|---|---|
| Manage SSH servers, hosts, IP addresses, ports | server, host, ssh, ip, port, deploy, infrastructure | `i-rs-server` | `skills/i-rs-server/SKILL.md` |
| Store/retrieve passwords, credentials, secrets | password, credential, account, login, secret, keychain | `i-rs-password` | `skills/i-rs-password/SKILL.md` |
| Save/organize URLs, website links, resources | bookmark, url, link, website, resource | `i-rs-bookmark` | `skills/i-rs-bookmark/SKILL.md` |
| Take notes, text snippets, general information | note, memo, snippet, text, content | `i-rs-note` | `skills/i-rs-note/SKILL.md` |
| Manage todo lists, tasks, mark as done | todo, task, done, checklist, pending | `i-rs-todo` | `skills/i-rs-todo/SKILL.md` |

### Health Tracking
| User wants to... | Keywords | Tool | Skill |
|---|---|---|---|
| Track weight, BMI, health metrics, trends | weight, bmi, health, trend, chart, fitness, diet | `i-rs-weight` | `skills/i-rs-weight/SKILL.md` |
| Track mood, emotions, mental health | mood, emotion, feeling, mental, happy, sad | `i-rs-mood` | `skills/i-rs-mood/SKILL.md` |
| Track water intake, hydration | water, drink, hydration, glass, ml | `i-rs-water` | `skills/i-rs-water/SKILL.md` |
| Track daily steps, walking | step, walk, activity, exercise | `i-rs-step` | `skills/i-rs-step/SKILL.md` |
| Track medicine dosage, pill intake | dose, medicine, pill, medication, take drug | `i-rs-dose` | `skills/i-rs-dose/SKILL.md` |
| Track menstrual cycle, period | cycle, period, menstrual, ovulation, period tracker | `i-rs-cycle` | `skills/i-rs-cycle/SKILL.md` |
| Track sedentary time, sit reminders | sit, sedentary, break, stand up, reminder | `i-rs-sit` | `skills/i-rs-sit/SKILL.md` |
| Track allergy attacks, symptoms | allergy, attack, symptom, reaction | `i-rs-allergy` | `skills/i-rs-allergy/SKILL.md` |
| Track calorie intake, food calories | cal, calorie, food, diet, nutrition | `i-rs-cal` | `skills/i-rs-cal/SKILL.md` |
| Track fasting periods, intermittent fast | fast, fasting, intermittent, diet | `i-rs-fast` | `skills/i-rs-fast/SKILL.md` |
| Track habits, daily routines, streaks | habit, routine, streak, daily, checkin | `i-rs-habit` | `skills/i-rs-habit/SKILL.md` |
| Track sleep patterns, duration, quality | sleep, bedtime, wake, rest, quality | `i-rs-sleep` | `skills/i-rs-sleep/SKILL.md` |

### Reminders & Expiry
| User wants to... | Keywords | Tool | Skill |
|---|---|---|---|
| Track domain names, expiry dates, SSL | domain, dns, expiry, registrar, ssl, renewal | `i-rs-domain` | `skills/i-rs-domain/SKILL.md` |
| Set reminders, track events, deadlines | remind, reminder, event, deadline, birthday, schedule | `i-rs-remind` | `skills/i-rs-remind/SKILL.md` |
| Track subscriptions, renewal dates | sub, subscription, renew, billing, plan | `i-rs-sub` | `skills/i-rs-sub/SKILL.md` |
| Track item purchase dates, replacement cycles | bestby, purchase, replace, expiry date | `i-rs-bestby` | `skills/i-rs-bestby/SKILL.md` |

### Finance & Data
| User wants to... | Keywords | Tool | Skill |
|---|---|---|---|
| Track accounting, transactions, income/expense | ledger, accounting, transaction, money, budget | `i-rs-ledger` | `skills/i-rs-ledger/SKILL.md` |
| Track recurring expenses, bills | recur, recurring, fixed expense, bill | `i-rs-recur` | `skills/i-rs-recur/SKILL.md` |
| Store key-value data, configuration | kv, key-value, store, config, data | `i-rs-kv` | `skills/i-rs-kv/SKILL.md` |
| Store API keys, tokens securely in keychain | keys, api key, token, credential, secure | `i-rs-keys` | `skills/i-rs-keys/SKILL.md` |

### Food & Lifestyle
| User wants to... | Keywords | Tool | Skill |
|---|---|---|---|
| Track meals, breakfast/lunch/dinner | meal, food, eat, breakfast, lunch, dinner | `i-rs-meal` | `skills/i-rs-meal/SKILL.md` |
| Track cravings, junk food, snack attacks | pig, crave, junk food, snack, binge | `i-rs-pig` | `skills/i-rs-pig/SKILL.md` |
| Manage grocery shopping list | grocery, shopping list, food list, purchase | `i-rs-grocery` | `skills/i-rs-grocery/SKILL.md` |

### Time & Productivity
| User wants to... | Keywords | Tool | Skill |
|---|---|---|---|
| Track duration, time spent, focus sessions | tick, time, duration, timer, focus, pomodoro | `i-rs-tick` | `skills/i-rs-tick/SKILL.md` |
| Capture ideas, inspiration, creative thoughts | spark, idea, inspiration, creative, thought | `i-rs-spark` | `skills/i-rs-spark/SKILL.md` |
| Track wish list, goals, things to buy | want, wish, goal, desire, shopping list | `i-rs-want` | `skills/i-rs-want/SKILL.md` |

### Home & Care
| User wants to... | Keywords | Tool | Skill |
|---|---|---|---|
| Track bedsheet changes, laundry | sheet, bedsheet, change, laundry | `i-rs-sheet` | `skills/i-rs-sheet/SKILL.md` |
| Track toothbrush replacement | toothbrush, replace, dental, oral | `i-rs-toothbrush` | `skills/i-rs-toothbrush/SKILL.md` |
| Track towel changes | towel, change, linen | `i-rs-towel` | `skills/i-rs-towel/SKILL.md` |
| Track mattress/pillow replacement | bed, mattress, pillow, replace | `i-rs-bed` | `skills/i-rs-bed/SKILL.md` |
| Track air conditioner cleaning | ac, air conditioner, clean, maintenance | `i-rs-ac` | `skills/i-rs-ac/SKILL.md` |
| Track appliance filter cleaning | filter, clean, appliance, maintenance | `i-rs-filter` | `skills/i-rs-filter/SKILL.md` |
| Track water purifier filter replacement | purify, water filter, replace, cartridge | `i-rs-purify` | `skills/i-rs-purify/SKILL.md` |

### Pet Care
| User wants to... | Keywords | Tool | Skill |
|---|---|---|---|
| Track pet feeding, meals | feedpet, pet food, feed, meal | `i-rs-feedpet` | `skills/i-rs-feedpet/SKILL.md` |
| Track pet bathing, grooming | petbath, bath, groom, clean | `i-rs-petbath` | `skills/i-rs-petbath/SKILL.md` |
| Track dog walking, exercise | walkdog, walk, exercise, pet | `i-rs-walkdog` | `skills/i-rs-walkdog/SKILL.md` |
| Track aquarium maintenance, water change | aqua, aquarium, fish tank, water change | `i-rs-aqua` | `skills/i-rs-aqua/SKILL.md` |

## Decision Algorithm

```
1. Parse user intent for domain keywords
2. If ONE tool matches → recommend it directly
3. If MULTIPLE tools match:
   - Recommend the PRIMARY tool first
   - Note secondary tools as supplementary
4. If NO tool matches → do NOT recommend any i-rs tool
5. If unclear → ask the user for clarification
```

## Common Multi-tool Scenarios

| Scenario | Primary Tool | Secondary |
|---|---|---|
| "Save a website login" | `i-rs-password` | `i-rs-bookmark` (if URL bookmarking needed) |
| "Add a server with notes" | `i-rs-server` | `i-rs-note` (if free-form notes needed) |
| "Remind me when a domain expires" | `i-rs-domain` | `i-rs-remind` (for reminder notification) |
| "Track meal calories" | `i-rs-meal` | `i-rs-cal` (for calorie estimation) |
| "Track fasting with water intake" | `i-rs-fast` | `i-rs-water` (for hydration tracking) |

## What find-i-rs Does NOT Do

- It does NOT execute i-rs CLI commands — it only recommends which skill to load
- It does NOT replace the individual tool skills — load the recommended skill for full command reference
- It does NOT handle non-i-rs tasks — route those normally

## Trigger Phrases

This skill triggers when the user mentions ANY of the following:

```
server, ssh, password, credential, bookmark, url, link,
note, todo, domain, dns, remind, reminder, weight, bmi,
health, mood, water, step, dose, medicine, pill,
cycle, period, sit, sedentary, allergy, cal, calorie,
fast, fasting, habit, routine, streak, sleep, bedtime,
sub, subscription, renew, bestby,
purchase, grocery, shopping list, ledger, accounting, money, recur, kv, keys,
api key, meal, food, pig, crave, tick, time, spark,
idea, want, wish, sheet, bedsheet, toothbrush, towel,
bed, mattress, ac, filter, purify, feedpet, pet,
walkdog, aqua, aquarium, i-rs, CLI, terminal, track
```

If there's even a 1% chance the user might need an i-rs tool, load this skill first.

### data

Manage data (export, import, clear).

```bash
i-rs-find-i-rs data export
i-rs-find-i-rs data import [FILE]
i-rs-find-i-rs data clear
```

### example

Show usage examples.

```bash
i-rs-find-i-rs example
```

### skill

Show skill information.

```bash
i-rs-find-i-rs skill [summary|content|raw]
```

## Examples

```bash
# JSON output
i-rs-find-i-rs list --json
```
