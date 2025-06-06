use crate::{auth::AuthBuilder, error::ApiError, queries, state::AppState};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use runelink_types::{NewServer, NewServerMember, ServerWithChannels};
use uuid::Uuid;

/// POST /api/servers
pub async fn create_server(
    State(state): State<AppState>,
    Json(new_server): Json<NewServer>,
) -> Result<impl IntoResponse, ApiError> {
    // TODO: get user id/session tokens
    AuthBuilder::new(Some(new_server.user_id))
        .admin()
        .build(&state.db_pool)
        .await?;
    let server = queries::insert_server(&state, &new_server).await?;
    let new_member = NewServerMember::admin(
        new_server.user_id,
        new_server.user_domain,
    );
    queries::add_user_to_server(&state.db_pool, server.id, &new_member).await?;
    Ok((StatusCode::CREATED, Json(server)))
}

/// GET /api/servers
pub async fn list_servers(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_all_servers(&state).await.map(Json)
}

/// GET /api/servers/{server_id}
pub async fn get_server_by_id_handler(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_server_by_id(&state, server_id).await.map(Json)
}

/// GET /api/servers/{server_id}/with_channels
pub async fn get_server_with_channels_handler(
    State(state): State<AppState>,
    Path(server_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let (server, channels) = tokio::join!(
        queries::get_server_by_id(&state, server_id),
        queries::get_channels_by_server(&state.db_pool, server_id),
    );
    Ok(Json(ServerWithChannels {
        server: server?,
        channels: channels?,
    }))
}

/// GET /api/users/{user_id}/servers
pub async fn list_server_memberships_by_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    queries::get_all_memberships_for_user(&state, user_id).await.map(Json)
}
