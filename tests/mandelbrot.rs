//! Mandelbrot example from plotters
//!
//! This tests the `draw_pixel()` method with a complex plot.

use std::ops::Range;
use std::process::exit;

use image::RgbaImage;
use plotters::prelude::*;
use plotters_wxdragon::WxBackend;
use wxdragon::{self as wx, DeviceContext};

#[test]
fn test_mandelbrot() {
    let _ = wx::main(|_| {
        let width: u32 = 800;
        let height: u32 = 600;
        let mut bitmap = wx::Bitmap::new(width as i32, height as i32).unwrap();
        let mut dc = wx::MemoryDC::new();
        dc.select_object(&mut bitmap);
        let backend = WxBackend::new(&dc);
        draw_mandelbrot(backend).unwrap();
        dc.select_object(&mut wx::Bitmap::null_bitmap());
        let rgba_data = bitmap.get_rgba_data().unwrap();
        assert_eq!(rgba_data.len(), (width * height * 4) as usize);
        let image = RgbaImage::from_raw(width, height, rgba_data).unwrap();
        image.save("tests/mandelbrot.png").unwrap();
        exit(0)
    });
}

fn draw_mandelbrot<C: DeviceContext>(
    backend: WxBackend<C>,
) -> Result<(), Box<dyn std::error::Error>> {
    let root_area = backend.into_drawing_area();

    root_area.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root_area)
        .margin(20)
        .x_label_area_size(10)
        .y_label_area_size(10)
        .build_cartesian_2d(-2.1f64..0.6f64, -1.2f64..1.2f64)?;

    chart
        .configure_mesh()
        .disable_x_mesh()
        .disable_y_mesh()
        .draw()?;

    let plotting_area = chart.plotting_area();

    let range = plotting_area.get_pixel_range();

    let (pw, ph) = (range.0.end - range.0.start, range.1.end - range.1.start);
    let (xr, yr) = (chart.x_range(), chart.y_range());

    for (x, y, c) in mandelbrot_set(xr, yr, (pw as usize, ph as usize), 100) {
        if c != 100 {
            plotting_area.draw_pixel(
                (x, y),
                &MandelbrotHSL::get_color(c as f64 / 100.0),
            )?;
        } else {
            plotting_area.draw_pixel((x, y), &BLACK)?;
        }
    }

    root_area.present().expect("present");
    Ok(())
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
