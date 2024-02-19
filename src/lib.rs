use plotters_backend::{
    BackendColor, BackendCoord, BackendTextStyle, DrawingBackend, DrawingErrorKind,
};
use ratatui::style::{Color as RataColor, Style as RataStyle};
use ratatui::widgets::canvas;
use ratatui::{layout, text};

#[cfg(feature = "widget")]
mod widget;
#[cfg(feature = "widget")]
pub use widget::*;

pub const CHAR_PIXEL_SIZE: u32 = 4;

pub struct RatatuiBackend<'a, 'b> {
    pub canvas: &'a mut canvas::Context<'b>,
    pub size:   layout::Rect,
}

impl<'a, 'b> DrawingBackend for RatatuiBackend<'a, 'b> {
    type ErrorType = Error;

    fn get_size(&self) -> (u32, u32) { rect_to_size(self.size) }

    fn ensure_prepared(&mut self) -> Result { Ok(()) }

    fn present(&mut self) -> Result { Ok(()) }

    fn draw_pixel(&mut self, coord: BackendCoord, color: BackendColor) -> Result {
        self.canvas.draw(&canvas::Points {
            coords: &[backend_to_canvas_coords(coord, self.size)],
            color:  convert_color(color),
        });
        Ok(())
    }

    fn draw_text<TStyle: BackendTextStyle>(
        &mut self,
        text: &str,
        style: &TStyle,
        mut coord: BackendCoord,
    ) -> Result {
        let width = text.chars().count();
        coord.0 -= (width as u32 * CHAR_PIXEL_SIZE / 2) as i32;

        let (x, y) = backend_to_canvas_coords(coord, self.size);
        self.canvas.print(x, y, text::Line::styled(text.to_string(), convert_style(style)));
        Ok(())
    }

    fn draw_line<S: plotters_backend::BackendStyle>(
        &mut self,
        coord1: BackendCoord,
        coord2: BackendCoord,
        style: &S,
    ) -> std::result::Result<(), DrawingErrorKind<Self::ErrorType>> {
        let (x1, y1) = backend_to_canvas_coords(coord1, self.size);
        let (x2, y2) = backend_to_canvas_coords(coord2, self.size);

        self.canvas.draw(&canvas::Line::new(x1, y1, x2, y2, convert_color(style.color())));
        Ok(())
    }

    fn draw_circle<S: plotters_backend::BackendStyle>(
        &mut self,
        coord: BackendCoord,
        radius: u32,
        style: &S,
        _fill: bool,
    ) -> std::result::Result<(), DrawingErrorKind<Self::ErrorType>> {
        let (x, y) = backend_to_canvas_coords(coord, self.size);
        self.canvas.draw(&canvas::Circle {
            x,
            y,
            radius: radius.into(),
            color: convert_color(style.color()),
        });
        Ok(())
    }

    fn draw_rect<S: plotters_backend::BackendStyle>(
        &mut self,
        coord1: BackendCoord,
        coord2: BackendCoord,
        style: &S,
        _fill: bool,
    ) -> std::result::Result<(), DrawingErrorKind<Self::ErrorType>> {
        let (x1, y1) = backend_to_canvas_coords(coord1, self.size);
        let (x2, y2) = backend_to_canvas_coords(coord2, self.size);

        self.canvas.draw(&canvas::Rectangle {
            x:      x1.min(x2),
            y:      y1.min(y2),
            width:  (x2 - x1).abs(),
            height: (y2 - y1).abs(),
            color:  convert_color(style.color()),
        });
        Ok(())
    }

    fn estimate_text_size<TStyle: BackendTextStyle>(
        &self,
        text: &str,
        _style: &TStyle,
    ) -> std::result::Result<(u32, u32), DrawingErrorKind<Self::ErrorType>> {
        Ok((text.chars().count() as u32 * CHAR_PIXEL_SIZE, CHAR_PIXEL_SIZE))
    }
}

fn rect_to_size(rect: layout::Rect) -> (u32, u32) {
    (u32::from(rect.width) * CHAR_PIXEL_SIZE, u32::from(rect.height) * CHAR_PIXEL_SIZE)
}

fn backend_to_canvas_coords((x, y): BackendCoord, rect: layout::Rect) -> (f64, f64) {
    let (width, height) = rect_to_size(rect);

    let x = f64::from(x) / f64::from(width);
    let mut y = f64::from(y) / f64::from(height);
    y = 1. - y;
    (x, y)
}

fn convert_color(color: BackendColor) -> RataColor {
    RataColor::Rgb(color.rgb.0, color.rgb.1, color.rgb.2)
}
fn convert_style(style: &impl BackendTextStyle) -> RataStyle {
    RataStyle::default().fg(convert_color(style.color()))
}

pub type Result<T = ()> = std::result::Result<T, DrawingErrorKind<Error>>;

#[derive(Debug, thiserror::Error)]
pub enum Error {}

// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use plotters_backend::DrawingBackend;
    use ratatui::{
        assert_buffer_eq,
        buffer::Buffer,
        layout::Rect,
        symbols::Marker,
        widgets::{canvas::Canvas, Widget},
    };

    use crate::{convert_color, BackendColor, RatatuiBackend};

    struct RectTest<'a> {
        start: (i32, i32),
        stop: (i32, i32),
        fill: bool,
        expected: Vec<&'a str>,
    }

    fn rect_test(test: &RectTest) {
        println!("rect_test:  start: {:?}  stop: {:?}  fill: {:?}", test.start, test.stop, test.fill);
        let mut buffer = Buffer::empty(Rect::new(0, 0, 10, 10));
        let red = BackendColor { alpha: 1.0, rgb: (0xff, 0x00, 0x00) };
        let canvas = Canvas::default()
            .marker(Marker::Braille)
            .x_bounds([0.0, 1.0])
            .y_bounds([0.0, 1.0])
            .paint(|context| {
                let mut backend = RatatuiBackend { canvas: context, size: Rect::new(0, 0, 10, 10) };
                backend.draw_rect(test.start, test.stop, &red, test.fill).unwrap();
            });
        canvas.render(buffer.area, &mut buffer);

        let mut expected = Buffer::with_lines(test.expected.clone());
        let fg = convert_color(red);
        // Set the fg color for non-space elements
        // Is there a less convoluted way to achieve the following?
        for i in expected
            .content
            .iter()
            .enumerate()
            .filter(|(_, cell)| match cell.symbol() {
                " " => false,
                _ => true,
            })
            .map(|(i, _)| i)
            .collect::<Vec<usize>>()
        {
            let pos = expected.pos_of(i);
            expected.get_mut(pos.0, pos.1).set_fg(fg);
        }
        assert_buffer_eq!(buffer, expected);
    }

    // Why are the outputs for ((0,0),(0,0)) and ((0,0),(1,1)) and ((1,1),(1,1)) identical?
    #[test]
    fn test_rects() {
        let tests: [RectTest; 20] = [
            RectTest {
                start: (0, 0),
                stop: (0, 0),
                fill: false,
                expected: vec![
                    "⠁         ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                ],
            },
            RectTest {
                start: (0, 0),
                stop: (1, 1),
                fill: false,
                expected: vec![
                    "⠁         ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                ],
            },
            RectTest {
                start: (1, 1),
                stop: (0, 0),
                fill: false,
                expected: vec![
                    "⠁         ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                ],
            },
            RectTest {
                start: (1, 1),
                stop: (1, 1),
                fill: false,
                expected: vec![
                    "⠁         ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                ],
            },
            RectTest {
                start: (1, 1),
                stop: (1, 1),
                fill: true,
                expected: vec![
                    "⠁         ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                ],
            },
            RectTest {
                start: (1, 1),
                stop: (2, 2),
                fill: false,
                expected: vec![
                    "⠛         ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                ],
            },
            RectTest {
                start: (1, 1),
                stop: (2, 2),
                fill: true,
                expected: vec![
                    "⠛         ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                ],
            },
            RectTest {
                start: (0, 0),
                stop: (2, 4),
                fill: false,
                expected: vec![
                    "⣿         ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                ],
            },
            RectTest {
                start: (0, 0),
                stop: (2, 4),
                fill: true,
                expected: vec![
                    "⣿         ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                ],
            },
            RectTest {
                start: (2, 4),
                stop: (3, 7),
                fill: false,
                expected: vec![
                    "⢀⡀        ",
                    "⠸⠇        ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                ],
            },
            RectTest {
                start: (2, 4),
                stop: (3, 7),
                fill: true,
                expected: vec![
                    "⢀⡀        ",
                    "⠸⠇        ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                ],
            },
            RectTest {
                start: (2, 4),
                stop: (5, 9),
                fill: false,
                expected: vec![
                    "⢀⣀⡀       ",
                    "⢸ ⡇       ",
                    "⠈⠉⠁       ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                ],
            },
            RectTest {
                start: (5, 9),
                stop: (2, 4),
                fill: false,
                expected: vec![
                    "⢀⣀⡀       ",
                    "⢸ ⡇       ",
                    "⠈⠉⠁       ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                ],
            },
            RectTest {
                start: (2, 4),
                stop: (5, 9),
                fill: true,
                expected: vec![
                    "⢀⣀⡀       ",
                    "⢸⣿⡇       ",
                    "⠈⠉⠁       ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                ],
            },
            RectTest {
                start: (5, 9),
                stop: (2, 4),
                fill: true,
                expected: vec![
                    "⢀⣀⡀       ",
                    "⢸⣿⡇       ",
                    "⠈⠉⠁       ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                    "          ",
                ],
            },
            RectTest {
                start: (0, 0),
                stop: (20, 40),
                fill: false,
                expected: vec![
                    "⡏⠉⠉⠉⠉⠉⠉⠉⠉⢹",
                    "⡇        ⢸",
                    "⡇        ⢸",
                    "⡇        ⢸",
                    "⡇        ⢸",
                    "⡇        ⢸",
                    "⡇        ⢸",
                    "⡇        ⢸",
                    "⡇        ⢸",
                    "⣇⣀⣀⣀⣀⣀⣀⣀⣀⣸",
                ],
            },
            RectTest {
                start: (1, 1),
                stop: (20, 40),
                fill: false,
                expected: vec![
                    "⡏⠉⠉⠉⠉⠉⠉⠉⠉⢹",
                    "⡇        ⢸",
                    "⡇        ⢸",
                    "⡇        ⢸",
                    "⡇        ⢸",
                    "⡇        ⢸",
                    "⡇        ⢸",
                    "⡇        ⢸",
                    "⡇        ⢸",
                    "⣇⣀⣀⣀⣀⣀⣀⣀⣀⣸",
                ],
            },
            RectTest {
                start: (1, 1),
                stop: (20, 40),
                fill: true,
                expected: vec![
                    "⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿",
                    "⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿",
                    "⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿",
                    "⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿",
                    "⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿",
                    "⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿",
                    "⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿",
                    "⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿",
                    "⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿",
                    "⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿",
                ],
            },
            RectTest {
                start: (2, 2),
                stop: (19, 39),
                fill: false,
                expected: vec![
                    "⢰⠒⠒⠒⠒⠒⠒⠒⠒⡆",
                    "⢸        ⡇",
                    "⢸        ⡇",
                    "⢸        ⡇",
                    "⢸        ⡇",
                    "⢸        ⡇",
                    "⢸        ⡇",
                    "⢸        ⡇",
                    "⢸        ⡇",
                    "⠸⠤⠤⠤⠤⠤⠤⠤⠤⠇",
                ],
            },
            RectTest {
                start: (2, 2),
                stop: (19, 39),
                fill: true,
                expected: vec![
                    "⢰⣶⣶⣶⣶⣶⣶⣶⣶⡆",
                    "⢸⣿⣿⣿⣿⣿⣿⣿⣿⡇",
                    "⢸⣿⣿⣿⣿⣿⣿⣿⣿⡇",
                    "⢸⣿⣿⣿⣿⣿⣿⣿⣿⡇",
                    "⢸⣿⣿⣿⣿⣿⣿⣿⣿⡇",
                    "⢸⣿⣿⣿⣿⣿⣿⣿⣿⡇",
                    "⢸⣿⣿⣿⣿⣿⣿⣿⣿⡇",
                    "⢸⣿⣿⣿⣿⣿⣿⣿⣿⡇",
                    "⢸⣿⣿⣿⣿⣿⣿⣿⣿⡇",
                    "⠸⠿⠿⠿⠿⠿⠿⠿⠿⠇",
                ],
            },
        ];
        for test in tests {
            rect_test(&test);
        }
    }
}
