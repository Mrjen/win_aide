use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct AppConfig {
    pub version: u32,
    pub shortcuts: Vec<Shortcut>,
    pub settings: Settings,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Shortcut {
    pub id: String,
    pub name: String,
    pub exe_name: String,
    pub exe_path: String,
    pub modifier: Modifier,
    pub key: char,
    pub enabled: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Modifier {
    Alt,
    Ctrl,
    Win,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Settings {
    pub auto_start: bool,
    pub start_minimized: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            shortcuts: Vec::new(),
            settings: Settings {
                auto_start: false,
                start_minimized: true,
            },
        }
    }
}

impl Modifier {
    pub fn display_name(&self) -> &str {
        match self {
            Modifier::Alt => "Alt",
            Modifier::Ctrl => "Ctrl",
            Modifier::Win => "Win",
        }
    }

    pub fn all() -> &'static [Modifier] {
        &[Modifier::Alt, Modifier::Ctrl, Modifier::Win]
    }
}

/// 获取配置文件目录 ~/.win_aide/
pub fn config_dir() -> PathBuf {
    let home = dirs::home_dir().expect("无法获取用户目录");
    home.join(".win_aide")
}

/// 获取配置文件路径 ~/.win_aide/config.json
pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

/// 加载配置，文件不存在则返回默认配置并创建文件
pub fn load_config() -> AppConfig {
    let path = config_path();
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        let config = AppConfig::default();
        save_config(&config);
        config
    }
}

/// 保存配置到 JSON 文件
pub fn save_config(config: &AppConfig) {
    let dir = config_dir();
    if !dir.exists() {
        fs::create_dir_all(&dir).expect("无法创建配置目录");
    }
    let path = config_path();
    let json = serde_json::to_string_pretty(config).expect("序列化配置失败");
    fs::write(&path, json).expect("写入配置文件失败");
}

/// 检查快捷键是否冲突（同一 modifier + key 组合）
pub fn has_conflict(
    shortcuts: &[Shortcut],
    modifier: &Modifier,
    key: char,
    exclude_id: Option<&str>,
) -> bool {
    shortcuts.iter().any(|s| {
        s.modifier == *modifier
            && s.key.to_ascii_uppercase() == key.to_ascii_uppercase()
            && exclude_id.map_or(true, |id| s.id != id)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.version, 1);
        assert!(config.shortcuts.is_empty());
        assert!(!config.settings.auto_start);
        assert!(config.settings.start_minimized);
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = AppConfig {
            version: 1,
            shortcuts: vec![Shortcut {
                id: "test-id".to_string(),
                name: "Chrome".to_string(),
                exe_name: "chrome.exe".to_string(),
                exe_path: "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe".to_string(),
                modifier: Modifier::Alt,
                key: 'C',
                enabled: true,
            }],
            settings: Settings {
                auto_start: true,
                start_minimized: true,
            },
        };

        let json = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_has_conflict() {
        let shortcuts = vec![Shortcut {
            id: "1".to_string(),
            name: "Chrome".to_string(),
            exe_name: "chrome.exe".to_string(),
            exe_path: "chrome.exe".to_string(),
            modifier: Modifier::Alt,
            key: 'C',
            enabled: true,
        }];

        // 相同组合应冲突
        assert!(has_conflict(&shortcuts, &Modifier::Alt, 'C', None));
        // 大小写不敏感
        assert!(has_conflict(&shortcuts, &Modifier::Alt, 'c', None));
        // 不同修饰键不冲突
        assert!(!has_conflict(&shortcuts, &Modifier::Ctrl, 'C', None));
        // 不同字母不冲突
        assert!(!has_conflict(&shortcuts, &Modifier::Alt, 'V', None));
        // 排除自身不冲突
        assert!(!has_conflict(&shortcuts, &Modifier::Alt, 'C', Some("1")));
    }

    #[test]
    fn test_modifier_display() {
        assert_eq!(Modifier::Alt.display_name(), "Alt");
        assert_eq!(Modifier::Ctrl.display_name(), "Ctrl");
        assert_eq!(Modifier::Win.display_name(), "Win");
    }
}
