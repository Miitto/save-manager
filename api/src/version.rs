use dioxus::{fullstack::MultipartFormData, prelude::*};

use crate::{SaveId, UserPreview, VersionId};

#[cfg(feature = "server")]
use crate::auth::RequireUser;

#[cfg(feature = "server")]
use crate::query_user_save_access;

#[cfg(feature = "server")]
use crate::UserAccessExt;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Version {
    pub id: VersionId,
    pub save_id: SaveId,
    pub version: i32,
    pub label: String,
    pub timestamp: i32,
    pub by: UserPreview,
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
            timestamp: v.timestamp,
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
        timestamp: version.timestamp,
        by: UserPreview {
            id: version.user_id,
            username: version.username,
        },
    })
}

pub fn is_version_name_valid(name: &str) -> bool {
    !name.trim().is_empty() && !name.contains('/') && !name.contains('\\')
}

#[post("/api/save/{save_id}/create", auth: crate::auth::Session, db: crate::ServerDb)]
pub async fn create_version(
    save_id: i32,
    mut form: MultipartFormData,
) -> Result<Version, ServerFnError> {
    let user = auth.require_user()?;

    let access = query_user_save_access(user.id, save_id, &db.0).await?;

    if !access.can_edit() {
        warn!(
            "User {} attempted to create a version for save {} without permission",
            user.id, save_id
        );
        return Err(HttpError::new(
            StatusCode::UNAUTHORIZED,
            "You do not have permission to create a version for this save".to_string(),
        )
        .into());
    }

    let mut label = None;
    let mut file_bytes = None;
    let mut file_name = None;

    while let Ok(Some(field)) = form.next_field().await {
        let name = field.name().unwrap_or_default();
        match name {
            "label" => label = Some(field.text().await.unwrap_or_default()),
            "file" => {
                file_name = field.file_name().map(|s| s.trim().to_string());
                file_bytes = Some(field.bytes().await.unwrap_or_default());
            }
            _ => {}
        }
    }

    if label.is_none()
        || file_bytes.is_none()
        || label.as_mut().is_some_and(|l| l.trim().is_empty())
        || file_bytes.as_ref().is_some_and(|b| b.is_empty())
    {
        return Err(HttpError::new(
            StatusCode::BAD_REQUEST,
            "Missing required fields".to_string(),
        )
        .into());
    }

    if file_name.as_ref().is_none() || file_name.as_ref().unwrap().is_empty() {
        return Err(
            HttpError::new(StatusCode::BAD_REQUEST, "Missing file name".to_string()).into(),
        );
    }

    let label = label.unwrap().trim().to_string();
    let file_bytes = file_bytes.unwrap();
    let file_name = file_name.unwrap();

    if !is_version_name_valid(&label) {
        return Err(
            HttpError::new(StatusCode::BAD_REQUEST, "Invalid version label".to_string()).into(),
        );
    }

    #[derive(sqlx::FromRow)]
    struct SaveIdentRow {
        name: String,
        game: crate::Game,
    }

    let version = sqlx::query_as::<_, DbVersion>(
        "INSERT INTO versions (save_id, label, by, version) VALUES ($1, $2, $3, (SELECT COALESCE(MAX(version), 0) FROM versions WHERE save_id = $1) + 1) RETURNING id, save_id, version, label, timestamp, by as user_id, (SELECT username FROM users WHERE id = by) as username;",
    )
    .bind(save_id)
    .bind(label)
    .bind(user.id)
    .fetch_one(&db.0)
    .await
    .map_err(|e| {
        error!("Failed to create version: {e:?}");
        ServerFnError::ServerError {
            message: "Internal server error".to_string(),
            code: 500,
            details: None,
        }
    })?;

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

    let file_path = format!(
        "./saves/{}/{:?}/{}/{}.zip",
        user.username, game, name, version.version
    );

    debug!("Creating version file at: {}", file_path);

    let file = std::fs::File::create(&file_path).map_err(|e| {
        error!("Failed to create version file: {e:?}");
        ServerFnError::ServerError {
            message: "Internal server error".to_string(),
            code: 500,
            details: None,
        }
    })?;

    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    use std::io::Write;

    zip.start_file(file_name, options);
    zip.write_all(&file_bytes);
    zip.finish();

    Ok(Version {
        id: version.id,
        save_id: version.save_id,
        version: version.version,
        label: version.label,
        timestamp: version.timestamp,
        by: UserPreview {
            id: version.user_id,
            username: version.username,
        },
    })
}

#[delete("/api/save/{save_id}/{version_id}", auth: crate::auth::Session, db: crate::ServerDb)]
pub async fn delete_version(save_id: i32, version_id: i32) -> Result<(), ServerFnError> {
    let user = auth.require_user()?;

    let access = query_user_save_access(user.id, save_id, &db.0).await?;

    if !access.can_edit() {
        warn!(
            "User {} attempted to delete version {} without permission",
            user.id, version_id
        );
        return Err(HttpError::new(
            StatusCode::UNAUTHORIZED,
            "You do not have permission to delete this version".to_string(),
        )
        .into());
    }

    #[derive(sqlx::FromRow)]
    struct SaveIdentRow {
        name: String,
        game: crate::Game,
        version: i32,
    };

    let SaveIdentRow { name, game, version } =
        sqlx::query_as::<_, SaveIdentRow>("SELECT s.name, s.game, v.version FROM saves s JOIN versions v ON s.id = v.save_id WHERE s.id = $1 AND v.id = $2")
            .bind(save_id)
            .bind(version_id)
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

    let file_path = format!(
        "./saves/{}/{:?}/{}/{}.zip",
        user.username, game, name, version
    );

    debug!("Deleting version file at: {}", file_path);

    std::fs::remove_file(&file_path).map_err(|e| {
        error!("Failed to delete version file: {e:?}");
        ServerFnError::ServerError {
            message: "Internal server error".to_string(),
            code: 500,
            details: None,
        }
    })?;

    sqlx::query("DELETE FROM versions WHERE id = $1")
        .bind(version_id)
        .execute(&db.0)
        .await
        .map_err(|e| {
            error!("Failed to delete version: {e:?}");
            ServerFnError::ServerError {
                message: "Internal server error".to_string(),
                code: 500,
                details: None,
            }
        })?;

    Ok(())
}

#[cfg(feature = "server")]
struct VersionFile {
    pub save_name: String,
    pub game: crate::Game,
    pub version: i32,
    pub path: std::path::PathBuf,
}

#[cfg(feature = "server")]
async fn get_version_file(
    save_id: i32,
    version_id: i32,
    auth: &crate::auth::Session,
    db: &crate::ServerDb,
) -> Result<VersionFile, ServerFnError> {
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

    #[derive(sqlx::FromRow)]
    struct SaveIdentRow {
        name: String,
        game: crate::Game,
        version: i32,
        username: String,
    }

    let SaveIdentRow { name, game, version, username } =
        sqlx::query_as::<_, SaveIdentRow>("SELECT s.name, s.game, v.version, u.username FROM saves s LEFT JOIN versions v ON s.id = v.save_id LEFT JOIN users u ON u.id = s.owner WHERE s.id = $1 AND v.id = $2")
            .bind(save_id)
            .bind(version_id)
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

    let file_path = format!("./saves/{}/{:?}/{}/{}.zip", username, game, name, version);

    Ok(VersionFile {
        save_name: name,
        game: game,
        version,
        path: std::path::PathBuf::from(file_path),
    })
}

#[get("/api/save/{save_id}/{version_id}/download", auth: crate::auth::Session, db: crate::ServerDb)]
pub async fn download_version(
    save_id: i32,
    version_id: i32,
) -> Result<dioxus_fullstack::FileStream, ServerFnError> {
    use tokio_util::io::ReaderStream;
    let file_path = get_version_file(save_id, version_id, &auth, &db).await?;

    let file_name = format!(
        "{:?}_{}_v{}.zip",
        file_path.game, file_path.save_name, file_path.version
    );

    let meta = file_path.path.metadata().map_err(|e| {
        error!("Failed to get version file metadata: {e:?}");
        ServerFnError::ServerError {
            message: "Internal server error".to_string(),
            code: 500,
            details: None,
        }
    })?;

    let size = meta.len();
    let file = tokio::fs::File::open(&file_path.path).await.map_err(|e| {
        error!("Failed to open version file: {e:?}");
        ServerFnError::ServerError {
            message: "Internal server error".to_string(),
            code: 500,
            details: None,
        }
    })?;

    let stream = ReaderStream::new(file);

    let body = axum::body::Body::from_stream(stream).into_data_stream();

    Ok(dioxus::fullstack::FileStream::from_raw(
        file_name,
        Some(size),
        "application/zip".to_string(),
        body,
    ))
}
