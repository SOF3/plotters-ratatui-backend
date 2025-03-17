use std::time::Duration;
use std::{io, iter};

use anyhow::Context as _;
use crossterm::event::{Event, KeyCode, KeyEvent};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, ExecutableCommand};
use flexi_logger::FileSpec;
use plotters::coord;
use plotters::prelude::DrawingArea;
use plotters_ratatui_backend::{widget_fn, AreaResult, RatatuiBackend};
use ratatui::layout::Direction;
use ratatui::prelude::{Constraint, CrosstermBackend, Layout};
use ratatui::Terminal;

pub fn main_boilerplate(
    draw_fns: &[fn(DrawingArea<RatatuiBackend, coord::Shift>) -> AreaResult],
) -> anyhow::Result<()> {
    flexi_logger::Logger::try_with_env()
        .context("parse RUST_LOG")?
        .log_to_file(FileSpec::default())
        .start()
        .context("logger setup")?;

    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    loop {
        terminal.draw(|frame| {
            let rects = Layout::new(
                Direction::Horizontal,
                std::iter::repeat(Constraint::Ratio(1, draw_fns.len() as u32)).take(draw_fns.len()),
            )
            .split(frame.area());
            for (&rect, &draw_fn) in iter::zip(&*rects, draw_fns) {
                frame.render_widget(widget_fn(draw_fn), rect);
            }
        })?;
        if event::poll(Duration::from_secs(1))? {
            if let Event::Key(KeyEvent { code: KeyCode::Char('q'), .. }) = event::read()? {
                break;
            }
        }
    }

    io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(())
}
