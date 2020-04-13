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
    pub zookeeper_nodes: ListState<&'a str>,
    pub zookeeper_wchc: Vec<&'a str>,
    pub zookeeper_wchc_all: Vec<Vec<&'a str>>,
    pub kafka_brokers: ListState<&'a str>,
    pub fibers: ListState<String>,
    pub fiber_dump: String,
    pub fiber_dump_all: Vec<String>,
    pub scroll: u16,
    pub barchart: Vec<(&'a str, u64)>,
    pub zio_keeper_nodes: ListState<&'a str>,
    pub akka_nodes: ListState<&'a str>,

}

impl<'a> App<'a> {
    pub fn new(title: &'a str, zookeeper_nodes: Vec<&'a str>, zookeeper_wchc_all: Vec<Vec<&'a str>>, fibers: Vec<String>, fiber_dump_all: Vec<String>) -> App<'a> {
        let a: Vec<&'a str> = zookeeper_wchc_all[0].to_owned();
        //let b: &'a str = &fiber_dump_all[0];
        //let b: Vec<&str> = a.to_owned().iter().map(|z| z.as_str().clone()).collect::<Vec<&str>>().to_owned();
        App {
            title,
            should_quit: false,
            tabs: TabsState::new(vec!["Zookeeper", "Kafka", "ZIO", "ZIO-Keeper", "Akka"]),
            zookeeper_nodes: ListState::new(zookeeper_nodes),
            zookeeper_wchc: a,
            zookeeper_wchc_all: zookeeper_wchc_all,
            kafka_brokers: ListState::new(KAFKA_BROKERS.to_vec()),
            fibers: ListState::new(fibers),
            fiber_dump: "".to_string(),
            fiber_dump_all: fiber_dump_all,
            scroll: 0,
            barchart: EVENTS.to_vec(),
            zio_keeper_nodes: ListState::new(ZIO_KEEPER_NODES.to_vec()),
            akka_nodes: ListState::new(AKKA_NODES.to_vec()),
        }
    }

    pub fn on_up(&mut self) {
        let tab = self.tabs.index;
        if tab == 0 {
            self.zookeeper_nodes.select_previous();
            let n = self.zookeeper_nodes.selected;
            self.zookeeper_wchc = self.zookeeper_wchc_all[n].to_owned();
        } else if tab == 2 {
            self.fibers.select_previous();
            let n = self.fibers.selected;
            self.fiber_dump = self.fiber_dump_all[n].to_owned();
        }
    }

    pub fn on_down(&mut self) {
        let tab = self.tabs.index;
        if tab == 0 {
            self.zookeeper_nodes.select_next();
            let n = self.zookeeper_nodes.selected;
            self.zookeeper_wchc = self.zookeeper_wchc_all[n].to_owned();
        } else if tab == 2 {
            self.fibers.select_next();
            let n = self.fibers.selected;
            self.fiber_dump = self.fiber_dump_all[n].to_owned();
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
        if tab == 2 {
            let fd = zmx_client::get_dump().unwrap();//TODO take care of error
            let list: Vec<UIFiber> = formatter::printable_tree(fd);
            let mut fib_labels = list.iter().map(|f| f.label.clone()).collect();
            let mut fib_dumps = list.iter().map(|f| f.dump.to_owned()).collect::<Vec<String>>();

            //let mut l: &Vec<&'a str> = &fib_labels.iter().map(|x| x.clone().to_owned().as_str()).collect();
            self.fibers.items.clear();
            self.fibers.items.append(&mut fib_labels);
            self.fibers.selected = 0;
            self.fiber_dump = fib_dumps[0].to_owned();
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
        if tab == 2 {
            if self.scroll > 0 {
                self.scroll -= 1;
            }
        }
    }

    pub fn on_page_down(&mut self) {
        let tab = self.tabs.index;
        if tab == 2 {
            let n = self.fiber_dump.clone().lines().collect::<Vec<&str>>().len();
            if self.scroll < n as u16 {
                self.scroll += 1;
            }
        }
    }

    pub fn on_tick(&mut self) {
        let event = self.barchart.pop().unwrap();
        self.barchart.insert(0, event);
    }
}

const KAFKA_BROKERS: [&'static str; 24] = [
    "Broker1", "Broker2", "Broker3", "Broker4", "Broker5", "Broker6", "Broker7", "Broker8", "Broker9", "Broker10",
    "Broker11", "Broker12", "Broker13", "Broker14", "Broker15", "Broker16", "Broker17", "Broker18", "Broker19",
    "Broker20", "Broker21", "Broker22", "Broker23", "Broker24",
];

const ZIO_KEEPER_NODES: [&'static str; 24] = [
    "Broker1", "Broker2", "Broker3", "Broker4", "Broker5", "Broker6", "Broker7", "Broker8", "Broker9", "Broker10",
    "Broker11", "Broker12", "Broker13", "Broker14", "Broker15", "Broker16", "Broker17", "Broker18", "Broker19",
    "Broker20", "Broker21", "Broker22", "Broker23", "Broker24",
];

const AKKA_NODES: [&'static str; 24] = [
    "Broker1", "Broker2", "Broker3", "Broker4", "Broker5", "Broker6", "Broker7", "Broker8", "Broker9", "Broker10",
    "Broker11", "Broker12", "Broker13", "Broker14", "Broker15", "Broker16", "Broker17", "Broker18", "Broker19",
    "Broker20", "Broker21", "Broker22", "Broker23", "Broker24",
];

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
