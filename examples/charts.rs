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
use plotters::prelude::{ChartBuilder, DrawingArea, LabelAreaPosition, SeriesLabelPosition};
use plotters::series::LineSeries;
use plotters::style::{IntoTextStyle as _, RGBColor};
use plotters_ratatui_backend::{widget_fn, AreaResult, RatatuiBackend};
use ratatui::prelude::{Constraint, CrosstermBackend, Layout};
use ratatui::Terminal;

fn main() -> anyhow::Result<()> {
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
            let rects = Layout::new()
                .constraints([Constraint::Ratio(1, DRAW_FNS.len() as u32); DRAW_FNS.len()])
                .split(frame.size());
            for (&rect, &draw_fn) in iter::zip(&*rects, DRAW_FNS) {
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

const DRAW_FNS: &[fn(DrawingArea<RatatuiBackend, coord::Shift>) -> AreaResult] = &[|area| {
    let mut chart = ChartBuilder::on(&area)
        .margin(10)
        .margin_left(30)
        .caption("Example 1", "".into_text_style(&area).with_color(plotters::style::WHITE))
        .set_label_area_size(LabelAreaPosition::Left, 1)
        .set_label_area_size(LabelAreaPosition::Bottom, 1)
        .build_cartesian_2d(-1.0..1.0, -1.0..1.0)?;
    chart
        .draw_series(LineSeries::new(
            (-100..=100).map(|x| (f64::from(x) / 100.0, (f64::from(x) / 10.0).cos())),
            plotters::style::RED,
        ))?
        .label("foo");
    chart
        .draw_series(LineSeries::new(
            (-100..=100).map(|x| (f64::from(x) / 100.0, (f64::from(x) / 10.).powi(2))),
            plotters::style::YELLOW,
        ))?
        .label("bar");
    chart
        .configure_mesh()
        .disable_mesh()
        .axis_style(plotters::style::GREEN)
        .axis_desc_style("".with_color(plotters::style::WHITE))
        .label_style("".with_color(RGBColor(200, 130, 120)))
        .draw()?;
    chart
        .configure_series_labels()
        .position(SeriesLabelPosition::UpperLeft)
        .border_style(plotters::style::CYAN)
        .label_font("".into_text_style(&area).with_color(plotters::style::MAGENTA))
        .background_style(plotters::style::WHITE)
        .legend_area_size(20)
        .draw()?;
    area.present()
}];
