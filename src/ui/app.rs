use std::iter::Iterator;

use crate::ui::formatter;
use crate::ui::model::UIFiber;
use crate::zio::zmx_client;

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
    pub jmx_addr: String
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
    pub slick: Option<SlickTab>,
}

impl<'a> App<'a> {
    pub fn new(title: &'a str, zio_zmx_addr: String) -> App<'a> {
        App {
            title,
            should_quit: false,
            tabs: TabsState::new(vec![
                Tab { kind: TabKind::ZMX, title: "ZMX" },
                Tab { kind: TabKind::Slick, title: "Slick" },
            ]),
            zmx: ZMXTab::new(zio_zmx_addr),
            slick: Some(SlickTab { jmx_addr: "localhost:9019".to_string() }),
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
            TabKind::Slick => {}
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
        match self.tabs.current().kind {
            TabKind::ZMX => self.zmx.tick(),
            TabKind::Slick => {}
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
