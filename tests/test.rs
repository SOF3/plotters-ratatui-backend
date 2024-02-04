use std::io;
use std::time::Duration;

use crossterm::event::{Event, KeyCode, KeyModifiers};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::{event, ExecutableCommand};
use plotters::coord::Shift;
//use plotters::prelude::{ChartBuilder, DrawingArea, LabelAreaPosition, SeriesLabelPosition, *};
use plotters::prelude::{DrawingArea, *};
//use plotters::series::LineSeries;
//use plotters::style::{IntoTextStyle as _, RGBColor};
use plotters_ratatui_backend::{widget_fn, AreaResult, RatatuiBackend};
use ratatui::prelude::{Alignment, Constraint, CrosstermBackend, Direction, Layout};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph};
use ratatui::Terminal;

// -----------------------------------------------------------------------------

const CHAR_PIXEL_SIZE: (i32, i32) = (2, 4);

// -----------------------------------------------------------------------------

fn quit() -> anyhow::Result<()> {
    io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    eprintln!("User requested to immediately exit tests.");
    std::process::exit(-1);
}

// -----------------------------------------------------------------------------

fn render(
    test_name: &str,
    description: impl ToString,
    draw_fn: impl Fn(DrawingArea<RatatuiBackend<'_, '_>, Shift>) -> AreaResult,
) -> anyhow::Result<bool> {
    let description = description.to_string().replace("\n", "\n    ");
    let question = format!(
        "Does the description:\n\n    '{description}'\n\nmatch what's displayed above?  Press 'y' \
         or 'n'",
    );

    let title = format!(" {test_name} ");

    let para = Paragraph::new(question).alignment(Alignment::Left).block(
        Block::default()
            .title(title)
            .padding(Padding::uniform(1))
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded),
    );

    enable_raw_mode()?;
    io::stdout().execute(EnterAlternateScreen)?;

    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    terminal.draw(|frame| {
        let rects =
            Layout::new(Direction::Vertical, [Constraint::Ratio(4, 5), Constraint::Ratio(1, 5)])
                .split(frame.size());

        frame.render_widget(widget_fn(&draw_fn), rects[0]);
        frame.render_widget(para, rects[1]);
    })?;

    #[allow(unused_assignments)]
    let mut retval = false;

    loop {
        if event::poll(Duration::from_secs(1))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('n') => {
                        retval = false;
                        break;
                    }
                    KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => quit()?,
                    KeyCode::Char('q') => quit()?,
                    KeyCode::Char('y') => {
                        retval = true;
                        break;
                    }
                    _ => (),
                },
                _ => (),
            }
        }
    }

    io::stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;

    Ok(retval)
}

// -----------------------------------------------------------------------------

#[test]
fn test_path_element_line() {
    assert_eq!(
        true,
        render(
            "PathElement thin",
            "A red slash from the upper left to lower right corner.  The line is a single pixel.",
            |area| {
                let (x, y) = area.dim_in_pixel();
                let points = vec![(0, 0), (x as i32, y as i32)];
                area.draw(&PathElement::new(points, RED.stroke_width(1))).unwrap();
                area.present().unwrap();
                Ok(())
            }
        )
        .unwrap()
    );
}

// -----------------------------------------------------------------------------

#[test]
fn test_path_element_block() {
    assert_eq!(
        true,
        render(
            "PathElement thick",
            "A red slash from the upper left to lower right corner, on a white \
             background.\nBecause of the char scope of colors, the line is drawn in jaggy \
             character blocks.",
            |area| {
                let (x, y) = area.dim_in_pixel();
                area.fill(&WHITE).unwrap();
                let points = vec![(0, 0), (x as i32, y as i32)];
                area.draw(&PathElement::new(points, RED.stroke_width(1))).unwrap();
                area.present().unwrap();
                Ok(())
            }
        )
        .unwrap()
    );
}

// -----------------------------------------------------------------------------

#[test]
fn test_circle_unfilled() {
    assert_eq!(
        true,
        render("Unfilled Circle", "A red bordered empty circle", |area| {
            let (x, y) = area.dim_in_pixel();
            let (x, y) = (x as i32 / 2, y as i32 / 2);
            area.draw(&Circle::new((x, y), x.min(y), RED)).unwrap();
            area.present().unwrap();
            Ok(())
        })
        .unwrap()
    );
}

// -----------------------------------------------------------------------------

#[test]
fn test_circle_filled() {
    assert_eq!(
        true,
        render("Filled Circle", "A filled in red circle", |area| {
            let (x, y) = area.dim_in_pixel();
            let (x, y) = (x as i32 / 2, y as i32 / 2);
            area.draw(&Circle::new((x, y), x.min(y), RED.filled())).unwrap();
            area.present().unwrap();
            Ok(())
        })
        .unwrap()
    );
}

// -----------------------------------------------------------------------------

#[test]
fn test_circle_concentric_filled() {
    assert_eq!(
        true,
        render(
            "Concentric Filled Circles",
            "A multicolored series of concentric filled circles.  The outer circle is expected to \
             be a closer approximation to circular, but given the discrete nature of character \
             cells with respect to color, the inner \"circles\" will be extremely jaggy.",
            |area| {
                let (x, y) = area.dim_in_pixel();
                let (x, y) = (x as i32 / 2, y as i32 / 2);

                let colors = [RED, GREEN, BLUE, MAGENTA, CYAN, YELLOW];
                for (r, color) in (1..x.min(y)).step_by(10).rev().zip(colors.iter().cycle()) {
                    area.draw(&Circle::new((x, y), r, color.filled())).unwrap();
                }
                area.present().unwrap();
                Ok(())
            }
        )
        .unwrap()
    );
}

// -----------------------------------------------------------------------------

#[test]
fn test_circle_concentric_unfilled() {
    assert_eq!(
        true,
        render("Concentric Circles", "A series of concentric circles", |area| {
            let (x, y) = area.dim_in_pixel();
            let (x, y) = (x as i32 / 2, y as i32 / 2);
            for r in (1..x.min(y)).step_by(10) {
                area.draw(&Circle::new((x, y), r, RED)).unwrap();
            }
            area.present().unwrap();
            Ok(())
        })
        .unwrap()
    );
}

// -----------------------------------------------------------------------------

#[test]
fn test_square_filled() {
    assert_eq!(
        true,
        render("Filled Rect", "A filled in red square in the centre of the display area", |area| {
            let (x, y) = area.dim_in_pixel();
            let (x, y) = area.map_coordinate(&(x as i32, y as i32));

            let mid = (x as i32 / 2, y as i32 / 2);
            let n = x.min(y) as i32 / 4;

            area.draw(&Rectangle::new(
                [(mid.0 - n, mid.1 - n), (mid.0 + n, mid.1 + n)],
                RED.filled(),
            ))
            .unwrap();

            area.present().unwrap();
            Ok(())
        })
        .unwrap()
    );
}

// -----------------------------------------------------------------------------

#[test]
fn test_square_unfilled() {
    assert_eq!(
        true,
        render(
            "Unfilled Square",
            "A red bordered empty square in the center of the display area",
            |area| {
                let (x, y) = area.dim_in_pixel();
                let (x, y) = area.map_coordinate(&(x as i32, y as i32));

                let n = x.min(y) as i32 / 4;
                let mid = (x as i32 / 2, y as i32 / 2);

                area.draw(&Rectangle::new([(mid.0 - n, mid.1 - n), (mid.0 + n, mid.1 + n)], RED))
                    .unwrap();

                area.present().unwrap();
                Ok(())
            }
        )
        .unwrap()
    );
}

// -----------------------------------------------------------------------------

#[test]
fn test_square_unfilled_inverted_coords() {
    assert_eq!(
        true,
        render(
            "Unfilled squares from inverted coords",
            "A red unfilled square in the center of the display area.\nBlue, green, yellow, and \
             cyan unfilled squares to the upper left, upper right, lower left, and lower right \
             respectively.",
            |area| {
                let (x, y) = area.dim_in_pixel();
                let (x, y) = (x as i32, y as i32);

                let n = x.min(y) / 8;
                let mid = (x / 2, y / 2);
                let ul = (x / 4, y / 4);
                let ur = (x * 3 / 4, y / 4);
                let ll = (x / 4, y * 3 / 4);
                let lr = (x * 3 / 4, y * 3 / 4);

                let rects = [(ul, BLUE), (ur, GREEN), (mid, RED), (ll, YELLOW), (lr, CYAN)];

                for ((x, y), color) in rects {
                    area.draw(&Rectangle::new([(x + n, y + n), (x - n, y - n)], color)).unwrap();
                }

                area.present().unwrap();
                Ok(())
            }
        )
        .unwrap()
    );
}

// -----------------------------------------------------------------------------

#[test]
fn test_square_filled_and_unfilled() {
    assert_eq!(
        true,
        render(
            "Filled and Unfilled squares are same size",
            "Filled green (upper), blue (lower), yellow (left), and cyan (right) \
             squares,\ninterspersed with white bordered unfilled squares.\nTouching edges are the \
             same length.  All enclosed inside a red border.",
            |area| {
                let (x, y) = area.dim_in_pixel();

                // Midpoint of area
                let (x, y) = (x as i32 / 2, y as i32 / 2);

                // Scoot along to char boundary
                let (x, y) = (x + x % CHAR_PIXEL_SIZE.0 + 1, y + y % CHAR_PIXEL_SIZE.1 + 1);

                // Unit length is defined in multiples of char height (4), which conveniently is a multiple of char width (2).
                let n = 3 * CHAR_PIXEL_SIZE.1;

                let base = [(x, y), (x + n - 1, y + n - 1)];

                let t = [(base[0].0, base[0].1 - n), (base[1].0, base[1].1 - n)];
                let b = [(base[0].0, base[0].1 + n), (base[1].0, base[1].1 + n)];
                let l = [(base[0].0 - n, base[0].1), (base[1].0 - n, base[1].1)];
                let r = [(base[0].0 + n, base[0].1), (base[1].0 + n, base[1].1)];

                let tl = [(base[0].0 - n, base[0].1 - n), (base[1].0 - n, base[1].1 - n)];
                let bl = [(base[0].0 - n, base[0].1 + n), (base[1].0 - n, base[1].1 + n)];
                let tr = [(base[0].0 + n, base[0].1 - n), (base[1].0 + n, base[1].1 - n)];
                let br = [(base[0].0 + n, base[0].1 + n), (base[1].0 + n, base[1].1 + n)];

                let border = [(tl[0].0 - 1, tl[0].1 - 1), (br[0].0 + n, br[1].1 + 1)];

                let white = WHITE.stroke_width(1);

                let rects = [
                    (base, white),
                    (t, GREEN.filled()),
                    (b, BLUE.filled()),
                    (l, YELLOW.filled()),
                    (r, CYAN.filled()),
                    (tl, white),
                    (bl, white),
                    (tr, white),
                    (br, white),
                    (border, RED.stroke_width(1)),
                ];

                for (coords, color) in rects {
                    area.draw(&Rectangle::new(coords, color)).unwrap();
                }

                area.present().unwrap();
                Ok(())
            }
        )
        .unwrap()
    );
}

// -----------------------------------------------------------------------------
