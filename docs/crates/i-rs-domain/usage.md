# i-rs-domain Usage Guide

## Commands Overview

| Command | Description |
|---------|-------------|
| `add` | Add a new domain |
| `list` | List all domains |
| `get` | Get domain details |
| `update` | Update a domain |
| `delete` | Delete a domain |

## add

Add a new domain.

```bash
i-rs-domain add <NAME> <EXPIRY_DATE> [OPTIONS]
```

### Arguments

| Argument | Description | Required |
|----------|-------------|----------|
| `NAME` | Domain name | Yes |
| `EXPIRY_DATE` | Expiry date (YYYY-MM-DD) | Yes |

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-r` | `--registrar` | Domain registrar (optional) |
| `-p` | `--password` | Registrar password (stored in keychain, optional) |
| `-t` | `--tag` | Tags (can be specified multiple times) |
| `-m` | `--remark` | Remarks (can be specified multiple times) |

---

## list

List all domains, optionally filtered by tag.

```bash
i-rs-domain list [OPTIONS]
```

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-t` | `--tag` | Filter by tag |

---

## get

Get detailed information about a domain.

```bash
i-rs-domain get <NAME> [OPTIONS]
```

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-s` | `--show-password` | Show password from keychain |

---

## update

Update an existing domain.

```bash
i-rs-domain update <NAME> [OPTIONS]
```

### Options

| Short | Long | Description |
|-------|------|-------------|
| `-e` | `--expiry-date` | New expiry date (YYYY-MM-DD) |
| `-r` | `--registrar` | New registrar |
| `-p` | `--password` | New password (stored in keychain) |
| `-t` | `--tag` | New tags |
| `-m` | `--remark` | New remarks |

---

## delete

Delete a domain.

```bash
i-rs-domain delete <NAME>
```