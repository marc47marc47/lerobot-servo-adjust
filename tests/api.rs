use std::collections::HashMap;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use serde_json::json;
use tower::util::ServiceExt;
use axum::body;

use lerobot_servo_adjust::api::{self, AppState};
use lerobot_servo_adjust::model::{Joint, Profile};
use lerobot_servo_adjust::store::Store;

fn build_app(tmp: &tempfile::TempDir) -> Router {
    let root = tmp.path().to_path_buf();
    std::fs::create_dir_all(root.join("robots")).unwrap();
    std::fs::create_dir_all(root.join("teleoperators")).unwrap();
    let store = Arc::new(Store::new(root));
    let state = AppState { store, base_url: None, read_only: false };
    Router::new().merge(api::router(state))
}

#[tokio::test]
async fn api_crud_profile() {
    let tmp = tempfile::tempdir().unwrap();
    let app = build_app(&tmp);

    // list empty
    let res = app
        .clone()
        .oneshot(Request::builder().uri("/api/profiles?kind=robots").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body = body::to_bytes(res.into_body(), 1024 * 1024).await.unwrap();
    let v: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(v["items"].as_array().unwrap().len(), 0);

    // create
    let mut map = HashMap::new();
    map.insert(
        "j1".to_string(),
        Joint { id: 1, drive_mode: 0, homing_offset: 0, range_min: 1, range_max: 10 },
    );
    let profile = Profile(map);
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/profiles/robots")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&json!({"name": "p1", "profile": profile})).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::CREATED);

    // list has p1
    let res = app
        .clone()
        .oneshot(Request::builder().uri("/api/profiles?kind=robots").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let v: serde_json::Value = serde_json::from_slice(&body::to_bytes(res.into_body(), 1024 * 1024).await.unwrap()).unwrap();
    assert_eq!(v["items"], json!(["p1"]));

    // get p1
    let res = app
        .clone()
        .oneshot(Request::builder().uri("/api/profiles/robots/p1").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);

    // patch p1
    let patch = json!({"j1": {"range_max": 20}});
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri("/api/profiles/robots/p1")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&patch).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // put p1
    let mut map2 = HashMap::new();
    map2.insert(
        "j1".to_string(),
        Joint { id: 2, drive_mode: 1, homing_offset: 5, range_min: 2, range_max: 30 },
    );
    let profile2 = Profile(map2);
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/profiles/robots/p1")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&profile2).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);

    // delete p1
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri("/api/profiles/robots/p1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::NO_CONTENT);
}
