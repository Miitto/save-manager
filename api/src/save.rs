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

impl Game {
    pub fn iter() -> impl Iterator<Item = Game> {
        [Game::IntoTheRadius2, Game::Satisfactory].into_iter()
    }
}

pub type SaveId = i32;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct Save {
    pub id: SaveId,
    pub name: String,
    pub game: Game,
    pub owner: crate::UserId,
    pub version_count: i32,
    pub most_recent_version: Option<i32>,
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

    let saves = sqlx::query_as::<_, Save>(
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

    Ok(saves.into_iter().collect())
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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "server", derive(sqlx::FromRow))]
pub struct NamedUserAccess {
    #[cfg_attr(feature = "server", sqlx(flatten))]
    pub user: UserPreview,
    pub access: UserAccess,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct SaveAccess {
    pub owner: UserPreview,
    pub access_list: Vec<NamedUserAccess>,
}

#[post("/api/save/{save_id}/access", auth: crate::auth::Session, db: crate::ServerDb)]
pub async fn get_save_access(save_id: SaveId) -> Result<SaveAccess, ServerFnError> {
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

    let owner = sqlx::query_as::<_, UserPreview>(
        "SELECT id, username FROM users WHERE id = (SELECT owner FROM saves WHERE id = $1)",
    )
    .bind(save_id)
    .fetch_one(&db.0)
    .await
    .map_err(|e| {
        error!("Database error while fetching save owner: {}", e);
        ServerFnError::ServerError {
            message: "Internal server error".to_string(),
            code: 500,
            details: None,
        }
    })?;

    let access_list = sqlx::query_as::<_, NamedUserAccess>(
        "SELECT u.id, u.username, usa.access FROM user_save_access usa LEFT JOIN users u ON usa.user_id = u.id WHERE usa.save_id = $1 GROUP BY u.username",
    )
    .bind(save_id)
    .fetch_all(&db.0)
    .await
    .map_err(|e| {
        error!("Database error while fetching save access: {}", e);
        ServerFnError::ServerError {
            message: "Internal server error".to_string(),
            code: 500,
            details: None,
        }
    })?;

    Ok(SaveAccess { owner, access_list })
}

#[post("/api/save/{save_id}/access/user", auth: crate::auth::Session, db: crate::ServerDb)]
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
        sqlx::query_as::<_, Save>("SELECT s.id, s.name, s.game, s.owner, COUNT(v.id) as version_count, MAX(v.timestamp) as most_recent_version FROM saves s LEFT JOIN versions v ON v.save_id = s.id WHERE s.id = $1 GROUP BY s.id;")
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

#[post("/api/save/create", auth: crate::auth::Session, db: crate::ServerDb)]
pub async fn create_save(name: String, game: Game) -> Result<Save, ServerFnError> {
    let user = auth.require_user()?;

    std::fs::create_dir_all(format!("./saves/{}/{:?}/{}", user.username, game, name)).map_err(
        |e| {
            error!("Failed to create save directory: {e:?}");
            ServerFnError::ServerError {
                message: "Internal server error".to_string(),
                code: 500,
                details: None,
            }
        },
    )?;

    #[derive(sqlx::FromRow)]
    struct SaveIdRow {
        id: SaveId,
    }

    let SaveIdRow { id } = sqlx::query_as::<_, SaveIdRow>(
        "INSERT INTO saves (name, game, owner) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(&name)
    .bind(game)
    .bind(user.id)
    .fetch_one(&db.0)
    .await
    .map_err(|e| {
        error!("Failed to create save: {e:?}");
        ServerFnError::ServerError {
            message: "Internal server error".to_string(),
            code: 500,
            details: None,
        }
    })?;

    Ok(Save {
        id,
        name,
        game,
        owner: user.id,
        version_count: 0,
        most_recent_version: None,
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

    #[derive(sqlx::FromRow)]
    struct SaveIdentRow {
        name: String,
        game: crate::Game,
    }

    let SaveIdentRow { name, game } =
        sqlx::query_as::<_, SaveIdentRow>("SELECT name, game FROM saves WHERE id = $1")
            .bind(save_id)
            .fetch_one(&db.0)
            .await
            .map_err(|e| {
                error!("Failed to fetch save game: {e:?}");
                ServerFnError::ServerError {
                    message: "Internal server error".to_string(),
                    code: 500,
                    details: None,
                }
            })?;

    let folder_path = format!("./saves/{}/{:?}/{}", user.username, game, name);

    debug!("Deleting save folder at: {}", folder_path);

    std::fs::remove_dir_all(folder_path).map_err(|e| {
        error!("Failed to delete save folder: {e:?}");
        ServerFnError::ServerError {
            message: "Internal server error".to_string(),
            code: 500,
            details: None,
        }
    })?;

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

#[cfg(feature = "server")]
#[derive(sqlx::FromRow)]
struct OwnedSave {
    pub id: SaveId,
    pub owner: crate::UserId,
}

#[post("/api/save/{save_id}/access/add", auth: crate::auth::Session, db: crate::ServerDb)]
pub async fn add_user_save_access(save_id: i32, username: String) -> Result<(), ServerFnError> {
    let user = auth.require_user()?;

    let save = sqlx::query_as::<_, OwnedSave>("SELECT id, owner FROM saves WHERE id = $1")
        .bind(save_id)
        .fetch_one(&db.0)
        .await
        .map_err(|e| {
            error!("Failed to fetch save: {e:?}");
            ServerFnError::ServerError {
                message: "Internal server error".to_string(),
                code: 500,
                details: None,
            }
        })?;

    if user.id != save.owner {
        warn!(
            "User {} attempted to add access for user {} to save {} without permission",
            user.id, username, save_id
        );
        return Err(HttpError::new(
            StatusCode::UNAUTHORIZED,
            "You do not have permission to modify this save".to_string(),
        )
        .into());
    }

    let new_user =
        sqlx::query_as::<_, UserPreview>("SELECT id, username FROM users WHERE username = $1")
            .bind(username)
            .fetch_one(&db.0)
            .await
            .map_err(|e| {
                error!("Failed to fetch user: {e:?}");
                ServerFnError::ServerError {
                    message: "User does not exist.".to_string(),
                    code: 500,
                    details: None,
                }
            })?;

    if new_user.id == save.owner {
        return Err(HttpError::new(
            StatusCode::BAD_REQUEST,
            "Cannot modify access for the owner of the save".to_string(),
        )
        .into());
    }

    sqlx::query(
        "INSERT OR IGNORE INTO user_save_access (user_id, save_id, access) VALUES ($1, $2, $3)",
    )
    .bind(new_user.id)
    .bind(save_id)
    .bind(UserAccess::View as i32)
    .execute(&db.0)
    .await
    .map_err(|e| {
        error!("Failed to add user save access: {e:?}");
        ServerFnError::ServerError {
            message: "Internal server error".to_string(),
            code: 500,
            details: None,
        }
    })?;

    Ok(())
}

#[post("/api/save/{save_id}/access/remove", auth: crate::auth::Session, db: crate::ServerDb)]
pub async fn remove_user_save_access(save_id: i32, username: String) -> Result<(), ServerFnError> {
    let user = auth.require_user()?;

    let save = sqlx::query_as::<_, OwnedSave>("SELECT id, owner FROM saves WHERE id = $1 LIMIT 1")
        .bind(save_id)
        .fetch_one(&db.0)
        .await
        .map_err(|e| {
            error!("Failed to fetch save: {e:?}");
            ServerFnError::ServerError {
                message: "Internal server error".to_string(),
                code: 500,
                details: None,
            }
        })?;

    if user.id != save.owner {
        warn!(
            "User {} attempted to remove access for user {} from save {} without permission",
            user.id, username, save_id
        );
        return Err(HttpError::new(
            StatusCode::UNAUTHORIZED,
            "You do not have permission to modify this save".to_string(),
        )
        .into());
    }

    let target_user =
        sqlx::query_as::<_, UserPreview>("SELECT id, username FROM users WHERE username = $1")
            .bind(username)
            .fetch_one(&db.0)
            .await
            .map_err(|e| {
                error!("Failed to fetch user: {e:?}");
                ServerFnError::ServerError {
                    message: "User does not exist.".to_string(),
                    code: 500,
                    details: None,
                }
            })?;

    if target_user.id == save.owner {
        return Err(HttpError::new(
            StatusCode::BAD_REQUEST,
            "Cannot modify access for the owner of the save".to_string(),
        )
        .into());
    }

    sqlx::query("DELETE FROM user_save_access WHERE user_id = $1 AND save_id = $2")
        .bind(target_user.id)
        .bind(save_id)
        .execute(&db.0)
        .await
        .map_err(|e| {
            error!("Failed to remove user save access: {e:?}");
            ServerFnError::ServerError {
                message: "Internal server error".to_string(),
                code: 500,
                details: None,
            }
        })?;

    Ok(())
}

#[post("/api/save/{save_id}/access/update", auth: crate::auth::Session, db: crate::ServerDb)]
pub async fn update_user_save_access(
    save_id: i32,
    username: String,
    access: UserAccess,
) -> Result<(), ServerFnError> {
    let user = auth.require_user()?;

    let save = sqlx::query_as::<_, OwnedSave>("SELECT id, owner FROM saves WHERE id = $1 LIMIT 1")
        .bind(save_id)
        .fetch_one(&db.0)
        .await
        .map_err(|e| {
            error!("Failed to fetch save: {e:?}");
            ServerFnError::ServerError {
                message: "Internal server error".to_string(),
                code: 500,
                details: None,
            }
        })?;

    if user.id != save.owner {
        warn!(
            "User {} attempted to update access for user {} on save {} without permission",
            user.id, username, save_id
        );
        return Err(HttpError::new(
            StatusCode::UNAUTHORIZED,
            "You do not have permission to modify this save".to_string(),
        )
        .into());
    }

    let target_user =
        sqlx::query_as::<_, UserPreview>("SELECT id, username FROM users WHERE username = $1")
            .bind(username)
            .fetch_one(&db.0)
            .await
            .map_err(|e| {
                error!("Failed to fetch user: {e:?}");
                ServerFnError::ServerError {
                    message: "User does not exist.".to_string(),
                    code: 500,
                    details: None,
                }
            })?;

    if target_user.id == save.owner {
        return Err(HttpError::new(
            StatusCode::BAD_REQUEST,
            "Cannot modify access for the owner of the save".to_string(),
        )
        .into());
    }

    sqlx::query("UPDATE user_save_access SET access = $1 WHERE user_id = $2 AND save_id = $3")
        .bind(access as i32)
        .bind(target_user.id)
        .bind(save_id)
        .execute(&db.0)
        .await
        .map_err(|e| {
            error!("Failed to update user save access: {e:?}");
            ServerFnError::ServerError {
                message: "Internal server error".to_string(),
                code: 500,
                details: None,
            }
        })?;

    Ok(())
}

macro_rules! display_match {
            ($formatter:ident, $enum:ty, $self:ident, $(($variant:ident, $name:literal)$(,)?)*) => {
                match $self {
                    $(<$enum>::$variant => write!($formatter, $name),)*
                }
            };
        }

impl std::fmt::Display for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_match!(
            f,
            Game,
            self,
            (IntoTheRadius2, "Into the Radius 2"),
            (Satisfactory, "Satisfactory"),
        )
    }
}

impl std::fmt::Display for UserAccess {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        display_match!(
            f,
            UserAccess,
            self,
            (View, "View"),
            (Edit, "Edit"),
            (Owner, "Owner"),
        )
    }
}
