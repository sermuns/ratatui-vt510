use ratatui::{
    backend::{Backend, ClearType, WindowSize},
    buffer::Cell,
    layout::{Position, Size},
};
use serialport::SerialPort;
use std::{
    io::{self, Write},
    time::Duration,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SerialBackendError {
    #[error("serial backend IO error: {0}")]
    Io(#[from] io::Error),
}

pub struct Vt510Backend {
    port: Box<dyn SerialPort>,
    size: Size,
}

impl Vt510Backend {
    pub fn new(
        serial_port_path: impl AsRef<str>,
        timeout: Duration,
        baud_rate: u32,
        width: u16,
        height: u16,
    ) -> Result<Self, serialport::Error> {
        let port = serialport::new(serial_port_path.as_ref(), baud_rate)
            .timeout(timeout)
            .open()?;
        Ok(Self {
            port,
            size: Size::new(width, height),
        })
    }

    fn write_raw(&mut self, s: &str) -> Result<(), SerialBackendError> {
        self.port.write_all(s.as_bytes())?;
        Ok(())
    }

    fn move_cursor(&mut self, row: u16, col: u16) -> Result<(), SerialBackendError> {
        // https://vt100.net/docs/vt510-rm/CUP.html
        self.write_raw(&format!("\x1B[{};{}H", row, col))
    }
}

impl Backend for Vt510Backend {
    type Error = SerialBackendError;

    fn draw<'a, I>(&mut self, content: I) -> Result<(), Self::Error>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        for (x, y, cell) in content {
            self.move_cursor(y + 1, x + 1)?;

            let ch = cell.symbol().chars().next().unwrap_or(' ');
            self.write_raw(&ch.to_string())?;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        self.port.flush()?;
        Ok(())
    }

    fn hide_cursor(&mut self) -> Result<(), Self::Error> {
        // https://vt100.net/docs/vt510-rm/DECTCEM.html
        self.write_raw("\x1B[?25l")
    }

    fn show_cursor(&mut self) -> Result<(), Self::Error> {
        // https://vt100.net/docs/vt510-rm/DECTCEM.html
        self.write_raw("\x1B[?25h")
    }

    fn set_cursor_position<P>(&mut self, position: P) -> Result<(), Self::Error>
    where
        P: Into<Position>,
    {
        let pos = position.into();
        self.move_cursor(pos.y + 1, pos.x + 1)
    }

    // NOTE: could be implemented with https://vt100.net/docs/vt510-rm/DSR-CPR.html
    fn get_cursor_position(&mut self) -> Result<Position, Self::Error> {
        Err(SerialBackendError::Io(io::Error::new(
            io::ErrorKind::Unsupported,
            "cursor position query not yet implemented",
        )))
    }

    fn clear(&mut self) -> Result<(), Self::Error> {
        // https://vt100.net/docs/vt510-rm/ED.html
        self.write_raw("\x1B[2J\x1B[H")
    }

    fn clear_region(&mut self, clear_type: ClearType) -> Result<(), Self::Error> {
        if let ClearType::All = clear_type {
            self.clear()?;
        }
        Ok(())
    }

    fn size(&self) -> Result<Size, Self::Error> {
        Ok(self.size)
    }

    fn window_size(&mut self) -> Result<WindowSize, Self::Error> {
        Ok(WindowSize {
            columns_rows: self.size,
            pixels: Size::new(0, 0), // pixels are ignored for serial VT510
        })
    }
}
