use std::collections::VecDeque;
use std::iter::Iterator;

use crate::akka_actor_tree::model::{ActorNode, AkkaActorTreeSettings};
use crate::jmx_client::model::{HikariMetrics, JMXConnectionSettings, SlickConfig, SlickMetrics};
use crate::ui::formatter;
use crate::ui::model::UIFiber;
use crate::zio::model::{Fiber, FiberCount, FiberStatus};

pub enum TabKind {
    ZMX,
    Slick,
    AkkaActorTree,
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

pub struct ZMXTab {
    pub fibers: ListState<String>,
    pub selected_fiber_dump: (String, u16),
    pub fiber_dump_all: Vec<String>,
    pub scroll: u16,
    pub fiber_counts: VecDeque<FiberCount>,
}

impl ZMXTab {
    pub const MAX_FIBER_COUNT_MEASURES: usize = 100;

    pub fn new() -> ZMXTab {
        ZMXTab {
            fibers: ListState::new(vec![]),
            selected_fiber_dump: ("".to_string(), 1),
            fiber_dump_all: vec![],
            scroll: 0,
            fiber_counts: VecDeque::new(),
        }
    }

    fn append_fiber_count(&mut self, c: FiberCount) {
        if self.fiber_counts.len() > ZMXTab::MAX_FIBER_COUNT_MEASURES {
            self.fiber_counts.pop_front();
        }
        self.fiber_counts.push_back(c);
    }

    pub fn select_prev_fiber(&mut self) {
        if !self.fibers.items.is_empty() {
            self.fibers.select_previous();
            self.on_fiber_change()
        }
    }

    pub fn select_next_fiber(&mut self) {
        if !self.fibers.items.is_empty() {
            self.fibers.select_next();
            self.on_fiber_change()
        }
    }

    pub fn on_fiber_change(&mut self) {
        let n = self.fibers.selected;
        self.selected_fiber_dump = ZMXTab::prepare_dump(self.fiber_dump_all[n].clone());
        self.scroll = 0;
    }

    pub fn replace_fiber_dump(&mut self, dump: Vec<Fiber>) {
        let list: Vec<UIFiber> = formatter::printable_tree(dump, true)
            .iter()
            .map(|(label, fb)| UIFiber { label: label.to_owned(), dump: fb.dump.to_owned() })
            .collect();
        let mut fib_labels = list.iter().map(|f| f.label.clone()).collect();
        let mut fib_dumps = list.iter().map(|f| f.dump.to_owned()).collect::<Vec<String>>();

        self.fibers.items.clear();
        self.fibers.items.append(&mut fib_labels);
        self.fibers.selected = 0;
        self.selected_fiber_dump = ZMXTab::prepare_dump(fib_dumps[0].clone());
        self.fiber_dump_all.clear();
        self.fiber_dump_all.append(&mut fib_dumps);
    }

    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.scroll < self.selected_fiber_dump.1 {
            self.scroll += 1;
        }
    }

    pub fn append_fiber_dump_for_counts(&mut self, dump: Vec<Fiber>) {
        let mut count = FiberCount { done: 0, suspended: 0, running: 0, finishing: 0 };
        for f in dump.iter() {
            match f.status {
                FiberStatus::Done => { count.done += 1 }
                FiberStatus::Finishing => { count.finishing += 1 }
                FiberStatus::Running => { count.running += 1 }
                FiberStatus::Suspended => { count.suspended += 1 }
            }
        }
        self.append_fiber_count(count);
    }

    fn prepare_dump(s: String) -> (String, u16) {
        (s.clone(), s.lines().collect::<Vec<&str>>().len() as u16)
    }
}

pub struct SlickTab {
    pub has_hikari: bool,
    pub slick_metrics: VecDeque<SlickMetrics>,
    pub slick_config: SlickConfig,
    pub hikari_metrics: VecDeque<HikariMetrics>,
}

impl SlickTab {
    pub const MAX_SLICK_MEASURES: usize = 25;
    pub const MAX_HIKARI_MEASURES: usize = 100;

    pub fn new() -> SlickTab {
        SlickTab {
            has_hikari: false,
            slick_metrics: VecDeque::new(),
            slick_config: SlickConfig { max_threads: 0, max_queue_size: 0 },
            hikari_metrics: VecDeque::new(),
        }
    }

    pub fn replace_slick_config(&mut self, m: SlickConfig) {
        self.slick_config = m
    }

    pub fn append_slick_metrics(&mut self, m: SlickMetrics) {
        if self.slick_metrics.len() > SlickTab::MAX_SLICK_MEASURES {
            self.slick_metrics.pop_front();
        }
        self.slick_metrics.push_back(m);
    }

    pub fn append_hikari_metrics(&mut self, m: HikariMetrics) {
        if self.hikari_metrics.len() > SlickTab::MAX_HIKARI_MEASURES {
            self.hikari_metrics.pop_front();
        }
        self.hikari_metrics.push_back(m);
    }
}

pub struct AkkaActorTreeTab {
    pub actors: ListState<String>,
    pub actor_counts: VecDeque<u64>,
}

impl AkkaActorTreeTab {
    pub const MAX_ACTOR_COUNT_MEASURES: usize = 25;

    pub fn new() -> AkkaActorTreeTab {
        AkkaActorTreeTab { actors: ListState::new(vec![]), actor_counts: VecDeque::new() }
    }

    pub fn update_actor_tree(&mut self, actors: Vec<ActorNode>) {
        let mut list: Vec<String> = formatter::printable_tree(actors, false)
            .iter()
            .map(|x| x.0.to_owned())
            .collect();

        self.actors.items.clear();
        self.actors.items.append(&mut list);
        self.actors.selected = 0;
    }

    pub fn select_prev_actor(&mut self) {
        self.actors.select_previous();
    }

    pub fn select_next_actor(&mut self) {
        self.actors.select_next();
    }

    pub fn append_actor_count(&mut self, c: u64) {
        if self.actor_counts.len() > AkkaActorTreeTab::MAX_ACTOR_COUNT_MEASURES {
            self.actor_counts.pop_front();
        }
        self.actor_counts.push_back(c);
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
        if !self.items.is_empty() && self.selected < self.items.len() - 1 {
            self.selected += 1
        }
    }
}

pub struct App<'a> {
    pub title: &'a str,
    pub should_quit: bool,
    pub exit_reason: Option<String>,
    pub tabs: TabsState<'a>,
    pub zmx: Option<ZMXTab>,
    pub slick: Option<SlickTab>,
    pub actor_tree: Option<AkkaActorTreeTab>,
}

impl<'a> App<'a> {
    pub fn new(
        title: &'a str,
        zio_zmx_addr: Option<String>,
        jmx: Option<JMXConnectionSettings>,
        akka_actor_tree: Option<AkkaActorTreeSettings>) -> App<'a> {
        let mut tabs: Vec<Tab> = vec![];

        if let Some(_) = zio_zmx_addr {
            tabs.push(Tab { kind: TabKind::ZMX, title: "ZMX" })
        }

        if let Some(_) = jmx {
            tabs.push(Tab { kind: TabKind::Slick, title: "Slick" })
        }

        if let Some(_) = akka_actor_tree {
            tabs.push(Tab { kind: TabKind::AkkaActorTree, title: "Actor Tree" })
        }

        App {
            title,
            should_quit: false,
            exit_reason: None,
            tabs: TabsState::new(tabs),
            zmx: zio_zmx_addr.map(|_| ZMXTab::new()),
            slick: jmx.map(|_| SlickTab::new()),
            actor_tree: akka_actor_tree.map(|_| AkkaActorTreeTab::new()),
        }
    }

    pub fn on_up(&mut self) {
        match self.tabs.current().kind {
            TabKind::ZMX => self.zmx.as_mut().unwrap().select_prev_fiber(),
            TabKind::Slick => {}
            TabKind::AkkaActorTree => self.actor_tree.as_mut().unwrap().select_prev_actor(),
        }
    }

    pub fn on_down(&mut self) {
        match self.tabs.current().kind {
            TabKind::ZMX => self.zmx.as_mut().unwrap().select_next_fiber(),
            TabKind::Slick => {}
            TabKind::AkkaActorTree => self.actor_tree.as_mut().unwrap().select_next_actor(),
        }
    }

    pub fn on_right(&mut self) {
        self.tabs.next();
    }

    pub fn on_left(&mut self) {
        self.tabs.previous();
    }

    pub fn on_key(&mut self, c: char) {
        match c {
            'q' => self.quit(None),
            _ => {}
        }
    }

    pub fn quit(&mut self, error: Option<String>) {
        self.should_quit = true;
        self.exit_reason = error;
    }

    pub fn on_page_up(&mut self) {
        match self.tabs.current().kind {
            TabKind::ZMX => self.zmx.as_mut().unwrap().scroll_up(),
            TabKind::Slick => {}
            TabKind::AkkaActorTree => {}
        }
    }

    pub fn on_page_down(&mut self) {
        match self.tabs.current().kind {
            TabKind::ZMX => self.zmx.as_mut().unwrap().scroll_down(),
            TabKind::Slick => {}
            TabKind::AkkaActorTree => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ui::app::{ZMXTab, ListState};
    use crate::zio::zmx_client::StubZMXClient;
    use crate::zio::model::{Fiber, FiberStatus};
    use std::collections::VecDeque;

    #[test]
    fn zmx_tab_dumps_fibers() {
        let fiber1 = Fiber {
            id: 1,
            parent_id: None,
            status: FiberStatus::Running,
            dump: "1".to_owned(),
        };
        let fiber2 = Fiber {
            id: 2,
            parent_id: Some(1),
            status: FiberStatus::Suspended,
            dump: "2".to_owned(),
        };
        let fiber4 = Fiber {
            id: 4,
            parent_id: None,
            status: FiberStatus::Done,
            dump: "4".to_owned(),
        };

        let fibers = vec![fiber1, fiber2, fiber4];

        let mut tab = ZMXTab {
            fibers: ListState::new(vec!["Fiber #1".to_owned()]),
            selected_fiber_dump: ("".to_string(), 0),
            fiber_dump_all: vec![],
            scroll: 0,
            fiber_counts: VecDeque::new()
        };

        tab.replace_fiber_dump(fibers);

        assert_eq!(tab.fiber_dump_all, vec!["1", "2", "4"]);
        assert_eq!(tab.fibers.items, vec![
            "├─#1   Running",
            "│ └─#2 Suspended",
            "└─#4   Done"
        ]);
        assert_eq!(tab.fibers.selected, 0);
    }
}
