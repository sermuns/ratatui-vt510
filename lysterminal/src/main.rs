use clap::Parser;
use color_eyre::{Result, eyre::Context};
use ratatui::prelude::*;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

mod app;

use crate::app::App;

#[derive(Parser)]
#[command(version)]
struct Args {
    /// where to serve the TUI
    serial_port: String,
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let interrupted = Arc::new(AtomicBool::new(false));

    let i = interrupted.clone();
    ctrlc::set_handler(move || {
        println!("interrupted!");
        i.store(true, Ordering::SeqCst);
    })
    .wrap_err("error setting Ctrl-C handler")?;

    #[cfg(feature = "vt510")]
    let args = {
        let args = Args::parse();
        println!("starting serving on {}", &args.serial_port);
        args
    };
    #[cfg(feature = "vt510")]
    let backend = {
        use std::time::Duration;
        ratatui_vt510::Vt510Backend::new(
            args.serial_port,
            Duration::from_millis(500),
            57_600,
            80,
            24,
        )?
    };
    #[cfg(not(feature = "vt510"))]
    let backend = {
        use ratatui::crossterm::terminal::enable_raw_mode;
        enable_raw_mode()?;
        CrosstermBackend::new(std::io::stdout())
    };

    let terminal = Terminal::new(backend)?;

    let app = App::new(interrupted, terminal);
    app.run()?;

    #[cfg(not(feature = "vt510"))]
    ratatui::crossterm::terminal::disable_raw_mode()?;
    println!("exiting..");

    Ok(())
}
