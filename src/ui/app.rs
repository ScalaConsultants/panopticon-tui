use std::iter::Iterator;

use crate::ui::formatter;
use crate::ui::model::UIFiber;
use crate::zio::zmx_client;

pub struct TabsState<'a> {
    pub titles: Vec<&'a str>,
    pub index: usize,
}

impl<'a> TabsState<'a> {
    pub fn new(titles: Vec<&'a str>) -> TabsState {
        TabsState { titles, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }

    pub fn previous(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.index = self.titles.len() - 1;
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
    pub zio_zmx_addr: String,
    pub fibers: ListState<String>,
    pub selected_fiber_dump: (String, u16),
    pub fiber_dump_all: Vec<String>,
    pub scroll: u16,
    pub barchart: Vec<(&'a str, u64)>,

}

impl<'a> App<'a> {
    fn dump_len(s: &String) -> u16 {
        s.lines().collect::<Vec<&str>>().len() as u16
    }

    pub fn new(title: &'a str, zio_zmx_addr: String, fibers: Vec<String>, fiber_dump_all: Vec<String>) -> App<'a> {
        App {
            title,
            should_quit: false,
            tabs: TabsState::new(vec!["ZIO"]),
            zio_zmx_addr: zio_zmx_addr,
            fibers: ListState::new(fibers),
            selected_fiber_dump: ("".to_string(), 1),
            fiber_dump_all: fiber_dump_all,
            scroll: 0,
            barchart: EVENTS.to_vec(),
        }
    }

    pub fn on_up(&mut self) {
        let tab = self.tabs.index;
        if tab == 0 {
            self.fibers.select_previous();
            let n = self.fibers.selected;
            let dump = self.fiber_dump_all[n].to_owned();
            self.selected_fiber_dump = (dump.clone(), App::dump_len(&dump));
        }
    }

    pub fn on_down(&mut self) {
        let tab = self.tabs.index;
        if tab == 0 {
            self.fibers.select_next();
            let n = self.fibers.selected;
            let dump = self.fiber_dump_all[n].to_owned();
            self.selected_fiber_dump = (dump.clone(), App::dump_len(&dump));
        }
    }

    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }

    pub fn on_enter(&mut self) {
        let tab = self.tabs.index;
        if tab == 0 {
            let addr = self.zio_zmx_addr.to_owned();
            let fd = zmx_client::get_dump(&addr).expect(format!("Couldn't get fiber dump from {}", addr.to_owned()).as_str());

            let list: Vec<UIFiber> = formatter::printable_tree(fd);
            let mut fib_labels = list.iter().map(|f| f.label.clone()).collect();
            let mut fib_dumps = list.iter().map(|f| f.dump.to_owned()).collect::<Vec<String>>();

            self.fibers.items.clear();
            self.fibers.items.append(&mut fib_labels);
            self.fibers.selected = 0;
            let dump = fib_dumps[0].to_owned();
            self.selected_fiber_dump = (dump.clone(), App::dump_len(&dump));
            self.fiber_dump_all.clear();
            self.fiber_dump_all.append(&mut fib_dumps);
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
        let tab = self.tabs.index;
        if tab == 0 {
            if self.scroll > 0 {
                self.scroll -= 1;
            }
        }
    }

    pub fn on_page_down(&mut self) {
        let tab = self.tabs.index;
        if tab == 0 {
            if self.scroll < self.selected_fiber_dump.1 {
                self.scroll += 1;
            }
        }
    }

    pub fn on_tick(&mut self) {
        let event = self.barchart.pop().unwrap();
        self.barchart.insert(0, event);
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
