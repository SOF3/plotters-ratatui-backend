// Modified from https://github.com/plotters-rs/plotters/blob/master/plotters/examples/normal-dist.rs

use plotters::prelude::*;
use rand::SeedableRng;
use rand_distr::{Distribution, Normal};
use rand_xorshift::XorShiftRng;

mod boilerplate;

fn main() -> anyhow::Result<()> {
    boilerplate::main_boilerplate(&[|root| {
        let sd = 0.13;

        let random_points: Vec<(f64, f64)> = {
            let norm_dist = Normal::new(0.5, sd).unwrap();
            let mut x_rand = XorShiftRng::from_seed(*b"MyFragileSeed123");
            let mut y_rand = XorShiftRng::from_seed(*b"MyFragileSeed321");
            let x_iter = norm_dist.sample_iter(&mut x_rand);
            let y_iter = norm_dist.sample_iter(&mut y_rand);
            x_iter.zip(y_iter).take(5000).collect()
        };

        let areas = root.split_by_breakpoints([944], [80]);

        let mut x_hist_ctx = ChartBuilder::on(&areas[0])
            .y_label_area_size(40)
            .build_cartesian_2d((0.0..1.0).step(0.01).use_round().into_segmented(), 0..250)?;
        let mut y_hist_ctx = ChartBuilder::on(&areas[3])
            .x_label_area_size(40)
            .build_cartesian_2d(0..250, (0.0..1.0).step(0.01).use_round())?;
        let mut scatter_ctx = ChartBuilder::on(&areas[2])
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(0f64..1f64, 0f64..1f64)?;
        scatter_ctx.configure_mesh().disable_x_mesh().disable_y_mesh().draw()?;
        scatter_ctx.draw_series(
            random_points.iter().map(|(x, y)| Circle::new((*x, *y), 2, GREEN.filled())),
        )?;
        let x_hist = Histogram::vertical(&x_hist_ctx)
            .style(GREEN.filled())
            .margin(0)
            .data(random_points.iter().map(|(x, _)| (*x, 1)));
        let y_hist = Histogram::horizontal(&y_hist_ctx)
            .style(GREEN.filled())
            .margin(0)
            .data(random_points.iter().map(|(_, y)| (*y, 1)));
        x_hist_ctx.draw_series(x_hist)?;
        y_hist_ctx.draw_series(y_hist)?;

        Ok(())
    }])
}
