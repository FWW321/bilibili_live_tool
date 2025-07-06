use reqwest::{Client, header::HeaderMap, cookie::Jar};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use crate::error::{Result, BiliError};

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/96.0.4664.110 Safari/537.36";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub code: i32,
    pub message: String,
    pub data: Option<T>,
    pub msg: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn is_success(&self) -> bool {
        self.code == 0
    }
    
    pub fn get_message(&self) -> &str {
        self.msg.as_deref().unwrap_or(&self.message)
    }
}

#[derive(Debug, Clone)]
pub struct BilibiliClient {
    client: Client,
    jar: Arc<Jar>,
}

impl BilibiliClient {
    pub fn new() -> Result<Self> {
        let jar = Arc::new(Jar::default());
        let client = Client::builder()
            .cookie_provider(jar.clone())
            .user_agent(USER_AGENT)
            .build()?;
        
        Ok(Self {
            client,
            jar,
        })
    }
    
    pub fn with_cookies(cookie_str: &str) -> Result<Self> {
        let jar = Arc::new(Jar::default());
        let client = Client::builder()
            .cookie_provider(jar.clone())
            .user_agent(USER_AGENT)
            .build()?;
        
        // 解析并添加cookies
        let cookies = Self::parse_cookies(cookie_str)?;
        Self::add_cookies_to_jar(&jar, &cookies);
        
        Ok(Self {
            client,
            jar,
        })
    }
    
    pub fn with_cookies_map(cookies: &HashMap<String, String>) -> Result<Self> {
        let jar = Arc::new(Jar::default());
        let client = Client::builder()
            .cookie_provider(jar.clone())
            .user_agent(USER_AGENT)
            .build()?;
        
        // 直接添加cookies
        Self::add_cookies_to_jar(&jar, cookies);
        
        Ok(Self {
            client,
            jar,
        })
    }
    
    fn add_cookies_to_jar(jar: &Arc<Jar>, cookies: &HashMap<String, String>) {
        // 为B站的主要域名添加cookies
        let domains = [
            "https://bilibili.com",
            "https://www.bilibili.com", 
            "https://api.bilibili.com",
            "https://api.live.bilibili.com",
            "https://live.bilibili.com",
            "https://link.bilibili.com",
        ];
        
        for domain in &domains {
            if let Ok(url) = domain.parse() {
                for (key, value) in cookies {
                    let cookie = format!("{}={}", key, value);
                    jar.add_cookie_str(&cookie, &url);
                }
            }
        }
    }
    
    pub fn parse_cookies(cookie_str: &str) -> Result<HashMap<String, String>> {
        let mut cookies = HashMap::new();
        let regex = regex::Regex::new(r"(\w+)=([^;]+)(?:;|$)").unwrap();
        
        for cap in regex.captures_iter(cookie_str) {
            let key = cap.get(1).unwrap().as_str().to_string();
            let value = urlencoding::decode(cap.get(2).unwrap().as_str())
                .map_err(|e| BiliError::General(format!("解析cookie失败: {}", e)))?
                .to_string();
            cookies.insert(key, value);
        }
        
        Ok(cookies)
    }
    
    pub fn get_default_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("accept", "application/json, text/plain, */*".parse().unwrap());
        headers.insert("accept-language", "zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6".parse().unwrap());
        headers.insert("content-type", "application/x-www-form-urlencoded; charset=UTF-8".parse().unwrap());
        headers.insert("origin", "https://link.bilibili.com".parse().unwrap());
        headers.insert("referer", "https://link.bilibili.com/p/center/index".parse().unwrap());
        headers.insert("sec-ch-ua", r#""Microsoft Edge";v="129", "Not=A?Brand";v="8", "Chromium";v="129""#.parse().unwrap());
        headers.insert("sec-ch-ua-mobile", "?0".parse().unwrap());
        headers.insert("sec-ch-ua-platform", r#""Windows""#.parse().unwrap());
        headers.insert("sec-fetch-dest", "empty".parse().unwrap());
        headers.insert("sec-fetch-mode", "cors".parse().unwrap());
        headers.insert("sec-fetch-site", "same-site".parse().unwrap());
        headers.insert("user-agent", USER_AGENT.parse().unwrap());
        headers
    }
    
    pub async fn get<T: for<'de> Deserialize<'de>>(&self, url: &str) -> Result<ApiResponse<T>> {
        let response = self.client
            .get(url)
            .headers(Self::get_default_headers())
            .send()
            .await?;
        
        let json: ApiResponse<T> = response.json().await?;
        
        if !json.is_success() {
            return Err(BiliError::api_error(json.code, json.get_message().to_string()));
        }
        
        Ok(json)
    }
    
    pub async fn post<T: for<'de> Deserialize<'de>>(&self, url: &str, data: &[(&str, &str)]) -> Result<ApiResponse<T>> {
        let response = self.client
            .post(url)
            .headers(Self::get_default_headers())
            .form(data)
            .send()
            .await?;
        
        let json: ApiResponse<T> = response.json().await?;
        
        if !json.is_success() {
            return Err(BiliError::api_error(json.code, json.get_message().to_string()));
        }
        
        Ok(json)
    }
    
    pub async fn post_json<T: for<'de> Deserialize<'de>, D: Serialize>(&self, url: &str, data: &D) -> Result<ApiResponse<T>> {
        let response = self.client
            .post(url)
            .headers(Self::get_default_headers())
            .json(data)
            .send()
            .await?;
        
        let json: ApiResponse<T> = response.json().await?;
        
        if !json.is_success() {
            return Err(BiliError::api_error(json.code, json.get_message().to_string()));
        }
        
        Ok(json)
    }
    
    pub fn get_client(&self) -> &Client {
        &self.client
    }
    
    pub fn get_jar(&self) -> &Arc<Jar> {
        &self.jar
    }
}

impl Default for BilibiliClient {
    fn default() -> Self {
        Self::new().unwrap()
    }
} 