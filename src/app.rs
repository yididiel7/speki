use rusqlite::Connection;
use tui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Tabs},
    Frame,
};

use crate::{
    logic::{
        add_card::{DepState, NewCard},
        incread::MainInc,
        review::ReviewList,
    },
    utils::widgets::textinput::Field,
    MyType, SpekiPaths,
};

pub struct TabsState {
    pub tabs: Vec<Box<dyn Tab>>,
    pub index: usize,
}

impl TabsState {
    pub fn new(
        conn: &Arc<Mutex<Connection>>,
        audio_handle: &rodio::OutputStreamHandle,
    ) -> TabsState {
        let mut tabs: Vec<Box<dyn Tab>> = vec![];

        let revlist = ReviewList::new(conn, audio_handle);
        let addcards = NewCard::new(conn, DepState::None);
        let incread = MainInc::new(conn);
        let importer = Importer::new(conn);

        tabs.push(Box::new(revlist));
        tabs.push(Box::new(addcards));
        tabs.push(Box::new(incread));
        tabs.push(Box::new(importer));

        TabsState { tabs, index: 0 }
    }
    pub fn next(&mut self) {
        if self.index < self.tabs.len() - 1 {
            self.index += 1;
        }
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        }
    }

    fn swap_left(&mut self) {
        if self.index == 0 {
            return;
        }
        self.tabs.swap(self.index, self.index - 1);
        self.previous();
    }
    fn swap_right(&mut self) {
        if self.index == self.tabs.len() - 1 {
            return;
        }
        self.tabs.swap(self.index, self.index + 1);
        self.next();
    }

    fn keyhandler(
        &mut self,
        conn: &Arc<Mutex<Connection>>,
        key: MyKey,
        audio: &rodio::OutputStreamHandle,
        paths: &SpekiPaths,
    ) {
        self.tabs[self.index].keyhandler(conn, key, audio, paths);
    }
}

use crate::logic::import::Importer;
use std::sync::{Arc, Mutex};

pub struct App {
    pub tabs: TabsState,
    pub should_quit: bool,
    pub display_help: bool,
    pub conn: Arc<Mutex<Connection>>,
    pub audio: rodio::OutputStream,
    pub audio_handle: rodio::OutputStreamHandle,
    pub paths: SpekiPaths,
}

impl App {
    pub fn new(display_help: bool, paths: SpekiPaths) -> App {
        let conn = Arc::new(Mutex::new(
            Connection::open(&paths.database).expect("Failed to connect to database."),
        ));
        let (audio, audio_handle) = rodio::OutputStream::try_default().unwrap();

        let tabs = TabsState::new(&conn, &audio_handle);
        App {
            tabs,
            display_help,
            should_quit: false,
            conn,
            audio,
            audio_handle,
            paths,
        }
    }

    pub fn keyhandler(&mut self, key: MyKey) {
        match key {
            MyKey::Tab => self.tabs.next(),
            MyKey::BackTab => self.tabs.previous(),
            MyKey::SwapTab => self.tabs.swap_right(),
            MyKey::BackSwapTab => self.tabs.swap_left(),
            MyKey::F(1) => self.display_help = !self.display_help,
            MyKey::Alt('q') => self.should_quit = true,
            _ => {}
        };
        self.tabs
            .keyhandler(&self.conn, key, &self.audio_handle, &self.paths);
    }

    pub fn draw(&mut self, f: &mut Frame<MyType>) {
        let mut area = f.size();
        if self.display_help {
            area = self.render_help(f, area);
        }

        let chunks = Layout::default()
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);

        let block = Block::default().style(Style::default().bg(Color::Rgb(20, 31, 31)));
        f.render_widget(block, f.size());

        let titles = self
            .tabs
            .tabs
            .iter()
            .map(|t| {
                Spans::from(Span::styled(
                    t.get_title(),
                    Style::default().fg(Color::Green),
                ))
            })
            .collect();

        let tabs = Tabs::new(titles)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(Style::default().fg(Color::Yellow))
            .select(self.tabs.index);

        f.render_widget(tabs, chunks[0]);
        self.tabs.tabs[self.tabs.index].render(f, chunks[1], &self.conn, &self.paths);
    }

    fn render_help(&self, f: &mut Frame<MyType>, area: Rect) -> Rect {
        let mut field = Field::new();
        let help_msg = self.tabs.tabs[self.tabs.index].get_manual();
        field.replace_text(help_msg);
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Ratio(2, 3), Constraint::Ratio(1, 3)].as_ref())
            .split(area);
        field.render(f, chunks[1], false);
        chunks[0]
    }
}

use crate::MyKey;

pub trait Tab {
    fn keyhandler(
        &mut self,
        conn: &Arc<Mutex<Connection>>,
        key: MyKey,
        audio: &rodio::OutputStreamHandle,
        paths: &SpekiPaths,
    );
    fn render(
        &mut self,
        f: &mut Frame<MyType>,
        area: Rect,
        conn: &Arc<Mutex<Connection>>,
        paths: &SpekiPaths,
    );
    fn get_title(&self) -> String;
    fn get_manual(&self) -> String {
        String::new()
    }
}
