use plotters::prelude::{ChartBuilder, LabelAreaPosition, SeriesLabelPosition};
use plotters::series::LineSeries;
use plotters::style::{IntoTextStyle as _, RGBColor};

mod boilerplate;

fn main() -> anyhow::Result<()> {
    boilerplate::main_boilerplate(&[|area| {
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
    }])
}
