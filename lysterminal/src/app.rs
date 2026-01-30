#[cfg(not(feature = "vt510"))]
use std::io::Stdout;
use std::sync::{Arc, atomic::AtomicBool};

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

#[derive(Debug)]
pub struct App {
    /// should the application be running?
    running: bool,

    /// i.e. CTRL-C has been pressed
    externally_interrupted: Arc<AtomicBool>,
}

impl App {
    pub fn new(externally_interrupted: Arc<AtomicBool>) -> Self {
        Self {
            externally_interrupted,
            running: false,
        }
    }

    pub fn run(mut self, mut terminal: AppTerminal) -> color_eyre::Result<()> {
        enable_raw_mode()?;
        terminal.hide_cursor()?;
        terminal.clear()?;

        self.running = true;
        loop {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_input()?;

            if !self.running
                || !self
                    .externally_interrupted
                    .load(std::sync::atomic::Ordering::SeqCst)
            {
                break;
            }
        }
        disable_raw_mode()?;
        terminal.clear()?;
        terminal.show_cursor()?;
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        let title = Line::from("Ratatui Simple Template")
            .bold()
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

    #[cfg(feature = "vt510")]
    fn handle_input(&mut self) -> color_eyre::Result<()> {
        // Handle events for VT510 backend
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
