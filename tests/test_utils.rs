//! Testing utilities for non-regression tests

use std::fs;
use std::io;
use std::process;

use anyhow::{Context, Result};
use image::RgbaImage;
use plotters_wxdragon::WxBackend;
use wxdragon::{self as wx};

/// Executes a plotter drawing function and compares the output to an expected
/// image file.
///
/// This function sets up a wxWidgets `MemoryDC`, draws on it using the
/// provided `draw_fn`, then compares the resulting bitmap with a reference
/// image loaded from `{path_root}.png`. If the images do not match, the test
/// will fail.
///
/// # Arguments
///
/// * `width`: width of the drawing area.
/// * `height`: height of the drawing area.
/// * `path_root`: used to build the path `"{path_root}.png"` to the reference
///   PNG image for non-regression comparison.
/// * `draw_fn`: closure that performs the drawing operations.
///
/// # Returns
///
/// Returns `Ok(())` if the drawing and comparison are successful, or an `Err`
/// if any part of the process fails (e.g., image loading, drawing errors,
/// image mismatch).
pub fn run_plotters_image_test<F>(
    width: u32,
    height: u32,
    path_root: &str,
    draw_fn: F,
) -> Result<()>
where
    F: FnOnce(WxBackend<wx::MemoryDC>) -> Result<()> + Send + 'static,
{
    let reference_png = format!("{path_root}.png");
    let actual_png = format!("{path_root}_actual.png"); // saved if mismatch
    let _ = wx::main(move |_| {
        let result = (|| -> Result<()> {
            // setup the backend with an empty bitmap
            let mut bitmap = wx::Bitmap::new(width as i32, height as i32)
                .context("failed to create bitmap")?;
            let mut dc = wx::MemoryDC::new();
            dc.select_object(&mut bitmap);
            let backend = WxBackend::new(&dc);

            // draw with user-provided closure
            draw_fn(backend).context("error while drawing")?;

            // convert to an image for comparison
            dc.select_object(&mut wx::Bitmap::null_bitmap());
            let rgba_data = bitmap
                .get_rgba_data()
                .context("failed to obtain image rgba data")?;
            anyhow::ensure!(
                rgba_data.len() == (width * height * 4) as usize,
                "RGBA data length mismatch"
            );
            let image = RgbaImage::from_raw(width, height, rgba_data)
                .context("failed to create RgbaImage from bitmap")?;

            // non-regression comparison
            let expected = image::load(
                io::BufReader::new(
                    fs::File::open(&reference_png).with_context(|| {
                        format!("failed to open {reference_png}")
                    })?,
                ),
                image::ImageFormat::Png,
            )
            .with_context(|| "failed to load {reference_png}")?;
            if expected == image::DynamicImage::ImageRgba8(image.clone()) {
                Ok(())
            } else {
                // regenerate expected image by uncommenting this and
                // commenting the non-regression part
                image
                    .save(&actual_png)
                    .context("failed to save {actual_png}")?;
                let message = format!(
                    "ERROR: image mismatch.
Compare the following two files manually, then \
update the reference image if needed.
  reference image: {reference_png}
  actual image   : {actual_png}
"
                );
                anyhow::bail!(message)
            }
        })();
        if let Err(e) = result {
            panic!("{}", error_chain_string(&*e.into_boxed_dyn_error()));
        }
        process::exit(0);
    });
    Ok(())
}

// Helper to get the full error chain string
fn error_chain_string(err: &dyn std::error::Error) -> String {
    let mut messages = Vec::new();
    let mut current: Option<&dyn std::error::Error> = Some(err);
    while let Some(e) = current {
        messages.push(e.to_string());
        current = e.source();
    }
    messages.join("\n -> ")
}
