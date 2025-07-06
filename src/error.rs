use thiserror::Error;

pub type Result<T> = std::result::Result<T, BiliError>;

#[derive(Error, Debug)]
pub enum BiliError {
    #[error("网络请求错误: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("JSON解析错误: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("配置错误: {0}")]
    Config(#[from] config::ConfigError),
    
    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("数字解析错误: {0}")]
    ParseInt(#[from] std::num::ParseIntError),
    
    #[error("URL解析错误: {0}")]
    UrlParse(#[from] url::ParseError),
    
    #[error("二维码生成错误: {0}")]
    QRCode(String),
    
    #[error("登录失败: {0}")]
    Login(String),
    
    #[error("认证失败: {0}")]
    Auth(String),
    
    #[error("直播操作失败: {0}")]
    Live(String),
    
    #[error("弹幕发送失败: {0}")]
    Bullet(String),
    
    #[error("API响应错误: code={0}, message={1}")]
    Api(i32, String),
    
    #[error("验证失败: {0}")]
    Validation(String),
    
    #[error("超时错误: {0}")]
    Timeout(String),
    
    #[error("权限不足: {0}")]
    Permission(String),
    
    #[error("资源不存在: {0}")]
    NotFound(String),
    
    #[error("内部错误: {0}")]
    Internal(String),
    
    #[error("{0}")]
    General(String),
}

impl BiliError {
    /// 创建API错误
    pub fn api_error(code: i32, message: impl Into<String>) -> Self {
        BiliError::Api(code, message.into())
    }
    
    /// 创建通用错误
    pub fn general(message: impl Into<String>) -> Self {
        BiliError::General(message.into())
    }
    
    /// 创建登录错误
    pub fn login(message: impl Into<String>) -> Self {
        BiliError::Login(message.into())
    }
    
    /// 创建认证错误
    pub fn auth(message: impl Into<String>) -> Self {
        BiliError::Auth(message.into())
    }
    
    /// 创建直播错误
    pub fn live(message: impl Into<String>) -> Self {
        BiliError::Live(message.into())
    }
    
    /// 创建弹幕错误
    pub fn bullet(message: impl Into<String>) -> Self {
        BiliError::Bullet(message.into())
    }
    
    /// 创建QR码错误
    pub fn qrcode(message: impl Into<String>) -> Self {
        BiliError::QRCode(message.into())
    }
    
    /// 创建验证错误
    pub fn validation(message: impl Into<String>) -> Self {
        BiliError::Validation(message.into())
    }
    
    /// 创建超时错误
    pub fn timeout(message: impl Into<String>) -> Self {
        BiliError::Timeout(message.into())
    }
    
    /// 创建权限错误
    pub fn permission(message: impl Into<String>) -> Self {
        BiliError::Permission(message.into())
    }
    
    /// 创建资源不存在错误
    pub fn not_found(message: impl Into<String>) -> Self {
        BiliError::NotFound(message.into())
    }
    
    /// 创建内部错误
    pub fn internal(message: impl Into<String>) -> Self {
        BiliError::Internal(message.into())
    }
    
    /// 判断是否为网络错误
    pub fn is_network_error(&self) -> bool {
        matches!(self, BiliError::Network(_))
    }
    
    /// 判断是否为认证错误
    pub fn is_auth_error(&self) -> bool {
        matches!(self, BiliError::Auth(_) | BiliError::Login(_) | BiliError::Permission(_))
    }
    
    /// 判断是否为API错误
    pub fn is_api_error(&self) -> bool {
        matches!(self, BiliError::Api(_, _))
    }
    
    /// 获取错误代码（如果是API错误）
    pub fn error_code(&self) -> Option<i32> {
        if let BiliError::Api(code, _) = self {
            Some(*code)
        } else {
            None
        }
    }
    
    /// 判断是否为可重试的错误
    pub fn is_retryable(&self) -> bool {
        match self {
            BiliError::Network(_) | BiliError::Timeout(_) => true,
            BiliError::Api(code, _) if *code == 503 || *code == 429 => true,
            _ => false,
        }
    }
}