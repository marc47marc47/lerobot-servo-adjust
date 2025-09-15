use askama::Template;
use axum::{
    extract::{Form, Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use serde::Deserialize;

use crate::api::AppState;
use crate::store::Kind;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/", get(index))
        .route("/profiles/:kind/:profile", get(view_profile))
        .route("/profiles/:kind/:profile", post(update_profile))
        .route("/arm/:kind/:profile", get(view_arm))
        .route("/arm/:kind/:profile", post(update_arm))
        .route("/assets/lerobot-arm.jpg", get(arm_image))
        .with_state(state)
}

#[derive(Template)]
#[template(path = "index.html")]
struct IndexTemplate<'a> {
    title: &'a str,
    robots: Vec<String>,
    leaders: Vec<String>,
}

async fn index(State(state): State<AppState>) -> impl IntoResponse {
    let robots = state
        .store
        .list_profiles(Kind::Robots)
        .map(|v| v.into_iter().map(|m| m.name).collect())
        .unwrap_or_default();
    let leaders = state
        .store
        .list_profiles(Kind::Teleoperators)
        .map(|v| v.into_iter().map(|m| m.name).collect())
        .unwrap_or_default();
    IndexTemplate { title: "LeRobot Servo Adjust", robots, leaders }
}

#[derive(Template)]
#[template(path = "profile.html")]
struct ProfileTemplate<'a> {
    title: &'a str,
    kind: String,
    name: String,
    json: String,
    error: Option<String>,
}

async fn view_profile(State(state): State<AppState>, Path((kind, profile)): Path<(String, String)>) -> impl IntoResponse {
    let kind_str = kind.clone();
    let k = match kind.as_str() {
        "robots" => Kind::Robots,
        "teleoperators" => Kind::Teleoperators,
        _ => return ProfileTemplate { title: "Profile", kind: kind_str, name: profile, json: String::new(), error: Some("invalid kind".into()) },
    };
    match state.store.read_profile(k, &profile) {
        Ok(p) => ProfileTemplate { title: "Profile", kind: kind_str, name: profile, json: serde_json::to_string_pretty(&p).unwrap_or_default(), error: None },
        Err(e) => ProfileTemplate { title: "Profile", kind: kind_str, name: profile, json: String::new(), error: Some(format!("{}", e)) },
    }
}

#[derive(Deserialize)]
struct UpdateForm {
    action: Option<String>,
    json: Option<String>,
}

async fn update_profile(State(state): State<AppState>, Path((kind, profile)): Path<(String, String)>, Form(form): Form<UpdateForm>) -> impl IntoResponse {
    let kind_str = kind.clone();
    let k = match kind.as_str() {
        "robots" => Kind::Robots,
        "teleoperators" => Kind::Teleoperators,
        _ => return ProfileTemplate { title: "Profile", kind: kind_str, name: profile, json: String::new(), error: Some("invalid kind".into()) }.into_response(),
    };
    let api_url = state.base_url.as_deref().map(|u| format!("{}/api/profiles/{}/{}", u, kind_str, profile));
    if matches!(form.action.as_deref(), Some("delete")) {
        if let Some(url) = api_url.as_deref() {
            match reqwest::Client::new().delete(url).send().await {
                Ok(resp) if resp.status().is_success() => return Redirect::to("/").into_response(),
                Ok(resp) => return ProfileTemplate { title: "Profile", kind: kind_str, name: profile, json: String::new(), error: Some(format!("delete failed: {}", resp.status())) }.into_response(),
                Err(e) => return ProfileTemplate { title: "Profile", kind: kind_str, name: profile, json: String::new(), error: Some(format!("request error: {}", e)) }.into_response(),
            }
        } else {
            match state.store.delete_profile(k, &profile) {
                Ok(_) => return Redirect::to("/").into_response(),
                Err(e) => return ProfileTemplate { title: "Profile", kind: kind_str, name: profile, json: String::new(), error: Some(format!("{}", e)) }.into_response(),
            }
        }
    }
    if let Some(json) = form.json {
        if let Some(url) = api_url.as_deref() {
            let resp = reqwest::Client::new()
                .put(url)
                .header("content-type", "application/json")
                .body(json.clone())
                .send()
                .await;
            match resp {
                Ok(resp) if resp.status().is_success() => Redirect::to(&format!("/profiles/{}/{}", kind_str, profile)).into_response(),
                Ok(resp) => ProfileTemplate { title: "Profile", kind: kind_str, name: profile, json, error: Some(format!("update failed: {}", resp.status())) }.into_response(),
                Err(e) => ProfileTemplate { title: "Profile", kind: kind_str, name: profile, json, error: Some(format!("request error: {}", e)) }.into_response(),
            }
        } else {
            match serde_json::from_str::<crate::model::Profile>(&json) {
                Ok(p) => match state.store.write_profile(k, &profile, &p, true) {
                    Ok(_) => Redirect::to(&format!("/profiles/{}/{}", kind_str, profile)).into_response(),
                    Err(e) => ProfileTemplate { title: "Profile", kind: kind_str, name: profile, json, error: Some(format!("{}", e)) }.into_response(),
                },
                Err(e) => ProfileTemplate { title: "Profile", kind: kind_str, name: profile, json, error: Some(format!("invalid json: {}", e)) }.into_response(),
            }
        }
    } else {
        ProfileTemplate { title: "Profile", kind: kind_str, name: profile, json: String::new(), error: Some("missing json".into()) }.into_response()
    }
}

// ---------------- Arm UI ----------------

#[derive(Template)]
#[template(path = "arm.html")]
struct ArmTemplate {
    title: String,
    kind: String,
    name: String,
    label_prefix: String,
    selected: Option<u8>,
    has_selection: bool,
    joint_name: Option<String>,
    has_joint: bool,
    joint_label: String,
    selected_n: u8,
    id_v: i32,
    drive_mode_v: i32,
    homing_offset_v: i32,
    range_min_v: i32,
    range_max_v: i32,
    error: Option<String>,
    message: Option<String>,
    hotspots: Vec<Hotspot>,
    robots_btns: Vec<(String, bool)>,
    leaders_btns: Vec<(String, bool)>,
    read_only: bool,
}

#[derive(Debug, Clone)]
struct Hotspot { n: u8, top: u8, left: u8, selected: bool, label: String }

#[derive(Deserialize)]
struct ArmQuery { sel: Option<u8> }

async fn view_arm(State(state): State<AppState>, Path((kind, profile)): Path<(String, String)>, Query(q): Query<ArmQuery>) -> Response {
    let label_prefix = if kind == "teleoperators" { "L" } else { "F" }.to_string();
    let sel = q.sel.filter(|v| (1..=6).contains(v));

    let robots: Vec<String> = state
        .store
        .list_profiles(Kind::Robots)
        .map(|v| v.into_iter().map(|m| m.name).collect())
        .unwrap_or_default();
    let leaders: Vec<String> = state
        .store
        .list_profiles(Kind::Teleoperators)
        .map(|v| v.into_iter().map(|m| m.name).collect())
        .unwrap_or_default();
    let robots_btns: Vec<(String, bool)> = robots.iter().cloned().map(|n| (n.clone(), kind == "robots" && profile == n)).collect();
    let leaders_btns: Vec<(String, bool)> = leaders.iter().cloned().map(|n| (n.clone(), kind == "teleoperators" && profile == n)).collect();

    let mut tpl = ArmTemplate {
        title: format!("Arm - {} / {}", &kind, &profile),
        kind: kind.clone(),
        name: profile.clone(),
        label_prefix,
        selected: sel,
        has_selection: sel.is_some(),
        joint_name: None,
        has_joint: false,
        joint_label: String::new(),
        selected_n: sel.unwrap_or(0),
        id_v: 0,
        drive_mode_v: 0,
        homing_offset_v: 0,
        range_min_v: 0,
        range_max_v: 0,
        error: None,
        message: None,
        hotspots: Vec::new(),
        robots_btns,
        leaders_btns,
        read_only: state.read_only,
    };

    let coords: [(u8, u8); 6] = match kind.as_str() {
        "robots" => [(86,79), (68,77), (25,85), (22,70), (18,56), (35,49)],
        _ => [(83,36), (58,31), (20,45), (19,28), (15,16), (30,9)],
    };

    let k = match kind.as_str() { "robots" => Kind::Robots, "teleoperators" => Kind::Teleoperators, _ => Kind::Robots };
    let mut id_to_name: std::collections::HashMap<i32, String> = Default::default();
    if let Ok(p) = state.store.read_profile(k, &profile) {
        for (name, j) in p.0.iter() {
            id_to_name.insert(j.id, name.clone());
        }
    }
    for (i, (t, l)) in coords.iter().enumerate() {
        let n = (i as u8)+1;
        let label = id_to_name.get(&(n as i32)).cloned().unwrap_or_else(|| format!("{}{}", tpl.label_prefix, n));
        tpl.hotspots.push(Hotspot { n, top: *t, left: *l, selected: sel == Some(n), label });
    }
    if let Some(s) = sel {
        match state.store.read_profile(k, &profile) {
            Ok(p) => {
                for (name, j) in p.0.iter() {
                    if j.id == s as i32 {
                        tpl.joint_name = Some(name.clone());
                        tpl.has_joint = true;
                        tpl.joint_label = name.clone();
                        tpl.id_v = j.id;
                        tpl.drive_mode_v = j.drive_mode;
                        tpl.homing_offset_v = j.homing_offset;
                        tpl.range_min_v = j.range_min;
                        tpl.range_max_v = j.range_max;
                        break;
                    }
                }
                if tpl.joint_name.is_none() {
                    tpl.error = Some(format!("joint with id={} not found", s));
                }
            }
            Err(e) => tpl.error = Some(format!("{}", e)),
        }
    }
    <ArmTemplate as askama_axum::IntoResponse>::into_response(tpl)
}

#[derive(Deserialize)]
struct ArmUpdateForm {
    action: Option<String>,
    id: u8,
    drive_mode: i32,
    homing_offset: i32,
    range_min: i32,
    range_max: i32,
}

async fn update_arm(State(state): State<AppState>, Path((kind, profile)): Path<(String, String)>, Form(form): Form<ArmUpdateForm>) -> Response {
    let kind_str = kind.clone();
    let k = match kind.as_str() { "robots" => Kind::Robots, "teleoperators" => Kind::Teleoperators, _ => Kind::Robots };
    let prof = match state.store.read_profile(k, &profile) {
        Ok(p) => p,
        Err(e) => return <ArmTemplate as askama_axum::IntoResponse>::into_response(ArmTemplate { title: "Arm".into(), kind: kind_str, name: profile, label_prefix: String::new(), selected: None, has_selection: false, joint_name: None, has_joint: false, joint_label: String::new(), selected_n: 0, id_v: 0, drive_mode_v: 0, homing_offset_v: 0, range_min_v: 0, range_max_v: 0, error: Some(format!("{}", e)), message: None, hotspots: vec![], robots_btns: vec![], leaders_btns: vec![], read_only: state.read_only }),
    };
    let mut joint_name: Option<String> = None;
    for (name, j) in prof.0.iter() {
        if j.id == form.id as i32 { joint_name = Some(name.clone()); break; }
    }
    let Some(jname) = joint_name else {
        return <ArmTemplate as askama_axum::IntoResponse>::into_response(ArmTemplate { title: "Arm".into(), kind: kind_str, name: profile, label_prefix: String::new(), selected: None, has_selection: false, joint_name: None, has_joint: false, joint_label: String::new(), selected_n: 0, id_v: 0, drive_mode_v: 0, homing_offset_v: 0, range_min_v: 0, range_max_v: 0, error: Some("invalid id".into()), message: None, hotspots: vec![], robots_btns: vec![], leaders_btns: vec![], read_only: state.read_only });
    };

    let jname_key = jname.clone();
    let body = serde_json::json!({
        jname_key: {
            "id": form.id as i32,
            "drive_mode": form.drive_mode,
            "homing_offset": form.homing_offset,
            "range_min": form.range_min,
            "range_max": form.range_max
        }
    });

    if let Some(base) = state.base_url.as_deref() {
        let url = format!("{}/api/profiles/{}/{}", base, kind_str, profile);
        match reqwest::Client::new().patch(url).json(&body).send().await {
            Ok(resp) if resp.status().is_success() => Redirect::to(&format!("/arm/{}/{}?sel={}", kind_str, profile, form.id)).into_response(),
            Ok(resp) => <ArmTemplate as askama_axum::IntoResponse>::into_response(ArmTemplate { title: "Arm".into(), kind: kind_str, name: profile, label_prefix: String::new(), selected: Some(form.id), has_selection: true, joint_name: Some(jname.clone()), has_joint: true, joint_label: jname.clone(), selected_n: form.id, id_v: form.id as i32, drive_mode_v: form.drive_mode, homing_offset_v: form.homing_offset, range_min_v: form.range_min, range_max_v: form.range_max, error: Some(format!("update failed: {}", resp.status())), message: None, hotspots: vec![], robots_btns: vec![], leaders_btns: vec![], read_only: state.read_only }),
            Err(e) => <ArmTemplate as askama_axum::IntoResponse>::into_response(ArmTemplate { title: "Arm".into(), kind: kind_str, name: profile, label_prefix: String::new(), selected: Some(form.id), has_selection: true, joint_name: Some(jname.clone()), has_joint: true, joint_label: jname.clone(), selected_n: form.id, id_v: form.id as i32, drive_mode_v: form.drive_mode, homing_offset_v: form.homing_offset, range_min_v: form.range_min, range_max_v: form.range_max, error: Some(format!("request error: {}", e)), message: None, hotspots: vec![], robots_btns: vec![], leaders_btns: vec![], read_only: state.read_only }),
        }
    } else {
        let mut p = prof;
        if let Some(j) = p.0.get_mut(&jname) {
            j.id = form.id as i32;
            j.drive_mode = form.drive_mode;
            j.homing_offset = form.homing_offset;
            j.range_min = form.range_min;
            j.range_max = form.range_max;
        }
        match state.store.write_profile(k, &profile, &p, true) {
            Ok(_) => Redirect::to(&format!("/arm/{}/{}?sel={}", kind_str, profile, form.id)).into_response(),
            Err(e) => <ArmTemplate as askama_axum::IntoResponse>::into_response(ArmTemplate { title: "Arm".into(), kind: kind_str, name: profile, label_prefix: String::new(), selected: Some(form.id), has_selection: true, joint_name: Some(jname.clone()), has_joint: true, joint_label: jname, selected_n: form.id, id_v: form.id as i32, drive_mode_v: form.drive_mode, homing_offset_v: form.homing_offset, range_min_v: form.range_min, range_max_v: form.range_max, error: Some(format!("{}", e)), message: None, hotspots: vec![], robots_btns: vec![], leaders_btns: vec![], read_only: state.read_only }),
        }
    }
}

async fn arm_image() -> Response {
    match tokio::fs::read("lerobot-arm.jpg").await {
        Ok(bytes) => (
            [(header::CONTENT_TYPE, "image/jpeg")],
            bytes,
        )
            .into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "image not found").into_response(),
    }
}
