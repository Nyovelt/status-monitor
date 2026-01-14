use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use crate::{
    db,
    models::{ClientResponse, CreateClientRequest, CreateClientResponse},
    AppState,
};

pub async fn list_clients(
    State(state): State<AppState>,
) -> Result<Json<Vec<ClientResponse>>, StatusCode> {
    let clients = db::get_all_clients(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(clients.into_iter().map(ClientResponse::from).collect()))
}

pub async fn get_client(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ClientResponse>, StatusCode> {
    let client = db::get_client_by_id(&state.db, &id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(ClientResponse::from(client)))
}

pub async fn create_client(
    State(state): State<AppState>,
    Json(input): Json<CreateClientRequest>,
) -> Result<(StatusCode, Json<CreateClientResponse>), StatusCode> {
    let client = db::create_client(&state.db, &input.hostname)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((
        StatusCode::CREATED,
        Json(CreateClientResponse {
            id: client.id,
            hostname: client.hostname,
            token: client.token,
        }),
    ))
}

pub async fn delete_client(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    let deleted = db::delete_client(&state.db, &id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
