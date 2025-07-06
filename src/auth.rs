use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::sleep;
use crate::client::{BilibiliClient, ApiResponse};
use crate::qr::{QRCode, QRCodeData};
use crate::error::{Result, BiliError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginData {
    pub url: String,
    pub qrcode_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginStatusData {
    pub code: i32,
    pub message: String,
    pub url: Option<String>,
    pub refresh_token: Option<String>,
    pub timestamp: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub uid: u64,
    pub room_id: u64,
    pub csrf: String,
    pub cookies: HashMap<String, String>,
}

pub struct Auth {
    client: BilibiliClient,
}

impl Auth {
    pub fn new() -> Result<Self> {
        Ok(Self {
            client: BilibiliClient::new()?,
        })
    }
    
    pub fn with_client(client: BilibiliClient) -> Self {
        Self { client }
    }
    
    /// 生成登录二维码
    pub async fn generate_qrcode(&self) -> Result<QRCodeData> {
        let url = "https://passport.bilibili.com/x/passport-login/web/qrcode/generate";
        
        let response: ApiResponse<LoginData> = self.client.get(url).await?;
        
        let data = response.data.ok_or_else(|| BiliError::Login("获取二维码数据失败".to_string()))?;
        
        Ok(QRCodeData {
            url: data.url,
            qrcode_key: data.qrcode_key,
        })
    }
    
    /// 检查二维码登录状态
    pub async fn check_login_status(&self, qrcode_key: &str) -> Result<(LoginStatusData, Option<HashMap<String, String>>)> {
        let url = format!("https://passport.bilibili.com/x/passport-login/web/qrcode/poll?qrcode_key={}", qrcode_key);
        
        let response = self.client.get_client()
            .get(&url)
            .headers(BilibiliClient::get_default_headers())
            .send()
            .await?;
        
        // 先获取cookies，再解析JSON
        let mut cookies = HashMap::new();
        for cookie in response.cookies() {
            cookies.insert(cookie.name().to_string(), cookie.value().to_string());
        }
        
        let cookies_dict = if cookies.is_empty() {
            None
        } else {
            Some(cookies)
        };
        
        let json: ApiResponse<LoginStatusData> = response.json().await?;
        
        let status_data = json.data.unwrap_or_else(|| LoginStatusData {
            code: json.code,
            message: json.message,
            url: None,
            refresh_token: None,
            timestamp: None,
        });
        
        Ok((status_data, cookies_dict))
    }
    
    /// 二维码登录流程
    pub async fn qr_login(&self) -> Result<UserInfo> {
        // 生成二维码
        let qr_data = self.generate_qrcode().await?;
        
        println!("请扫描以下二维码登录:");
        QRCode::print_unicode_to_terminal(&qr_data.url)?;
        // println!("二维码链接: {}", qr_data.url);
        println!("等待扫描二维码...");
        
        let mut login_cookies: Option<HashMap<String, String>> = None;
        let mut last_status_code = -1; // 记录上次状态码，避免重复打印
        
        // 轮询登录状态
        loop {
            let (status, cookies) = self.check_login_status(&qr_data.qrcode_key).await?;
            
            // 保存cookies
            if let Some(cookies_dict) = cookies {
                login_cookies = Some(cookies_dict);
            }
            
            // 只有状态变化时才打印消息
            if status.code != last_status_code {
                match status.code {
                    0 => {
                        println!("登录成功!");
                        break;
                    }
                    86038 => {
                        return Err(BiliError::Login("二维码已失效，请重新生成".to_string()));
                    }
                    86090 => {
                        println!("二维码已扫描，等待确认...");
                    }
                    86101 => {
                        // 已经在开始时显示过了，不再重复显示
                    }
                    _ => {
                        return Err(BiliError::Login(format!("登录失败: {}", status.message)));
                    }
                }
                last_status_code = status.code;
            }
            
            // 如果登录成功，跳出循环
            if status.code == 0 {
                break;
            }
            
            sleep(Duration::from_secs(2)).await;
        }
        
        // 使用获取到的cookies
        let cookies = login_cookies.ok_or_else(|| BiliError::Login("未获取到登录cookies".to_string()))?;
        
        // 获取用户信息
        println!("正在获取用户信息...");
        let user_info = self.get_user_info(&cookies).await?;
        
        Ok(user_info)
    }
    
    /// 获取用户信息
    pub async fn get_user_info(&self, cookies: &HashMap<String, String>) -> Result<UserInfo> {
        let dede_user_id = cookies.get("DedeUserID")
            .ok_or_else(|| BiliError::Auth("未找到用户ID".to_string()))?;
        
        let csrf = cookies.get("bili_jct")
            .ok_or_else(|| BiliError::Auth("未找到CSRF token".to_string()))?;
        
        let uid: u64 = dede_user_id.parse()
            .map_err(|_| BiliError::Auth("用户ID格式错误".to_string()))?;
        
        // 获取直播间ID
        let room_id = self.get_room_id(uid).await?;
        
        Ok(UserInfo {
            uid,
            room_id,
            csrf: csrf.clone(),
            cookies: cookies.clone(),
        })
    }
    
    /// 根据用户ID获取直播间ID
    pub async fn get_room_id(&self, uid: u64) -> Result<u64> {
        let url = format!("https://api.live.bilibili.com/room/v2/Room/room_id_by_uid?uid={}", uid);
        
        #[derive(Deserialize)]
        struct RoomIdData {
            room_id: u64,
        }
        
        let response: ApiResponse<RoomIdData> = self.client.get(&url).await?;
        let data = response.data.ok_or_else(|| BiliError::Auth("获取直播间ID失败".to_string()))?;
        
        Ok(data.room_id)
    }
    
    /// 从cookie字符串解析cookies
    pub fn parse_cookie_string(cookie_str: &str) -> Result<HashMap<String, String>> {
        BilibiliClient::parse_cookies(cookie_str)
    }
    
    /// 将cookies转换为字符串
    pub fn cookies_to_string(cookies: &HashMap<String, String>) -> String {
        cookies.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("; ")
    }
    
    /// 验证cookies是否有效
    pub async fn validate_cookies(&self, cookies: &HashMap<String, String>) -> Result<bool> {
        let client = BilibiliClient::with_cookies(&Self::cookies_to_string(cookies))?;
        
        // 尝试获取用户信息来验证cookies
        let result = client.get::<serde_json::Value>("https://api.bilibili.com/x/web-interface/nav").await;
        
        match result {
            Ok(response) => {
                if let Some(data) = response.data {
                    if let Some(is_login) = data.get("isLogin") {
                        return Ok(is_login.as_bool().unwrap_or(false));
                    }
                }
                Ok(false)
            }
            Err(_) => Ok(false),
        }
    }
}

impl Default for Auth {
    fn default() -> Self {
        Self::new().unwrap()
    }
} 