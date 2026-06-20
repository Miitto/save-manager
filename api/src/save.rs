use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::auth::RequireUser;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "server", derive(sqlx::Type))]
pub enum Game {
    IntoTheRadius2,
    Satisfactory,
}

pub type SaveId = i32;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Save {
    pub id: SaveId,
    pub name: String,
    pub game: Game,
    pub owner: crate::UserId,
    pub version_count: i32,
    pub most_recent_version: Option<std::time::SystemTime>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct DbSave {
    pub id: SaveId,
    pub name: String,
    pub game: Game,
    pub owner: crate::UserId,
    pub version_count: i32,
    pub most_recent_version: Option<i32>,
}

impl DbSave {
    pub fn to_save(&self) -> Save {
        Save {
            id: self.id,
            name: self.name.clone(),
            game: self.game,
            owner: self.owner,
            version_count: self.version_count,
            most_recent_version: self
                .most_recent_version
                .map(|ts| std::time::UNIX_EPOCH + std::time::Duration::from_secs(ts as u64)),
        }
    }
}

pub type VersionId = i32;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct UserPreview {
    pub id: crate::UserId,
    pub username: String,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "server", derive(sqlx::Type))]
pub enum UserAccess {
    View,
    Edit,
    Owner,
}

impl UserAccess {
    pub fn can_edit(&self) -> bool {
        matches!(self, UserAccess::Edit | UserAccess::Owner)
    }

    pub fn can_view(&self) -> bool {
        matches!(
            self,
            UserAccess::View | UserAccess::Edit | UserAccess::Owner
        )
    }
}

pub trait UserAccessExt {
    fn can_edit(&self) -> bool;
    fn can_view(&self) -> bool;
}

impl UserAccessExt for Option<UserAccess> {
    fn can_edit(&self) -> bool {
        self.is_some_and(|access| access.can_edit())
    }

    fn can_view(&self) -> bool {
        self.is_some_and(|access| access.can_view())
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct UserSaveAccess {
    pub user_id: crate::UserId,
    pub save_id: SaveId,
    pub access: UserAccess,
}

#[post("/api/saves", auth: crate::auth::Session, db: crate::ServerDb)]
pub async fn get_user_saves(user_id: crate::UserId) -> Result<Vec<Save>, ServerFnError> {
    let user = auth.require_user()?;

    if user.id != user_id {
        return Err(HttpError::new(
            StatusCode::UNAUTHORIZED,
            format!(
                "You are not authorized to view this user's saves (You: {}| Them: {})",
                user.id, user_id
            ),
        )
        .into());
    }

    let saves = sqlx::query_as::<_, DbSave>(
        "SELECT s.id, s.name, s.game, s.owner, COUNT(v.id) as version_count, MAX(v.timestamp) as most_recent_version FROM saves s LEFT JOIN versions v ON s.id = v.save_id LEFT JOIN user_save_access usa ON s.id = usa.save_id WHERE (usa.user_id = $1 OR s.owner = $1) GROUP BY s.id;",
    )
    .bind(user_id)
    .fetch_all(&db.0)
    .await
    .map_err(|e| {
        error!("Database error while fetching user saves: {}", e);
        ServerFnError::ServerError {
            message: "Internal server error".to_string(),
            code: 500,
            details: None,
        }
    })?;

    Ok(saves.into_iter().map(|s| s.to_save()).collect())
}

#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct Access {
    pub access: UserAccess,
}

#[cfg(feature = "server")]
pub(crate) async fn query_user_save_access(
    user_id: crate::UserId,
    save_id: SaveId,
    db: &sqlx::SqlitePool,
) -> Result<Option<UserAccess>, ServerFnError> {
    sqlx::query_as::<_, Access>(
        "SELECT
        CASE WHEN s.owner = $1 THEN 2 ELSE usa.access END as access
        FROM saves s
        LEFT JOIN user_save_access usa ON s.id = usa.save_id
        WHERE s.id = $2 AND (s.owner = $1 OR usa.user_id = $1)",
    )
    .bind(user_id)
    .bind(save_id)
    .fetch_optional(db)
    .await
    .map(|access| access.map(|a| a.access))
    .map_err(|e| {
        error!("Database error while fetching user save access: {}", e);
        ServerFnError::ServerError {
            message: "Internal server error".to_string(),
            code: 500,
            details: None,
        }
    })
}

#[post("/api/save/{save_id}/access", auth: crate::auth::Session, db: crate::ServerDb)]
pub async fn get_user_save_access(save_id: SaveId) -> Result<Option<UserAccess>, ServerFnError> {
    let user = if let Ok(u) = auth.require_user() {
        u
    } else {
        return Ok(None);
    };

    query_user_save_access(user.id, save_id, &db.0).await
}

#[post("/api/save/{save_id}", auth: crate::auth::Session, db: crate::ServerDb)]
pub async fn get_save_details(save_id: SaveId) -> Result<Save, ServerFnError> {
    let user = auth.require_user()?;

    if query_user_save_access(user.id, save_id, &db.0)
        .await?
        .is_none()
    {
        warn!("User {} attempeted to access save {}", user.id, save_id);
        return Err(HttpError::new(
            StatusCode::UNAUTHORIZED,
            "You do not have permission to view this save".to_string(),
        )
        .into());
    }

    let save =
        sqlx::query_as::<_, DbSave>("SELECT s.id, s.name, s.game, s.owner, COUNT(v.id) as version_count, MAX(v.timestamp) as most_recent_version FROM saves s LEFT JOIN versions v ON v.save_id = s.id WHERE s.id = $1 GROUP BY s.id;")
            .bind(save_id)
            .fetch_one(&db.0).await.map_err(|e| {
                error!("Database error while fetching save details: {}", e);
                ServerFnError::ServerError {
                    message: "Internal server error".to_string(),
                    code: 500,
                    details: None,
                }
    })?;

    Ok(save.to_save())
}

#[post("/api/save/{save_id}/name", auth: crate::auth::Session, db: crate::ServerDb)]
pub async fn get_save_name(save_id: i32) -> Result<String, ServerFnError> {
    let user = auth.require_user()?;

    if query_user_save_access(user.id, save_id, &db.0)
        .await?
        .is_none()
    {
        warn!(
            "User {} attempted to get the name of save {}",
            user.id, save_id
        );
        return Err(HttpError::new(
            StatusCode::UNAUTHORIZED,
            "You do not have permission to view this save".to_string(),
        )
        .into());
    }

    #[derive(sqlx::FromRow)]
    struct Name {
        name: String,
    }

    sqlx::query_as::<_, Name>("SELECT name FROM saves WHERE id = $1")
        .bind(save_id)
        .fetch_one(&db.0)
        .await
        .map(|s| s.name)
        .map_err(|e| {
            error!("Failed to get save name: {e:?}");
            ServerFnError::ServerError {
                message: "Internal server error".to_string(),
                code: 500,
                details: None,
            }
        })
}

#[delete("/api/save/{save_id}", auth: crate::auth::Session, db: crate::ServerDb)]
pub async fn delete_save(save_id: i32) -> Result<(), ServerFnError> {
    let user = auth.require_user()?;

    let access = query_user_save_access(user.id, save_id, &db.0).await?;

    if !matches!(access, Some(UserAccess::Owner)) {
        warn!(
            "User {} attempted to delete save {} without permission",
            user.id, save_id
        );
        return Err(HttpError::new(
            StatusCode::UNAUTHORIZED,
            "You do not have permission to delete this save".to_string(),
        )
        .into());
    }

    sqlx::query("DELETE FROM versions WHERE save_id = $1")
        .bind(save_id)
        .execute(&db.0)
        .await
        .map_err(|e| {
            error!("Failed to delete versions for save: {e:?}");
            ServerFnError::ServerError {
                message: "Internal server error".to_string(),
                code: 500,
                details: None,
            }
        })?;

    sqlx::query("DELETE FROM user_save_access WHERE save_id = $1")
        .bind(save_id)
        .execute(&db.0)
        .await
        .map_err(|e| {
            error!("Failed to delete user save access for save: {e:?}");
            ServerFnError::ServerError {
                message: "Internal server error".to_string(),
                code: 500,
                details: None,
            }
        })?;

    sqlx::query("DELETE FROM saves WHERE id = $1")
        .bind(save_id)
        .execute(&db.0)
        .await
        .map_err(|e| {
            error!("Failed to delete save: {e:?}");
            ServerFnError::ServerError {
                message: "Internal server error".to_string(),
                code: 500,
                details: None,
            }
        })?;

    Ok(())
}

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Game as G;

        macro_rules! g {
            ($self:ident, $(($variant:ident, $name:literal)$(,)?)*) => {
                match $self {
                    $(G::$variant => write!(f, $name),)*
                }
            };
        }

        g!(
            self,
            (IntoTheRadius2, "Into the Radius 2"),
            (Satisfactory, "Satisfactory"),
        )
    }
}
