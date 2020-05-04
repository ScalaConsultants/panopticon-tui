use std::iter::Iterator;

use crate::ui::formatter;
use crate::ui::model::UIFiber;
use crate::jmx_client::model::{JMXConnectionSettings, SlickMetrics, SlickConfig, HikariMetrics};
use crate::jmx_client::client::JMXClient;
use crate::zio::zmx_client;
use jmx::MBeanClient;
use std::collections::VecDeque;

pub enum TabKind {
    ZMX,
    Slick,
}

pub struct Tab<'a> {
    pub kind: TabKind,
    pub title: &'a str,
}

pub struct TabsState<'a> {
    pub tabs: Vec<Tab<'a>>,
    pub index: usize,
}

impl<'a> TabsState<'a> {
    pub fn new(tabs: Vec<Tab<'a>>) -> TabsState {
        TabsState { tabs, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.tabs.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.tabs.len() - 1;
        }
    }

    pub fn current(&self) -> &Tab<'a> {
        &self.tabs[self.index]
    }

    pub fn titles(&self) -> Vec<&'a str> {
        self.tabs.iter().map(|x| x.title).collect()
    }
}

pub struct ZMXTab<'a> {
    pub zio_zmx_addr: String,
    pub fibers: ListState<String>,
    pub selected_fiber_dump: (String, u16),
    pub fiber_dump_all: Vec<String>,
    pub scroll: u16,
    pub barchart: Vec<(&'a str, u64)>,
}

impl<'a> ZMXTab<'a> {
    fn new(zio_zmx_addr: String) -> ZMXTab<'a> {
        ZMXTab {
            zio_zmx_addr,
            fibers: ListState::new(vec![]),
            selected_fiber_dump: ("".to_string(), 1),
            fiber_dump_all: vec![],
            scroll: 0,
            barchart: EVENTS.to_vec(),
        }
    }

    fn select_prev_fiber(&mut self) {
        self.fibers.select_previous();
        self.on_fiber_change()
    }

    fn select_next_fiber(&mut self) {
        self.fibers.select_next();
        self.on_fiber_change()
    }

    fn on_fiber_change(&mut self) {
        let n = self.fibers.selected;
        self.selected_fiber_dump = ZMXTab::prepare_dump(self.fiber_dump_all[n].clone());
        self.scroll = 0;
    }

    fn dump_fibers(&mut self) {
        let addr = self.zio_zmx_addr.to_owned();
        let fd = zmx_client::get_dump(&addr).expect(format!("Couldn't get fiber dump from {}", addr.to_owned()).as_str());

        let list: Vec<UIFiber> = formatter::printable_tree(fd);
        let mut fib_labels = list.iter().map(|f| f.label.clone()).collect();
        let mut fib_dumps = list.iter().map(|f| f.dump.to_owned()).collect::<Vec<String>>();

        self.fibers.items.clear();
        self.fibers.items.append(&mut fib_labels);
        self.fibers.selected = 0;
        self.selected_fiber_dump = ZMXTab::prepare_dump(fib_dumps[0].clone());
        self.fiber_dump_all.clear();
        self.fiber_dump_all.append(&mut fib_dumps);
    }

    fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    fn scroll_down(&mut self) {
        if self.scroll < self.selected_fiber_dump.1 {
            self.scroll += 1;
        }
    }

    fn tick(&mut self) {
        let event = self.barchart.pop().unwrap();
        self.barchart.insert(0, event);
    }

    fn prepare_dump(s: String) -> (String, u16) {
        (s.clone(), s.lines().collect::<Vec<&str>>().len() as u16)
    }
}

pub struct SlickTab {
    pub jmx_connection_settings: JMXConnectionSettings,
    pub jmx: JMXClient,
    pub has_hikari: bool,
    pub slick_error: Option<String>,
    pub slick_metrics: VecDeque<SlickMetrics>,
    pub slick_config: SlickConfig,
    pub hikari_metrics: VecDeque<HikariMetrics>,
}

impl SlickTab {
    pub const MAX_SLICK_MEASURES: usize = 25;
    pub const MAX_HIKARI_MEASURES: usize = 100;

    fn append_slick_metrics(&mut self, m: SlickMetrics) {
        if self.slick_metrics.len() > SlickTab::MAX_SLICK_MEASURES {
            self.slick_metrics.pop_front();
        }
        self.slick_metrics.push_back(m);
    }

    fn append_hikari_metrics(&mut self, m: HikariMetrics) {
        if self.hikari_metrics.len() > SlickTab::MAX_HIKARI_MEASURES {
            self.hikari_metrics.pop_front();
        }
        self.hikari_metrics.push_back(m);
    }

    fn initialize(&mut self) {
        match self.jmx.get_slick_config() {
            Ok(config) => {
                self.slick_error = None;
                self.slick_config = config;
                let dyn_data = self.jmx.get_slick_metrics().unwrap();
                self.slick_metrics = VecDeque::from(vec![SlickMetrics::ZERO; SlickTab::MAX_SLICK_MEASURES]);
                self.append_slick_metrics(dyn_data);
            }
            Err(_) =>
                self.slick_error = Some("No slick jmx metrics found. Are you sure you have registerMbeans=true in your slick config?".to_owned()),
        }
        self.has_hikari = self.jmx.get_hikari_metrics().is_ok();
    }

    fn tick(&mut self) {
        if self.slick_error.is_none() {
            self.append_slick_metrics(self.jmx.get_slick_metrics().unwrap());
        }
        if self.has_hikari {
            self.append_hikari_metrics(self.jmx.get_hikari_metrics().unwrap());
        }
    }
}

pub struct ListState<I> {
    pub items: Vec<I>,
    pub selected: usize,
}

impl<I> ListState<I> {
    fn new(items: Vec<I>) -> ListState<I> {
        ListState { items, selected: 0 }
    }
    fn select_previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }
    fn select_next(&mut self) {
        if self.selected < self.items.len() - 1 {
            self.selected += 1
        }
    }
}

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub tabs: TabsState<'a>,
    pub zmx: ZMXTab<'a>,
    pub jmx_settings: Option<JMXConnectionSettings>,
    pub jmx_connection_error: Option<String>,
    pub slick: Option<SlickTab>,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, zio_zmx_addr: String, jmx: Option<JMXConnectionSettings>) -> App<'a> {
        let mut tabs: Vec<Tab> = vec![Tab { kind: TabKind::ZMX, title: "ZMX" }];

        if let Some(_) = jmx {
            tabs.push(Tab { kind: TabKind::Slick, title: "Slick" })
        }

        App {
            title,
            should_quit: false,
            tabs: TabsState::new(tabs),
            zmx: ZMXTab::new(zio_zmx_addr),
            jmx_settings: jmx,
            jmx_connection_error: Some("Not connected yet".to_owned()),
            slick: None,
        }
    }

    pub fn connect_to_jmx(&mut self) {
        match &self.jmx_settings {
            None => { self.jmx_connection_error = Some("No jmx connection settings specified".to_owned()) }
            Some(conn) => {
                let url = jmx::MBeanAddress::service_url(format!(
                    "service:jmx:rmi://{}/jndi/rmi://{}/jmxrmi",
                    &conn.address, &conn.address
                ));

                match MBeanClient::connect(url) {
                    Err(e) => self.jmx_connection_error = Some(e.to_string()),
                    Ok(c) => {
                        let client = JMXClient { connection: c, db_pool_name: conn.db_pool_name.clone() };
                        self.jmx_connection_error = None;
                        let mut slick_tab: SlickTab = SlickTab {
                            jmx_connection_settings: conn.to_owned(),
                            jmx: client,
                            has_hikari: false,
                            slick_error: None,
                            slick_metrics: VecDeque::new(),
                            slick_config: SlickConfig { max_queue_size: 0, max_threads: 0 },
                            hikari_metrics: VecDeque::new(),
                        };
                        slick_tab.initialize();
                        self.slick = Some(slick_tab);
                    }
                }
            }
        }
    }

    pub fn on_up(&mut self) {
        match self.tabs.current().kind {
            TabKind::ZMX => self.zmx.select_prev_fiber(),
            TabKind::Slick => {}
        }
    }

    pub fn on_down(&mut self) {
        match self.tabs.current().kind {
            TabKind::ZMX => self.zmx.select_next_fiber(),
            TabKind::Slick => {}
        }
    }

    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }

    pub fn on_enter(&mut self) {
        match self.tabs.current().kind {
            TabKind::ZMX => self.zmx.dump_fibers(),
            TabKind::Slick => self.connect_to_jmx()
        }
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    pub fn on_page_up(&mut self) {
        match self.tabs.current().kind {
            TabKind::ZMX => self.zmx.scroll_up(),
            TabKind::Slick => {}
        }
    }

    pub fn on_page_down(&mut self) {
        match self.tabs.current().kind {
            TabKind::ZMX => self.zmx.scroll_down(),
            TabKind::Slick => {}
        }
    }

    pub fn on_tick(&mut self) {
        self.zmx.tick();
        if let Some(s) = &mut self.slick {
            s.tick();
        }
    }
}

const EVENTS: [(&'static str, u64); 24] = [
    ("B1", 9),
    ("B2", 12),
    ("B3", 5),
    ("B4", 8),
    ("B5", 2),
    ("B6", 4),
    ("B7", 5),
    ("B8", 9),
    ("B9", 14),
    ("B10", 15),
    ("B11", 1),
    ("B12", 0),
    ("B13", 4),
    ("B14", 6),
    ("B15", 4),
    ("B16", 6),
    ("B17", 4),
    ("B18", 7),
    ("B19", 13),
    ("B20", 8),
    ("B21", 11),
    ("B22", 9),
    ("B23", 3),
    ("B24", 5),
];
