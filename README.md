# end* (endptr)

Terminal UI HTTP client. Postman-compatible, keyboard-driven.

![Rust](https://img.shields.io/badge/rust-2024-orange)

## Features

- Send HTTP requests (GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS)
- Save/load requests in **Postman Collection v2.1** format
- Multiple collections — one `.json` file per collection in `.endptr/`
- Folders inside collections (nested groups)
- **Secrets** — define variables, use them in URLs/headers/body with `{{var}}` syntax
- **Auth** — None, Basic Auth, Bearer, OAuth 2.0, JWT
- **Query params** — toggle enabled/disabled per param
- Pretty-printed JSON responses
- Keyboard-only navigation

## Layout

```
┌──────────────┬──────────────────────────────────────────────────┐
│ Requests     │ [GET    ▾] [ {{api}}/users?page=1      ] [ ▶ ]  │
│ ▼ api.json   ├──────────────────────────────────────────────────┤
│   ▶ Auth     │ Body  Headers  Auth  Params                      │
│   ▼ Users    │ ──────────────────────────────────────────────── │
│     GET List │ {                                                │
│     POST New │   "users": [...]                                 │
│──────────────│ }                                                │
│ Secrets      │                                                  │
│ api          │ 200 OK  42ms                                     │
└──────────────┴──────────────────────────────────────────────────┘
```

## Storage

```
.endptr/
├── collection.json   ← default collection (Postman v2.1)
├── my-api.json       ← additional collections
└── secrets.json      ← key-value variables
```

Collections are fully compatible with Postman — import/export directly.

## Secrets & Templates

Define secrets in `.endptr/secrets.json`:

```json
{
  "api": "http://localhost:8080",
  "token": "my-secret-token"
}
```

Use them anywhere — URL, headers, body, params:

```
{{api}}/users          →  http://localhost:8080/users
Authorization: {{token}}
```

## Syntax Highlighting

JSON is highlighted automatically — in response body and in the Body tab (when not editing).

| Token | Color |
|-------|-------|
| Keys | Cyan |
| String values | Green |
| Numbers | Yellow |
| `true` / `false` | Magenta |
| `null` | Dark gray |
| `{ } [ ] : ,` | White |

Body tab behavior:
- **Focused** → plain editable input with cursor
- **Not focused** → highlighted + pretty-printed (falls back to plain text if not valid JSON)

## Keybinds

### Global

| Key | Action |
|-----|--------|
| `Ctrl+Q` | Quit |
| `?` | Toggle help |
| `Tab` / `Shift+Tab` | Cycle panel focus |
| `Ctrl+Enter` / `F5` | Send request |
| `Ctrl+S` | Save current request (prompts for name) |

### Sidebar (Requests)

| Key | Action |
|-----|--------|
| `j` / `k` or `↑` / `↓` | Navigate list |
| `Enter` | Load request / expand or collapse folder |
| `n` | New request (or new collection if on collection node) |
| `f` | New folder |
| `d` | Delete selected (confirms first) |
| `s` | Switch to Secrets tab |

### Sidebar (Secrets)

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate |
| `Enter` | Edit secret value |
| `n` | Add secret |
| `d` | Delete secret |
| `s` | Switch to Requests tab |

### Method selector

| Key | Action |
|-----|--------|
| `←` / `→` or `h` / `l` | Cycle HTTP method |
| `Enter` | Send request |

### URL bar

| Key | Action |
|-----|--------|
| Type | Edit URL |
| `←` / `→` / `Home` / `End` | Move cursor |
| `Enter` | Send request |

### Request panel

| Key | Action |
|-----|--------|
| `[` / `]` | Cycle tabs (Body → Headers → Auth → Params) |

**Body tab:**

| Key | Action |
|-----|--------|
| Type | Edit body |
| `y` | Copy body to clipboard |

**Headers / Params tabs:**

| Key | Action |
|-----|--------|
| `j` / `k` | Navigate list |
| `n` | Add entry |
| `d` | Delete selected |
| `Space` | Toggle param enabled/disabled (Params tab only) |

**Auth tab:**

| Key | Action |
|-----|--------|
| `←` / `→` | Change auth type (None / Basic Auth / Bearer / OAuth 2.0 / JWT) |
| `Tab` / `↓` | Next input field |
| `Shift+Tab` / `↑` | Previous field |
| Type | Edit focused field |

### Response panel

| Key | Action |
|-----|--------|
| `j` / `k` or `↑` / `↓` | Scroll |
| `y` | Copy response body to clipboard |

## Auth types

| Type | Sends |
|------|-------|
| None | — |
| Basic Auth | `Authorization: Basic base64(user:pass)` |
| Bearer | `Authorization: Bearer <token>` |
| OAuth 2.0 | `Authorization: Bearer <token>` |
| JWT | `Authorization: Bearer <token>` |

Auth is saved in Postman's `auth` field format.

## Build & Run

```bash
cargo run
```

Requires Rust 2024 edition (rustc ≥ 1.85).
