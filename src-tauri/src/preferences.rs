use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

#[derive(serde::Serialize, serde::Deserialize)]
struct PreferencesData {
    #[serde(default = "default_include_task")]
    include_task: bool,
}

fn default_include_task() -> bool {
    true
}

pub struct Preferences {
    path: PathBuf,
    include_task: AtomicBool,
}

impl Preferences {
    pub fn load(path: PathBuf) -> Self {
        let include_task = fs::read(&path)
            .ok()
            .and_then(|raw| serde_json::from_slice::<PreferencesData>(&raw).ok())
            .map(|data| data.include_task)
            .unwrap_or(true);

        Self {
            path,
            include_task: AtomicBool::new(include_task),
        }
    }

    pub fn include_task(&self) -> bool {
        self.include_task.load(Ordering::Relaxed)
    }

    pub fn set_include_task(&self, include_task: bool) -> Result<(), String> {
        self.include_task.store(include_task, Ordering::Relaxed);

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|error| format!("创建配置目录失败: {error}"))?;
        }
        let temporary = with_suffix(&self.path, ".tmp");
        let raw = serde_json::to_vec(&PreferencesData { include_task })
            .map_err(|error| format!("序列化设置失败: {error}"))?;
        fs::write(&temporary, raw).map_err(|error| format!("写入设置失败: {error}"))?;
        fs::rename(&temporary, &self.path).map_err(|error| format!("替换设置失败: {error}"))?;
        Ok(())
    }
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut value = path.as_os_str().to_os_string();
    value.push(suffix);
    PathBuf::from(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_enabled_and_roundtrips() {
        let directory = tempfile::tempdir().expect("temp dir");
        let path = directory.path().join("preferences.json");

        let preferences = Preferences::load(path.clone());
        assert!(preferences.include_task());

        preferences.set_include_task(false).expect("save preferences");
        let reloaded = Preferences::load(path);
        assert!(!reloaded.include_task());
    }
}
