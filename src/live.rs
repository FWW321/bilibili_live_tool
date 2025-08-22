use serde::{Deserialize, Serialize, Deserializer};
use std::collections::HashMap;
use crate::client::{BilibiliClient, ApiResponse};
use crate::error::Result;

// 自定义反序列化函数，用于将字符串转换为数字
fn deserialize_string_to_u32<'de, D>(deserializer: D) -> std::result::Result<u32, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrNumber {
        String(String),
        Number(u32),
    }
    
    match StringOrNumber::deserialize(deserializer)? {
        StringOrNumber::String(s) => s.parse::<u32>().map_err(Error::custom),
        StringOrNumber::Number(n) => Ok(n),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveStreamData {
    pub change: i32,
    pub live_key: String,
    pub need_face_auth: bool,
    pub notice: NoticeData,
    pub protocols: Vec<Protocol>,
    pub qr: String,
    pub room_type: i32,
    pub rtmp: RtmpData,
    pub rtmp_backup: Option<serde_json::Value>,
    pub service_source: String,
    pub status: String,
    pub sub_session_key: String,
    pub try_time: String,
    pub up_stream_extra: UpStreamExtra,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoticeData {
    pub button_text: String,
    pub button_url: String,
    pub msg: String,
    pub status: i32,
    pub title: String,
    #[serde(rename = "type")]
    pub notice_type: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpStreamExtra {
    pub isp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtmpData {
    pub addr: String,
    pub code: String,
    pub new_link: String,
    pub provider: String,
    #[serde(rename = "type")]
    pub rtmp_type: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Protocol {
    pub protocol: String,
    pub addr: String,
    pub code: String,
    pub new_link: String,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaData {
    #[serde(deserialize_with = "deserialize_string_to_u32")]
    pub id: u32,
    pub name: String,
    #[serde(deserialize_with = "deserialize_string_to_u32")]
    pub parent_id: u32,
    pub parent_name: String,
    #[serde(deserialize_with = "deserialize_string_to_u32")]
    pub act_id: u32,
    pub hot_status: u32,
    #[serde(deserialize_with = "deserialize_string_to_u32")]
    pub lock_status: u32,
    pub pic: String,
    #[serde(default)]
    pub complex_area_name: String,
    pub area_type: u32,
    #[serde(default)]
    pub pinyin: String,
    #[serde(default)]
    pub old_area_id: String,
    #[serde(default)]
    pub pk_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaListData {
    pub data: Vec<AreaCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AreaCategory {
    pub id: u32,
    pub name: String,
    pub list: Vec<AreaData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveStartData {
    pub room_id: u64,
    pub platform: String,
    pub area_v2: u32,
    pub backup_stream: String,
    pub csrf_token: String,
    pub csrf: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveStopData {
    pub room_id: u64,
    pub platform: String,
    pub csrf_token: String,
    pub csrf: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleUpdateData {
    pub room_id: u64,
    pub platform: String,
    pub title: String,
    pub csrf_token: String,
    pub csrf: String,
}

pub struct Live {
    client: BilibiliClient,
    room_id: u64,
    csrf: String,
}

impl Live {
    pub fn new(room_id: u64, csrf: String, cookie_str: &str) -> Result<Self> {
        let client = BilibiliClient::with_cookies(cookie_str)?;
        Ok(Self {
            client,
            room_id,
            csrf,
        })
    }
    
    pub fn new_with_cookies_map(room_id: u64, csrf: String, cookies: &std::collections::HashMap<String, String>) -> Result<Self> {
        let client = BilibiliClient::with_cookies_map(cookies)?;
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
    
    /// 开始直播
    pub async fn start_live(&self, area_id: u32) -> Result<LiveStreamData> {
        let url = "https://api.live.bilibili.com/room/v1/Room/startLive";
        
        let mut params = HashMap::new();
        params.insert("room_id".to_string(), self.room_id.to_string());
        params.insert("area_v2".to_string(), area_id.to_string());
        params.insert("platform".to_string(), "pc_link".to_string());
        params.insert("backup_stream".to_string(), "0".to_string());
        params.insert("type".to_string(), "2".to_string());
        params.insert("csrf_token".to_string(), self.csrf.clone());
        params.insert("csrf".to_string(), self.csrf.clone());
        
        // 使用App签名增强安全性
        let signed_params = crate::sign::Signer::sign_live_request(params);
        let data: Vec<_> = signed_params.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        
        let response: ApiResponse<LiveStreamData> = self.client.post(url, &data).await?;
        let stream_data = response.data.ok_or_else(|| crate::error::BiliError::Live("获取推流信息失败".to_string()))?;
        
        Ok(stream_data)
    }
    
    /// 停止直播
    pub async fn stop_live(&self) -> Result<()> {
        let url = "https://api.live.bilibili.com/room/v1/Room/stopLive";
        
        let mut params = HashMap::new();
        params.insert("room_id".to_string(), self.room_id.to_string());
        params.insert("platform".to_string(), "pc_link".to_string());
        params.insert("csrf_token".to_string(), self.csrf.clone());
        params.insert("csrf".to_string(), self.csrf.clone());
        
        // 使用App签名增强安全性
        let signed_params = crate::sign::Signer::sign_live_request(params);
        let data: Vec<_> = signed_params.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        
        let _response: ApiResponse<serde_json::Value> = self.client.post(url, &data).await?;
        
        Ok(())
    }
    
    /// 设置直播标题
    pub async fn set_title(&self, title: &str) -> Result<()> {
        let url = "https://api.live.bilibili.com/room/v1/Room/update";
        
        let mut params = HashMap::new();
        params.insert("room_id".to_string(), self.room_id.to_string());
        params.insert("platform".to_string(), "pc_link".to_string());
        params.insert("title".to_string(), title.to_string());
        params.insert("csrf_token".to_string(), self.csrf.clone());
        params.insert("csrf".to_string(), self.csrf.clone());
        
        // 使用App签名增强安全性
        let signed_params = crate::sign::Signer::sign_live_request(params);
        let data: Vec<_> = signed_params.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        
        let _response: ApiResponse<serde_json::Value> = self.client.post(url, &data).await?;
        
        Ok(())
    }
    
    /// 设置直播分区
    pub async fn set_area(&self, area_id: u32) -> Result<()> {
        let url = "https://api.live.bilibili.com/room/v1/Room/update";
        
        let mut params = HashMap::new();
        params.insert("room_id".to_string(), self.room_id.to_string());
        params.insert("area_id".to_string(), area_id.to_string());
        params.insert("activity_id".to_string(), "0".to_string());
        params.insert("platform".to_string(), "pc_link".to_string());
        params.insert("csrf_token".to_string(), self.csrf.clone());
        params.insert("csrf".to_string(), self.csrf.clone());
        
        // 使用App签名增强安全性
        let signed_params = crate::sign::Signer::sign_live_request(params);
        let data: Vec<_> = signed_params.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();
        
        let _response: ApiResponse<serde_json::Value> = self.client.post(url, &data).await?;
        
        Ok(())
    }
    
    /// 获取直播分区列表
    pub async fn get_area_list(&self) -> Result<Vec<AreaCategory>> {
        let url = "https://api.live.bilibili.com/room/v1/Area/getList?show_pinyin=1";
        
        let response: ApiResponse<Vec<AreaCategory>> = self.client.get(url).await?;
        let area_data = response.data.ok_or_else(|| crate::error::BiliError::Live("获取分区列表失败".to_string()))?;
        
        Ok(area_data)
    }
    
    /// 获取直播间信息
    pub async fn get_room_info(&self) -> Result<serde_json::Value> {
        let url = format!("https://api.live.bilibili.com/room/v1/Room/get_info?room_id={}", self.room_id);
        
        let response: ApiResponse<serde_json::Value> = self.client.get(&url).await?;
        let room_info = response.data.ok_or_else(|| crate::error::BiliError::Live("获取直播间信息失败".to_string()))?;
        
        Ok(room_info)
    }
    
    /// 获取直播状态
    pub async fn get_live_status(&self) -> Result<i32> {
        let room_info = self.get_room_info().await?;
        
        if let Some(live_status) = room_info.get("live_status") {
            if let Some(status) = live_status.as_i64() {
                return Ok(status as i32);
            }
        }
        
        Err(crate::error::BiliError::Live("获取直播状态失败".to_string()))
    }
    
    /// 检查是否正在直播
    pub async fn is_live(&self) -> Result<bool> {
        let status = self.get_live_status().await?;
        Ok(status == 1)
    }
    
    /// 获取当前直播标题
    pub async fn get_current_title(&self) -> Result<String> {
        let room_info = self.get_room_info().await?;
        
        if let Some(title) = room_info.get("title") {
            if let Some(title_str) = title.as_str() {
                return Ok(title_str.to_string());
            }
        }
        
        Err(crate::error::BiliError::Live("获取直播标题失败".to_string()))
    }
    
    /// 获取当前直播分区
    pub async fn get_current_area(&self) -> Result<(u32, String)> {
        let room_info = self.get_room_info().await?;
        
        let area_id = room_info.get("area_id")
            .and_then(|v| v.as_u64())
            .ok_or_else(|| crate::error::BiliError::Live("获取分区ID失败".to_string()))?;
        
        let area_name = room_info.get("area_name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::BiliError::Live("获取分区名称失败".to_string()))?;
        
        Ok((area_id as u32, area_name.to_string()))
    }
    
    /// 获取直播间统计信息
    pub async fn get_live_stats(&self) -> Result<serde_json::Value> {
        let url = format!("https://api.live.bilibili.com/xlive/web-room/v1/index/getInfoByRoom?room_id={}", self.room_id);
        
        let response: ApiResponse<serde_json::Value> = self.client.get(&url).await?;
        let stats = response.data.ok_or_else(|| crate::error::BiliError::Live("获取直播间统计信息失败".to_string()))?;
        
        Ok(stats)
    }
    
    /// 获取推流地址和推流码
    pub fn parse_stream_info(&self, stream_data: &LiveStreamData) -> (String, String) {
        let server = stream_data.rtmp.addr.clone();
        let stream_key = stream_data.rtmp.code.clone();
        (server, stream_key)
    }
    
    /// 格式化推流信息输出
    pub fn format_stream_info(&self, stream_data: &LiveStreamData) -> String {
        let (server, stream_key) = self.parse_stream_info(stream_data);
        
        format!(
            "推流服务器: {}\n推流码: {}",
            server,
            stream_key
        )
    }
    
    /// 保存推流信息到文件
    pub async fn save_stream_info_to_file(&self, stream_data: &LiveStreamData, file_path: &str) -> Result<()> {
        let info = self.format_stream_info(stream_data);
        tokio::fs::write(file_path, info).await?;
        Ok(())
    }
    
    /// 获取房间号
    pub fn get_room_id(&self) -> u64 {
        self.room_id
    }
    
    /// 获取CSRF token
    pub fn get_csrf(&self) -> &str {
        &self.csrf
    }
} 