use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use md5::{Md5, Digest};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use urlencoding::encode;

type HmacSha256 = Hmac<Sha256>;

pub struct Signer;

impl Signer {
    /// APP密钥和版本信息
    const APP_KEY: &'static str = "1d8b6e7d45233436";
    const APP_SECRET: &'static str = "560c52ccd288fed045859ed18bffd973";
    const LIVEHIME_BUILD: &'static str = "105101";
    const LIVEHIME_VERSION: &'static str = "5.15.1";
    
    /// WBI签名用的混合密钥表
    const MIXIN_KEY_ENC_TAB: [usize; 64] = [
        46, 47, 18, 2, 53, 8, 23, 32, 15, 50, 10, 31, 58, 3, 45, 35,
        27, 43, 5, 49, 33, 9, 42, 19, 29, 28, 14, 39, 12, 38, 41, 13,
        37, 48, 7, 16, 24, 55, 40, 61, 26, 17, 0, 1, 60, 51, 30, 4,
        22, 25, 54, 21, 56, 59, 6, 63, 57, 62, 11, 36, 20, 34, 44, 52,
    ];

    /// 获取当前时间戳（秒）
    fn current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// App签名 - 对请求数据进行签名
    pub fn app_sign(mut data: HashMap<String, String>) -> HashMap<String, String> {
        // 添加必要的字段
        data.insert("access_key".to_string(), "".to_string());
        data.insert("ts".to_string(), Self::current_timestamp().to_string());
        data.insert("build".to_string(), Self::LIVEHIME_BUILD.to_string());
        data.insert("version".to_string(), Self::LIVEHIME_VERSION.to_string());
        data.insert("appkey".to_string(), Self::APP_KEY.to_string());

        // 按照key排序
        let mut sorted_keys: Vec<_> = data.keys().collect();
        sorted_keys.sort();

        // 构建查询字符串
        let mut query_string = String::new();
        for (i, key) in sorted_keys.iter().enumerate() {
            if i > 0 {
                query_string.push('&');
            }
            let value = &data[*key];
            query_string.push_str(&format!("{}={}", key, encode(value)));
        }

        // 计算签名
        let sign_input = format!("{}{}", query_string, Self::APP_SECRET);
        let mut hasher = Md5::new();
        hasher.update(sign_input.as_bytes());
        let sign = format!("{:x}", hasher.finalize());

        // 添加签名字段
        data.insert("sign".to_string(), sign);
        
        data
    }

    /// WBI签名 - 基于img_key和sub_key的混合密钥签名
    pub fn wbi_sign(mut params: HashMap<String, String>, img_key: &str, sub_key: &str) -> HashMap<String, String> {
        // 获取混合密钥
        let mixin_key = Self::get_mixin_key(img_key, sub_key);
        
        // 添加时间戳
        params.insert("wts".to_string(), Self::current_timestamp().to_string());
        
        // 按key排序
        let mut sorted_keys: Vec<_> = params.keys().collect();
        sorted_keys.sort();
        
        // 构建查询字符串并过滤特殊字符
        let mut query_string = String::new();
        for (i, key) in sorted_keys.iter().enumerate() {
            if i > 0 {
                query_string.push('&');
            }
            let value = &params[*key];
            let filtered_value: String = value
                .chars()
                .filter(|c| !"!'()*".contains(*c))
                .collect();
            query_string.push_str(&format!("{}={}", key, encode(&filtered_value)));
        }
        
        // 计算签名
        let sign_input = format!("{}{}", query_string, mixin_key);
        let mut hasher = Md5::new();
        hasher.update(sign_input.as_bytes());
        let w_rid = format!("{:x}", hasher.finalize());
        
        // 添加签名字段
        params.insert("w_rid".to_string(), w_rid);
        
        params
    }

    /// 获取混合密钥
    fn get_mixin_key(img_key: &str, sub_key: &str) -> String {
        let combined = format!("{}{}", img_key, sub_key);
        let mut result = String::new();
        
        for &index in &Self::MIXIN_KEY_ENC_TAB {
            if let Some(ch) = combined.chars().nth(index) {
                result.push(ch);
            }
        }
        
        result.chars().take(32).collect()
    }

    /// HMAC-SHA256签名
    pub fn hmac_sha256(key: &str, message: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(key.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(message.as_bytes());
        let result = mac.finalize();
        hex::encode(result.into_bytes())
    }

    /// 为直播API请求添加签名
    pub fn sign_live_request(params: HashMap<String, String>) -> HashMap<String, String> {
        Self::app_sign(params)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_app_sign() {
        let mut params = HashMap::new();
        params.insert("room_id".to_string(), "123456".to_string());
        params.insert("platform".to_string(), "pc_link".to_string());
        
        let signed = Signer::app_sign(params);
        
        assert!(signed.contains_key("sign"));
        assert!(signed.contains_key("appkey"));
        assert!(signed.contains_key("ts"));
        assert_eq!(signed.get("appkey"), Some(&Signer::APP_KEY.to_string()));
    }

    #[test]
    fn test_wbi_sign() {
        let mut params = HashMap::new();
        params.insert("mid".to_string(), "123456".to_string());
        
        let signed = Signer::wbi_sign(params, "img_key_example", "sub_key_example");
        
        assert!(signed.contains_key("w_rid"));
        assert!(signed.contains_key("wts"));
    }

    #[test]
    fn test_mixin_key() {
        let key = Signer::get_mixin_key("1234567890abcdef", "fedcba0987654321");
        assert_eq!(key.len(), 32);
    }
}