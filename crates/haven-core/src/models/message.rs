use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: i64,
    pub channel_id: i64,
    pub user_id: i64,
    pub username: String,
    pub content: String,
    pub reply_to: Option<i64>,
    pub is_pinned: bool,
    pub is_edited: bool,
    pub created_at: String,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: i64,
    pub message_id: i64,
    pub filename: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    pub message_id: i64,
    pub user_id: i64,
    pub emoji: String,
}
