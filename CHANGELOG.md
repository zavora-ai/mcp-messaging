# Changelog

## [1.5.0] - 2026-08-13

### Changed
- Upgraded to rmcp 3.1.2 and raised the minimum supported Rust version to 1.94.1.
- Added MCP 2026-07-28 stateless request handling while retaining MCP 2025-11-25 initialization compatibility.

### Added
- Per-request identity and protocol metadata, on-demand discovery/cache hints, and the configured Tasks and sealed MRTR approval policies.

## [1.4.0] - 2026-05-27

### Added
- `mark_as_read` — set read receipts with user ID and timestamp
- `recall_message` — unsend messages (clears content, marks recalled)
- `format_message` — generate styled HTML (font, size, color, background, bold, italic, underline, alignment)
- Read-by tracking on all messages (who read, when)
- Recalled state on messages

## [1.3.0] - 2026-05-27

### Added
- Rich media message types: HTML, image, video, audio, file, location, contact
- Media fields: `media_url`, `mime_type`, `thumbnail_url`, `file_name`, `file_size`, `duration_seconds`
- Location sharing: `lat`, `lon` fields
- Reply threading: `reply_to` field

## [1.2.0] - 2026-05-27

### Added
- `send_fcm` — Firebase Cloud Messaging (Android, iOS, Web)
- `FCM_SERVER_KEY` environment variable

## [1.1.0] - 2026-05-27

### Added
- `send_sms_africa` — Africa's Talking (Kenya, Nigeria, Uganda, Tanzania, 20+ countries)
- `send_sms_europe` — Vonage/Nexmo (Europe, 200+ countries)
- `send_sms_asia` — Sinch (Asia-Pacific, Australia, 200+ countries)
- `get_message_status` — delivery status tracking
- `set_queue_priority` — update priority of queued messages

## [1.0.0] - 2026-05-27

### Added
- `send_push` — push notifications via ntfy.sh (free, global)
- `broadcast` — multi-topic push broadcast
- `send_sms` — SMS via Twilio (Americas)
- `fire_webhook` — POST events to any URL
- `subscribe_webhook` — subscribe URL to topic
- `create_channel` — direct, group, broadcast channels
- `send_message` — in-app messaging
- `get_messages` — channel message history
- `enqueue` — priority message queue with delayed delivery
- `dequeue` — consume messages from queue
- `queue_status` — queue depth and age
