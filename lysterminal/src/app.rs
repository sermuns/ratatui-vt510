use ratatui::{
    crossterm::event::KeyCode,
    prelude::*,
    widgets::{Block, Paragraph},
};
use std::sync::{Arc, atomic::AtomicBool};
use strum::{FromRepr, IntoEnumIterator};

#[cfg(feature = "vt510")]
use ratatui_vt510::Vt510Backend;
#[cfg(feature = "vt510")]
type AppTerminal = ratatui::Terminal<Vt510Backend>;

#[cfg(not(feature = "vt510"))]
use {ratatui::prelude::CrosstermBackend, std::io::Stdout};
#[cfg(not(feature = "vt510"))]
type AppTerminal = ratatui::Terminal<CrosstermBackend<Stdout>>;

pub struct App {
    /// should the application be running?
    running: bool,

    /// i.e. CTRL-C has been pressed
    externally_interrupted: Arc<AtomicBool>,

    terminal: AppTerminal,

    current_menu: Menu,
}

#[derive(FromRepr, Clone, Copy)]
enum MainMenuButton {
    BadApple,
}

trait Browse {
    fn next(self) -> Self;
    fn previous(self) -> Self;
}

impl<T> Browse for T
where
    T: Copy,
{
    fn next(self) -> Self {
        let current_index = self as usize;
        let next_index = current_index.saturating_add(1);
        Self::from_repr(next_index).unwrap_or(self)
    }

    fn previous(self) -> Self {
        let current_index = self as usize;
        let previous_index = current_index.saturating_sub(1);
        Self::from_repr(previous_index).unwrap_or(self)
    }
}

enum Menu {
    Main { focus: MainMenuButton },
    BadApple,
}

enum Command {
    Up,
    Down,
    Left,
    Right,
    Confirm,
    Cancel,
}

trait ToCommand {
    fn to_command(self) -> Option<Command>;
}

impl ToCommand for KeyCode {
    fn to_command(self) -> Option<Command> {
        match self {
            KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('k') => Some(Command::Up),
            KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('j') => Some(Command::Down),
            KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('h') => Some(Command::Left),
            KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('l') => Some(Command::Right),
            KeyCode::Enter | KeyCode::Char(' ') => Some(Command::Confirm),
            KeyCode::Esc => Some(Command::Cancel),
            _ => None,
        }
    }
}

impl App {
    pub fn new(externally_interrupted: Arc<AtomicBool>, terminal: AppTerminal) -> Self {
        Self {
            externally_interrupted,
            running: true,
            terminal,
            current_menu: Menu::Main {
                focus: MainMenuButton::BadApple,
            },
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
            self.terminal
                .draw(|frame| render(&self.current_menu, frame))?;
            self.handle_input()?;
        }

        self.terminal.clear()?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    fn quit(&mut self) {
        self.running = false;
    }

    fn handle_input(&mut self) -> color_eyre::Result<()> {
        #[cfg(feature = "vt510")]
        {
            let Ok(chars) = self.terminal.backend_mut().read() else {
                return Ok(());
            };
            println!("{:?}", chars);
            match chars.first().unwrap().to_ascii_lowercase() as char {
                'q' => self.quit(),
                _ => {}
            }
        }
        #[cfg(not(feature = "vt510"))]
        {
            use ratatui::crossterm::event::{self, Event, KeyCode, KeyModifiers};

            let Event::Key(key_event) = event::read()? else {
                return Ok(());
            };

            if key_event.code == KeyCode::Char('c') && key_event.modifiers == KeyModifiers::CONTROL
            {
                self.quit();
            }

            let Some(command) = key_event.code.to_command() else {
                return Ok(());
            };

            #[allow(clippy::single_match)]
            match self.current_menu {
                Menu::Main { mut focus } => focus = focus.next(),
                Menu::BadApple => match command {
                    Command::Cancel => {
                        self.current_menu = Menu::Main {
                            focus: MainMenuButton::BadApple,
                        }
                    }
                    _ => {}
                },
            }
        }

        Ok(())
    }
}

fn render(current_menu: &Menu, frame: &mut Frame) {
    let [title_bar, body, bottom_bar] = frame.area().layout(&Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ]));
}
