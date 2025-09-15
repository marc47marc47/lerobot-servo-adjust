use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Joint {
    pub id: i32,
    pub drive_mode: i32,
    pub homing_offset: i32,
    pub range_min: i32,
    pub range_max: i32,
}

impl Joint {
    pub fn validate(&self) -> Result<(), String> {
        if self.id <= 0 {
            return Err("id must be > 0".into());
        }
        if self.range_min >= self.range_max {
            return Err("range_min must be < range_max".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Profile(pub std::collections::HashMap<String, Joint>);

impl Profile {
    pub fn validate(&self) -> Result<(), String> {
        for (name, joint) in &self.0 {
            joint
                .validate()
                .map_err(|e| format!("joint `{}` invalid: {}", name, e))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joint_validation() {
        let ok = Joint { id: 1, drive_mode: 0, homing_offset: 0, range_min: 10, range_max: 20 };
        assert!(ok.validate().is_ok());

        let bad_id = Joint { id: 0, ..ok.clone() };
        assert!(bad_id.validate().is_err());

        let bad_range = Joint { range_min: 5, range_max: 5, ..ok };
        assert!(bad_range.validate().is_err());
    }

    #[test]
    fn profile_validation() {
        let mut map = std::collections::HashMap::new();
        map.insert(
            "shoulder_pan".to_string(),
            Joint { id: 1, drive_mode: 0, homing_offset: 0, range_min: 100, range_max: 200 },
        );
        let profile = Profile(map);
        assert!(profile.validate().is_ok());
    }
}

