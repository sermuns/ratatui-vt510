use clap::Parser;
use color_eyre::{Result, eyre::Context};
use ratatui::{
    Terminal,
    widgets::{Block, Borders, Paragraph},
};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

mod backend;
use crate::backend::SerialBackend;

#[derive(clap::Parser)]
struct Args {
    /// where to serve the TUI
    serial_port: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let port = serialport::new(&args.serial_port, 115_200)
        .timeout(Duration::from_millis(10))
        .open()?;

    let running = Arc::new(AtomicBool::new(true));

    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .wrap_err("error setting Ctrl-C handler")?;

    println!("starting serving on {}", &args.serial_port);

    let backend = SerialBackend::new(port, 80, 24);
    let mut terminal = Terminal::new(backend)?;

    terminal.show_cursor()?;
    terminal.clear()?;
    terminal.flush()?;

    let mut counter = 0;
    let mut last_tick = Instant::now();

    while running.load(Ordering::SeqCst) {
        terminal.draw(|f| {
            let area = f.area();

            let widget = Paragraph::new(format!("Hello VT510 over serial\nCounter: {}", counter))
                .block(Block::default().title("Ratatui 0.30").borders(Borders::ALL));

            f.render_widget(widget, area);
        })?;

        if last_tick.elapsed() >= Duration::from_secs(1) {
            counter += 1;
            last_tick = Instant::now();
        }

        std::thread::sleep(Duration::from_millis(50));
    }

    terminal.show_cursor()?;
    terminal.clear()?;
    terminal.flush()?;
    terminal.draw(|f| f.render_widget(Paragraph::new("Exiting.."), f.area()))?;

    println!("exiting..");
    Ok(())
}
