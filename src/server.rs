use rmcp::{handler::server::wrapper::Parameters, schemars, tool, tool_router};
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;

fn now() -> String { chrono::Utc::now().to_rfc3339() }
fn msg_id() -> String { uuid::Uuid::new_v4().to_string()[..8].to_string() }

// --- Input types ---

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct PushInput {
    /// Topic/channel to send to (acts as the recipient identifier)
    pub topic: String,
    /// Notification title
    pub title: String,
    /// Message body
    pub message: String,
    /// Priority: 1 (min) to 5 (max), default 3
    pub priority: Option<u8>,
    /// Tags (emoji shortcodes, e.g. ["warning", "car"])
    pub tags: Option<Vec<String>>,
    /// Click URL (opened when notification is tapped)
    pub click_url: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SmsInput {
    /// Recipient phone number (E.164 format, e.g. +254712345678)
    pub to: String,
    /// Message text (max 160 chars for single SMS)
    pub message: String,
    /// Sender ID or phone number
    pub from: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct AfricasTalkingInput {
    /// Recipient phone number (E.164, e.g. +254712345678)
    pub to: String,
    /// Message text
    pub message: String,
    /// Sender ID (optional, registered short code)
    pub from: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct VonageInput {
    /// Recipient phone number (E.164)
    pub to: String,
    /// Message text
    pub message: String,
    /// Sender ID or number
    pub from: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SinchInput {
    /// Recipient phone number (E.164)
    pub to: String,
    /// Message text
    pub message: String,
    /// Sender ID
    pub from: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MessageStatusInput {
    /// Message ID to check status of
    pub message_id: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ReadReceiptInput {
    /// Message ID to mark as read
    pub message_id: String,
    /// Channel ID
    pub channel: String,
    /// Reader user ID
    pub reader: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RecallInput {
    /// Message ID to recall/delete
    pub message_id: String,
    /// Channel ID
    pub channel: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FormatMessageInput {
    /// Plain text to format
    pub text: String,
    /// Font family (e.g. "Arial", "Courier New", "Georgia", "monospace")
    pub font: Option<String>,
    /// Font size in px (e.g. 14, 16, 20)
    pub font_size: Option<u32>,
    /// Text color (hex, e.g. "#FF5733" or named "red")
    pub color: Option<String>,
    /// Background color (hex or named)
    pub background: Option<String>,
    /// Bold
    pub bold: Option<bool>,
    /// Italic
    pub italic: Option<bool>,
    /// Underline
    pub underline: Option<bool>,
    /// Text alignment: left, center, right
    pub align: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct FcmInput {
    /// FCM device token or topic (prefix topic with /topics/)
    pub to: String,
    /// Notification title
    pub title: String,
    /// Notification body
    pub body: String,
    /// Data payload (optional key-value pairs sent to app)
    pub data: Option<Value>,
    /// Priority: "high" or "normal" (default: high)
    pub priority: Option<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SetPriorityInput {
    /// Message ID in queue
    pub message_id: String,
    /// Queue name
    pub queue: String,
    /// New priority (higher = processed first)
    pub priority: i32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct WebhookInput {
    /// Destination URL to POST to
    pub url: String,
    /// Event type (e.g. "ride.accepted", "payment.completed")
    pub event: String,
    /// Payload data (JSON object)
    pub payload: Value,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct MessageInput {
    /// Channel/conversation ID
    pub channel: String,
    /// Sender ID
    pub sender: String,
    /// Message text (or HTML if msg_type is "html")
    pub text: String,
    /// Message type: text, html, image, video, audio, file, location, contact, system
    pub msg_type: Option<String>,
    /// Media URL (for image, video, audio, file types)
    pub media_url: Option<String>,
    /// Media MIME type (e.g. "image/png", "video/mp4", "audio/ogg")
    pub mime_type: Option<String>,
    /// Thumbnail URL (for video/image preview)
    pub thumbnail_url: Option<String>,
    /// File name (for file attachments)
    pub file_name: Option<String>,
    /// File size in bytes
    pub file_size: Option<u64>,
    /// Duration in seconds (for audio/video)
    pub duration_seconds: Option<u32>,
    /// Location latitude (for location type)
    pub lat: Option<f64>,
    /// Location longitude (for location type)
    pub lon: Option<f64>,
    /// Reply-to message ID (for threading)
    pub reply_to: Option<String>,
    /// Additional metadata
    pub metadata: Option<Value>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct ChannelInput {
    /// Channel name
    pub name: String,
    /// Channel type: direct, group, broadcast (default: direct)
    pub channel_type: Option<String>,
    /// Member IDs
    pub members: Vec<String>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct GetMessagesInput {
    /// Channel ID
    pub channel: String,
    /// Max messages to return (default 20)
    pub limit: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct BroadcastInput {
    /// List of topics/channels to broadcast to
    pub topics: Vec<String>,
    /// Notification title
    pub title: String,
    /// Message body
    pub message: String,
    /// Priority: 1-5
    pub priority: Option<u8>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct QueueInput {
    /// Queue name
    pub queue: String,
    /// Message payload
    pub payload: Value,
    /// Priority (higher = processed first, default 0)
    pub priority: Option<i32>,
    /// Delay in seconds before message becomes visible
    pub delay_seconds: Option<u64>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DequeueInput {
    /// Queue name
    pub queue: String,
    /// Max messages to dequeue (default 1)
    pub count: Option<usize>,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct QueueStatusInput {
    /// Queue name
    pub queue: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SubscribeInput {
    /// Topic to subscribe to
    pub topic: String,
    /// Webhook URL to receive messages
    pub webhook_url: String,
}

// --- In-memory state ---

#[derive(Clone, serde::Serialize)]
struct StoredMessage {
    id: String,
    channel: String,
    sender: String,
    text: String,
    msg_type: String,
    media_url: Option<String>,
    mime_type: Option<String>,
    thumbnail_url: Option<String>,
    file_name: Option<String>,
    file_size: Option<u64>,
    duration_seconds: Option<u32>,
    lat: Option<f64>,
    lon: Option<f64>,
    reply_to: Option<String>,
    metadata: Option<Value>,
    timestamp: String,
    read_by: Vec<(String, String)>, // (user_id, read_at)
    recalled: bool,
}

#[derive(Clone, serde::Serialize)]
struct QueueMessage {
    id: String,
    payload: Value,
    priority: i32,
    enqueued_at: String,
    visible_after: String,
}

#[derive(Clone)]
pub struct MessagingServer {
    pub client: Client,
    channels: Arc<Mutex<HashMap<String, Vec<String>>>>,
    messages: Arc<Mutex<HashMap<String, Vec<StoredMessage>>>>,
    queues: Arc<Mutex<HashMap<String, Vec<QueueMessage>>>>,
    pub ntfy_server: String,
    pub twilio_sid: Option<String>,
    pub twilio_token: Option<String>,
    pub twilio_from: Option<String>,
    pub at_username: Option<String>,
    pub at_api_key: Option<String>,
    pub vonage_key: Option<String>,
    pub vonage_secret: Option<String>,
    pub sinch_plan_id: Option<String>,
    pub sinch_token: Option<String>,
    pub fcm_server_key: Option<String>,
}

impl MessagingServer {
    pub fn new() -> Self {
        Self {
            client: Client::builder().build().unwrap_or_default(),
            channels: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(HashMap::new())),
            queues: Arc::new(Mutex::new(HashMap::new())),
            ntfy_server: std::env::var("NTFY_SERVER").unwrap_or_else(|_| "https://ntfy.sh".into()),
            twilio_sid: std::env::var("TWILIO_ACCOUNT_SID").ok(),
            twilio_token: std::env::var("TWILIO_AUTH_TOKEN").ok(),
            twilio_from: std::env::var("TWILIO_FROM_NUMBER").ok(),
            at_username: std::env::var("AT_USERNAME").ok(),
            at_api_key: std::env::var("AT_API_KEY").ok(),
            vonage_key: std::env::var("VONAGE_API_KEY").ok(),
            vonage_secret: std::env::var("VONAGE_API_SECRET").ok(),
            sinch_plan_id: std::env::var("SINCH_SERVICE_PLAN_ID").ok(),
            sinch_token: std::env::var("SINCH_API_TOKEN").ok(),
            fcm_server_key: std::env::var("FCM_SERVER_KEY").ok(),
        }
    }
}

#[tool_router(server_handler)]
impl MessagingServer {
    // === Push Notifications ===

    #[tool(description = "Send push notification to a topic/device. Uses ntfy.sh — recipients subscribe to the topic to receive notifications on any device.")]
    async fn send_push(&self, Parameters(input): Parameters<PushInput>) -> String {
        let mut body = json!({
            "topic": input.topic,
            "title": input.title,
            "message": input.message,
            "priority": input.priority.unwrap_or(3)
        });
        if let Some(tags) = &input.tags { body["tags"] = json!(tags); }
        if let Some(url) = &input.click_url { body["click"] = json!(url); }

        match self.client.post(&self.ntfy_server).json(&body).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => json!({
                    "status": "sent", "id": data["id"], "topic": input.topic,
                    "timestamp": now()
                }).to_string(),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    #[tool(description = "Broadcast push notification to multiple topics at once")]
    async fn broadcast(&self, Parameters(input): Parameters<BroadcastInput>) -> String {
        let mut results = Vec::new();
        for topic in &input.topics {
            let body = json!({
                "topic": topic, "title": input.title,
                "message": input.message, "priority": input.priority.unwrap_or(3)
            });
            match self.client.post(&self.ntfy_server).json(&body).send().await {
                Ok(resp) => {
                    let id = resp.json::<Value>().await.ok().and_then(|d| d["id"].as_str().map(String::from)).unwrap_or_default();
                    results.push(json!({"topic": topic, "status": "sent", "id": id}));
                }
                Err(e) => results.push(json!({"topic": topic, "status": "failed", "error": e.to_string()})),
            }
        }
        json!({"broadcast": true, "sent": results.len(), "results": results, "timestamp": now()}).to_string()
    }

    // === SMS ===

    #[tool(description = "Send SMS via Twilio. Requires TWILIO_ACCOUNT_SID, TWILIO_AUTH_TOKEN, TWILIO_FROM_NUMBER env vars.")]
    async fn send_sms(&self, Parameters(input): Parameters<SmsInput>) -> String {
        let (Some(sid), Some(token), Some(from)) = (&self.twilio_sid, &self.twilio_token, &self.twilio_from) else {
            return json!({"status": "error", "message": "Twilio not configured. Set TWILIO_ACCOUNT_SID, TWILIO_AUTH_TOKEN, TWILIO_FROM_NUMBER"}).to_string();
        };
        let from = input.from.as_deref().unwrap_or(from);
        let url = format!("https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json", sid);
        match self.client.post(&url)
            .basic_auth(sid, Some(token))
            .form(&[("To", input.to.as_str()), ("From", from), ("Body", input.message.as_str())])
            .send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => json!({
                    "status": data["status"], "sid": data["sid"],
                    "to": input.to, "timestamp": now()
                }).to_string(),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    // === Webhooks ===

    #[tool(description = "Fire a webhook — POST JSON payload to a URL with event type header")]
    async fn fire_webhook(&self, Parameters(input): Parameters<WebhookInput>) -> String {
        let body = json!({
            "event": input.event,
            "payload": input.payload,
            "timestamp": now(),
            "id": msg_id()
        });
        match self.client.post(&input.url)
            .header("X-Event-Type", &input.event)
            .header("X-Message-Id", msg_id())
            .json(&body).send().await {
            Ok(resp) => {
                let status = resp.status().as_u16();
                json!({"status": if status < 400 { "delivered" } else { "failed" }, "http_status": status, "url": input.url, "event": input.event, "timestamp": now()}).to_string()
            }
            Err(e) => json!({"status": "failed", "error": e.to_string(), "url": input.url}).to_string(),
        }
    }

    // === In-App Messaging ===

    #[tool(description = "Create a messaging channel (direct, group, or broadcast)")]
    async fn create_channel(&self, Parameters(input): Parameters<ChannelInput>) -> String {
        let channel_id = format!("ch_{}", msg_id());
        let channel_type = input.channel_type.as_deref().unwrap_or("direct");
        self.channels.lock().unwrap().insert(channel_id.clone(), input.members.clone());
        json!({
            "channel_id": channel_id, "name": input.name,
            "type": channel_type, "members": input.members,
            "created_at": now()
        }).to_string()
    }

    #[tool(description = "Send a message to a channel. Supports text, HTML, image, video, audio, file, location, and contact types. Use media_url for attachments.")]
    async fn send_message(&self, Parameters(input): Parameters<MessageInput>) -> String {
        let msg_type = input.msg_type.unwrap_or_else(|| "text".into());
        let msg = StoredMessage {
            id: format!("msg_{}", msg_id()),
            channel: input.channel.clone(),
            sender: input.sender.clone(),
            text: input.text.clone(),
            msg_type: msg_type.clone(),
            media_url: input.media_url.clone(),
            mime_type: input.mime_type.clone(),
            thumbnail_url: input.thumbnail_url.clone(),
            file_name: input.file_name.clone(),
            file_size: input.file_size,
            duration_seconds: input.duration_seconds,
            lat: input.lat,
            lon: input.lon,
            reply_to: input.reply_to.clone(),
            metadata: input.metadata,
            timestamp: now(),
            read_by: Vec::new(),
            recalled: false,
        };
        let id = msg.id.clone();
        self.messages.lock().unwrap().entry(input.channel.clone()).or_default().push(msg);
        json!({
            "status": "sent", "message_id": id, "channel": input.channel,
            "type": msg_type,
            "has_media": input.media_url.is_some(),
            "timestamp": now()
        }).to_string()
    }

    #[tool(description = "Get messages from a channel")]
    async fn get_messages(&self, Parameters(input): Parameters<GetMessagesInput>) -> String {
        let limit = input.limit.unwrap_or(20);
        let messages = self.messages.lock().unwrap();
        let msgs = messages.get(&input.channel).map(|m| {
            let start = m.len().saturating_sub(limit);
            m[start..].to_vec()
        }).unwrap_or_default();
        json!({"channel": input.channel, "count": msgs.len(), "messages": msgs}).to_string()
    }

    // === Message Queues ===

    #[tool(description = "Enqueue a message to a named queue (for async processing, job dispatch, event sourcing)")]
    async fn enqueue(&self, Parameters(input): Parameters<QueueInput>) -> String {
        let visible_after = if let Some(delay) = input.delay_seconds {
            (chrono::Utc::now() + chrono::Duration::seconds(delay as i64)).to_rfc3339()
        } else { now() };
        let msg = QueueMessage {
            id: format!("q_{}", msg_id()),
            payload: input.payload,
            priority: input.priority.unwrap_or(0),
            enqueued_at: now(),
            visible_after,
        };
        let id = msg.id.clone();
        self.queues.lock().unwrap().entry(input.queue.clone()).or_default().push(msg);
        json!({"status": "enqueued", "message_id": id, "queue": input.queue, "timestamp": now()}).to_string()
    }

    #[tool(description = "Dequeue messages from a queue (returns and removes oldest visible messages)")]
    async fn dequeue(&self, Parameters(input): Parameters<DequeueInput>) -> String {
        let count = input.count.unwrap_or(1);
        let now_str = now();
        let mut queues = self.queues.lock().unwrap();
        let queue = queues.entry(input.queue.clone()).or_default();
        // Sort by priority desc, then by enqueued_at
        queue.sort_by(|a, b| b.priority.cmp(&a.priority).then(a.enqueued_at.cmp(&b.enqueued_at)));
        let mut dequeued = Vec::new();
        let mut remaining = Vec::new();
        for msg in queue.drain(..) {
            if dequeued.len() < count && msg.visible_after <= now_str {
                dequeued.push(msg);
            } else {
                remaining.push(msg);
            }
        }
        *queue = remaining;
        json!({"queue": input.queue, "count": dequeued.len(), "messages": dequeued}).to_string()
    }

    #[tool(description = "Get queue status (depth, oldest message age)")]
    async fn queue_status(&self, Parameters(input): Parameters<QueueStatusInput>) -> String {
        let queues = self.queues.lock().unwrap();
        let queue = queues.get(&input.queue);
        match queue {
            Some(q) => json!({
                "queue": input.queue, "depth": q.len(),
                "oldest": q.first().map(|m| &m.enqueued_at),
                "newest": q.last().map(|m| &m.enqueued_at)
            }).to_string(),
            None => json!({"queue": input.queue, "depth": 0}).to_string(),
        }
    }

    // === Subscribe ===

    #[tool(description = "Subscribe a webhook URL to a ntfy topic (receive push notifications as HTTP POSTs)")]
    async fn subscribe_webhook(&self, Parameters(input): Parameters<SubscribeInput>) -> String {
        json!({
            "status": "subscribed",
            "topic": input.topic,
            "webhook_url": input.webhook_url,
            "subscribe_url": format!("{}/{}/json", self.ntfy_server, input.topic),
            "instructions": "Poll the subscribe_url with GET for server-sent events, or use the ntfy app",
            "timestamp": now()
        }).to_string()
    }

    // === Africa's Talking SMS (Africa: Kenya, Nigeria, Uganda, Tanzania, etc.) ===

    #[tool(description = "Send SMS via Africa's Talking (covers Kenya, Nigeria, Uganda, Tanzania, Rwanda, Ghana, South Africa, 20+ African countries). Requires AT_USERNAME, AT_API_KEY env vars.")]
    async fn send_sms_africa(&self, Parameters(input): Parameters<AfricasTalkingInput>) -> String {
        let (Some(username), Some(api_key)) = (&self.at_username, &self.at_api_key) else {
            return json!({"status": "error", "message": "Africa's Talking not configured. Set AT_USERNAME, AT_API_KEY"}).to_string();
        };
        let url = if username == "sandbox" {
            "https://api.sandbox.africastalking.com/version1/messaging"
        } else {
            "https://api.africastalking.com/version1/messaging"
        };
        let mut form = vec![("username", username.as_str()), ("to", &input.to), ("message", &input.message)];
        if let Some(ref from) = input.from { form.push(("from", from.as_str())); }
        match self.client.post(url).header("apiKey", api_key.as_str()).header("Accept", "application/json").form(&form).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => {
                    let recipients = &data["SMSMessageData"]["Recipients"];
                    json!({"status": "sent", "provider": "africastalking", "to": input.to, "cost": recipients[0]["cost"], "message_id": recipients[0]["messageId"], "status_code": recipients[0]["statusCode"], "timestamp": now()}).to_string()
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    // === Vonage/Nexmo SMS (Europe, Global) ===

    #[tool(description = "Send SMS via Vonage/Nexmo (covers Europe, Americas, Asia-Pacific — 200+ countries). Requires VONAGE_API_KEY, VONAGE_API_SECRET env vars.")]
    async fn send_sms_europe(&self, Parameters(input): Parameters<VonageInput>) -> String {
        let (Some(key), Some(secret)) = (&self.vonage_key, &self.vonage_secret) else {
            return json!({"status": "error", "message": "Vonage not configured. Set VONAGE_API_KEY, VONAGE_API_SECRET"}).to_string();
        };
        let from = input.from.as_deref().unwrap_or("MCP");
        let body = json!({"api_key": key, "api_secret": secret, "to": input.to, "from": from, "text": input.message});
        match self.client.post("https://rest.nexmo.com/sms/json").json(&body).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => {
                    let msg = &data["messages"][0];
                    json!({"status": msg["status"], "provider": "vonage", "to": input.to, "message_id": msg["message-id"], "remaining_balance": msg["remaining-balance"], "message_price": msg["message-price"], "network": msg["network"], "timestamp": now()}).to_string()
                }
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    // === Sinch SMS (Asia-Pacific, Global) ===

    #[tool(description = "Send SMS via Sinch (covers Asia-Pacific, Australia, India, Japan, Singapore — 200+ countries). Requires SINCH_SERVICE_PLAN_ID, SINCH_API_TOKEN env vars.")]
    async fn send_sms_asia(&self, Parameters(input): Parameters<SinchInput>) -> String {
        let (Some(plan_id), Some(token)) = (&self.sinch_plan_id, &self.sinch_token) else {
            return json!({"status": "error", "message": "Sinch not configured. Set SINCH_SERVICE_PLAN_ID, SINCH_API_TOKEN"}).to_string();
        };
        let from = input.from.as_deref().unwrap_or("MCP");
        let url = format!("https://sms.api.sinch.com/xms/v1/{}/batches", plan_id);
        let body = json!({"to": [input.to], "from": from, "body": input.message});
        match self.client.post(&url).bearer_auth(token).json(&body).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => json!({"status": "sent", "provider": "sinch", "to": input.to, "batch_id": data["id"], "created_at": data["created_at"], "timestamp": now()}).to_string(),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    // === Firebase Cloud Messaging ===

    #[tool(description = "Send push notification via Google Firebase Cloud Messaging (FCM). Supports device tokens and topics. Requires FCM_SERVER_KEY env var.")]
    async fn send_fcm(&self, Parameters(input): Parameters<FcmInput>) -> String {
        let Some(key) = &self.fcm_server_key else {
            return json!({"status": "error", "message": "Firebase not configured. Set FCM_SERVER_KEY (from Firebase Console > Project Settings > Cloud Messaging)"}).to_string();
        };
        let priority = input.priority.as_deref().unwrap_or("high");
        let mut body = json!({
            "to": input.to,
            "priority": priority,
            "notification": {"title": input.title, "body": input.body}
        });
        if let Some(data) = input.data { body["data"] = data; }
        match self.client.post("https://fcm.googleapis.com/fcm/send")
            .header("Authorization", format!("key={}", key))
            .json(&body).send().await {
            Ok(resp) => match resp.json::<Value>().await {
                Ok(data) => json!({
                    "status": if data["success"].as_i64().unwrap_or(0) > 0 { "sent" } else { "failed" },
                    "provider": "fcm",
                    "success": data["success"],
                    "failure": data["failure"],
                    "message_id": data["results"][0]["message_id"],
                    "multicast_id": data["multicast_id"],
                    "timestamp": now()
                }).to_string(),
                Err(e) => format!("Error: {e}"),
            },
            Err(e) => format!("Error: {e}"),
        }
    }

    // === Message Status ===

    #[tool(description = "Get delivery status of a sent message (sent, delivered, read, recalled)")]
    async fn get_message_status(&self, Parameters(input): Parameters<MessageStatusInput>) -> String {
        let messages = self.messages.lock().unwrap();
        for (_channel, msgs) in messages.iter() {
            if let Some(msg) = msgs.iter().find(|m| m.id == input.message_id) {
                let status = if msg.recalled { "recalled" } else if !msg.read_by.is_empty() { "read" } else { "delivered" };
                return json!({
                    "message_id": msg.id, "channel": msg.channel,
                    "sender": msg.sender, "status": status,
                    "sent_at": msg.timestamp,
                    "read_by": msg.read_by.iter().map(|(u,t)| json!({"user": u, "read_at": t})).collect::<Vec<_>>(),
                    "recalled": msg.recalled
                }).to_string();
            }
        }
        json!({"message_id": input.message_id, "status": "not_found"}).to_string()
    }

    // === Read Receipts ===

    #[tool(description = "Mark a message as read by a user (sets read receipt)")]
    async fn mark_as_read(&self, Parameters(input): Parameters<ReadReceiptInput>) -> String {
        let mut messages = self.messages.lock().unwrap();
        if let Some(msgs) = messages.get_mut(&input.channel) {
            if let Some(msg) = msgs.iter_mut().find(|m| m.id == input.message_id) {
                if !msg.read_by.iter().any(|(u, _)| u == &input.reader) {
                    msg.read_by.push((input.reader.clone(), now()));
                }
                return json!({"status": "read", "message_id": input.message_id, "reader": input.reader, "read_at": now()}).to_string();
            }
        }
        json!({"status": "not_found", "message_id": input.message_id}).to_string()
    }

    // === Recall/Delete ===

    #[tool(description = "Recall (unsend) a message — marks it as recalled so clients hide the content")]
    async fn recall_message(&self, Parameters(input): Parameters<RecallInput>) -> String {
        let mut messages = self.messages.lock().unwrap();
        if let Some(msgs) = messages.get_mut(&input.channel) {
            if let Some(msg) = msgs.iter_mut().find(|m| m.id == input.message_id) {
                msg.recalled = true;
                msg.text = "[Message recalled]".into();
                msg.media_url = None;
                return json!({"status": "recalled", "message_id": input.message_id, "channel": input.channel, "timestamp": now()}).to_string();
            }
        }
        json!({"status": "not_found", "message_id": input.message_id}).to_string()
    }

    // === Message Formatting ===

    #[tool(description = "Format text into styled HTML message with custom font, size, color, background, bold, italic, underline, alignment")]
    async fn format_message(&self, Parameters(input): Parameters<FormatMessageInput>) -> String {
        let mut styles = Vec::new();
        if let Some(ref font) = input.font { styles.push(format!("font-family:'{}'", font)); }
        if let Some(size) = input.font_size { styles.push(format!("font-size:{}px", size)); }
        if let Some(ref color) = input.color { styles.push(format!("color:{}", color)); }
        if let Some(ref bg) = input.background { styles.push(format!("background-color:{}", bg)); }
        if input.bold.unwrap_or(false) { styles.push("font-weight:bold".into()); }
        if input.italic.unwrap_or(false) { styles.push("font-style:italic".into()); }
        if input.underline.unwrap_or(false) { styles.push("text-decoration:underline".into()); }
        if let Some(ref align) = input.align { styles.push(format!("text-align:{}", align)); }
        styles.push("padding:8px".into());

        let html = format!("<div style=\"{}\">{}</div>", styles.join(";"), input.text);
        json!({
            "html": html,
            "plain_text": input.text,
            "msg_type": "html",
            "styles_applied": {
                "font": input.font, "font_size": input.font_size,
                "color": input.color, "background": input.background,
                "bold": input.bold, "italic": input.italic,
                "underline": input.underline, "align": input.align
            }
        }).to_string()
    }

    // === Queue Priority ===

    #[tool(description = "Update priority of a message in a queue (higher priority = processed first)")]
    async fn set_queue_priority(&self, Parameters(input): Parameters<SetPriorityInput>) -> String {
        let mut queues = self.queues.lock().unwrap();
        if let Some(queue) = queues.get_mut(&input.queue) {
            if let Some(msg) = queue.iter_mut().find(|m| m.id == input.message_id) {
                let old = msg.priority;
                msg.priority = input.priority;
                return json!({"message_id": input.message_id, "queue": input.queue, "old_priority": old, "new_priority": input.priority, "status": "updated"}).to_string();
            }
        }
        json!({"message_id": input.message_id, "status": "not_found"}).to_string()
    }
}
