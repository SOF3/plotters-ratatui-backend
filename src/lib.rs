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

const WORD_PIXEL_SIZE: u32 = 4;

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
        coord: BackendCoord,
    ) -> Result {
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
        Ok((text.len() as u32, WORD_PIXEL_SIZE))
    }
}

fn rect_to_size(rect: layout::Rect) -> (u32, u32) {
    (u32::from(rect.width) * WORD_PIXEL_SIZE, u32::from(rect.height) * WORD_PIXEL_SIZE)
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
