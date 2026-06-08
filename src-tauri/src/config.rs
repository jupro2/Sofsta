use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub id: String,
    pub name: String,
    pub path: String,
    pub args: String,
    pub enabled: bool,
    pub delay_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mode {
    pub id: String,
    pub name: String,
    pub icon_emoji: String,
    pub color: String,
    pub auto_start: bool,
    pub apps: Vec<AppEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub modes: Vec<Mode>,
    pub exit_delay_minutes: u32,
    pub auto_start_enabled: bool,
    pub silent_launch: bool,
}

/// 旧格式（用于迁移）
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LegacyAppConfig {
    #[serde(default)]
    apps: Vec<AppEntry>,
    #[serde(default = "default_exit_delay")]
    exit_delay_minutes: u32,
    #[serde(default = "default_true")]
    auto_start_enabled: bool,
    #[serde(default = "default_true")]
    silent_launch: bool,
}

fn default_exit_delay() -> u32 {
    5
}
fn default_true() -> bool {
    true
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            modes: vec![Mode {
                id: uuid::Uuid::new_v4().to_string(),
                name: "默认模式".to_string(),
                icon_emoji: "📋".to_string(),
                color: "#3B82F6".to_string(),
                auto_start: true,
                apps: Vec::new(),
            }],
            exit_delay_minutes: 5,
            auto_start_enabled: true,
            silent_launch: true,
        }
    }
}

fn config_dir() -> PathBuf {
    let base = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    base.join("AutoStartManager")
}

fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

impl AppConfig {
    /// 加载配置文件，不存在则返回默认配置；旧格式自动迁移
    pub fn load() -> Self {
        let path = config_path();
        if !path.exists() {
            return Self::default();
        }

        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => return Self::default(),
        };

        // 尝试新格式
        if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
            return config;
        }

        // 尝试旧格式迁移
        if let Ok(legacy) = serde_json::from_str::<LegacyAppConfig>(&content) {
            let mut config = AppConfig {
                modes: Vec::new(),
                exit_delay_minutes: legacy.exit_delay_minutes,
                auto_start_enabled: legacy.auto_start_enabled,
                silent_launch: legacy.silent_launch,
            };

            if legacy.apps.is_empty() {
                // 空配置，使用默认
                return AppConfig::default();
            }

            // 将旧的 apps 迁移到「默认模式」
            config.modes.push(Mode {
                id: uuid::Uuid::new_v4().to_string(),
                name: "默认模式".to_string(),
                icon_emoji: "📋".to_string(),
                color: "#3B82F6".to_string(),
                auto_start: true,
                apps: legacy.apps,
            });

            // 保存迁移后的配置
            let _ = config.save();
            return config;
        }

        Self::default()
    }

    /// 保存配置文件
    pub fn save(&self) -> Result<(), String> {
        let dir = config_dir();
        fs::create_dir_all(&dir).map_err(|e| format!("创建配置目录失败: {}", e))?;

        let path = config_path();
        let json = serde_json::to_string_pretty(self).map_err(|e| format!("序列化失败: {}", e))?;
        fs::write(&path, json).map_err(|e| format!("写入配置文件失败: {}", e))?;
        Ok(())
    }

    /// 通过 exe 路径提取程序名称
    pub fn extract_name(path: &str) -> String {
        std::path::Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string()
    }

    /// 查找模式
    pub fn find_mode(&self, mode_id: &str) -> Option<&Mode> {
        self.modes.iter().find(|m| m.id == mode_id)
    }

    /// 查找模式（可变）
    pub fn find_mode_mut(&mut self, mode_id: &str) -> Option<&mut Mode> {
        self.modes.iter_mut().find(|m| m.id == mode_id)
    }
}
