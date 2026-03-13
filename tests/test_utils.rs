//! Testing utilities for non-regression tests

use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;
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
/// image loaded from `{reference_png}`. If the images do not match, the test
/// will fail.
///
/// In case of image mismatch, the actual producted image i saved in
/// subdirectory `/outputs/` of the directory containing `{reference_png}`.
///
/// # Arguments
///
/// * `width`: width of the drawing area.
/// * `height`: height of the drawing area.
/// * `reference_png`: path to the reference PNG image for non-regression
///   comparison.
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
    reference_png: impl Into<PathBuf>,
    draw_fn: F,
) -> Result<()>
where
    F: FnOnce(WxBackend<wx::MemoryDC>) -> Result<()> + Send + 'static,
{
    let reference_png = reference_png.into();
    let (output_dir, actual_png) = {
        let output_dir = reference_png
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .join("outputs");
        let actual_png = output_dir.join(
            reference_png
                .file_name()
                .context("Invalid reference_png path: missing filename")?,
        );
        (output_dir, actual_png)
    };

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
                        format!("failed to open {}", reference_png.display())
                    })?,
                ),
                image::ImageFormat::Png,
            )
            .with_context(|| {
                format!("failed to load {}", reference_png.display())
            })?;

            // save actual image for later comparison in case of failure
            fs::create_dir_all(&output_dir).context(format!(
                "failed to create directory {}",
                output_dir.display()
            ))?;
            image
                .save(&actual_png)
                .context(format!("failed to save {}", actual_png.display()))?;

            if expected == image::DynamicImage::ImageRgba8(image.clone()) {
                Ok(())
            } else {
                let message = format!(
                    "ERROR: image mismatch.
Compare the following two files manually, then \
update the reference image if needed.
  reference image: {}
  actual image   : {}
",
                    reference_png.display(),
                    actual_png.display()
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
