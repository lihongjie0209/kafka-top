use crate::cli::Cli;
use crate::event::{Event, EventStream};
use crate::state::*;
use crate::ui;
use anyhow::Result;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::collections::HashMap;
use std::io::{stdout, Write};
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard = 0,
    Topics = 1,
    ConsumerGroups = 2,
}

impl Tab {
    pub fn next(&self) -> Self {
        match self {
            Tab::Dashboard => Tab::Topics,
            Tab::Topics => Tab::ConsumerGroups,
            Tab::ConsumerGroups => Tab::Dashboard,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Tab::Dashboard => Tab::ConsumerGroups,
            Tab::Topics => Tab::Dashboard,
            Tab::ConsumerGroups => Tab::Topics,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    Normal,
    TopicDetail,
    GroupDetail,
}

pub struct App {
    pub cli: Cli,
    pub client: Option<crate::kafka::KafkaClient>,

    // Navigation
    pub should_quit: bool,
    pub current_tab: Tab,
    pub current_view: View,
    pub selected_topic: Option<String>,
    pub selected_group: Option<String>,

    // Data states
    pub dashboard_state: LoadState<crate::state::DashboardData>,
    pub topics_state: LoadState<crate::state::TopicsData>,
    pub topic_detail_state: LoadState<crate::state::TopicDetailData>,
    pub groups_state: LoadState<crate::state::GroupsData>,
    pub group_detail_state: LoadState<crate::state::GroupDetailData>,

    // List selection
    pub topic_list_index: usize,
    pub group_list_index: usize,

    // Rate tracking: (topic, partition) -> (offset, timestamp)
    pub prev_offsets: HashMap<(String, i32), (i64, Instant)>,
    pub prev_topic_offsets: HashMap<(String, i32), (i64, Instant)>,
}

impl App {
    pub fn new(cli: Cli) -> Self {
        Self {
            cli,
            client: None,
            should_quit: false,
            current_tab: Tab::Dashboard,
            current_view: View::Normal,
            selected_topic: None,
            selected_group: None,
            dashboard_state: LoadState::Idle,
            topics_state: LoadState::Idle,
            topic_detail_state: LoadState::Idle,
            groups_state: LoadState::Idle,
            group_detail_state: LoadState::Idle,
            topic_list_index: 0,
            group_list_index: 0,
            prev_offsets: HashMap::new(),
            prev_topic_offsets: HashMap::new(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // Initialize Kafka client
        match crate::kafka::KafkaClient::new(&self.cli) {
            Ok(client) => {
                self.client = Some(client);
            }
            Err(e) => {
                eprintln!("Failed to create Kafka client: {}", e);
                std::process::exit(1);
            }
        }

        // Terminal setup
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        terminal.clear()?;

        // Event stream
        let mut events = EventStream::new(self.cli.refresh_interval);

        // Initial fetch
        self.fetch_current_view().await;

        // Main loop
        while !self.should_quit {
            terminal.draw(|f| {
                ui::render(f, &self);
            })?;

            match events.next().await {
                Some(Event::Key(key)) => {
                    if self.handle_key(key) {
                        self.fetch_current_view().await;
                    }
                }
                Some(Event::Tick) => {
                    self.force_fetch_current_view().await;
                }
                None => break,
            }
        }

        // Restore terminal
        terminal.clear()?;
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        terminal.backend_mut().flush()?;

        Ok(())
    }

    /// Returns true if the current view needs to be refreshed
    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> bool {
        use crossterm::event::{KeyCode, KeyEventKind, KeyModifiers};

        match key.code {
            KeyCode::Char('q') | KeyCode::Char('Q') => {
                self.should_quit = true;
                false
            }
            KeyCode::Char('r') | KeyCode::Char('R') => {
                self.reset_current_view();
                true
            }
            KeyCode::Tab if key.kind == KeyEventKind::Press => {
                if self.current_view == View::Normal {
                    self.current_tab = self.current_tab.next();
                    self.reset_current_view();
                    true
                } else {
                    false
                }
            }
            KeyCode::BackTab if key.kind == KeyEventKind::Press => {
                if self.current_view == View::Normal {
                    self.current_tab = self.current_tab.prev();
                    self.reset_current_view();
                    true
                } else {
                    false
                }
            }
            KeyCode::Enter => {
                self.drill_down();
                self.needs_refresh_on_view_change()
            }
            KeyCode::Esc => {
                self.go_back();
                self.needs_refresh_on_view_change()
            }
            KeyCode::Up | KeyCode::Char('k') if key.modifiers == KeyModifiers::NONE => {
                self.scroll_up();
                false
            }
            KeyCode::Down | KeyCode::Char('j') if key.modifiers == KeyModifiers::NONE => {
                self.scroll_down();
                false
            }
            _ => false,
        }
    }

    fn needs_refresh_on_view_change(&self) -> bool {
        self.current_view != View::Normal
    }

    fn drill_down(&mut self) {
        if self.current_view != View::Normal {
            return;
        }
        match self.current_tab {
            Tab::Topics => {
                if let LoadState::Loaded(ref data) = self.topics_state {
                    if !data.topics.is_empty() {
                        let idx = self.topic_list_index.min(data.topics.len() - 1);
                        self.selected_topic = Some(data.topics[idx].name.clone());
                        self.current_view = View::TopicDetail;
                        self.topic_detail_state = LoadState::Idle;
                    }
                }
            }
            Tab::ConsumerGroups => {
                if let LoadState::Loaded(ref data) = self.groups_state {
                    if !data.groups.is_empty() {
                        let idx = self.group_list_index.min(data.groups.len() - 1);
                        self.selected_group = Some(data.groups[idx].group_id.clone());
                        self.current_view = View::GroupDetail;
                        self.group_detail_state = LoadState::Idle;
                    }
                }
            }
            _ => {}
        }
    }

    fn go_back(&mut self) {
        match self.current_view {
            View::TopicDetail | View::GroupDetail => {
                self.current_view = View::Normal;
                self.selected_topic = None;
                self.selected_group = None;
            }
            _ => {}
        }
    }

    fn scroll_up(&mut self) {
        match self.current_tab {
            Tab::Topics => {
                if self.topic_list_index > 0 {
                    self.topic_list_index -= 1;
                }
            }
            Tab::ConsumerGroups => {
                if self.group_list_index > 0 {
                    self.group_list_index -= 1;
                }
            }
            _ => {}
        }
    }

    fn scroll_down(&mut self) {
        match self.current_tab {
            Tab::Topics => {
                if let LoadState::Loaded(ref data) = self.topics_state {
                    if !data.topics.is_empty() && self.topic_list_index + 1 < data.topics.len() {
                        self.topic_list_index += 1;
                    }
                }
            }
            Tab::ConsumerGroups => {
                if let LoadState::Loaded(ref data) = self.groups_state {
                    if !data.groups.is_empty() && self.group_list_index + 1 < data.groups.len() {
                        self.group_list_index += 1;
                    }
                }
            }
            _ => {}
        }
    }

    fn reset_current_view(&mut self) {
        match self.current_view {
            View::Normal => match self.current_tab {
                Tab::Dashboard => self.dashboard_state = LoadState::Idle,
                Tab::Topics => self.topics_state = LoadState::Idle,
                Tab::ConsumerGroups => self.groups_state = LoadState::Idle,
            },
            View::TopicDetail => {
                self.topic_detail_state = LoadState::Idle;
            }
            View::GroupDetail => {
                self.group_detail_state = LoadState::Idle;
            }
        }
    }

    async fn fetch_current_view(&mut self) {
        self.fetch_view_data(false).await;
    }

    async fn force_fetch_current_view(&mut self) {
        self.fetch_view_data(true).await;
    }

    async fn fetch_view_data(&mut self, force: bool) {
        let client = match &self.client {
            Some(c) => c.clone(),
            None => return,
        };

        macro_rules! fetch_impl {
            ($state:expr, $fut:expr $(, $update:expr)?) => {{
                if !force && !$state.is_idle() {
                    return;
                }
                let is_refresh = matches!($state, LoadState::Loaded(_));
                if !is_refresh {
                    *$state = LoadState::Loading;
                }
                #[allow(unused_mut)]
                match $fut.await {
                    Ok(mut data) => {
                        $( $update(&mut data); )?
                        *$state = LoadState::Loaded(data);
                    }
                    Err(e) => {
                        if !is_refresh {
                            *$state = LoadState::Error(e.to_string());
                        }
                    }
                }
            }};
        }

        match self.current_view {
            View::Normal => match self.current_tab {
                Tab::Dashboard => {
                    fetch_impl!(&mut self.dashboard_state, client.get_dashboard_data());
                }
                Tab::Topics => {
                    let filter = self.cli.topic_filter.clone();
                    fetch_impl!(&mut self.topics_state, client.get_topics(filter.as_deref()), |data: &mut TopicsData| {
                        let len = data.topics.len();
                        self.topic_list_index = self.topic_list_index.min(len.saturating_sub(1));
                    });
                }
                Tab::ConsumerGroups => {
                    let filter = self.cli.group_filter.clone();
                    fetch_impl!(&mut self.groups_state, client.list_consumer_groups(filter.as_deref()), |data: &mut GroupsData| {
                        let len = data.groups.len();
                        self.group_list_index = self.group_list_index.min(len.saturating_sub(1));
                    });
                }
            },
            View::TopicDetail => {
                let topic = self.selected_topic.clone();
                fetch_impl!(&mut self.topic_detail_state, async {
                    match topic {
                        Some(t) => client.get_topic_detail(&t).await,
                        None => Err(anyhow::anyhow!("No topic selected")),
                    }
                }, |data: &mut TopicDetailData| {
                    let now = Instant::now();
                    let mut new_offsets = HashMap::new();
                    for p in &mut data.partitions {
                        let key = (data.topic.clone(), p.id);
                        if let Some(&(prev_offset, prev_time)) = self.prev_topic_offsets.get(&key) {
                            let elapsed = now.saturating_duration_since(prev_time).as_secs_f64();
                            if elapsed > 0.0 && p.log_end_offset >= prev_offset {
                                p.producer_rate = (p.log_end_offset - prev_offset) as f64 / elapsed;
                            }
                        }
                        new_offsets.insert(key, (p.log_end_offset, now));
                    }
                    self.prev_topic_offsets = new_offsets;
                });
            }
            View::GroupDetail => {
                let group = self.selected_group.clone();
                fetch_impl!(&mut self.group_detail_state, async {
                    match group {
                        Some(g) => client.get_group_detail(&g).await,
                        None => Err(anyhow::anyhow!("No group selected")),
                    }
                }, |data: &mut GroupDetailData| {
                    let now = Instant::now();
                    let mut new_offsets = HashMap::new();
                    for a in &mut data.assignments {
                        let key = (a.topic.clone(), a.partition_id);
                        if let Some(&(prev_offset, prev_time)) = self.prev_offsets.get(&key) {
                            let elapsed = now.saturating_duration_since(prev_time).as_secs_f64();
                            if elapsed > 0.0 && a.current_offset >= prev_offset {
                                a.consumer_rate = (a.current_offset - prev_offset) as f64 / elapsed;
                            }
                        }
                        new_offsets.insert(key, (a.current_offset, now));
                    }
                    self.prev_offsets = new_offsets;
                });
            }
        }
    }
}
