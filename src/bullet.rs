use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::client::{BilibiliClient, ApiResponse};
use crate::error::{Result, BiliError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulletData {
    pub msg: String,
    pub color: u32,
    pub fontsize: u32,
    pub rnd: u64,
    pub roomid: u64,
    pub csrf_token: String,
    pub csrf: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulletResponse {
    pub code: i32,
    pub msg: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

pub struct Bullet {
    client: BilibiliClient,
    room_id: u64,
    csrf: String,
}

impl Bullet {
    pub fn new(room_id: u64, csrf: String, cookie_str: &str) -> Result<Self> {
        let client = BilibiliClient::with_cookies(cookie_str)?;
        Ok(Self {
            client,
            room_id,
            csrf,
        })
    }
    
    pub fn with_client(client: BilibiliClient, room_id: u64, csrf: String) -> Self {
        Self {
            client,
            room_id,
            csrf,
        }
    }
    
    /// 发送弹幕
    pub async fn send_bullet(&self, msg: &str) -> Result<String> {
        self.send_bullet_with_options(msg, None, None).await
    }
    
    /// 发送带选项的弹幕
    pub async fn send_bullet_with_options(&self, msg: &str, color: Option<u32>, fontsize: Option<u32>) -> Result<String> {
        let url = "https://api.live.bilibili.com/msg/send";
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let color_str = color.unwrap_or(16777215).to_string();
        let fontsize_str = fontsize.unwrap_or(25).to_string();
        let timestamp_str = timestamp.to_string();
        let room_id_str = self.room_id.to_string();
        
        let data = vec![
            ("msg", msg),
            ("color", color_str.as_str()),
            ("fontsize", fontsize_str.as_str()),
            ("rnd", timestamp_str.as_str()),
            ("roomid", room_id_str.as_str()),
            ("csrf_token", self.csrf.as_str()),
            ("csrf", self.csrf.as_str()),
        ];
        
        let response = self.client.get_client()
            .post(url)
            .headers(BilibiliClient::get_default_headers())
            .form(&data)
            .send()
            .await?;
        
        let bullet_response: BulletResponse = response.json().await?;
        
        match bullet_response.code {
            0 => Ok("发送成功".to_string()),
            1003212 => Err(BiliError::Bullet("超出限制长度".to_string())),
            -101 => Err(BiliError::Bullet("未登录".to_string())),
            -400 => Err(BiliError::Bullet("参数错误".to_string())),
            10031 => Err(BiliError::Bullet("发送频率过高".to_string())),
            _ => Err(BiliError::Bullet(format!("未知错误: {}", bullet_response.msg))),
        }
    }
    
    /// 发送带颜色的弹幕
    pub async fn send_colored_bullet(&self, msg: &str, color: u32) -> Result<String> {
        self.send_bullet_with_options(msg, Some(color), None).await
    }
    
    /// 发送带字体大小的弹幕
    pub async fn send_sized_bullet(&self, msg: &str, fontsize: u32) -> Result<String> {
        self.send_bullet_with_options(msg, None, Some(fontsize)).await
    }
    
    /// 批量发送弹幕
    pub async fn send_bullets(&self, messages: Vec<&str>) -> Result<Vec<(String, Result<String>)>> {
        let mut results = Vec::new();
        
        for msg in messages {
            let result = self.send_bullet(msg).await;
            results.push((msg.to_string(), result));
            
            // 防止发送过快，等待1秒
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
        
        Ok(results)
    }
    
    /// 获取弹幕颜色常量
    pub fn get_color_white() -> u32 { 16777215 }
    pub fn get_color_red() -> u32 { 16711680 }
    pub fn get_color_green() -> u32 { 65280 }
    pub fn get_color_blue() -> u32 { 255 }
    pub fn get_color_yellow() -> u32 { 16776960 }
    pub fn get_color_purple() -> u32 { 16711935 }
    pub fn get_color_cyan() -> u32 { 65535 }
    
    /// 获取字体大小常量
    pub fn get_fontsize_small() -> u32 { 18 }
    pub fn get_fontsize_normal() -> u32 { 25 }
    pub fn get_fontsize_large() -> u32 { 36 }
    
    /// 验证弹幕内容
    pub fn validate_message(msg: &str) -> Result<()> {
        if msg.is_empty() {
            return Err(BiliError::Bullet("弹幕内容不能为空".to_string()));
        }
        
        if msg.len() > 20 {
            return Err(BiliError::Bullet("弹幕内容过长，最多20个字符".to_string()));
        }
        
        // 检查是否包含敏感词汇
        let sensitive_words = vec!["fuck", "shit", "damn"];
        for word in sensitive_words {
            if msg.to_lowercase().contains(word) {
                return Err(BiliError::Bullet("弹幕包含敏感词汇".to_string()));
            }
        }
        
        Ok(())
    }
    
    /// 发送验证过的弹幕
    pub async fn send_validated_bullet(&self, msg: &str) -> Result<String> {
        Self::validate_message(msg)?;
        self.send_bullet(msg).await
    }
    
    /// 获取弹幕历史记录
    pub async fn get_bullet_history(&self) -> Result<Vec<serde_json::Value>> {
        let url = format!("https://api.live.bilibili.com/xlive/web-room/v1/dM/gethistory?roomid={}", self.room_id);
        
        let response: ApiResponse<serde_json::Value> = self.client.get(&url).await?;
        let data = response.data.ok_or_else(|| BiliError::Bullet("获取弹幕历史失败".to_string()))?;
        
        if let Some(room) = data.get("room") {
            if let Some(history) = room.get("history") {
                if let Some(history_array) = history.as_array() {
                    return Ok(history_array.clone());
                }
            }
        }
        
        Ok(vec![])
    }
    
    /// 获取直播间弹幕配置
    pub async fn get_bullet_config(&self) -> Result<serde_json::Value> {
        let url = format!("https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo?id={}", self.room_id);
        
        let response: ApiResponse<serde_json::Value> = self.client.get(&url).await?;
        let config = response.data.ok_or_else(|| BiliError::Bullet("获取弹幕配置失败".to_string()))?;
        
        Ok(config)
    }
    
    /// 获取房间号
    pub fn get_room_id(&self) -> u64 {
        self.room_id
    }
    
    /// 获取CSRF token
    pub fn get_csrf(&self) -> &str {
        &self.csrf
    }
    
    /// 创建弹幕
    pub fn create_bullet_data(&self, msg: &str, color: Option<u32>, fontsize: Option<u32>) -> BulletData {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        BulletData {
            msg: msg.to_string(),
            color: color.unwrap_or(Self::get_color_white()),
            fontsize: fontsize.unwrap_or(Self::get_fontsize_normal()),
            rnd: timestamp,
            roomid: self.room_id,
            csrf_token: self.csrf.clone(),
            csrf: self.csrf.clone(),
        }
    }
} 