use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::error::{Result, BiliError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub room_id: Option<String>,
    pub cookie_str: Option<String>,
    pub csrf: Option<String>,
    pub last_settings: Option<LastSettings>,
    pub retry_count: u32,
    pub retry_delay: u64,
    pub timeout: u64,
    // 推流信息
    pub stream_server: Option<String>,
    pub stream_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LastSettings {
    pub live_title: String,
    pub area_id: Option<u32>,
    pub sub_area_id: Option<u32>,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            room_id: None,
            cookie_str: None,
            csrf: None,
            last_settings: None,
            retry_count: 3,
            retry_delay: 1000,
            timeout: 30000,
            stream_server: None,
            stream_key: None,
        }
    }
}

impl Config {
    /// 加载配置文件
    pub fn load() -> Result<Self> {
        let config_path = Self::get_config_path();
        if !config_path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }
        
        let config_str = std::fs::read_to_string(&config_path)
            .map_err(|e| BiliError::general(format!("读取配置文件失败: {}", e)))?;
        
        let config: Config = toml::from_str(&config_str)
            .map_err(|e| BiliError::general(format!("解析配置文件失败: {}", e)))?;
        
        Ok(config)
    }
    
    /// 保存配置
    pub fn save(&self) -> Result<()> {
        let config_path = Self::get_config_path();
        
        // 确保目录存在
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| BiliError::general(format!("创建配置目录失败: {}", e)))?;
        }
        
        let config_str = toml::to_string_pretty(self)
            .map_err(|e| BiliError::general(format!("序列化配置失败: {}", e)))?;
        
        std::fs::write(config_path, config_str)
            .map_err(|e| BiliError::general(format!("写入配置文件失败: {}", e)))?;
        
        Ok(())
    }
    
    /// 获取程序根目录
    fn get_app_dir() -> PathBuf {
        // 优先尝试获取可执行文件所在目录
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(parent) = exe_path.parent() {
                return parent.to_path_buf();
            }
        }
        
        // 如果获取不到，使用当前工作目录
        std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
    }
    
    /// 获取配置文件路径
    pub fn get_config_path() -> PathBuf {
        let mut path = Self::get_app_dir();
        path.push("config.toml");
        path
    }
    
    /// 获取Cookies文件路径
    pub fn get_cookies_path() -> PathBuf {
        let mut path = Self::get_app_dir();
        path.push("cookies.txt");
        path
    }
    
    /// 获取日志文件路径
    pub fn get_log_path() -> PathBuf {
        let mut path = Self::get_app_dir();
        path.push("bilibili_live_tool.log");
        path
    }
    
    /// 检查是否有认证信息
    pub fn has_credentials(&self) -> bool {
        self.room_id.is_some() && 
        self.cookie_str.is_some() && 
        self.csrf.is_some()
    }
    
    /// 设置认证信息
    pub fn set_credentials(&mut self, room_id: String, cookie_str: String, csrf: String) {
        self.room_id = Some(room_id);
        self.cookie_str = Some(cookie_str);
        self.csrf = Some(csrf);
    }
    
    /// 清除认证信息
    pub fn clear_credentials(&mut self) {
        self.room_id = None;
        self.cookie_str = None;
        self.csrf = None;
    }
    
    /// 保存最近的设置
    pub fn save_last_settings(&mut self, title: String, area_id: Option<u32>, sub_area_id: Option<u32>) -> Result<()> {
        self.last_settings = Some(LastSettings {
            live_title: title,
            area_id,
            sub_area_id,
            last_used: Some(chrono::Utc::now()),
        });
        self.save()
    }
    
    /// 获取房间ID
    pub fn get_room_id(&self) -> Option<u64> {
        self.room_id.as_ref().and_then(|id| id.parse().ok())
    }
    
    /// 保存推流信息（安全保存，不会覆盖其他配置）
    pub fn save_stream_info(&mut self, server: String, key: String) -> Result<()> {
        // 重新加载最新的配置文件，确保不丢失其他设置
        let mut latest_config = Self::load()?;
        latest_config.stream_server = Some(server);
        latest_config.stream_key = Some(key);
        latest_config.save()?;
        
        // 更新当前实例的推流信息
        self.stream_server = latest_config.stream_server.clone();
        self.stream_key = latest_config.stream_key.clone();
        
        Ok(())
    }
    
    /// 清除推流信息（安全清除，不会覆盖其他配置）
    pub fn clear_stream_info(&mut self) -> Result<()> {
        // 重新加载最新的配置文件，确保不丢失其他设置
        let mut latest_config = Self::load()?;
        latest_config.stream_server = None;
        latest_config.stream_key = None;
        latest_config.save()?;
        
        // 更新当前实例的推流信息
        self.stream_server = None;
        self.stream_key = None;
        
        Ok(())
    }
    
    /// 检查是否有推流信息
    pub fn has_stream_info(&self) -> bool {
        self.stream_server.is_some() && self.stream_key.is_some()
    }
    
    /// 获取推流信息
    pub fn get_stream_info(&self) -> Option<(String, String)> {
        if let (Some(server), Some(key)) = (&self.stream_server, &self.stream_key) {
            Some((server.clone(), key.clone()))
        } else {
            None
        }
    }
} 