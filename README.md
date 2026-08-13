# Messaging & Notifications MCP Server

[![Crates.io](https://img.shields.io/crates/v/mcp-messaging.svg)](https://crates.io/crates/mcp-messaging)
[![Docs.rs](https://docs.rs/mcp-messaging/badge.svg)](https://docs.rs/mcp-messaging)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)
[![ADK-Rust Enterprise](https://img.shields.io/badge/ADK--Rust-Enterprise-purple.svg)](https://enterprise.adk-rust.com)
[![Registry Ready](https://img.shields.io/badge/ADK_Registry-Ready-green.svg)](https://enterprise.adk-rust.com)

Full-stack messaging for [ADK-Rust Enterprise](https://enterprise.adk-rust.com) agents. Provides 20 MCP tools covering push notifications, SMS (4 regional providers), webhooks, in-app messaging with rich media, message queues, and broadcasts.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                        mcp-messaging (20 tools)                      │
├──────────┬──────────┬──────────┬──────────────┬─────────────────────┤
│   Push   │   SMS    │ Webhooks │   In-App     │      Queues         │
├──────────┼──────────┼──────────┼──────────────┼─────────────────────┤
│ ntfy.sh  │ Twilio   │ Any URL  │ Channels     │ Priority queues     │
│ Firebase │ AT (🌍)  │          │ Rich media   │ Delayed delivery    │
│          │ Vonage(🇪🇺)│         │ Read receipts│ Dequeue/status      │
│          │ Sinch(🌏) │          │ Recall       │                     │
│          │          │          │ Formatting   │                     │
└──────────┴──────────┴──────────┴──────────────┴─────────────────────┘
```

## Key Principles

- **Global SMS coverage** — Africa's Talking, Twilio, Vonage, Sinch cover every continent.
- **Free push notifications** — ntfy.sh requires zero configuration or API keys.
- **Rich messaging** — HTML, images, video, audio, files, locations, reply threading.
- **Message lifecycle** — send → deliver → read receipt → recall.
- **Built-in queues** — priority-based message queues for async job dispatch.
- **Registry-ready** — ships with `mcp-server.toml` for automatic ADK-Rust Enterprise onboarding.

## Tools (20)

### Push Notifications

| Tool | Provider | Description |
|------|----------|-------------|
| `send_push` | ntfy.sh | Send push notification (free, no auth) |
| `broadcast` | ntfy.sh | Broadcast to multiple topics at once |
| `send_fcm` | Firebase | Send via Google Firebase Cloud Messaging |

### SMS

| Tool | Provider | Coverage |
|------|----------|----------|
| `send_sms` | Twilio | Americas, global |
| `send_sms_africa` | Africa's Talking | Kenya, Nigeria, Uganda, Tanzania, Rwanda, Ghana, South Africa |
| `send_sms_europe` | Vonage/Nexmo | Europe, UK, 200+ countries |
| `send_sms_asia` | Sinch | Australia, India, Japan, Singapore, 200+ countries |

### Webhooks

| Tool | Description |
|------|-------------|
| `fire_webhook` | POST JSON payload to any URL with event headers |
| `subscribe_webhook` | Subscribe a URL to receive topic notifications |

### In-App Messaging

| Tool | Description |
|------|-------------|
| `create_channel` | Create direct, group, or broadcast channel |
| `send_message` | Send text, HTML, image, video, audio, file, or location |
| `get_messages` | Retrieve channel message history |
| `get_message_status` | Check sent/delivered/read/recalled status |
| `mark_as_read` | Set read receipt for a message |
| `recall_message` | Unsend a message (clears content) |
| `format_message` | Generate styled HTML with fonts, colors, backgrounds |

### Message Queues

| Tool | Description |
|------|-------------|
| `enqueue` | Add message to priority queue (with optional delay) |
| `dequeue` | Remove and return oldest visible messages |
| `queue_status` | Get queue depth and age |
| `set_queue_priority` | Update priority of queued message |

## Installation

### From crates.io

```bash
cargo install mcp-messaging
```

### Build from source

```bash
git clone https://github.com/zavora-ai/mcp-messaging
cd mcp-messaging
cargo build --release
```

### Claude Desktop

```json
{
  "mcpServers": {
    "messaging": {
      "command": "mcp-messaging",
      "env": {
        "AT_USERNAME": "your_username",
        "AT_API_KEY": "your_key"
      }
    }
  }
}
```

### Kiro

Add to `.kiro/settings/mcp.json`:

```json
{
  "mcpServers": {
    "messaging": {
      "command": "mcp-messaging",
      "env": {
        "FCM_SERVER_KEY": "your_firebase_key"
      }
    }
  }
}
```

### Cursor

Add to `.cursor/mcp.json`:

```json
{
  "mcpServers": {
    "messaging": { "command": "mcp-messaging" }
  }
}
```

## Configuration

### Environment Variables

| Variable | Provider | Required |
|----------|----------|:--------:|
| `NTFY_SERVER` | ntfy.sh (default: https://ntfy.sh) | No |
| `FCM_SERVER_KEY` | Firebase Cloud Messaging | No |
| `TWILIO_ACCOUNT_SID` | Twilio | No |
| `TWILIO_AUTH_TOKEN` | Twilio | No |
| `TWILIO_FROM_NUMBER` | Twilio | No |
| `AT_USERNAME` | Africa's Talking | No |
| `AT_API_KEY` | Africa's Talking | No |
| `VONAGE_API_KEY` | Vonage/Nexmo | No |
| `VONAGE_API_SECRET` | Vonage/Nexmo | No |
| `SINCH_SERVICE_PLAN_ID` | Sinch | No |
| `SINCH_API_TOKEN` | Sinch | No |

All providers are optional. The server starts with zero configuration — push notifications via ntfy.sh and in-app messaging work immediately. SMS providers activate when their env vars are set.

## Quick Start

### Send a push notification (zero config)

```json
{
  "name": "send_push",
  "arguments": {
    "topic": "rider-james-4521",
    "title": "Ride Accepted",
    "message": "Driver John is 3 minutes away. Toyota Vitz, KBZ 123A",
    "priority": 4,
    "tags": ["car", "white_check_mark"]
  }
}
```

### Send rich media message

```json
{
  "name": "send_message",
  "arguments": {
    "channel": "trip-4521",
    "sender": "driver_john",
    "text": "Voice note",
    "msg_type": "audio",
    "media_url": "https://cdn.example.com/voice/msg123.ogg",
    "mime_type": "audio/ogg",
    "duration_seconds": 12
  }
}
```

### Format a styled message

```json
{
  "name": "format_message",
  "arguments": {
    "text": "Your ride is confirmed!",
    "font": "Georgia",
    "font_size": 18,
    "color": "#FFFFFF",
    "background": "#4CAF50",
    "bold": true,
    "align": "center"
  }
}
```

**Output:**

```html
<div style="font-family:'Georgia';font-size:18px;color:#FFFFFF;background-color:#4CAF50;font-weight:bold;text-align:center;padding:8px">Your ride is confirmed!</div>
```

### Send SMS to Africa

```json
{
  "name": "send_sms_africa",
  "arguments": {
    "to": "+254712345678",
    "message": "Your ride is arriving in 2 minutes. Driver: John, KBZ 123A"
  }
}
```

### Enqueue a job

```json
{
  "name": "enqueue",
  "arguments": {
    "queue": "ride_requests",
    "payload": {
      "rider": "james",
      "pickup": [-1.2921, 36.8219],
      "dropoff": [-1.1631, 36.9519]
    },
    "priority": 5
  }
}
```

### Mark message as read

```json
{
  "name": "mark_as_read",
  "arguments": {
    "message_id": "msg_a0e4fa8c",
    "channel": "trip-4521",
    "reader": "rider_james"
  }
}
```

### Recall a message

```json
{
  "name": "recall_message",
  "arguments": {
    "message_id": "msg_a0e4fa8c",
    "channel": "trip-4521"
  }
}
```

## Message Types

| Type | Required Fields | Optional Fields |
|------|----------------|-----------------|
| `text` | `text` | `reply_to` |
| `html` | `text` (HTML content) | `reply_to` |
| `image` | `text`, `media_url` | `mime_type`, `thumbnail_url` |
| `video` | `text`, `media_url` | `mime_type`, `thumbnail_url`, `duration_seconds` |
| `audio` | `text`, `media_url` | `mime_type`, `duration_seconds` |
| `file` | `text`, `media_url` | `mime_type`, `file_name`, `file_size` |
| `location` | `text`, `lat`, `lon` | `metadata` |
| `contact` | `text` | `metadata` (phone, email) |
| `system` | `text` | — |

## Message Lifecycle

```
send_message → delivered → mark_as_read → [recall_message]
     │              │            │                │
     ▼              ▼            ▼                ▼
  msg_id      get_messages   read_by[]     "[Message recalled]"
              get_status     timestamps     content cleared
```

## SMS Coverage Map

| Region | Provider | Countries |
|--------|----------|-----------|
| 🌍 Africa | Africa's Talking | KE, NG, UG, TZ, RW, GH, ZA, ET, CI, SN, CM, MW, ZM, BJ, BF |
| 🇺🇸 Americas | Twilio | US, CA, BR, MX, AR, CO, CL, PE |
| 🇪🇺 Europe | Vonage | UK, DE, FR, ES, IT, NL, SE, NO, PL, 200+ |
| 🌏 Asia-Pacific | Sinch | AU, IN, JP, SG, KR, TH, PH, ID, MY, 200+ |

## Use Cases

### Ride-Hailing App
```
Ride accepted → send_push (rider) + send_fcm (mobile app)
Driver arriving → send_sms_africa (fallback for no internet)
In-trip chat → create_channel + send_message (voice notes, location)
Trip complete → format_message (receipt HTML) + fire_webhook (billing)
```

### E-Commerce
```
Order placed → enqueue (fulfillment queue)
Shipped → send_push + send_sms
Delivered → mark_as_read (delivery confirmation)
Support chat → create_channel + send_message (images of damaged item)
```

### Team Communication
```
Announcement → broadcast (all team topics)
Direct message → create_channel (direct) + send_message
Read tracking → mark_as_read
Mistake → recall_message
```

## MCP Server Manifest

```toml
server_id = "mcp_messaging"
display_name = "Messaging"
version = "1.5.0"
domain = "messaging"
risk_level = "medium"
writes_allowed = "gated"
governance_gates = ["rate_limit_required"]
```

## Documentation

| Document | Description |
|----------|-------------|
| [mcp-server.toml](mcp-server.toml) | ADK-Rust Enterprise registry manifest |
| [Rust Docs](https://docs.rs/mcp-messaging) | Generated API documentation |

## Contributing

Contributions welcome. Priority areas:
- Apple Push Notification Service (APNs) support
- Message encryption (end-to-end)
- Typing indicators
- Message reactions/emoji
- Read receipt aggregation (group chats)
- Persistent storage backend (Redis, PostgreSQL)

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START -->
| [<img src="https://github.com/jkmaina.png" width="80px;" alt=""/><br /><sub><b>James Karanja Maina</b></sub>](https://github.com/jkmaina) |
|:---:|
<!-- ALL-CONTRIBUTORS-LIST:END -->

## License

Apache-2.0 — see [LICENSE](LICENSE) for details.

---

Part of the [ADK-Rust Enterprise](https://enterprise.adk-rust.com) MCP server ecosystem.

Built with ❤️ by [Zavora AI](https://zavora.ai)

## Registry Compliance

This server implements the [ADK MCP SDK](https://crates.io/crates/adk-mcp-sdk) contract:

- **HealthCheck** — async health probe for registry monitoring
- **mcp-server.toml** — manifest declaring tools, risk classes, and credentials
- **Structured tracing** — `RUST_LOG` env-filter for observability

## rmcp and MCP compatibility

This server is built with [`rmcp` 3.1.2](https://github.com/modelcontextprotocol/rust-sdk/releases/tag/rmcp-v3.1.2) and requires Rust 1.94.1 or newer. The rmcp 3 rollout retains legacy MCP initialization compatibility and targets MCP protocol revisions `2025-11-25` and `2026-07-28`.

## MCP 2026-07-28 rollout (P4 workflow/business)

This server uses `rmcp` 3.1.2 and `adk-mcp-sdk` 0.2 with a minimum supported
Rust version of **1.94.1**. It accepts stateless MCP 2026 requests with
per-request protocol, client identity, and capability metadata while retaining
the legacy MCP 2025-11-25 initialize flow for ordinary tools.

- **Tasks:** None; this server's operations are short-lived and execute directly.
- **MRTR approvals:** `send_sms`
- **Discovery and routing:** rmcp serves on-demand discovery and validates the
  per-request protocol envelope; HTTP deployments can route with `Mcp-Method`
  and `Mcp-Name`. The packaged binary currently uses stdio.
- **Caching:** `tools/list` returns a public `ttlMs` of 60,000 for MCP 2026;
  rmcp omits the cache fields for legacy clients.
- **Deprecated extensions:** this server does not add new Roots, Sampling, or
  dynamic client-registration dependencies.

Protected tools require `MCP_REQUEST_STATE_KEY` with at least 32 high-entropy
bytes. All replicas must share that key so sealed approval state can resume on
another instance. Approval state is bound to the client identity, tool, and
arguments and expires after two minutes. Missing identity, invalid state,
rejection, or legacy protocol use fails closed. Task records are process-local
for the current stdio runtime; use a durable task store before deploying the
server behind scale-to-zero HTTP infrastructure.
