use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use std::collections::HashMap;

use crate::{
    db,
    models::{AlertRule, AlertRuleInput},
    AppState,
};

pub async fn get_settings(
    State(state): State<AppState>,
) -> Result<Json<HashMap<String, String>>, StatusCode> {
    let settings = db::get_all_settings(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let map: HashMap<String, String> = settings.into_iter().map(|s| (s.key, s.value)).collect();

    Ok(Json(map))
}

pub async fn update_settings(
    State(state): State<AppState>,
    Json(settings): Json<HashMap<String, String>>,
) -> Result<StatusCode, StatusCode> {
    for (key, value) in settings {
        db::set_setting(&state.db, &key, &value)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    Ok(StatusCode::OK)
}

pub async fn get_alert_rules(
    State(state): State<AppState>,
) -> Result<Json<Vec<AlertRule>>, StatusCode> {
    let rules = db::get_alert_rules(&state.db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rules))
}

pub async fn create_alert_rule(
    State(state): State<AppState>,
    Json(input): Json<AlertRuleInput>,
) -> Result<(StatusCode, Json<AlertRule>), StatusCode> {
    let rule = db::create_alert_rule(&state.db, &input)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok((StatusCode::CREATED, Json(rule)))
}

pub async fn delete_alert_rule(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    let deleted = db::delete_alert_rule(&state.db, id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}
