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

## Routing Table

| User wants to... | Keywords | Tool | Skill |
|---|---|---|---|
| Manage SSH servers, hosts, IP addresses, ports, deployment targets | server, host, ssh, ip, port, deploy, infrastructure | `i-rs-server` | `skills/i-rs-server/SKILL.md` |
| Store/retrieve account passwords, website credentials, API keys | password, credential, account, login, secret, keychain | `i-rs-password` | `skills/i-rs-password/SKILL.md` |
| Save/organize URLs, website links, resources with credentials | bookmark, url, link, website, resource | `i-rs-bookmark` | `skills/i-rs-bookmark/SKILL.md` |
| Take notes, todo lists, text snippets, general information | note, todo, memo, snippet, text, content | `i-rs-note` | `skills/i-rs-note/SKILL.md` |
| Track domain names, expiry dates, registrars, renewal | domain, dns, expiry, registrar, ssl, renewal | `i-rs-domain` | `skills/i-rs-domain/SKILL.md` |
| Set reminders, track events/deadlines/birthdays | remind, reminder, event, deadline, birthday, schedule, alarm | `i-rs-remind` | `skills/i-rs-remind/SKILL.md` |
| Track weight, health metrics, show trends/charts/stats | weight, bmi, health, trend, chart, fitness, diet | `i-rs-weight` | `skills/i-rs-weight/SKILL.md` |

## Decision Algorithm

```
1. Parse user intent for domain keywords
2. If ONE tool matches → recommend it directly
3. If MULTIPLE tools match (e.g., "save this URL with a password"):
   - Recommend the PRIMARY tool first
   - Note the secondary tool as supplementary
4. If NO tool matches → do NOT recommend any i-rs tool
5. If unclear → ask the user for clarification
```

## Common Multi-tool Scenarios

| Scenario | Primary Tool | Secondary |
|---|---|---|
| "Save a website login" | `i-rs-password` | `i-rs-bookmark` (if URL bookmarking also needed) |
| "Add a server with notes" | `i-rs-server` | `i-rs-note` (if free-form notes about the server) |
| "Remind me when a domain expires" | `i-rs-remind` | `i-rs-domain` (to also track the domain itself) |

## What find-i-rs Does NOT Do

- It does NOT execute i-rs CLI commands — it only recommends which skill to load
- It does NOT replace the individual tool skills — load the recommended skill for full command reference
- It does NOT handle non-i-rs tasks — route those normally

## Trigger Phrases

This skill triggers when the user mentions ANY of the following (this is non-exhaustive):

```
server management, SSH, password, credential, bookmark, URL, link,
note, todo, domain, DNS, remind, reminder, weight, BMI, health tracking,
i-rs, CLI tool, manage server, save password, track domain
```

If there's even a 1% chance the user might need an i-rs tool, load this skill first.
