#[cfg(not(feature = "vt510"))]
use std::io::Stdout;
use std::{
    sync::{Arc, atomic::AtomicBool},
    thread::sleep,
    time::Duration,
};

use ratatui::{
    crossterm::terminal::{disable_raw_mode, enable_raw_mode},
    prelude::*,
    widgets::{Block, Paragraph},
};

#[cfg(feature = "vt510")]
use ratatui_vt510::Vt510Backend;
#[cfg(feature = "vt510")]
type AppTerminal = ratatui::Terminal<Vt510Backend>;

#[cfg(not(feature = "vt510"))]
use ratatui::prelude::CrosstermBackend;
#[cfg(not(feature = "vt510"))]
type AppTerminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

pub struct App {
    /// should the application be running?
    running: bool,

    /// i.e. CTRL-C has been pressed
    externally_interrupted: Arc<AtomicBool>,

    terminal: AppTerminal,
}

impl App {
    pub fn new(externally_interrupted: Arc<AtomicBool>, terminal: AppTerminal) -> Self {
        Self {
            externally_interrupted,
            running: true,
            terminal,
        }
    }

    pub fn run(mut self) -> color_eyre::Result<()> {
        self.terminal.hide_cursor()?;
        self.terminal.clear()?;

        while self.running
            && !self
                .externally_interrupted
                .load(std::sync::atomic::Ordering::SeqCst)
        {
            self.terminal.draw(render)?;
            self.handle_input()?;
        }

        self.terminal.clear()?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    fn quit(&mut self) {
        self.running = false;
    }

    #[cfg(feature = "vt510")]
    fn handle_input(&mut self) -> color_eyre::Result<()> {
        let Ok(chars) = self.terminal.backend_mut().read() else {
            return Ok(());
        };
        println!("{:?}", chars);
        match chars.first().unwrap().to_ascii_lowercase() as char {
            'q' => self.quit(),
            _ => {}
        }
        Ok(())
    }
    #[cfg(not(feature = "vt510"))]
    fn handle_input(&mut self) -> color_eyre::Result<()> {
        use ratatui::crossterm::event::{self, Event, KeyCode};

        #[allow(clippy::single_match)]
        match event::read()? {
            Event::Key(key_event) => {
                if key_event.code == KeyCode::Char('q') {
                    self.running = false;
                }
            }
            _ => {}
        }

        Ok(())
    }
}
fn render(frame: &mut Frame) {
    let title = Line::from("Ratatui Simple Template")
        // .bold()
        .blue()
        .centered();
    let text = "Hello, Ratatui!\n\n\
            Created using https://github.com/ratatui/templates\n\
            Press `Esc`, `Ctrl-C` or `q` to stop running.";
    frame.render_widget(
        Paragraph::new(text)
            .block(Block::bordered().title(title))
            .centered(),
        frame.area(),
    )
}
