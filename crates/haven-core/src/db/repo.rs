use crate::error::{HavenError, Result};
use crate::models::{
    Channel, Message, NewChannel, NewUser, User, VoiceChannelAccessible, Webhook,
};
use rusqlite::{params, Connection, OptionalExtension};

pub struct DbRepo;

impl DbRepo {
    // === Users ===
    pub fn create_user(conn: &Connection, user: &NewUser) -> Result<User> {
        conn.execute(
            "INSERT INTO users (username, display_name, password_hash, is_admin, is_guest)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                user.username,
                user.display_name,
                user.password_hash,
                user.is_admin as i32,
                user.is_guest as i32,
            ],
        )?;

        let id = conn.last_insert_rowid();
        Self::get_user_by_id(conn, id)?
            .ok_or_else(|| HavenError::Internal("Failed to fetch newly created user".into()))
    }

    pub fn get_user_by_id(conn: &Connection, id: i64) -> Result<Option<User>> {
        let mut stmt = conn.prepare(
            "SELECT id, username, display_name, password_hash, is_admin, is_guest,
                    avatar, banner, bio, name_color, is_banned, created_at
             FROM users WHERE id = ?1",
        )?;

        let user = stmt
            .query_row(params![id], |row| {
                Ok(User {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    display_name: row.get(2)?,
                    password_hash: row.get(3)?,
                    is_admin: row.get::<_, i32>(4)? != 0,
                    is_guest: row.get::<_, i32>(5)? != 0,
                    avatar: row.get(6)?,
                    banner: row.get(7)?,
                    bio: row.get(8)?,
                    name_color: row.get(9)?,
                    is_banned: row.get::<_, i32>(10)? != 0,
                    created_at: row.get(11)?,
                })
            })
            .optional()?;

        Ok(user)
    }

    pub fn get_user_by_username(conn: &Connection, username: &str) -> Result<Option<User>> {
        let mut stmt = conn.prepare(
            "SELECT id, username, display_name, password_hash, is_admin, is_guest,
                    avatar, banner, bio, name_color, is_banned, created_at
             FROM users WHERE username = ?1 COLLATE NOCASE",
        )?;

        let user = stmt
            .query_row(params![username], |row| {
                Ok(User {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    display_name: row.get(2)?,
                    password_hash: row.get(3)?,
                    is_admin: row.get::<_, i32>(4)? != 0,
                    is_guest: row.get::<_, i32>(5)? != 0,
                    avatar: row.get(6)?,
                    banner: row.get(7)?,
                    bio: row.get(8)?,
                    name_color: row.get(9)?,
                    is_banned: row.get::<_, i32>(10)? != 0,
                    created_at: row.get(11)?,
                })
            })
            .optional()?;

        Ok(user)
    }

    // === Channels ===
    pub fn create_channel(conn: &Connection, ch: &NewChannel) -> Result<Channel> {
        conn.execute(
            "INSERT INTO channels (code, name, topic, position, is_dm, voice_enabled, voice_user_limit, voice_bitrate, role_gate)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                ch.code,
                ch.name,
                ch.topic,
                ch.position,
                ch.is_dm as i32,
                ch.voice_enabled as i32,
                ch.voice_user_limit,
                ch.voice_bitrate,
                ch.role_gate,
            ],
        )?;

        let id = conn.last_insert_rowid();
        Self::get_channel_by_id(conn, id)?
            .ok_or_else(|| HavenError::Internal("Failed to fetch newly created channel".into()))
    }

    pub fn get_channel_by_id(conn: &Connection, id: i64) -> Result<Option<Channel>> {
        let mut stmt = conn.prepare(
            "SELECT id, code, name, topic, position, is_dm, is_temp_voice AS is_temp, voice_enabled,
                    voice_user_limit, voice_bitrate, role_gate, channel_type, created_at
             FROM channels WHERE id = ?1",
        )?;

        let channel = stmt
            .query_row(params![id], |row| {
                Ok(Channel {
                    id: row.get(0)?,
                    code: row.get(1)?,
                    name: row.get(2)?,
                    topic: row.get(3)?,
                    position: row.get(4)?,
                    is_dm: row.get::<_, i32>(5)? != 0,
                    is_temp: row.get::<_, i32>(6)? != 0,
                    voice_enabled: row.get::<_, i32>(7)? != 0,
                    voice_user_limit: row.get(8)?,
                    voice_bitrate: row.get(9)?,
                    role_gate: row.get(10)?,
                    channel_type: row.get(11)?,
                    created_at: row.get(12)?,
                })
            })
            .optional()?;

        Ok(channel)
    }

    pub fn get_channel_by_code(conn: &Connection, code: &str) -> Result<Option<Channel>> {
        let mut stmt = conn.prepare(
            "SELECT id, code, name, topic, position, is_dm, is_temp_voice AS is_temp, voice_enabled,
                    voice_user_limit, voice_bitrate, role_gate, channel_type, created_at
             FROM channels WHERE code = ?1",
        )?;

        let channel = stmt
            .query_row(params![code], |row| {
                Ok(Channel {
                    id: row.get(0)?,
                    code: row.get(1)?,
                    name: row.get(2)?,
                    topic: row.get(3)?,
                    position: row.get(4)?,
                    is_dm: row.get::<_, i32>(5)? != 0,
                    is_temp: row.get::<_, i32>(6)? != 0,
                    voice_enabled: row.get::<_, i32>(7)? != 0,
                    voice_user_limit: row.get(8)?,
                    voice_bitrate: row.get(9)?,
                    role_gate: row.get(10)?,
                    channel_type: row.get(11)?,
                    created_at: row.get(12)?,
                })
            })
            .optional()?;

        Ok(channel)
    }

    pub fn list_channels(conn: &Connection) -> Result<Vec<Channel>> {
        let mut stmt = conn.prepare(
            "SELECT id, code, name, topic, position, is_dm, is_temp_voice AS is_temp, voice_enabled,
                    voice_user_limit, voice_bitrate, role_gate, channel_type, created_at
             FROM channels WHERE is_dm = 0 ORDER BY position ASC, id ASC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Channel {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                topic: row.get(3)?,
                position: row.get(4)?,
                is_dm: row.get::<_, i32>(5)? != 0,
                is_temp: row.get::<_, i32>(6)? != 0,
                voice_enabled: row.get::<_, i32>(7)? != 0,
                voice_user_limit: row.get(8)?,
                voice_bitrate: row.get(9)?,
                role_gate: row.get(10)?,
                channel_type: row.get(11)?,
                created_at: row.get(12)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    // === Messages ===
    pub fn create_message(
        conn: &Connection,
        channel_id: i64,
        user_id: i64,
        _username: &str,
        content: &str,
        reply_to: Option<i64>,
    ) -> Result<Message> {
        conn.execute(
            "INSERT INTO messages (channel_id, user_id, content, reply_to)
             VALUES (?1, ?2, ?3, ?4)",
            params![channel_id, user_id, content, reply_to],
        )?;

        let id = conn.last_insert_rowid();
        let mut stmt = conn.prepare(
            "SELECT m.id, m.channel_id, m.user_id,
                    COALESCE(m.webhook_username, u.username, 'Unknown') AS username,
                    m.content, m.reply_to, 0 AS is_pinned,
                    (m.edited_at IS NOT NULL) AS is_edited,
                    m.created_at, COALESCE(m.edited_at, m.created_at) AS updated_at
             FROM messages m
             LEFT JOIN users u ON u.id = m.user_id
             WHERE m.id = ?1",
        )?;

        let msg = stmt.query_row(params![id], |row| {
            Ok(Message {
                id: row.get(0)?,
                channel_id: row.get(1)?,
                user_id: row.get(2)?,
                username: row.get(3)?,
                content: row.get(4)?,
                reply_to: row.get(5)?,
                is_pinned: row.get::<_, i32>(6)? != 0,
                is_edited: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?;

        Ok(msg)
    }

    pub fn list_messages(conn: &Connection, channel_id: i64, limit: i64) -> Result<Vec<Message>> {
        let mut stmt = conn.prepare(
            "SELECT m.id, m.channel_id, m.user_id,
                    COALESCE(m.webhook_username, u.username, 'Unknown') AS username,
                    m.content, m.reply_to, 0 AS is_pinned,
                    (m.edited_at IS NOT NULL) AS is_edited,
                    m.created_at, COALESCE(m.edited_at, m.created_at) AS updated_at
             FROM messages m
             LEFT JOIN users u ON u.id = m.user_id
             WHERE m.channel_id = ?1
             ORDER BY m.id DESC LIMIT ?2",
        )?;

        let rows = stmt.query_map(params![channel_id, limit], |row| {
            Ok(Message {
                id: row.get(0)?,
                channel_id: row.get(1)?,
                user_id: row.get(2)?,
                username: row.get(3)?,
                content: row.get(4)?,
                reply_to: row.get(5)?,
                is_pinned: row.get::<_, i32>(6)? != 0,
                is_edited: row.get::<_, i32>(7)? != 0,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        list.reverse();
        Ok(list)
    }

    // === Webhooks & Bots ===
    pub fn get_webhook_by_token(conn: &Connection, token: &str) -> Result<Option<Webhook>> {
        let mut stmt = conn.prepare(
            "SELECT id, name, token, channel_id, created_by, is_bot, can_use_voice, created_at
             FROM webhooks WHERE token = ?1",
        )?;

        let wh = stmt
            .query_row(params![token], |row| {
                Ok(Webhook {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    token: row.get(2)?,
                    channel_id: row.get(3)?,
                    created_by: row.get(4)?,
                    is_bot: row.get::<_, i32>(5)? != 0,
                    can_use_voice: row.get::<_, i32>(6)? != 0,
                    created_at: row.get(7)?,
                })
            })
            .optional()?;

        Ok(wh)
    }

    pub fn get_accessible_voice_channels(
        conn: &Connection,
        webhook: &Webhook,
    ) -> Result<Vec<VoiceChannelAccessible>> {
        let is_admin = if let Some(creator_id) = webhook.created_by {
            let mut stmt = conn.prepare("SELECT is_admin, is_banned FROM users WHERE id = ?1")?;
            let res = stmt.query_row(params![creator_id], |row| {
                let admin: i32 = row.get(0)?;
                let banned: i32 = row.get(1)?;
                Ok((admin != 0, banned != 0))
            })?;
            if res.1 {
                return Ok(Vec::new()); // Banned creator
            }
            res.0
        } else {
            false
        };

        if is_admin {
            let mut stmt = conn.prepare(
                "SELECT id, code, name, voice_enabled, voice_user_limit, voice_bitrate
                 FROM channels WHERE is_dm = 0 ORDER BY position ASC, id ASC",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok(VoiceChannelAccessible {
                    id: row.get(0)?,
                    code: row.get(1)?,
                    name: row.get(2)?,
                    voice_enabled: row.get::<_, i32>(3)? != 0,
                    voice_user_limit: row.get(4)?,
                    voice_bitrate: row.get(5)?,
                })
            })?;
            let mut out = Vec::new();
            for r in rows {
                out.push(r?);
            }
            return Ok(out);
        }

        let mut stmt = conn.prepare(
            "SELECT DISTINCT c.id, c.code, c.name, c.voice_enabled, c.voice_user_limit, c.voice_bitrate
             FROM channels c
             LEFT JOIN channel_members cm ON cm.channel_id = c.id AND cm.user_id = ?1
             WHERE c.is_dm = 0 AND (c.id = ?2 OR cm.user_id IS NOT NULL)
             ORDER BY c.position ASC, c.id ASC",
        )?;

        let creator_id = webhook.created_by.unwrap_or(-1);
        let rows = stmt.query_map(params![creator_id, webhook.channel_id], |row| {
            Ok(VoiceChannelAccessible {
                id: row.get(0)?,
                code: row.get(1)?,
                name: row.get(2)?,
                voice_enabled: row.get::<_, i32>(3)? != 0,
                voice_user_limit: row.get(4)?,
                voice_bitrate: row.get(5)?,
            })
        })?;

        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }
}
