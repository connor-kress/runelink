use crate::{
    auth::{Principal, authorize},
    error::ApiResult,
    ops,
    state::AppState,
};
use axum::{
    extract::{Json, Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use log::info;
use runelink_types::NewChannel;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct ChannelQueryParams {
    pub target_domain: Option<String>,
}

/// POST /servers/{server_id}/channels
pub async fn create(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(server_id): Path<Uuid>,
    Query(params): Query<ChannelQueryParams>,
    Json(new_channel): Json<NewChannel>,
) -> ApiResult<impl IntoResponse> {
    info!(
        "POST /servers/{server_id}/channels?target_domain={:?}\nnew_channel = {:#?}",
        params.target_domain, new_channel,
    );
    let session = authorize(
        &state,
        Principal::from_client_headers(&headers, &state)?,
        ops::channels::auth::create(server_id),
    )
    .await?;
    let channel = ops::channels::create(
        &state,
        &session,
        server_id,
        &new_channel,
        params.target_domain.as_deref(),
    )
    .await?;
    Ok((StatusCode::CREATED, Json(channel)))
}

/// GET /channels
pub async fn get_all(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(params): Query<ChannelQueryParams>,
) -> ApiResult<impl IntoResponse> {
    info!("GET /channels?target_domain={:?}", params.target_domain);
    let session = authorize(
        &state,
        Principal::from_client_headers(&headers, &state)?,
        ops::channels::auth::get_all(),
    )
    .await?;
    let channels = ops::channels::get_all(
        &state,
        &session,
        params.target_domain.as_deref(),
    )
    .await?;
    Ok((StatusCode::OK, Json(channels)))
}

/// GET /servers/{server_id}/channels
pub async fn get_by_server(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(server_id): Path<Uuid>,
    Query(params): Query<ChannelQueryParams>,
) -> ApiResult<impl IntoResponse> {
    info!(
        "GET /servers/{server_id}/channels?target_domain={:?}",
        params.target_domain
    );
    let session = authorize(
        &state,
        Principal::from_client_headers(&headers, &state)?,
        ops::channels::auth::get_by_server(server_id),
    )
    .await?;
    let channels = ops::channels::get_by_server(
        &state,
        &session,
        server_id,
        params.target_domain.as_deref(),
    )
    .await?;
    Ok((StatusCode::OK, Json(channels)))
}

/// GET /servers/{server_id}/channels/{channel_id}
pub async fn get_by_id(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((server_id, channel_id)): Path<(Uuid, Uuid)>,
    Query(params): Query<ChannelQueryParams>,
) -> ApiResult<impl IntoResponse> {
    info!(
        "GET /servers/{server_id}/channels/{channel_id}?target_domain={:?}",
        params.target_domain
    );
    let session = authorize(
        &state,
        Principal::from_client_headers(&headers, &state)?,
        ops::channels::auth::get_by_id(server_id),
    )
    .await?;
    let channel = ops::channels::get_by_id(
        &state,
        &session,
        server_id,
        channel_id,
        params.target_domain.as_deref(),
    )
    .await?;
    Ok((StatusCode::OK, Json(channel)))
}

/// DELETE /servers/{server_id}/channels/{channel_id}
pub async fn delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path((server_id, channel_id)): Path<(Uuid, Uuid)>,
    Query(params): Query<ChannelQueryParams>,
) -> ApiResult<impl IntoResponse> {
    info!(
        "DELETE /servers/{server_id}/channels/{channel_id}?target_domain={:?}",
        params.target_domain
    );
    let session = authorize(
        &state,
        Principal::from_client_headers(&headers, &state)?,
        ops::channels::auth::delete(server_id),
    )
    .await?;
    ops::channels::delete(
        &state,
        &session,
        server_id,
        channel_id,
        params.target_domain.as_deref(),
    )
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Federation endpoints (server-to-server authentication required).
pub mod federated {
    use super::*;

    /// POST /federation/servers/{server_id}/channels
    pub async fn create(
        State(state): State<AppState>,
        headers: HeaderMap,
        Path(server_id): Path<Uuid>,
        Json(new_channel): Json<NewChannel>,
    ) -> ApiResult<impl IntoResponse> {
        info!(
            "POST /federation/servers/{server_id}/channels\nnew_channel = {:#?}",
            new_channel
        );
        let session = authorize(
            &state,
            Principal::from_federation_headers(&headers, &state).await?,
            ops::channels::auth::federated::create(server_id),
        )
        .await?;
        let channel = ops::channels::create(
            &state,
            &session,
            server_id,
            &new_channel,
            None,
        )
        .await?;
        Ok((StatusCode::CREATED, Json(channel)))
    }

    /// GET /federation/channels
    pub async fn get_all(
        State(state): State<AppState>,
        headers: HeaderMap,
    ) -> ApiResult<impl IntoResponse> {
        info!("GET /federation/channels");
        let session = authorize(
            &state,
            Principal::from_federation_headers(&headers, &state).await?,
            ops::channels::auth::federated::get_all(),
        )
        .await?;
        let channels = ops::channels::get_all(&state, &session, None).await?;
        Ok((StatusCode::OK, Json(channels)))
    }

    /// GET /federation/servers/{server_id}/channels
    pub async fn get_by_server(
        State(state): State<AppState>,
        headers: HeaderMap,
        Path(server_id): Path<Uuid>,
    ) -> ApiResult<impl IntoResponse> {
        info!("GET /federation/servers/{server_id}/channels");
        let session = authorize(
            &state,
            Principal::from_federation_headers(&headers, &state).await?,
            ops::channels::auth::federated::get_by_server(server_id),
        )
        .await?;
        let channels =
            ops::channels::get_by_server(&state, &session, server_id, None)
                .await?;
        Ok((StatusCode::OK, Json(channels)))
    }

    /// GET /federation/servers/{server_id}/channels/{channel_id}
    pub async fn get_by_id(
        State(state): State<AppState>,
        headers: HeaderMap,
        Path((server_id, channel_id)): Path<(Uuid, Uuid)>,
    ) -> ApiResult<impl IntoResponse> {
        info!("GET /federation/servers/{server_id}/channels/{channel_id}");
        let session = authorize(
            &state,
            Principal::from_federation_headers(&headers, &state).await?,
            ops::channels::auth::federated::get_by_id(server_id),
        )
        .await?;
        let channel = ops::channels::get_by_id(
            &state, &session, server_id, channel_id, None,
        )
        .await?;
        Ok((StatusCode::OK, Json(channel)))
    }

    /// DELETE /federation/servers/{server_id}/channels/{channel_id}
    pub async fn delete(
        State(state): State<AppState>,
        headers: HeaderMap,
        Path((server_id, channel_id)): Path<(Uuid, Uuid)>,
    ) -> ApiResult<impl IntoResponse> {
        info!("DELETE /federation/servers/{server_id}/channels/{channel_id}");
        let session = authorize(
            &state,
            Principal::from_federation_headers(&headers, &state).await?,
            ops::channels::auth::federated::delete(server_id),
        )
        .await?;
        ops::channels::delete(&state, &session, server_id, channel_id, None)
            .await?;
        Ok(StatusCode::NO_CONTENT)
    }
}
