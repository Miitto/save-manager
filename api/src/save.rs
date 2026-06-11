use dioxus::prelude::*;

#[cfg(feature = "server")]
use crate::auth::RequireUser;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "server", derive(sqlx::Type))]
pub enum Game {
    IntoTheRadius2,
}

pub type SaveId = i32;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Save {
    pub id: SaveId,
    pub name: String,
    pub game: Game,
    pub version_count: i32,
}

pub type VersionId = i32;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct UserPreview {
    pub id: crate::UserId,
    pub username: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Version {
    pub id: VersionId,
    pub save_id: SaveId,
    pub version: i32,
    pub label: String,
    pub timestamp: std::time::SystemTime,
    pub by: UserPreview,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "server", derive(sqlx::Type))]
pub enum UserAccess {
    View,
    Edit,
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

    let saves = sqlx::query_as::<_, Save>(
        "SELECT s.id, s.name, s.game, COUNT(v.id) as version_count FROM saves s LEFT JOIN versions v ON s.id = v.save_id JOIN user_save_access usa ON s.id = usa.save_id WHERE usa.user_id = $1 GROUP BY s.id;",
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

    Ok(saves)
}

#[cfg(feature = "server")]
async fn get_user_save_access(
    user_id: crate::UserId,
    save_id: SaveId,
    db: &sqlx::SqlitePool,
) -> Result<Option<UserAccess>, ServerFnError> {
    sqlx::query_as::<_, UserSaveAccess>(
        "SELECT * FROM user_save_access WHERE user_id = $1 AND save_id = $2;",
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

#[post("/api/save/{save_id}", auth: crate::auth::Session, db: crate::ServerDb)]
pub async fn get_save_details(save_id: SaveId) -> Result<Save, ServerFnError> {
    let user = auth.require_user()?;

    if get_user_save_access(user.id, save_id, &db.0)
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
        sqlx::query_as::<_, Save>("SELECT s.id, s.name, s.game, COUNT(v.id) as version_count FROM saves s LEFT JOIN versions v ON v.save_id = s.id WHERE s.id = $1;")
            .bind(save_id)
            .fetch_one(&db.0).await.map_err(|e| {
                error!("Database error while fetching save details: {}", e);
                ServerFnError::ServerError {
                    message: "Internal server error".to_string(),
                    code: 500,
                    details: None,
                }
    })?;

    Ok(save)
}

#[cfg(feature = "server")]
#[derive(Debug, Clone, sqlx::FromRow)]
struct DbVersion {
    pub id: VersionId,
    pub save_id: SaveId,
    pub version: i32,
    pub label: String,
    pub timestamp: i32,
    pub user_id: crate::UserId,
    pub username: String,
}

#[post("/api/save/{save_id}/versions", auth: crate::auth::Session, db: crate::ServerDb)]
pub async fn get_save_versions(save_id: SaveId) -> Result<Vec<Version>, ServerFnError> {
    let user = auth.require_user()?;

    if get_user_save_access(user.id, save_id, &db.0)
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

    let versions = sqlx::query_as::<_, DbVersion>(
        "SELECT v.id, v.save_id, v.version, v.label, v.timestamp, u.id as user_id, u.username as username FROM versions v JOIN users u ON v.by = u.id WHERE v.save_id = $1 ORDER BY v.version DESC;",
    ).bind(save_id).fetch_all(&db.0).await.map_err(|e| {
        error!("Database error while fetching save versions: {}", e);
        ServerFnError::ServerError {
            message: "Internal server error".to_string(),
            code: 500,
            details: None,
        }
    })?;

    debug!("Save {} has {} versions", save_id, versions.len());

    Ok(versions
        .into_iter()
        .map(|v| Version {
            id: v.id,
            save_id: v.save_id,
            version: v.version,
            label: v.label,
            timestamp: std::time::UNIX_EPOCH + std::time::Duration::from_secs(v.timestamp as u64),
            by: UserPreview {
                id: v.user_id,
                username: v.username,
            },
        })
        .collect())
}

#[post("/api/save/{save_id}/{version_id}", auth: crate::auth::Session, db: crate::ServerDb)]
pub async fn get_version_details(save_id: i32, version_id: i32) -> Result<Version, ServerFnError> {
    let user = auth.require_user()?;

    if get_user_save_access(user.id, save_id, &db.0)
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

    let version = sqlx::query_as::<_, DbVersion>(
        "SELECT v.id, v.save_id, v.version, v.label, v.timestamp, u.id as user_id, u.username as username FROM versions v JOIN users u ON v.by = u.id WHERE v.id = $1 ORDER BY v.version DESC;",
    ).bind(version_id).fetch_one(&db.0).await.map_err(|e| {
        error!("Database error while fetching version: {}", e);
        ServerFnError::ServerError {
            message: "Internal server error".to_string(),
            code: 500,
            details: None,
        }
    })?;

    Ok(Version {
        id: version.id,
        save_id: version.save_id,
        version: version.version,
        label: version.label,
        timestamp: std::time::UNIX_EPOCH + std::time::Duration::from_secs(version.timestamp as u64),
        by: UserPreview {
            id: version.user_id,
            username: version.username,
        },
    })
}

#[post("/api/save/{save_id}/name", auth: crate::auth::Session, db: crate::ServerDb)]
pub async fn get_save_name(save_id: i32) -> Result<String, ServerFnError> {
    let user = auth.require_user()?;

    if get_user_save_access(user.id, save_id, &db.0)
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

#[post("/api/save/{save_id}/modify", auth: crate::auth::Session, db: crate::ServerDb)]
pub async fn can_modify_save(save_id: i32) -> Result<bool, ServerFnError> {
    let user = if let Ok(u) = auth.require_user() {
        u
    } else {
        return Ok(false);
    };

    let access = get_user_save_access(user.id, save_id, &db.0)
        .await
        .map_err(|e| {
            error!(
                "Failed to get user save access for user {}, save {}: {e:?}",
                user.id, save_id,
            );
            ServerFnError::ServerError {
                message: "Internal Server Error".to_string(),
                code: 500,
                details: None,
            }
        })?;

    Ok(access.is_some_and(|a| a == UserAccess::Edit))
}
