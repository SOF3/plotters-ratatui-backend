// Modified from https://github.com/plotters-rs/plotters/blob/master/plotters/examples/mandelbrot.rs

use std::ops::Range;

use plotters::prelude::*;

mod boilerplate;

fn main() -> anyhow::Result<()> {
    boilerplate::main_boilerplate(&[|area| {
        let mut chart = ChartBuilder::on(&area)
            .margin(20)
            .x_label_area_size(10)
            .y_label_area_size(10)
            .build_cartesian_2d(-2.1f64..0.6f64, -1.2f64..1.2f64)?;

        chart.configure_mesh().disable_x_mesh().disable_y_mesh().draw()?;

        let plotting_area = chart.plotting_area();

        let range = plotting_area.get_pixel_range();

        let (pw, ph) = (range.0.end - range.0.start, range.1.end - range.1.start);
        let (xr, yr) = (chart.x_range(), chart.y_range());

        for (x, y, c) in mandelbrot_set(xr, yr, (pw as usize, ph as usize), 100) {
            if c != 100 {
                plotting_area.draw_pixel((x, y), &MandelbrotHSL::get_color(c as f64 / 100.0))?;
            } else {
                plotting_area.draw_pixel((x, y), &BLACK)?;
            }
        }

        Ok(())
    }])
}

fn mandelbrot_set(
    real: Range<f64>,
    complex: Range<f64>,
    samples: (usize, usize),
    max_iter: usize,
) -> impl Iterator<Item = (f64, f64, usize)> {
    let step = (
        (real.end - real.start) / samples.0 as f64,
        (complex.end - complex.start) / samples.1 as f64,
    );
    (0..(samples.0 * samples.1)).map(move |k| {
        let c = (
            real.start + step.0 * (k % samples.0) as f64,
            complex.start + step.1 * (k / samples.0) as f64,
        );
        let mut z = (0.0, 0.0);
        let mut cnt = 0;
        while cnt < max_iter && z.0 * z.0 + z.1 * z.1 <= 1e10 {
            z = (z.0 * z.0 - z.1 * z.1 + c.0, 2.0 * z.0 * z.1 + c.1);
            cnt += 1;
        }
        (c.0, c.1, cnt)
    })
}
