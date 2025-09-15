use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use serde_json::Error as SerdeError;
use tracing::{error, info, instrument};
use thiserror::Error;

use crate::model::Profile;

#[derive(Debug, Clone, Copy)]
pub enum Kind {
    Robots,
    Teleoperators,
}

impl Kind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Kind::Robots => "robots",
            Kind::Teleoperators => "teleoperators",
        }
    }
}

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("json error: {0}")]
    Json(#[from] SerdeError),
    #[error("validation error: {0}")]
    Validation(String),
    #[error("not found: {0}")]
    NotFound(String),
}

#[derive(Debug, Clone)]
pub struct ProfileMeta {
    pub name: String,
    pub path: PathBuf,
}

pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    fn kind_dir(&self, kind: Kind) -> PathBuf {
        self.root.join(kind.as_str())
    }

    #[instrument(skip(self))]
    pub fn list_profiles(&self, kind: Kind) -> Result<Vec<ProfileMeta>, StoreError> {
        let mut metas = Vec::new();
        let dir = self.kind_dir(kind);
        if !dir.exists() {
            return Ok(metas);
        }
        for entry in walkdir::WalkDir::new(&dir).into_iter().filter_map(Result::ok) {
            let p = entry.path();
            if p.is_file() && p.extension() == Some(OsStr::new("json")) {
                if let Some(name) = p.file_stem().and_then(OsStr::to_str) {
                    metas.push(ProfileMeta { name: name.to_string(), path: p.to_path_buf() });
                }
            }
        }
        metas.sort_by(|a, b| a.name.cmp(&b.name));
        info!(kind = kind.as_str(), count = metas.len(), "list profiles");
        Ok(metas)
    }

    #[instrument(skip(self))]
    pub fn read_profile(&self, kind: Kind, name: &str) -> Result<Profile, StoreError> {
        let dir = self.kind_dir(kind);
        // try to find file by name under dir
        let mut found: Option<PathBuf> = None;
        for entry in walkdir::WalkDir::new(&dir).into_iter().filter_map(Result::ok) {
            let p = entry.path();
            if p.is_file() && p.extension() == Some(OsStr::new("json")) {
                if p.file_stem().and_then(OsStr::to_str) == Some(name) {
                    found = Some(p.to_path_buf());
                    break;
                }
            }
        }
        let path = found.ok_or_else(|| StoreError::NotFound(format!("{}:{}", kind.as_str(), name)))?;
        let data = fs::read_to_string(&path).map_err(|e| {
            error!(?e, ?path, "read file error");
            e
        })?;
        let profile: Profile = serde_json::from_str(&data).map_err(|e| {
            error!(?e, "json parse error");
            e
        })?;
        profile.validate().map_err(|e| {
            error!(error = %e, "validation error");
            StoreError::Validation(e)
        })?;
        Ok(profile)
    }

    #[instrument(skip(self, profile))]
    pub fn write_profile(&self, kind: Kind, name: &str, profile: &Profile, backup: bool) -> Result<PathBuf, StoreError> {
        profile.validate().map_err(StoreError::Validation)?;
        let dir = self.kind_dir(kind);
        fs::create_dir_all(&dir)?;
        let path = dir.join(format!("{}.json", name));
        let tmp = dir.join(format!("{}.json.tmp", name));
        let payload = serde_json::to_vec_pretty(profile)?;

        if backup && path.exists() {
            let bak = dir.join(format!("{}.json.bak", name));
            fs::copy(&path, bak)?;
        }

        {
            let mut f = fs::File::create(&tmp)?;
            f.write_all(&payload)?;
            f.sync_all()?;
        }
        // Atomic-ish replace
        fs::rename(&tmp, &path).map_err(|e| {
            error!(?e, ?tmp, ?path, "rename error");
            e
        })?;
        info!(kind = kind.as_str(), name, ?path, "write profile ok");
        Ok(path)
    }

    #[instrument(skip(self))]
    pub fn delete_profile(&self, kind: Kind, name: &str) -> Result<(), StoreError> {
        let dir = self.kind_dir(kind);
        let path = dir.join(format!("{}.json", name));
        if !path.exists() {
            return Err(StoreError::NotFound(format!("{}:{}", kind.as_str(), name)));
        }
        fs::remove_file(&path).map_err(|e| {
            error!(?e, ?path, "delete error");
            e
        })?;
        info!(kind = kind.as_str(), name, "delete profile ok");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Joint, Profile};
    use std::collections::HashMap;

    #[test]
    fn list_and_rw_profile() {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::new(dir.path().to_path_buf());

        let mut map = HashMap::new();
        map.insert("j1".to_string(), Joint { id: 1, drive_mode: 0, homing_offset: 0, range_min: 1, range_max: 10 });
        let p = Profile(map);

        // write
        let out = store.write_profile(Kind::Robots, "test_profile", &p, true).unwrap();
        assert!(out.exists());

        // list
        let metas = store.list_profiles(Kind::Robots).unwrap();
        assert!(metas.iter().any(|m| m.name == "test_profile"));

        // read
        let read = store.read_profile(Kind::Robots, "test_profile").unwrap();
        assert_eq!(read, p);
    }
}
