use super::{Color, Event, Point, Rect, Renderer, Widget};

use std::sync::Arc;

extern crate orbclient;

pub struct WindowRenderer<'a> {
    inner: &'a mut Box<orbclient::Window>,
}

impl<'a> WindowRenderer<'a> {
    pub fn new(inner: &'a mut Box<orbclient::Window>) -> WindowRenderer {
        WindowRenderer { inner: inner }
    }
}

impl<'a> Renderer for WindowRenderer<'a> {
    fn clear(&mut self, color: Color) {
        self.inner.set(orbclient::Color { data: color.data });
    }

    fn char(&mut self, pos: Point, c: char, color: Color) {
        self.inner.char(pos.x, pos.y, c, orbclient::Color { data: color.data });
    }

    fn rect(&mut self, rect: Rect, color: Color) {
        self.inner.rect(rect.x,
                        rect.y,
                        rect.width,
                        rect.height,
                        orbclient::Color { data: color.data });
    }
}

impl<'a> Drop for WindowRenderer<'a> {
    fn drop(&mut self) {
        self.inner.sync();
    }
}

pub struct Window {
    inner: Box<orbclient::Window>,
    pub widgets: Vec<Arc<Widget>>,
    pub widget_focus: usize,
    pub bg: Color,
}

impl Window {
    pub fn new(rect: Rect, title: &str) -> Box<Self> {
        Box::new(Window {
            inner: orbclient::Window::new(rect.x, rect.y, rect.width, rect.height, title).unwrap(),
            widgets: Vec::new(),
            widget_focus: 0,
            bg: Color::rgb(237, 233, 227),
        })
    }

    pub fn draw(&mut self) {
        let mut renderer = WindowRenderer::new(&mut self.inner);
        renderer.clear(self.bg);

        for i in 0..self.widgets.len() {
            if let Some(widget) = self.widgets.get(i) {
                widget.draw(&mut renderer, self.widget_focus == i);
            }
        }
    }

    pub fn exec(&mut self) {
        self.draw();
        'event: loop {
            let mut events = Vec::new();

            for orbital_event in self.inner.events() {
                match orbital_event.to_option() {
                    orbclient::EventOption::Mouse(mouse_event) => {
                        events.push(Event::Mouse {
                            point: Point::new(mouse_event.x, mouse_event.y),
                            left_button: mouse_event.left_button,
                            middle_button: mouse_event.middle_button,
                            right_button: mouse_event.right_button,
                        })
                    }
                    orbclient::EventOption::Key(key_event) => {
                        if key_event.pressed {
                            match key_event.scancode {
                                orbclient::K_BKSP => events.push(Event::Backspace),
                                orbclient::K_DEL => events.push(Event::Delete),
                                orbclient::K_HOME => events.push(Event::Home),
                                orbclient::K_END => events.push(Event::End),
                                orbclient::K_UP => events.push(Event::UpArrow),
                                orbclient::K_DOWN => events.push(Event::DownArrow),
                                orbclient::K_LEFT => events.push(Event::LeftArrow),
                                orbclient::K_RIGHT => events.push(Event::RightArrow),
                                _ => {
                                    match key_event.character {
                                        '\0' => (),
                                        '\x1B' => (),
                                        '\n' => events.push(Event::Enter),
                                        _ => events.push(Event::Text { c: key_event.character }),
                                    }
                                }
                            }
                        }
                    }
                    orbclient::EventOption::Quit(_quit_event) => break 'event,
                    _ => (),
                };
            }

            let mut redraw = false;
            for event in events.iter() {
                for i in 0..self.widgets.len() {
                    if let Some(widget) = self.widgets.get(i) {
                        if widget.event(*event, self.widget_focus == i, &mut redraw) {
                            if self.widget_focus != i {
                                self.widget_focus = i;
                                redraw = true;
                            }
                        }
                    }
                }
            }

            if redraw {
                self.draw();
            }
        }
    }
}
