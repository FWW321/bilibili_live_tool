use qrcode::{QrCode as QRCodeLib, Color};
use image::{Rgb, RgbImage};
use serde::{Deserialize, Serialize};
use std::io::{self, Write};
use crate::error::{Result, BiliError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QRCodeData {
    pub url: String,
    pub qrcode_key: String,
}

pub struct QRCode;

impl QRCode {
    /// 生成二维码ASCII字符串
    pub fn generate_ascii(data: &str) -> Result<String> {
        let qr = QRCodeLib::new(data)
            .map_err(|e| BiliError::QRCode(format!("生成二维码失败: {}", e)))?;
        
        let string = qr.render::<char>()
            .quiet_zone(false)
            .module_dimensions(2, 1)
            .build();
        
        Ok(string)
    }
    
    /// 生成二维码图片
    pub fn generate_image(data: &str) -> Result<RgbImage> {
        let qr = QRCodeLib::new(data)
            .map_err(|e| BiliError::QRCode(format!("生成二维码失败: {}", e)))?;
        
        let image = qr.render::<Rgb<u8>>()
            .max_dimensions(200, 200)
            .build();
        
        Ok(image)
    }
    
    /// 在终端中打印二维码
    pub fn print_to_terminal(data: &str) -> Result<()> {
        let qr = QRCodeLib::new(data)
            .map_err(|e| BiliError::QRCode(format!("生成二维码失败: {}", e)))?;
        
        let string = qr.render::<char>()
            .quiet_zone(false)
            .module_dimensions(2, 1)
            .build();
        
        println!("{}", string);
        Ok(())
    }
    
    /// 使用Unicode字符打印更好看的二维码
    pub fn print_unicode_to_terminal(data: &str) -> Result<()> {
        let qr = QRCodeLib::new(data)
            .map_err(|e| BiliError::QRCode(format!("生成二维码失败: {}", e)))?;
        
        let width = qr.width();
        let mut stdout = io::stdout();
        
        // 上边框
        print!("┌");
        for _ in 0..width {
            print!("─");
        }
        println!("┐");
        
        // 二维码内容 - 使用半格字符来调整比例，每个模块用一个字符
        for y in (0..width).step_by(2) {
            print!("│");
            for x in 0..width {
                let top_module = qr[(x, y)];
                let bottom_module = if y + 1 < width {
                    qr[(x, y + 1)]
                } else {
                    Color::Light
                };
                
                let char_to_print = match (top_module, bottom_module) {
                    (Color::Light, Color::Light) => " ",
                    (Color::Light, Color::Dark) => "▄",
                    (Color::Dark, Color::Light) => "▀",
                    (Color::Dark, Color::Dark) => "█",
                };
                print!("{}", char_to_print);
            }
            println!("│");
        }
        
        // 下边框
        print!("└");
        for _ in 0..width {
            print!("─");
        }
        println!("┘");
        
        stdout.flush()?;
        Ok(())
    }
    
    /// 保存二维码图片到文件
    pub fn save_image(data: &str, path: &str) -> Result<()> {
        let image = Self::generate_image(data)?;
        image.save(path)
            .map_err(|e| BiliError::QRCode(format!("保存二维码图片失败: {}", e)))?;
        Ok(())
    }
    
    /// 生成带边框的二维码ASCII字符串
    pub fn generate_ascii_with_border(data: &str) -> Result<String> {
        let qr = QRCodeLib::new(data)
            .map_err(|e| BiliError::QRCode(format!("生成二维码失败: {}", e)))?;
        
        let width = qr.width();
        let mut result = String::new();
        
        // 上边框
        result.push_str("┌");
        for _ in 0..width {
            result.push_str("─");
        }
        result.push_str("┐\n");
        
        // 二维码内容
        for y in 0..width {
            result.push_str("│");
            for x in 0..width {
                let module = qr[(x, y)];
                match module {
                    Color::Light => result.push_str(" "),
                    Color::Dark => result.push_str("█"),
                }
            }
            result.push_str("│\n");
        }
        
        // 下边框
        result.push_str("└");
        for _ in 0..width {
            result.push_str("─");
        }
        result.push_str("┘\n");
        
        Ok(result)
    }
} 