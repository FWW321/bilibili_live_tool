use clap::{Arg, Command};
use bilibili_live_tool::*;
use bilibili_live_tool::tui::TuiApp;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("bilibili_live_tool")
        .about("哔哩哔哩直播推流码获取工具")
        .version("0.1.0")
        .author("FWW")
        .arg(
            Arg::new("cli")
                .short('c')
                .long("cli")
                .help("使用传统命令行模式")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("config")
                .short('f')
                .long("config-file")
                .help("配置文件路径")
                .value_name("FILE"),
        )
        .get_matches();
    
    // 如果指定了CLI参数，使用传统命令行模式
    if matches.get_flag("cli") {
        return run_cli().await;
    }
    
    // 默认使用TUI模式
    run_tui().await
}

async fn run_tui() -> Result<()> {
    println!("正在启动...");
    
    // 加载配置
    let mut config = match Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("加载配置失败: {}", e);
            eprintln!("使用默认配置");
            Config::default()
        }
    };



    // 获取认证信息
    let user_info = if config.has_credentials() {
        println!("检测到已保存的认证信息，正在验证...");
        
        // 尝试使用已保存的认证信息
        let auth_result = Auth::new();
        let cookies_result = Auth::parse_cookie_string(&config.cookie_str.as_ref().unwrap());
        let room_id_result = config.room_id.as_ref().unwrap().parse::<u64>();
        
        match (auth_result, cookies_result, room_id_result) {
            (Ok(auth), Ok(cookies), Ok(room_id)) => {
                match auth.validate_cookies(&cookies).await {
                    Ok(true) => {
                        println!("认证信息有效，正在启动...");
                        auth::UserInfo {
                            uid: 0,
                            room_id,
                            csrf: config.csrf.as_ref().unwrap().clone(),
                            cookies,
                        }
                    }
                    Ok(false) => {
                        println!("认证信息已过期，开始扫码登录");
                        match login().await {
                            Ok(user_info) => {
                                save_credentials(&mut config, &user_info);
                                user_info
                            }
                            Err(e) => {
                                eprintln!("登录失败: {}", e);
                                return Err(e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("验证认证信息失败: {}", e);
                        println!("开始扫码登录");
                        match login().await {
                            Ok(user_info) => {
                                save_credentials(&mut config, &user_info);
                                user_info
                            }
                            Err(e) => {
                                eprintln!("登录失败: {}", e);
                                return Err(e);
                            }
                        }
                    }
                }
            }
            _ => {
                println!("解析已保存的认证信息失败，开始扫码登录");
                match login().await {
                    Ok(user_info) => {
                        save_credentials(&mut config, &user_info);
                        user_info
                    }
                    Err(e) => {
                        eprintln!("登录失败: {}", e);
                        return Err(e);
                    }
                }
            }
        }
    } else {
        println!("扫码登录");
        match login().await {
            Ok(user_info) => {
                save_credentials(&mut config, &user_info);
                user_info
            }
            Err(e) => {
                eprintln!("登录失败: {}", e);
                return Err(e);
            }
        }
    };

    // 创建Live实例
    let live = match Live::new_with_cookies_map(user_info.room_id, user_info.csrf.clone(), &user_info.cookies) {
        Ok(live) => live,
        Err(e) => {
            eprintln!("创建直播客户端失败: {}", e);
            return Err(e);
        }
    };

    // 创建TUI应用（在保存认证信息之后，确保config包含最新的登录信息）
    let app = TuiApp::new(config);

    // 运行TUI应用
    app.with_live(live, user_info).run().await
}

fn save_credentials(config: &mut Config, user_info: &auth::UserInfo) {
    let cookie_str = Auth::cookies_to_string(&user_info.cookies);
    config.set_credentials(
        user_info.room_id.to_string(),
        cookie_str,
        user_info.csrf.clone(),
    );
    if let Err(e) = config.save() {
        eprintln!("保存认证信息失败: {}", e);
        eprintln!("程序将继续运行，但下次启动时需要重新登录");
    }
}

async fn run_cli() -> Result<()> {
    println!("=== 哔哩哔哩直播推流码获取工具 ===");
    println!("版本: 0.1.0");
    println!("作者: Chace");
    println!();
    
    // 加载配置
    let mut config = match Config::load() {
        Ok(cfg) => cfg,
        Err(e) => {
            println!("加载配置失败: {}", e);
            println!("使用默认配置");
            Config::default()
        }
    };
    

    
    // 获取认证信息
    let user_info = if config.has_credentials() {
        println!("检测到已保存的认证信息，正在验证...");
        
        // 尝试使用已保存的认证信息
        let auth_result = Auth::new();
        let cookies_result = Auth::parse_cookie_string(&config.cookie_str.as_ref().unwrap());
        let room_id_result = config.room_id.as_ref().unwrap().parse::<u64>();
        
        match (auth_result, cookies_result, room_id_result) {
            (Ok(auth), Ok(cookies), Ok(room_id)) => {
                match auth.validate_cookies(&cookies).await {
                    Ok(true) => {
                        println!("认证信息有效");
                        auth::UserInfo {
                            uid: 0,
                            room_id,
                            csrf: config.csrf.as_ref().unwrap().clone(),
                            cookies,
                        }
                    }
                    Ok(false) => {
                        println!("认证信息已过期，开始扫码登录");
                        match login().await {
                            Ok(user_info) => {
                                // 保存新的认证信息
                                save_credentials(&mut config, &user_info);
                                user_info
                            }
                            Err(e) => {
                                println!("登录失败: {}", e);
                                return Err(e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("验证认证信息失败: {}", e);
                        println!("开始扫码登录");
                        match login().await {
                            Ok(user_info) => {
                                // 保存新的认证信息
                                save_credentials(&mut config, &user_info);
                                user_info
                            }
                            Err(e) => {
                                println!("登录失败: {}", e);
                                return Err(e);
                            }
                        }
                    }
                }
            }
            _ => {
                println!("解析已保存的认证信息失败，开始扫码登录");
                match login().await {
                    Ok(user_info) => {
                        // 保存新的认证信息
                        save_credentials(&mut config, &user_info);
                        user_info
                    }
                    Err(e) => {
                        println!("登录失败: {}", e);
                        return Err(e);
                    }
                }
            }
        }
    } else {
        println!("扫码登录");
        match login().await {
            Ok(user_info) => {
                // 保存新的认证信息
                save_credentials(&mut config, &user_info);
                user_info
            }
            Err(e) => {
                println!("登录失败: {}", e);
                return Err(e);
            }
        }
    };
    
    // 创建Live实例，使用HashMap格式的cookies
    let live = match Live::new_with_cookies_map(user_info.room_id, user_info.csrf.clone(), &user_info.cookies) {
        Ok(live) => live,
        Err(e) => {
            println!("创建直播客户端失败: {}", e);
            return Err(e);
        }
    };
    
    // 检查当前直播状态
    match live.is_live().await {
        Ok(is_live) => {
            if is_live {
                println!("检测到当前正在直播中");
                // 如果有保存的推流信息，显示出来
                if let Some((server, key)) = config.get_stream_info() {
                    println!("当前推流信息:");
                    println!("推流服务器: {}", server);
                    println!("推流码: {}", key);
                } else {
                    println!("但未找到保存的推流信息");
                }
            } else {
                println!("当前未在直播中");
            }
        }
        Err(e) => {
            println!("检查直播状态失败: {}", e);
        }
    }
    
    // 直接设置直播标题和分区，然后获取推流码
    println!("\n=== 设置直播信息 ===");
    
    // 设置直播标题
    if let Err(e) = set_title(&live).await {
        println!("设置直播标题失败: {}", e);
        println!("继续使用默认标题...");
    }
    
    // 设置直播分区
    if let Err(e) = set_area(&live).await {
        println!("设置直播分区失败: {}", e);
        println!("继续使用默认分区...");
    }
    
    // 获取推流码并开始直播
    println!("\n=== 获取推流码并开始直播 ===");
    if let Err(e) = start_live(&live, &mut config).await {
        println!("获取推流码失败: {}", e);
        return Err(e);
    }
    
    // 等待用户输入停止直播
    println!("\n已开启直播，请迅速进入第三方直播软件进行直播！");
    println!("下播时请输入Y或y关闭直播！");
    
    loop {
        print!("下播时请输入Y或y关闭直播：");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        if input.trim().to_lowercase() == "y" {
            break;
        }
    }
    
    // 停止直播
    if let Err(e) = stop_live(&live, &mut config).await {
        println!("停止直播时出错: {}", e);
        println!("请手动停止直播");
    }
    
    println!("直播已关闭！");
    println!("按任意键退出...");
    
    // 等待用户按任意键退出
    let _ = io::stdin().read_line(&mut String::new());
    
    println!("再见！");
    
    Ok(())
}



async fn login() -> Result<auth::UserInfo> {
    println!("=== 登录 ===");
    
    // 直接使用二维码登录，不再询问
    qr_login().await
}

async fn qr_login() -> Result<auth::UserInfo> {
    let auth = match Auth::new() {
        Ok(auth) => auth,
        Err(e) => {
            println!("创建认证客户端失败: {}", e);
            return Err(e);
        }
    };
    
    match auth.qr_login().await {
        Ok(user_info) => Ok(user_info),
        Err(e) => {
            println!("二维码登录失败: {}", e);
            Err(e)
        }
    }
}

async fn start_live(live: &Live, config: &mut Config) -> Result<()> {
    println!("正在获取推流码，请稍等...");
    
    // 获取当前分区ID
    let (current_area_id, current_area_name) = match live.get_current_area().await {
        Ok((id, name)) => (id, name),
        Err(e) => {
            println!("获取当前分区失败: {}", e);
            return Err(e);
        }
    };
    println!("使用分区: {} (ID: {})", current_area_name, current_area_id);
    
    let stream_data = match live.start_live(current_area_id).await {
        Ok(data) => data,
        Err(e) => {
            println!("开始直播失败: {}", e);
            return Err(e);
        }
    };
    
    println!("成功获取推流码!");
    println!("{}", live.format_stream_info(&stream_data));
    
    // 保存推流信息到配置文件
    let (rtmp_url, stream_key) = live.parse_stream_info(&stream_data);
    if let Err(e) = config.save_stream_info(rtmp_url, stream_key) {
        println!("保存推流信息失败: {}", e);
    }
    
    Ok(())
}

async fn stop_live(live: &Live, config: &mut Config) -> Result<()> {
    println!("正在停止直播...");
    
    let is_live = match live.is_live().await {
        Ok(status) => status,
        Err(e) => {
            println!("检查直播状态失败: {}", e);
            println!("尝试停止直播...");
            true // 假设正在直播，尝试停止
        }
    };
    
    if !is_live {
        println!("当前没有在直播");
        return Ok(());
    }
    
    match live.stop_live().await {
        Ok(_) => {
            println!("直播已停止");
            
            // 清除配置文件中的推流信息
            if let Err(e) = config.clear_stream_info() {
                println!("清除推流信息失败: {}", e);
            }
        }
        Err(e) => {
            println!("停止直播失败: {}", e);
            println!("请手动在B站直播间停止直播");
            return Err(e);
        }
    }
    
    Ok(())
}

async fn set_title(live: &Live) -> Result<()> {
    println!("=== 设置直播标题 ===");
    
    let current_title = match live.get_current_title().await {
        Ok(title) => title,
        Err(e) => {
            println!("获取当前标题失败: {}", e);
            "未知标题".to_string()
        }
    };
    println!("当前标题: {}", current_title);
    
    print!("请输入新标题（直接回车跳过）: ");
    if let Err(e) = io::stdout().flush() {
        println!("输出缓冲区刷新失败: {}", e);
    }
    
    let mut input = String::new();
    if let Err(e) = io::stdin().read_line(&mut input) {
        println!("读取输入失败: {}", e);
        return Ok(());
    }
    
    let new_title = input.trim();
    if !new_title.is_empty() {
        match live.set_title(new_title).await {
            Ok(_) => println!("标题设置成功"),
            Err(e) => {
                println!("设置标题失败: {}", e);
                println!("将继续使用当前标题");
            }
        }
    } else {
        println!("跳过标题设置，使用当前标题");
    }
    
    Ok(())
}

async fn set_area(live: &Live) -> Result<()> {
    println!("=== 设置直播分区 ===");
    
    let (current_area_id, current_area_name) = match live.get_current_area().await {
        Ok((id, name)) => (id, name),
        Err(e) => {
            println!("获取当前分区失败: {}", e);
            (0, "未知分区".to_string())
        }
    };
    println!("当前分区: {} (ID: {})", current_area_name, current_area_id);
    
    // 获取分区列表
    println!("正在获取分区列表...");
    let areas = match live.get_area_list().await {
        Ok(areas) => {
            println!("成功获取分区列表，共{}个主分区", areas.len());
            areas
        },
        Err(e) => {
            println!("获取分区列表失败: {}", e);
            
            // 输出详细错误信息用于调试
            println!("错误详情: {:?}", e);
            
            // 尝试直接使用当前分区
            println!("将使用当前分区继续");
            return Ok(());
        }
    };
    
    println!("可用分区:");
    for (i, category) in areas.iter().enumerate() {
        println!("{}. {}", i + 1, category.name);
        for (j, area) in category.list.iter().enumerate() {
            println!("   {}.{} {}", i + 1, j + 1, area.name);
        }
    }
    
    print!("请选择分区 (格式: 主分区.子分区, 如: 1.2, 直接回车跳过): ");
    if let Err(e) = io::stdout().flush() {
        println!("输出缓冲区刷新失败: {}", e);
    }
    
    let mut input = String::new();
    if let Err(e) = io::stdin().read_line(&mut input) {
        println!("读取输入失败: {}", e);
        return Ok(());
    }
    
    let input_trimmed = input.trim();
    if input_trimmed.is_empty() {
        println!("跳过分区设置，使用当前分区");
        return Ok(());
    }
    
    let parts: Vec<&str> = input_trimmed.split('.').collect();
    if parts.len() != 2 {
        println!("格式错误，使用当前分区");
        return Ok(());
    }
    
    let main_idx: usize = match parts[0].parse::<usize>() {
        Ok(idx) => idx - 1,
        Err(_) => {
            println!("输入格式错误，使用当前分区");
            return Ok(());
        }
    };
    
    let sub_idx: usize = match parts[1].parse::<usize>() {
        Ok(idx) => idx - 1,
        Err(_) => {
            println!("输入格式错误，使用当前分区");
            return Ok(());
        }
    };
    
    if main_idx >= areas.len() || sub_idx >= areas[main_idx].list.len() {
        println!("分区索引超出范围，使用当前分区");
        return Ok(());
    }
    
    let area_id = areas[main_idx].list[sub_idx].id;
    
    match live.set_area(area_id).await {
        Ok(_) => println!("分区设置成功"),
        Err(e) => {
            println!("设置分区失败: {}", e);
            println!("将继续使用当前分区");
        }
    }
    
    Ok(())
}
