use plotters::coord;
use plotters::prelude::{DrawingArea, DrawingAreaErrorKind, IntoDrawingArea};
use ratatui::layout;
use ratatui::prelude::Buffer;
use ratatui::widgets::canvas::Canvas;
use ratatui::widgets::Widget;

use crate::{Error, RatatuiBackend};

pub type AreaResult<T = ()> = Result<T, DrawingAreaErrorKind<Error>>;

/// A ratatui widget that draws a plotters chart.
pub struct PlottersWidget<D, E> {
    pub draw:          D,
    pub error_handler: E,
}

/// Creates a [ratatui widget](Widget) for drawing a plotters chart.
pub fn widget_fn(
    draw_fn: impl Fn(DrawingArea<RatatuiBackend, coord::Shift>) -> AreaResult,
) -> PlottersWidget<impl Draw, impl Fn(DrawingAreaErrorKind<Error>)> {
    PlottersWidget {
        draw:          DrawFn(draw_fn),
        error_handler: |err| log::error!("Plotters draw error: {err:?}"),
    }
}

pub struct DrawFn<F>(pub F);

impl<F: Fn(DrawingArea<RatatuiBackend, coord::Shift>) -> AreaResult> Draw for DrawFn<F> {
    fn draw(&self, b: DrawingArea<RatatuiBackend, coord::Shift>) -> AreaResult { (self.0)(b) }
}

pub trait Draw {
    fn draw(&self, b: DrawingArea<RatatuiBackend, coord::Shift>) -> AreaResult;
}

impl<D: Draw, E: Fn(DrawingAreaErrorKind<Error>)> Widget for PlottersWidget<D, E> {
    fn render(self, area: layout::Rect, buf: &mut Buffer) {
        let (size_x, size_y) = super::rect_to_size(area);
        log::error!("size: {area:?}, {size_x}, {size_y}");
        let canvas =
            Canvas::default().x_bounds([0.0, 1.0]).y_bounds([0.0, 1.0]).paint(move |canvas| {
                let backend = RatatuiBackend { canvas, size: area };
                let area = backend.into_drawing_area();
                if let Err(err) = self.draw.draw(area) {
                    (self.error_handler)(err);
                }
            });
        canvas.render(area, buf)
    }
}
