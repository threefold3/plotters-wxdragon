//! Example of drawing a $y=x^2$ function.
//!
//! This example shows basic integration between Plotters and wxDragon to
//! display a simple plot on an empty frame.
//!
//! This example is inspired by wxdragon example `dc_example`, inside which I
//! put the plotters curve example.

// We leave the glob import of plotters so as not to change the example.
use plotters::prelude::*;

// No glob import for wxdragon to avoid conflicts, but an alias `wx`, and
// import traits as needed.
use plotters_wxdragon::WxBackend;
use wxdragon::{self as wx, WindowEvents, WxWidget};

struct DrawingPanel {
    panel: wx::Panel,
}

impl DrawingPanel {
    fn new(parent: &wx::Frame) -> Self {
        let panel = wx::PanelBuilder::new(parent).build();
        panel.set_background_style(wx::BackgroundStyle::Paint);

        // Register the paint handler with a move closure
        panel.on_paint(move |_event| {
            // Create a PaintDC when handling paint events
            let dc = wx::AutoBufferedPaintDC::new(&panel);

            // Create a backend for plotters
            let backend = WxBackend::new(&dc).into_drawing_area();

            // Create a plotters plot as you would with any other backend
            let mut chart = ChartBuilder::on(&backend)
                .caption("y=x^2", ("sans-serif", 50).into_font())
                .margin(5)
                .x_label_area_size(30)
                .y_label_area_size(30)
                .build_cartesian_2d(-1f32..1f32, -0.1f32..1f32)
                .expect("plot grid");

            chart.configure_mesh().draw().expect("plot draw");

            chart
                .draw_series(LineSeries::new(
                    (-50..=50).map(|x| x as f32 / 50.0).map(|x| (x, x * x)),
                    &RED,
                ))
                .expect("draw series")
                .label("y = x^2")
                .legend(|(x, y)| {
                    PathElement::new(vec![(x, y), (x + 20, y)], RED)
                });

            chart
                .configure_series_labels()
                .background_style(WHITE.mix(0.8))
                .border_style(BLACK)
                .draw()
                .expect("configure labels");

            backend.present().expect("present");
        });

        // Also handle SIZE events to refresh when the window size changes
        panel.on_size(move |_event| {
            // Force a repaint when window size changes
            panel.refresh(true, None);
        });

        Self { panel }
    }
}

impl std::ops::Deref for DrawingPanel {
    type Target = wx::Panel;

    fn deref(&self) -> &Self::Target {
        &self.panel
    }
}

fn main() {
    let _ = wxdragon::main(|_| {
        let frame = wx::Frame::builder()
            .with_title("Plotters example y=x^2")
            .with_size(wx::Size::new(800, 600))
            .with_position(wx::Point::new(100, 100))
            .build();

        let drawing_panel = DrawingPanel::new(&frame);

        // Initial paint
        drawing_panel.refresh(true, None);

        frame.show(true);
    });
}
