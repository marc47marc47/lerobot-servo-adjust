use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub calib_root: PathBuf,
}

impl Config {
    pub fn from_env() -> Self {
        let root = std::env::var("CALIB_ROOT").ok();
        let calib_root = root
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("huggingface/lerobot/calibration"));
        Self { calib_root }
    }

    pub fn robots_dir(&self) -> PathBuf {
        self.calib_root.join("robots")
    }

    pub fn teleoperators_dir(&self) -> PathBuf {
        self.calib_root.join("teleoperators")
    }

    pub fn ensure_exists(&self) -> std::io::Result<()> {
        for p in [self.calib_root.as_path(), self.robots_dir().as_path(), self.teleoperators_dir().as_path()] {
            if !p.exists() {
                std::fs::create_dir_all(p)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn default_root() {
        // Rust 2024: environment mutation is unsafe; keep minimal and isolated.
        unsafe { env::remove_var("CALIB_ROOT"); }
        let cfg = Config::from_env();
        assert!(cfg.calib_root.ends_with("huggingface/lerobot/calibration"));
    }

    #[test]
    fn custom_root() {
        let dir = tempfile::tempdir().unwrap();
        unsafe { env::set_var("CALIB_ROOT", dir.path()); }
        let cfg = Config::from_env();
        assert_eq!(cfg.calib_root, dir.path());
        cfg.ensure_exists().unwrap();
        assert!(cfg.robots_dir().exists());
        assert!(cfg.teleoperators_dir().exists());
        unsafe { env::remove_var("CALIB_ROOT"); }
    }
}
