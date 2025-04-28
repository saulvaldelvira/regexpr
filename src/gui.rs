#![cfg_attr(rustfmt, rustfmt_skip)]

use std::cell::RefCell;
use std::fmt::Write;
use std::rc::Rc;

use fltk::app::Scheme;
use fltk::enums::{CallbackTrigger, Event, Shortcut};
use fltk::group::Group;
use fltk::menu::{MenuBar, MenuFlag};
use fltk::text::TextDisplay;
use fltk::{prelude::*, *};
use regexpr::Regex;

struct State {
    regex: Regex,
}

#[derive(Clone)]
struct Splitter {
    left: text::TextEditor,
    right: text::TextDisplay,
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

        let mut right = text::TextDisplay::new(x + w / 2 + 2, y, w / 2 - 2, h, None);
        right.set_buffer(text::TextBuffer::default());
        right.wrap_mode(text::WrapMode::AtBounds, 0);

        let group = Group::new(10, MENU_HEIGHT, 790, REGEX_HEIGHT, "");

        let mut td = TextDisplay::new(10, MENU_HEIGHT, 60, REGEX_HEIGHT, ": ");
        td.set_buffer(text::TextBuffer::default());
        td.buffer().unwrap().set_text("Enter regex: ");
        let mut regex_input = input::Input::new(70, MENU_HEIGHT, 790 - 60, REGEX_HEIGHT, "");

        group.end();


        regex_input.set_trigger(CallbackTrigger::Changed);

        let mut spl = Self {
            left,
            right,
            divider,
            state,
            regex_input,
            dragging: false,
        };

        spl.regex_input.set_callback({
            let spl = spl.clone();
            move |_| {
                let regex = spl.regex_input.value();
                let regex = Regex::compile(&regex).unwrap();
                spl.state.borrow_mut().regex = regex;
                spl.process()
            }
        });

        spl.left.set_callback({
            let spl = spl.clone();
            move |_| {
                spl.process()
            }
        });

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
                let max_x = self.right.x() + self.right.w() - 50;

                if x >= min_x && x <= max_x {
                    let left_w = x - self.left.x();
                    let right_w = self.right.x() + self.right.w() - x - 4;

                    self.left.resize(self.left.x(), self.left.y(), left_w, self.left.h());
                    self.divider.resize(x, self.divider.y(), 4, self.divider.h());
                    self.right.resize(x + 4, self.right.y(), right_w, self.right.h());

                    self.left.redraw();
                    self.divider.redraw();
                    self.right.redraw();
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

            let state = self.state.borrow();
            let matches = state.regex.find_matches(&text);

            let mut s = "".to_string();
            for m in matches {
                writeln!(s, "{}:{}", m.span().0, m.span().1).unwrap();
            }

            self.right.buffer().unwrap().set_text(&s);
        }
    }
}

const MENU_HEIGHT: i32 = 25;
const INITIAL_WIDTH: i32 = 800;
const INITIAL_HEIGHT: i32 = 600;
const REGEX_HEIGHT: i32 = 40;
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
