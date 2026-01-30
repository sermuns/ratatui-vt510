use clap::Parser;
use color_eyre::{Result, eyre::Context};
use ratatui::Terminal;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

mod app;

use crate::app::App;
#[cfg(feature = "vt510")]
use ratatui_vt510::Vt510Backend;

#[cfg(not(feature = "vt510"))]
use {ratatui::prelude::CrosstermBackend, std::io::stdout};

#[derive(Parser)]
#[command(version)]
struct Args {
    /// where to serve the TUI
    #[cfg(feature = "vt510")]
    serial_port: String,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let _args = Args::parse();

    let interrupted = Arc::new(AtomicBool::new(true));

    let i = interrupted.clone();
    ctrlc::set_handler(move || {
        i.store(false, Ordering::SeqCst);
    })
    .wrap_err("error setting Ctrl-C handler")?;

    #[cfg(feature = "vt510")]
    println!("starting serving on {}", &args.serial_port);
    #[cfg(feature = "vt510")]
    let backend = Vt510Backend::new("/dev/pts/4", Duration::from_millis(10), 115_200, 80, 26)?;
    #[cfg(not(feature = "vt510"))]
    let backend = CrosstermBackend::new(stdout());

    let terminal = Terminal::new(backend)?;

    let app = App::new(interrupted);
    app.run(terminal)?;

    #[cfg(feature = "vt510")]
    println!("exiting..");

    Ok(())
}
