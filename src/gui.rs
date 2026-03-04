#![cfg_attr(rustfmt, rustfmt_skip)]

use std::cell::RefCell;
use std::rc::Rc;

use fltk::app::Scheme;
use fltk::enums::{Align, CallbackTrigger, Color, Event, FrameType, Shortcut};
use fltk::frame::Frame;
use fltk::group::{Group, Pack, PackType};
use fltk::menu::{MenuBar, MenuFlag};
use fltk::{prelude::*, *};
use regexpr::Regex;

struct Match {
    offset: usize,
    str: String,
}

struct State {
    regex: Regex,
    matches: Vec<Match>,
}

#[derive(Clone)]
struct Splitter {
    left: text::TextEditor,
    right: Rc<RefCell<group::Scroll>>,
    divider: frame::Frame,
    dragging: bool,
    state: Rc<RefCell<State>>,
    regex_input: input::Input,
}

impl Splitter {
    fn new(x: i32, y: i32, w: i32, h: i32, state: Rc<RefCell<State>>) -> Self {
        let mut left = text::TextEditor::new(x, y, w / 2 - 2, h, None);
        left.set_buffer(text::TextBuffer::default());
        left.set_trigger(CallbackTrigger::Changed);

        let divider = frame::Frame::new(x + w / 2 - 2, y, 4, h, None);

        let mut right = group::Scroll::new(x + w / 2 + 2, y, w / 2 - 2, h, None);
        right.set_frame(enums::FrameType::NoBox);
        right.set_scrollbar_size(20);
        right.clear();
        right.end();
        /* right.set_buffer(text::TextBuffer::default()); */
        /* right.wrap_mode(text::WrapMode::AtBounds, 0); */

        let group = Group::new(10, MENU_HEIGHT, w - 20, REGEX_HEIGHT, "regex");
        /* let mut td = TextDisplay::new(10, MENU_HEIGHT, 240, REGEX_HEIGHT, ": "); */
        /* td.set_buffer(text::TextBuffer::default()); */
        /* td.buffer().unwrap().set_text("Enter regex: "); */
        let mut regex_input = input::Input::new(10, MENU_HEIGHT, w - 20 - 50, REGEX_HEIGHT, "");

        let mut btn = button::Button::new(10 + w - 20 - 50 + 5, MENU_HEIGHT, 40, REGEX_HEIGHT, "");
        btn.set_label("match");

        group.end();

        regex_input.set_trigger(CallbackTrigger::Changed);

        let spl = Self {
            left,
            right: Rc::new(RefCell::new(right)),
            divider,
            state,
            regex_input,
            dragging: false,
        };

        btn.set_callback({
            let spl = spl.clone();
            move |_| {
                let regex = spl.regex_input.value();
                let regex = Regex::compile(&regex).unwrap();
                spl.state.borrow_mut().regex = regex;
                spl.process();
            }
        });
        /* spl.left.set_callback({ */
        /*     let spl = spl.clone(); */
        /*     move |_| { */
        /*         spl.process() */
        /*     } */
        /* }); */

        spl
    }

    fn handle_event(&mut self, ev: Event) -> bool {
        match ev {
            Event::Push => {
                self.dragging = true;
                true
            }
            Event::Drag if self.dragging => {
                let x = app::event_coords().0;
                let min_x = self.left.x() + 50;

                let mut scroll = self.right.borrow_mut();

                let max_x = scroll.x() + scroll.w() - 50;

                if x >= min_x && x <= max_x {
                    let left_w = x - self.left.x();
                    let right_w = scroll.x() + scroll.w() - x - 4;

                    self.left.resize(self.left.x(), self.left.y(), left_w, self.left.h());
                    self.divider.resize(x, self.divider.y(), 4, self.divider.h());
                    let scry = scroll.y();
                    let scrh = scroll.h();
                    scroll.resize(x + 4, scry, right_w, scrh);

                    self.left.redraw();
                    self.divider.redraw();
                    scroll.redraw();
                }
                true
            }
            Event::Released => {
                self.dragging = false;
                true
            }
            _ => false,
        }
    }

    fn process(&self) {
        if let Some(buf) = self.left.buffer() {
            let text = buf.text();

            let mut state = self.state.borrow_mut();
            let State { regex, matches } = &mut *state;
            matches.clear();
            matches.extend(
                regex.find_matches(&text).map(|m| {
                    Match { offset: m.span().0, str: m.slice().to_string() }
                })
            );


            let mut scroll = self.right.borrow_mut();
            let width = scroll.width() - scroll.scrollbar_size() - 20;

            let mut pack = Pack::new(5, 10, width, 0, "");
            pack.set_spacing(5);
            pack.set_type(PackType::Vertical);
            for m in matches {
                let label_text = format!("[{}:{}] {}", m.offset, m.offset + m.str.len(), m.str);

                let mut frame = Frame::new(0, 0, width, 25, Some(&*label_text));
                frame.set_callback({
                    let (x, y) = (scroll.x(), scroll.y());
                    move |_| {
                        let mut f = Frame::new(x - 50, y + 40, 300, 500, "Info");
                        f.redraw();
                    }
                });
                frame.set_frame(FrameType::DownBox);
                frame.set_color(Color::from_rgb(240, 240, 255));
                frame.set_label_size(14);
                frame.set_align(Align::Left | Align::Inside);

                pack.add(&frame);
            }
            pack.end();
            scroll.clear();
            scroll.add(&pack);
            scroll.redraw();

            /* let matches = state.regex.find_matches(&text); */

            /* scroll.clear(); */


            /* for m in matches { */
            /*     let label_text = format!("{}:{}", m.span().0, m.span().1); */

            /*     let mut frame = Frame::new(0, 0, width, 25, Some(&*label_text)); */
            /*     frame.set_frame(FrameType::DownBox); */
            /*     frame.set_color(Color::from_rgb(240, 240, 255)); */
            /*     frame.set_label_size(14); */
            /*     frame.set_align(Align::Left | Align::Inside); */

            /*     pack.add(&frame); */
            /* } */

            /* pack.end(); */

            /* scroll.add(&pack); */

            /* scroll.redraw(); */
            /* scroll.scroll_to(0, 0); */
        }
    }
}

const MENU_HEIGHT: i32 = 25;
const INITIAL_WIDTH: i32 = 800;
const INITIAL_HEIGHT: i32 = 600;
const REGEX_HEIGHT: i32 = 30;
const STATUS_HEIGHT: i32 = 25;

fn menu_bar() {
    let mut menu = MenuBar::new(0, 0, INITIAL_WIDTH, MENU_HEIGHT, "");

    menu.add(
        "File/Exit",
        Shortcut::None,
        MenuFlag::Normal,
        move |_| {
            std::process::exit(0);
        }
    );
}

pub fn start_gui() -> Result<(), String> {
    let app = app::App::default().with_scheme(Scheme::Base);

    let mut win = window::Window::default()
        .with_size(INITIAL_WIDTH, INITIAL_HEIGHT)
        .with_label("Regexpr")
        .center_screen();

    let state = State {
        regex: Regex::compile("").unwrap(),
        matches: vec![],
    };
    let state = Rc::new(RefCell::new(state));

    let splitter = Splitter::new(
        10,
        MENU_HEIGHT + REGEX_HEIGHT,
        INITIAL_WIDTH - 10,
        INITIAL_HEIGHT - MENU_HEIGHT - STATUS_HEIGHT - REGEX_HEIGHT,
        Rc::clone(&state),
    );

    menu_bar();

    win.handle({
        let mut splt = splitter.clone();
        move |_, ev| splt.handle_event(ev)
    });

    win.resizable(&splitter.left);
    win.end();
    win.show();

    app.run().map_err(|err| {
        format!("Couldn't start GUI: {err}")
    })
}
