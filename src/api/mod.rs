use std::str::FromStr;
use std::sync::Arc;

use axum::{extract::{Path, Query, State}, http::StatusCode, response::IntoResponse, routing::{delete, get, patch, post, put}, Json, Router};
use serde::{Deserialize, Serialize};

use crate::model::Profile;
use crate::store::{Kind, Store, StoreError};

#[derive(Clone)]
pub struct AppState {
    pub store: Arc<Store>,
    pub base_url: Option<String>,
    pub read_only: bool,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/ping", get(ping))
        .route("/api/profiles", get(list_profiles))
        .route("/api/profiles/:kind/:profile", get(get_profile))
        .route("/api/profiles/:kind/:profile", put(put_profile))
        .route("/api/profiles/:kind/:profile", patch(patch_profile))
        .route("/api/profiles/:kind", post(create_profile))
        .route("/api/profiles/:kind/:profile", delete(delete_profile))
        .with_state(state)
}

#[derive(Serialize)]
struct Pong { message: &'static str }

async fn ping() -> Json<Pong> { Json(Pong { message: "pong" }) }

// ---- helpers ----

#[derive(Debug, Clone, Copy)]
struct KindParam(Kind);

impl FromStr for KindParam {
    type Err = ApiError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "robots" => Ok(Self(Kind::Robots)),
            "teleoperators" => Ok(Self(Kind::Teleoperators)),
            _ => Err(ApiError::bad_request("invalid kind", Some(serde_json::json!({"kind": s})))),
        }
    }
}

#[derive(Debug, Serialize)]
struct ApiErrorBody {
    code: u16,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<serde_json::Value>,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
    details: Option<serde_json::Value>,
}

impl ApiError {
    fn from_store(e: StoreError) -> Self {
        match e {
            StoreError::NotFound(msg) => Self { status: StatusCode::NOT_FOUND, message: msg, details: None },
            StoreError::Validation(msg) => Self { status: StatusCode::BAD_REQUEST, message: msg, details: None },
            StoreError::Json(err) => Self { status: StatusCode::BAD_REQUEST, message: "invalid json".into(), details: Some(serde_json::json!({"error": err.to_string()})) },
            StoreError::Io(err) => Self { status: StatusCode::INTERNAL_SERVER_ERROR, message: "io error".into(), details: Some(serde_json::json!({"error": err.to_string()})) },
        }
    }
    fn bad_request(msg: &str, details: Option<serde_json::Value>) -> Self {
        Self { status: StatusCode::BAD_REQUEST, message: msg.into(), details }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = ApiErrorBody { code: self.status.as_u16(), message: self.message, details: self.details };
        (self.status, Json(body)).into_response()
    }
}

// ---- endpoints ----

#[derive(Deserialize)]
struct ListQuery { kind: String }

#[derive(Serialize)]
struct ListResponse { items: Vec<String> }

async fn list_profiles(State(state): State<AppState>, Query(q): Query<ListQuery>) -> Result<Json<ListResponse>, ApiError> {
    let KindParam(kind) = KindParam::from_str(&q.kind)?;
    let items = state
        .store
        .list_profiles(kind)
        .map_err(ApiError::from_store)?
        .into_iter()
        .map(|m| m.name)
        .collect();
    Ok(Json(ListResponse { items }))
}

#[derive(Serialize)]
struct ProfileResponse(Profile);

async fn get_profile(State(state): State<AppState>, Path((kind, profile)): Path<(String, String)>) -> Result<Json<ProfileResponse>, ApiError> {
    let KindParam(kind) = KindParam::from_str(&kind)?;
    let p = state.store.read_profile(kind, &profile).map_err(ApiError::from_store)?;
    Ok(Json(ProfileResponse(p)))
}

async fn put_profile(State(state): State<AppState>, Path((kind, profile)): Path<(String, String)>, Json(body): Json<Profile>) -> Result<StatusCode, ApiError> {
    let KindParam(kind) = KindParam::from_str(&kind)?;
    state
        .store
        .write_profile(kind, &profile, &body, true)
        .map_err(ApiError::from_store)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct PatchJoint {
    id: Option<i32>,
    drive_mode: Option<i32>,
    homing_offset: Option<i32>,
    range_min: Option<i32>,
    range_max: Option<i32>,
}

type PatchBody = std::collections::HashMap<String, PatchJoint>;

async fn patch_profile(State(state): State<AppState>, Path((kind, profile)): Path<(String, String)>, Json(patch): Json<PatchBody>) -> Result<StatusCode, ApiError> {
    let KindParam(kind) = KindParam::from_str(&kind)?;
    let mut current = state.store.read_profile(kind, &profile).map_err(ApiError::from_store)?;
    for (name, pj) in patch {
        if let Some(mut j) = current.0.get(&name).cloned() {
            if let Some(v) = pj.id { j.id = v; }
            if let Some(v) = pj.drive_mode { j.drive_mode = v; }
            if let Some(v) = pj.homing_offset { j.homing_offset = v; }
            if let Some(v) = pj.range_min { j.range_min = v; }
            if let Some(v) = pj.range_max { j.range_max = v; }
            j.validate().map_err(|e| ApiError::bad_request("invalid joint", Some(serde_json::json!({"joint": name, "error": e}))))?;
            current.0.insert(name, j);
        } else {
            return Err(ApiError::bad_request("unknown joint", Some(serde_json::json!(name))));
        }
    }
    state.store.write_profile(kind, &profile, &current, true).map_err(ApiError::from_store)?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
struct CreateBody {
    name: String,
    #[serde(default)]
    profile: Option<Profile>,
}

async fn create_profile(State(state): State<AppState>, Path(kind): Path<String>, Json(body): Json<CreateBody>) -> Result<StatusCode, ApiError> {
    let KindParam(kind) = KindParam::from_str(&kind)?;
    let profile = body.profile.unwrap_or_else(|| Profile(Default::default()));
    state.store.write_profile(kind, &body.name, &profile, false).map_err(ApiError::from_store)?;
    Ok(StatusCode::CREATED)
}

async fn delete_profile(State(state): State<AppState>, Path((kind, profile)): Path<(String, String)>) -> Result<StatusCode, ApiError> {
    let KindParam(kind) = KindParam::from_str(&kind)?;
    state.store.delete_profile(kind, &profile).map_err(ApiError::from_store)?;
    Ok(StatusCode::NO_CONTENT)
}
