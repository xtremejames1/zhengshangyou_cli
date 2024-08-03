use crate::display::Renderable;
use crossterm::{
    cursor, queue,
    style::{self, Stylize},
    terminal,
};
use std::any::Any;
use std::collections::VecDeque;
use std::io::{self, Write};
use std::time::Duration;
use std::time::Instant;

pub struct Logger {
    log_queue: VecDeque<(String, Instant, Duration)>,
    new_log: bool,
}

impl Logger {
    pub fn new() -> Self {
        Self {
            log_queue: VecDeque::new(),
            new_log: false,
        }
    }

    pub fn log<T>(&mut self, s: T, timeout: Duration)
    where
        T: Into<String>,
    {
        self.log_queue
            .push_back((s.into(), Instant::now(), timeout));
        self.new_log = true;
    }

    pub fn clean(&mut self) {
        // Remove timed out logs
        self.log_queue.retain(|(_, instant, timeout)| {
            timeout.is_zero() || Instant::now().duration_since(*instant).lt(timeout)
        });
    }
}

impl Renderable for Logger {
    fn render_init(&self) -> Result<(), &'static str> {
        let log_width = terminal::size().unwrap().0 / 3 - 4;
        // Clear background
        queue!(io::stdout(), style::SetBackgroundColor(style::Color::Black),);
        for i in 0..log_width {
            for j in 0..10 {
                queue!(
                    io::stdout(),
                    cursor::MoveTo(2 + i, 2 + j),
                    style::Print(" ")
                );
            }
        }
        Ok(())
    }
    fn render_update(&mut self) -> Result<(), &'static str> {
        let old_logs = self.log_queue.clone();

        self.clean();

        if self.new_log || !old_logs.iter().all(|item| self.log_queue.contains(item)) {
            let log_width = terminal::size().unwrap().0 / 3 - 4;
            // Clear background
            queue!(io::stdout(), style::SetBackgroundColor(style::Color::Black),);
            for i in 0..log_width {
                for j in 0..10 {
                    queue!(
                        io::stdout(),
                        cursor::MoveTo(2 + i, 2 + j),
                        style::Print(" ")
                    );
                }
            }
            let mut height = 0;
            for log in self.log_queue.iter() {
                let log_height = log.0.len() as u16 / log_width;
                if height + log_height < 10 {
                    for i in 0..log_height + 1 {
                        let line = log
                            .0
                            .split_at((log_width * i).into())
                            .1
                            .split_at(if i == log_height {
                                log.0.len() % log_width as usize
                            } else {
                                (log_width).into()
                            })
                            .0;

                        queue!(
                            io::stdout(),
                            cursor::MoveTo(2, 2 + height),
                            style::SetBackgroundColor(style::Color::Black),
                            style::PrintStyledContent(line.green())
                        );
                        height += 1;
                    }
                }
            }
            io::stdout().flush();
            self.new_log = false;
        }
        Ok(())
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
