use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub display_name: Option<String>,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub is_admin: bool,
    pub is_guest: bool,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub bio: Option<String>,
    pub name_color: Option<String>,
    pub is_banned: bool,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub is_admin: bool,
    pub is_guest: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPublicProfile {
    pub id: i64,
    pub username: String,
    pub display_name: Option<String>,
    pub is_admin: bool,
    pub is_guest: bool,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub bio: Option<String>,
    pub name_color: Option<String>,
    pub is_banned: bool,
}

impl From<User> for UserPublicProfile {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            username: u.username,
            display_name: u.display_name,
            is_admin: u.is_admin,
            is_guest: u.is_guest,
            avatar: u.avatar,
            banner: u.banner,
            bio: u.bio,
            name_color: u.name_color,
            is_banned: u.is_banned,
        }
    }
}
