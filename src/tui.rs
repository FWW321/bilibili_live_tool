use std::io::{stdout, Stdout};
use std::time::Duration;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, Clear, Gauge, List, ListItem, ListState, Paragraph, Wrap,
    },
    Frame, Terminal,
};
use crate::{Live, Config, auth::UserInfo, error::Result};

#[derive(Clone)]
pub struct AppState {
    pub menu_state: ListState,
    pub selected_menu: usize,
    pub menu_items: Vec<String>,
    pub is_live: bool,
    pub show_area_search: bool,
    pub area_search_query: String,
    pub area_list: Vec<crate::live::AreaCategory>,
    pub filtered_areas: Vec<crate::live::AreaData>,
    pub area_state: ListState,
    pub current_title: String,
    pub current_area: String,
    pub show_title_input: bool,
    pub title_input: String,
    pub show_message: bool,
    pub message: String,
    pub message_type: MessageType,
    pub show_loading: bool,
    pub loading_message: String,
    pub stream_server: String,
    pub stream_key: String,
    pub show_help: bool,
}

#[derive(Clone)]
pub enum MessageType {
    Info,
    Success,
    Warning,
    Error,
}

impl Default for AppState {
    fn default() -> Self {
        let mut state = Self {
            menu_state: ListState::default(),
            selected_menu: 0,
            menu_items: Vec::new(),
            is_live: false,
            show_area_search: false,
            area_search_query: String::new(),
            area_list: Vec::new(),
            filtered_areas: Vec::new(),
            area_state: ListState::default(),
            current_title: "未设置".to_string(),
            current_area: "未设置".to_string(),
            show_title_input: false,
            title_input: String::new(),
            show_message: false,
            message: String::new(),
            message_type: MessageType::Info,
            show_loading: false,
            loading_message: String::new(),
            stream_server: String::new(),
            stream_key: String::new(),
            show_help: false,
        };
        state.update_menu_items();
        state.menu_state.select(Some(0));
        state
    }
}

impl AppState {
    pub fn next_menu(&mut self) {
        let i = match self.menu_state.selected() {
            Some(i) => {
                if i >= self.menu_items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.menu_state.select(Some(i));
        self.selected_menu = i;
    }

    pub fn previous_menu(&mut self) {
        let i = match self.menu_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.menu_items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.menu_state.select(Some(i));
        self.selected_menu = i;
    }

    pub fn show_message(&mut self, message: String, message_type: MessageType) {
        self.message = message;
        self.message_type = message_type;
        self.show_message = true;
    }

    pub fn hide_message(&mut self) {
        self.show_message = false;
    }

    pub fn show_loading(&mut self, message: String) {
        self.loading_message = message;
        self.show_loading = true;
    }

    pub fn hide_loading(&mut self) {
        self.show_loading = false;
    }

    pub fn filter_areas(&mut self, query: &str) {
        self.filtered_areas.clear();
        
        if query.is_empty() {
            // 如果查询为空，显示所有分区
            for category in &self.area_list {
                self.filtered_areas.extend(category.list.clone());
            }
        } else {
            // 搜索分区
            let query_lower = query.to_lowercase();
            for category in &self.area_list {
                for area in &category.list {
                    if area.name.to_lowercase().contains(&query_lower) 
                        || area.parent_name.to_lowercase().contains(&query_lower) {
                        self.filtered_areas.push(area.clone());
                    }
                }
            }
        }
        
        // 重置选择
        self.area_state.select(if self.filtered_areas.is_empty() { None } else { Some(0) });
    }

    pub fn next_area(&mut self) {
        if self.filtered_areas.is_empty() {
            return;
        }
        
        let i = match self.area_state.selected() {
            Some(i) => {
                if i >= self.filtered_areas.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.area_state.select(Some(i));
    }

    pub fn previous_area(&mut self) {
        if self.filtered_areas.is_empty() {
            return;
        }
        
        let i = match self.area_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.filtered_areas.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.area_state.select(Some(i));
    }

    pub fn get_selected_area(&self) -> Option<&crate::live::AreaData> {
        self.area_state.selected()
            .and_then(|i| self.filtered_areas.get(i))
    }

    /// 根据直播状态更新菜单项
    pub fn update_menu_items(&mut self) {
        // 如果菜单为空，初始化菜单
        if self.menu_items.is_empty() {
            self.menu_items.push("开始直播".to_string());
            self.menu_items.push("修改标题".to_string());
            self.menu_items.push("修改分区".to_string());
            self.menu_items.push("帮助".to_string());
            self.menu_items.push("退出程序".to_string());
            
            // 初始化时选择第一个菜单项
            self.selected_menu = 0;
            self.menu_state.select(Some(0));
        }
        
        // 根据直播状态更新第一个菜单项的文本
        if self.is_live {
            self.menu_items[0] = "结束直播".to_string();
        } else {
            self.menu_items[0] = "开始直播".to_string();
        }
    }

    /// 更新直播状态并更新菜单项文本
    pub fn set_live_status(&mut self, is_live: bool) {
        if self.is_live != is_live {
            self.is_live = is_live;
            self.update_menu_items();
        }
    }

    /// 设置推流信息
    pub fn set_stream_info(&mut self, server: String, key: String) {
        self.stream_server = server;
        self.stream_key = key;
    }

    /// 清空推流信息
    pub fn clear_stream_info(&mut self) {
        self.stream_server.clear();
        self.stream_key.clear();
    }

    /// 显示帮助
    pub fn show_help(&mut self) {
        self.show_help = true;
    }

    /// 隐藏帮助
    pub fn hide_help(&mut self) {
        self.show_help = false;
    }
    

}

pub struct TuiApp {
    pub state: AppState,
    pub live: Option<Live>,
    pub config: Config,
    pub user_info: Option<UserInfo>,
}

impl TuiApp {
    pub fn new(config: Config) -> Self {
        Self {
            state: AppState::default(),
            live: None,
            config,
            user_info: None,
        }
    }

    pub fn with_live(mut self, live: Live, user_info: UserInfo) -> Self {
        self.live = Some(live);
        self.user_info = Some(user_info);
        self
    }

    pub async fn run(mut self) -> Result<()> {
        // 设置终端
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // 初始化当前直播信息
        self.initialize_live_info().await;

        let result = self.run_app(&mut terminal).await;

        // 恢复终端
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen
        )?;
        terminal.show_cursor()?;

        result
    }

    async fn initialize_live_info(&mut self) {
        if let Some(live) = &self.live {
            // 更新直播状态
            if let Ok(is_live) = live.is_live().await {
                self.state.set_live_status(is_live);
                
                // 如果正在直播，从配置文件加载推流信息
                if is_live {
                    if let Some((server, key)) = self.config.get_stream_info() {
                        self.state.set_stream_info(server, key);
                    }
                }
            }

            // 更新标题
            if let Ok(title) = live.get_current_title().await {
                self.state.current_title = title;
            }

            // 更新分区
            if let Ok((_, area_name)) = live.get_current_area().await {
                self.state.current_area = area_name;
            }
        }
    }

    async fn run_app(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
        loop {
            terminal.draw(|f| self.ui(f))?;

            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        if !self.handle_key(key.code).await? {
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle_key(&mut self, key: KeyCode) -> Result<bool> {
        // 如果显示加载界面，忽略按键
        if self.state.show_loading {
            return Ok(true);
        }

        // 处理帮助弹窗
        if self.state.show_help {
            match key {
                KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {
                    self.state.hide_help();
                }
                _ => {}
            }
            return Ok(true);
        }

        // 处理消息框
        if self.state.show_message {
            self.state.hide_message();
            return Ok(true);
        }

        // 处理标题输入
        if self.state.show_title_input {
            match key {
                KeyCode::Enter => {
                    if !self.state.title_input.trim().is_empty() {
                        self.set_title().await?;
                    }
                    self.state.show_title_input = false;
                    self.state.title_input.clear();
                }
                KeyCode::Esc => {
                    self.state.show_title_input = false;
                    self.state.title_input.clear();
                }
                KeyCode::Char(c) => {
                    self.state.title_input.push(c);
                }
                KeyCode::Backspace => {
                    self.state.title_input.pop();
                }
                _ => {}
            }
            return Ok(true);
        }

        // 处理分区搜索
        if self.state.show_area_search {
            match key {
                KeyCode::Enter => {
                    if let Some(area) = self.state.get_selected_area() {
                        let area_id = area.id;
                        self.set_area(area_id).await?;
                        self.state.show_area_search = false;
                        self.state.area_search_query.clear();
                    }
                }
                KeyCode::Esc => {
                    self.state.show_area_search = false;
                    self.state.area_search_query.clear();
                }
                KeyCode::Up => {
                    self.state.previous_area();
                }
                KeyCode::Down => {
                    self.state.next_area();
                }
                KeyCode::Char(c) => {
                    self.state.area_search_query.push(c);
                    let query = self.state.area_search_query.clone();
                    self.state.filter_areas(&query);
                }
                KeyCode::Backspace => {
                    self.state.area_search_query.pop();
                    let query = self.state.area_search_query.clone();
                    self.state.filter_areas(&query);
                }
                _ => {}
            }
            return Ok(true);
        }

        // 处理主菜单
        match key {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(false),
            KeyCode::Up => self.state.previous_menu(),
            KeyCode::Down => self.state.next_menu(),
            KeyCode::Enter => {
                if let Some(menu_item) = self.state.menu_items.get(self.state.selected_menu) {
                    match menu_item.as_str() {
                        "开始直播" => self.handle_start_live().await?,
                        "修改标题" => self.handle_modify_title().await?,
                        "修改分区" => self.handle_modify_area().await?,
                        "结束直播" => self.handle_stop_live().await?,
                        "帮助" => self.handle_help().await?,
                        "退出程序" => return Ok(false),
                        _ => {}
                    }
                }
            }

            _ => {}
        }

        Ok(true)
    }

    async fn handle_start_live(&mut self) -> Result<()> {
        if self.state.is_live {
            self.state.show_message("已经在直播中".to_string(), MessageType::Warning);
            return Ok(());
        }

        if let Some(live) = &self.live {
            self.state.show_loading("正在开始直播...".to_string());
            
            // 获取当前分区ID
            let (area_id, _) = live.get_current_area().await.unwrap_or((0, "未知".to_string()));
            
            match live.start_live(area_id).await {
                Ok(stream_data) => {
                    let (rtmp_url, stream_key) = live.parse_stream_info(&stream_data);
                    
                    // 更新状态
                    self.state.set_live_status(true);
                    self.state.set_stream_info(rtmp_url.clone(), stream_key.clone());
                    
                    // 保存推流信息到配置文件
                    if let Err(e) = self.config.save_stream_info(rtmp_url.clone(), stream_key.clone()) {
                        eprintln!("保存推流信息失败: {}", e);
                    }
                    
                    self.state.hide_loading();
                    
                    let message = format!("直播已开启！\n推流地址: {}\n推流码: {}", rtmp_url, stream_key);
                    self.state.show_message(message, MessageType::Success);
                }
                Err(e) => {
                    self.state.hide_loading();
                    self.state.show_message(format!("开启直播失败: {}", e), MessageType::Error);
                }
            }
        }
        Ok(())
    }

    async fn handle_modify_title(&mut self) -> Result<()> {
        if self.live.is_some() {
            self.state.title_input = self.state.current_title.clone();
            self.state.show_title_input = true;
        }
        Ok(())
    }

    async fn handle_modify_area(&mut self) -> Result<()> {
        if let Some(live) = &self.live {
            if self.state.area_list.is_empty() {
                self.state.show_loading("正在加载分区列表...".to_string());
                
                match live.get_area_list().await {
                    Ok(areas) => {
                        self.state.area_list = areas;
                        self.state.filter_areas(""); // 显示所有分区
                        self.state.hide_loading();
                        self.state.show_area_search = true;
                    }
                    Err(e) => {
                        self.state.hide_loading();
                        self.state.show_message(format!("加载分区列表失败: {}", e), MessageType::Error);
                    }
                }
            } else {
                self.state.filter_areas(""); // 显示所有分区
                self.state.show_area_search = true;
            }
        }
        Ok(())
    }

    async fn handle_stop_live(&mut self) -> Result<()> {
        if !self.state.is_live {
            self.state.show_message("当前未在直播中".to_string(), MessageType::Warning);
            return Ok(());
        }

        if let Some(live) = &self.live {
            self.state.show_loading("正在结束直播...".to_string());
            
            match live.stop_live().await {
                Ok(_) => {
                    // 更新状态
                    self.state.set_live_status(false);
                    self.state.clear_stream_info();
                    
                    // 清除配置文件中的推流信息
                    if let Err(e) = self.config.clear_stream_info() {
                        eprintln!("清除推流信息失败: {}", e);
                    }
                    
                    self.state.hide_loading();
                    
                    self.state.show_message("直播已结束".to_string(), MessageType::Success);
                }
                Err(e) => {
                    self.state.hide_loading();
                    self.state.show_message(format!("结束直播失败: {}", e), MessageType::Error);
                }
            }
        }
        Ok(())
    }

    async fn handle_help(&mut self) -> Result<()> {
        self.state.show_help();
        Ok(())
    }
    


    async fn set_title(&mut self) -> Result<()> {
        if let Some(live) = &self.live {
            self.state.show_loading("正在设置标题...".to_string());
            
            match live.set_title(&self.state.title_input).await {
                Ok(_) => {
                    self.state.current_title = self.state.title_input.clone();
                    self.state.hide_loading();
                    self.state.show_message("标题设置成功".to_string(), MessageType::Success);
                }
                Err(e) => {
                    self.state.hide_loading();
                    self.state.show_message(format!("设置标题失败: {}", e), MessageType::Error);
                }
            }
        }
        Ok(())
    }

    async fn set_area(&mut self, area_id: u32) -> Result<()> {
        if let Some(live) = &self.live {
            self.state.show_loading("正在设置分区...".to_string());
            
            match live.set_area(area_id).await {
                Ok(_) => {
                    self.initialize_live_info().await;
                    self.state.hide_loading();
                    self.state.show_message("分区设置成功".to_string(), MessageType::Success);
                }
                Err(e) => {
                    self.state.hide_loading();
                    self.state.show_message(format!("设置分区失败: {}", e), MessageType::Error);
                }
            }
        }
        Ok(())
    }

    fn ui(&mut self, f: &mut Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(0),
                Constraint::Length(3),
            ])
            .split(f.area());

        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(25),  // 菜单宽度
                Constraint::Percentage(75),  // 信息显示宽度
            ])
            .split(chunks[0]);

        self.render_menu(f, main_chunks[0]);
        self.render_info(f, main_chunks[1]);
        
        self.render_status(f, chunks[1]);

        if self.state.show_title_input {
            self.render_title_input(f);
        }

        if self.state.show_area_search {
            self.render_area_search(f);
        }

        if self.state.show_message {
            self.render_message(f);
        }

        if self.state.show_loading {
            self.render_loading(f);
        }

        if self.state.show_help {
            self.render_help(f);
        }
    }

    fn render_menu(&mut self, f: &mut Frame, area: Rect) {
        let items: Vec<ListItem> = self.state.menu_items
            .iter()
            .enumerate()
            .map(|(i, item)| {
                let style = if i == self.state.selected_menu {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                
                ListItem::new(format!("  {}", item)).style(style)
            })
            .collect();

        let list = List::new(items)
            .block(Block::default()
                .title("📋 菜单")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
            .highlight_symbol("►");

        f.render_stateful_widget(list, area, &mut self.state.menu_state);
    }

    fn render_info(&self, f: &mut Frame, area: Rect) {
        // 直播信息
        let live_status = if self.state.is_live { "🔴 直播中" } else { "⚫ 未开播" };
        let mut info_text = vec![
            Line::from(vec![
                Span::styled("状态: ", Style::default().fg(Color::Gray)),
                Span::styled(live_status, if self.state.is_live { 
                    Style::default().fg(Color::Red) 
                } else { 
                    Style::default().fg(Color::Gray) 
                }),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("标题: ", Style::default().fg(Color::Gray)),
                Span::styled(&self.state.current_title, Style::default().fg(Color::Green)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("分区: ", Style::default().fg(Color::Gray)),
                Span::styled(&self.state.current_area, Style::default().fg(Color::Green)),
            ]),
        ];

        // 如果正在直播，显示推流信息
        if self.state.is_live && !self.state.stream_server.is_empty() {
            info_text.push(Line::from(""));
            info_text.push(Line::from(vec![
                Span::styled("推流服务器: ", Style::default().fg(Color::Gray)),
                Span::styled(&self.state.stream_server, Style::default().fg(Color::Cyan)),
            ]));
            info_text.push(Line::from(""));
            info_text.push(Line::from(vec![
                Span::styled("推流码: ", Style::default().fg(Color::Gray)),
                Span::styled(&self.state.stream_key, Style::default().fg(Color::Cyan)),
            ]));
        }

        let info_widget = Paragraph::new(info_text)
            .block(Block::default()
                .title("📊 直播信息")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Green)))
            .wrap(Wrap { trim: true });

        f.render_widget(info_widget, area);
    }

    fn render_status(&self, f: &mut Frame, area: Rect) {
        let status_text = format!("房间号: {} | 用户ID: {}", 
            self.live.as_ref().map(|l| l.get_room_id().to_string()).unwrap_or_else(|| "未知".to_string()),
            self.user_info.as_ref().map(|u| u.uid.to_string()).unwrap_or_else(|| "未知".to_string())
        );

        let status = Paragraph::new(status_text)
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));

        f.render_widget(status, area);
    }

    fn render_title_input(&self, f: &mut Frame) {
        let area = centered_rect(70, 30, f.area());
        
        f.render_widget(Clear, area);
        
        let input_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(5),  // 增加输入框高度
                Constraint::Length(3),
            ])
            .split(area);

        // 标题
        let title_widget = Paragraph::new("修改直播标题")
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title_widget, input_chunks[0]);

        // 输入框 - 添加光标显示
        let input_text = format!("{}█", self.state.title_input);  // 添加方块光标
        let input_widget = Paragraph::new(input_text)
            .style(Style::default().fg(Color::White).bg(Color::Black))
            .block(Block::default()
                .borders(Borders::ALL)
                .title("输入新标题")
                .border_style(Style::default().fg(Color::Cyan)))
            .wrap(Wrap { trim: false });
        f.render_widget(input_widget, input_chunks[1]);

        // 提示
        let hint = Paragraph::new("Enter: 确认 | Esc: 取消")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(hint, input_chunks[2]);
    }

    fn render_area_search(&mut self, f: &mut Frame) {
        let area = centered_rect(80, 70, f.area());
        
        f.render_widget(Clear, area);
        
        let search_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);

        // 标题
        let title_widget = Paragraph::new("修改直播分区")
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title_widget, search_chunks[0]);

        // 搜索框
        let search_query = self.state.area_search_query.clone();
        let search_widget = Paragraph::new(search_query.as_str())
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL).title("搜索分区 (输入关键词)"));
        f.render_widget(search_widget, search_chunks[1]);

        // 分区列表
        let filtered_areas = self.state.filtered_areas.clone();
        let items: Vec<ListItem> = filtered_areas
            .iter()
            .map(|area| {
                ListItem::new(format!("  {} - {}", area.parent_name, area.name))
            })
            .collect();

        let list = List::new(items)
            .block(Block::default()
                .title("分区列表")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Blue)))
            .highlight_style(Style::default().bg(Color::Blue).fg(Color::White))
            .highlight_symbol("►");

        f.render_stateful_widget(list, search_chunks[2], &mut self.state.area_state);

        // 提示
        let hint = Paragraph::new("↑/↓: 选择 | Enter: 确认 | Esc: 取消")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(hint, search_chunks[3]);
    }

    fn render_message(&self, f: &mut Frame) {
        let area = centered_rect(60, 30, f.area());
        
        f.render_widget(Clear, area);
        
        let message_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);

        // 消息类型和标题
        let (title, style) = match self.state.message_type {
            MessageType::Info => ("ℹ️ 信息", Style::default().fg(Color::Blue)),
            MessageType::Success => ("✅ 成功", Style::default().fg(Color::Green)),
            MessageType::Warning => ("⚠️ 警告", Style::default().fg(Color::Yellow)),
            MessageType::Error => ("❌ 错误", Style::default().fg(Color::Red)),
        };

        let title_widget = Paragraph::new(title)
            .style(style.add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title_widget, message_chunks[0]);

        // 消息内容
        let content_widget = Paragraph::new(self.state.message.as_str())
            .style(Style::default().fg(Color::White))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(content_widget, message_chunks[1]);

        // 提示
        let hint = Paragraph::new("按任意键关闭")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(hint, message_chunks[2]);
    }

    fn render_loading(&self, f: &mut Frame) {
        let area = centered_rect(50, 20, f.area());
        
        f.render_widget(Clear, area);
        
        let loading_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(1),
            ])
            .split(area);

        // 标题
        let title_widget = Paragraph::new("⏳ 正在处理...")
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title_widget, loading_chunks[0]);

        // 进度条
        let progress = Gauge::default()
            .block(Block::default().borders(Borders::ALL))
            .gauge_style(Style::default().fg(Color::Yellow))
            .percent(50)
            .label(self.state.loading_message.as_str());
        f.render_widget(progress, loading_chunks[1]);

        // 提示
        let hint = Paragraph::new("请稍候...")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(hint, loading_chunks[2]);
    }

    fn render_help(&self, f: &mut Frame) {
        let area = centered_rect(70, 80, f.area());
        
        f.render_widget(Clear, area);
        
        let help_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Min(0),
                Constraint::Length(1),
            ])
            .split(area);

        // 标题
        let title_widget = Paragraph::new("❓ 帮助信息")
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title_widget, help_chunks[0]);

        // 帮助内容
        let help_text = vec![
            Line::from("🎯 基本操作:"),
            Line::from(""),
            Line::from("  ↑/↓  - 选择菜单项"),
            Line::from("  Enter - 确认选择"),
            Line::from("  Esc/q - 退出程序"),
            Line::from(""),
            Line::from("📋 菜单说明:"),
            Line::from(""),
            Line::from("  • 开始直播 - 开启直播，获取推流码"),
            Line::from("  • 修改标题 - 修改当前直播间标题"),
            Line::from("  • 修改分区 - 修改当前直播间分区"),
            Line::from("  • 结束直播 - 结束当前直播"),
            Line::from("  • 帮助 - 显示此帮助信息"),
            Line::from("  • 退出程序 - 关闭应用程序"),
        ];

        let content_widget = Paragraph::new(help_text)
            .style(Style::default().fg(Color::White))
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true });
        f.render_widget(content_widget, help_chunks[1]);

        // 提示
        let hint = Paragraph::new("按 Enter/Esc/q 关闭帮助")
            .style(Style::default().fg(Color::Gray))
            .alignment(Alignment::Center);
        f.render_widget(hint, help_chunks[2]);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
} 