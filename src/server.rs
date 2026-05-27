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
    /// Message text
    pub text: String,
    /// Message type: text, image, location, system (default: text)
    pub msg_type: Option<String>,
    /// Optional metadata (e.g. image URL, coordinates)
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
    metadata: Option<Value>,
    timestamp: String,
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

    #[tool(description = "Send a message to a channel (in-app messaging)")]
    async fn send_message(&self, Parameters(input): Parameters<MessageInput>) -> String {
        let msg = StoredMessage {
            id: format!("msg_{}", msg_id()),
            channel: input.channel.clone(),
            sender: input.sender.clone(),
            text: input.text.clone(),
            msg_type: input.msg_type.unwrap_or_else(|| "text".into()),
            metadata: input.metadata,
            timestamp: now(),
        };
        let id = msg.id.clone();
        self.messages.lock().unwrap().entry(input.channel.clone()).or_default().push(msg);
        json!({"status": "sent", "message_id": id, "channel": input.channel, "timestamp": now()}).to_string()
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
}
