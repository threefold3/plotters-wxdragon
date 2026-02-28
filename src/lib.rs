//! `plotters-wxdragon` is a backend for [Plotters], to draw plots in a GUI window
//! managed by [wxDragon].
//!
//! [wxDragon] is a binding of wxWidgets for rust. wxWidgets is a cross-platform
//! GUI library toolkit for desktop applications, that uses the native GUI toolkit
//! on Windows, macOS, and Linux.
//!
//! [Plotters] is a Rust drawing library focusing on data plotting for both WASM
//! and native applications.
//!
//! [Plotters]: https://crates.io/crates/plotters
//! [wxDragon]: https://crates.io/crates/wxdragon
//!
//! ## Getting started
//!
//! ### Quick start
//!
//! 1. Follow [wxDragon] instructions to install the `wxdragon` crate. You will
//!    need the wxWidgets library for your OS so be sure to follow the
//!    instructions.
//!
//! 2. Clone this repository and run the `x2` plotting example to check that it
//!    works for you.
//!
//!    ```bash
//!    git clone https://github.com/threefold3/plotters-wxdragon.git
//!    cd plotters-wxdragon
//!    cargo run --example x2
//!    ```
//!
//!    This will open a new window displaying a simple `y=x^2` plot.
//!
//! ### Integrating with a project
//!
//! 1. Add the following to your `Cargo.toml` file:
//!
//!    ```toml
//!    [dependencies]
//!    wxdragon = "0.9"
//!    plotters = "0.3"
//!    plotters-wxdragon = "0.1"
//!    ```
//!
//! 2. Create an app with a `wxdragon::Panel`, and use the panel's `on_paint`
//!    handler to create a new `wxdragon::AutoBufferedPaintDC` (device context)
//!    each time, wrap it in a `WxBackend` and then draw on it.
//!
//!    ```rust
//!    use plotters::prelude::*;
//!    use plotters::style::text_anchor::{HPos, Pos, VPos};
//!    use plotters_wxdragon::WxBackend;
//!    use wxdragon::{self as wx, WindowEvents, WxWidget};
//!
//!    struct DrawingPanel {
//!        panel: wx::Panel,
//!    }
//!
//!    impl DrawingPanel {
//!        fn new(parent: &wx::Frame) -> Self {
//!            let panel = wx::PanelBuilder::new(parent).build();
//!            panel.set_background_style(wx::BackgroundStyle::Paint);
//!
//!            // Register the paint handler with a move closure
//!            panel.on_paint(move |_event| {
//!                // Create a device context (wxdragon has several types)
//!                // and a plotters backend
//!                let dc = wx::AutoBufferedPaintDC::new(&panel);
//!                let mut backend = WxBackend::new(&dc);
//!
//!                // Create a plotters plot as you would with any other backend
//!                backend
//!                    .draw_rect((300, 250), (500, 350), &BLACK, false)
//!                    .unwrap();
//!                let style = TextStyle::from(("monospace", 32.0).into_font())
//!                    .pos(Pos::new(HPos::Center, VPos::Center));
//!                backend
//!                    .draw_text("hello, world", &style, (400, 300))
//!                    .unwrap();
//!
//!                // Call present() when you are ready
//!                backend.present().expect("present");
//!            });
//!
//!            // Handle SIZE events to refresh when the window size changes
//!            panel.on_size(move |_event| {
//!                panel.refresh(true, None);
//!            });
//!
//!            Self { panel }
//!        }
//!    }
//!
//!    impl std::ops::Deref for DrawingPanel {
//!        type Target = wx::Panel;
//!
//!        fn deref(&self) -> &Self::Target {
//!            &self.panel
//!        }
//!    }
//!
//!    fn main() {
//!        let _ = wxdragon::main(|_| {
//!            let frame = wx::Frame::builder()
//!                .with_title("Getting started")
//!                // with this, wx produces a canvas of size 800 x 600
//!                .with_size(wx::Size::new(852, 689))
//!                .build();
//!
//!            let drawing_panel = DrawingPanel::new(&frame);
//!
//!            // Initial paint
//!            drawing_panel.refresh(true, None);
//!
//!            frame.show(true);
//!        });
//!    }
//!    ```
//!
//!    You can find more details in the examples for how to integrate a plot in
//!    your wxWidgets application:
//!    + `x2`: simple `y=x^2` plot in a wxWidgets frame
//!    + `text`: a single window that shows various text orientations and a
//!      toolbar that can modify application state
//!
//!    See also the existing tests, which illustrate that most existing
//!    plotters examples work without change. In these tests we write to to an
//!    in-memory device context instead of a device context linked to a
//!    `Panel`, and compare to a reference png images to ensure non-regression.
//!
//! ## How this works
//!
//! This crate implements a backend for [Plotters]. It uses the existing drawing
//! context of wxWidgets, and maps plotters drawing primitives to corresponding
//! calls fo the wxWidgets API.
//!
//! See also [`plotters-backend`] for reference on implementing a backend for
//! plotters.
//!
//! [`plotters-backend`]: https://docs.rs/plotters-backend/latest/plotters_backend/
//!
//! ## License
//!
//! This project is dual-licensed under [Apache 2.0](./LICENSE-APACHE) and
//! [`MIT`](./LICENSE-MIT) terms.

use plotters_backend::{
    BackendColor, DrawingBackend, FontFamily, FontStyle, FontTransform,
    text_anchor::{HPos, Pos, VPos},
};
use wxdragon::{self as wx, BackgroundMode, DeviceContext};

/// Bridge struct to allow plotters to plot on a [`wxdragon::DeviceContext`].
///
/// This backend works with any [`wxdragon::DeviceContext`] that implements the
/// required drawing primitives. For example:
/// - [`wxdragon::AutoBufferedPaintDC`] for drawing on a [`wxdragon::Panel`]
///   in a GUI application.
/// - [`wxdragon::MemoryDC`] for off-screen drawing to a [`wxdragon::Bitmap`].
///
/// # How to use
///
/// Create an app with a `wxdragon::Panel`, and use the panel's `on_paint`
/// handler to create a new `wxdragon::AutoBufferedPaintDC` (device context)
/// each time, wrap it in a `WxBackend` and then draw on it.
///
/// ```no_run
/// use plotters::prelude::*;
/// use plotters::style::text_anchor::{HPos, Pos, VPos};
/// use plotters_wxdragon::WxBackend;
/// use wxdragon::{self as wx, WindowEvents, WxWidget};
///
/// struct DrawingPanel {
///     panel: wx::Panel,
/// }
///
/// impl DrawingPanel {
///     fn new(parent: &wx::Frame) -> Self {
///         let panel = wx::PanelBuilder::new(parent).build();
///         panel.set_background_style(wx::BackgroundStyle::Paint);
///
///         // Register the paint handler with a move closure
///         panel.on_paint(move |_event| {
///             // Create a device context (wxdragon has several types)
///             // and a plotters backend
///             let dc = wx::AutoBufferedPaintDC::new(&panel);
///             let mut backend = WxBackend::new(&dc);
///
///             // Create a plotters plot as you would with any other backend
///             backend
///                 .draw_rect((300, 250), (500, 350), &BLACK, false)
///                 .unwrap();
///             let style = TextStyle::from(("monospace", 32.0).into_font())
///                 .pos(Pos::new(HPos::Center, VPos::Center));
///             backend
///                 .draw_text("hello, world", &style, (400, 300))
///                 .unwrap();
///
///             // Call present() when you are ready
///             backend.present().expect("present");
///         });
///
///         // Handle SIZE events to refresh when the window size changes
///         panel.on_size(move |_event| {
///             panel.refresh(true, None);
///         });
///
///         Self { panel }
///     }
/// }
///
/// impl std::ops::Deref for DrawingPanel {
///     type Target = wx::Panel;
///
///     fn deref(&self) -> &Self::Target {
///         &self.panel
///     }
/// }
///
/// let _ = wxdragon::main(|_| {
///     let frame = wx::Frame::builder()
///         .with_title("Getting started")
///         // with this, wx produces a canvas of size 800 x 600
///         .with_size(wx::Size::new(852, 689))
///         .build();
///
///     let drawing_panel = DrawingPanel::new(&frame);
///
///     // Initial paint
///     drawing_panel.refresh(true, None);
///
///     frame.show(true);
/// });
/// ```
pub struct WxBackend<'context, C>
where
    C: DeviceContext,
{
    context: &'context C,
}

impl<'context, C> WxBackend<'context, C>
where
    C: DeviceContext,
{
    /// Creates a new `WxBackend` from a `wxdragon::DeviceContext`.
    ///
    /// The `DeviceContext` is initialized with a white background color and
    /// transparent background mode.
    pub fn new(context: &'context C) -> WxBackend<'context, C> {
        let backend = WxBackend { context };
        backend.set_background_color(wx::Colour::rgb(255, 255, 255));
        backend.set_background_mode(wx::BackgroundMode::Transparent);
        backend.clear();
        backend
    }

    /// Clear the device context.
    pub fn clear(&self) {
        self.context.clear();
    }

    /// Set the background color of the device context.
    ///
    /// This setting affects the global background, and also the fill color of
    /// text labels.
    pub fn set_background_color(&self, color: wx::Colour) {
        self.context.set_background(color);
    }

    /// Set the background mode of the device context.
    ///
    /// This settings affects the fill color of text labels. Use
    /// [`BackgroundMode::Solid`] if you want text labels to have a solid
    /// background, otherwise leave the default
    /// [`BackgroundMode::Transparent`].
    pub fn set_background_mode(&self, mode: BackgroundMode) {
        self.context.set_background_mode(mode);
    }

    /// Set pen from plotters style.
    fn set_pen_style<S: plotters_backend::BackendStyle>(&self, style: &S) {
        let color = convert_color(style.color());
        let width = style.stroke_width() as i32;
        // FIXME: how to get info of other styles?
        let style = wx::PenStyle::Solid;
        self.context.set_pen(color, width, style);
    }

    /// Set brush from plotters style.
    fn set_brush_style(
        &self,
        fill: bool,
        color: plotters_backend::BackendColor,
    ) {
        let style = match fill {
            true => wx::BrushStyle::Solid,
            false => wx::BrushStyle::Transparent,
        };
        let color = convert_color(color);
        self.context.set_brush(color, style);
    }

    /// Sets the font style from plotters BackendTextStyle.
    ///
    /// Note: text background information is not present in
    /// plotters_backend::BackendTextStyle, but it can be controlled using
    /// [`WxBackend::set_background_mode`] and
    /// [`WxBackend::set_background_color`]
    fn set_font_style<TStyle: plotters_backend::BackendTextStyle>(
        &self,
        style: &TStyle,
    ) -> Result<(), ErrorInner> {
        self.context
            .set_text_background(self.context.get_background());
        let color = convert_color(style.color());
        self.context.set_text_foreground(color);
        // FIXME: There is a discrepancy with font size compared to the
        // BitmapBackend. For now using a coeficient 0.6. Note that in the
        // tests of an off-screen wxBitmap, the dpi value is 96.
        let point_size = (style.size() * 0.6) as i32;
        let (family, face_name) = match style.family() {
            // According to wx docs
            // https://docs.wxwidgets.org/3.2/interface_2wx_2font_8h.html
            FontFamily::Monospace => (wx::FontFamily::Teletype, "None"),
            FontFamily::SansSerif => (wx::FontFamily::Swiss, "None"),
            FontFamily::Serif => (wx::FontFamily::Roman, "None"),
            FontFamily::Name(name) => (wx::FontFamily::Default, name),
        };
        use wx::FontStyle::{Italic, Normal, Slant};
        let (style, weight) = match style.style() {
            FontStyle::Bold => (Normal, wx::FontWeight::Bold),
            FontStyle::Italic => (Italic, wx::FontWeight::Normal),
            FontStyle::Normal => (Normal, wx::FontWeight::Normal),
            FontStyle::Oblique => (Slant, wx::FontWeight::Normal),
        };
        let underlined = false;
        let font = wx::Font::builder()
            .with_point_size(point_size)
            .with_family(family)
            .with_style(style)
            .with_weight(weight)
            .with_underline(underlined)
            // NOTE: wxdragon could be improved here. `with_face_name()`
            // creates a string, and `build()` creates another string in its
            // call to `wx::dc::Font::new_with_details()`.
            .with_face_name(face_name)
            .build()
            .ok_or(ErrorInner::CreateFont)?;
        self.context.set_font(&font);
        Ok(())
    }
}

/// Convert color from plotters to wx
fn convert_color(color: plotters_backend::BackendColor) -> wx::Colour {
    let BackendColor { alpha, rgb } = color;
    let (r, g, b) = rgb;
    wx::Colour::new(r, g, b, (alpha * 255.0) as u8)
}

impl<'context, C> DrawingBackend for WxBackend<'context, C>
where
    C: DeviceContext,
{
    type ErrorType = Error;
    fn get_size(&self) -> (u32, u32) {
        let (width, height) = self.context.get_size();
        (width as u32, height as u32)
    }

    fn ensure_prepared(
        &mut self,
    ) -> Result<(), plotters_backend::DrawingErrorKind<Self::ErrorType>> {
        Ok(())
    }

    fn present(
        &mut self,
    ) -> Result<(), plotters_backend::DrawingErrorKind<Self::ErrorType>> {
        Ok(())
    }

    fn draw_pixel(
        &mut self,
        point: plotters_backend::BackendCoord,
        color: plotters_backend::BackendColor,
    ) -> Result<(), plotters_backend::DrawingErrorKind<Self::ErrorType>> {
        let (x, y) = point;
        let color = convert_color(color);
        let width = 1;
        let style = wx::PenStyle::Solid;
        self.context.set_pen(color, width, style);
        self.context.draw_point(x, y);
        Ok(())
    }

    fn draw_line<S: plotters_backend::BackendStyle>(
        &mut self,
        from: plotters_backend::BackendCoord,
        to: plotters_backend::BackendCoord,
        style: &S,
    ) -> Result<(), plotters_backend::DrawingErrorKind<Self::ErrorType>> {
        self.set_pen_style(style);
        let (x1, y1) = from;
        let (x2, y2) = to;
        self.context.draw_line(x1, y1, x2, y2);
        Ok(())
    }

    fn draw_path<
        S: plotters_backend::BackendStyle,
        I: IntoIterator<Item = plotters_backend::BackendCoord>,
    >(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), plotters_backend::DrawingErrorKind<Self::ErrorType>> {
        self.set_pen_style(style);
        let points: Vec<wx::dc::Point> = path
            .into_iter()
            .map(|(x, y)| wx::dc::Point::new(x, y))
            .collect();
        let x_offset = 0;
        let y_offset = 0;
        self.context.draw_lines(&points[..], x_offset, y_offset);
        Ok(())
    }

    fn draw_circle<S: plotters_backend::BackendStyle>(
        &mut self,
        center: plotters_backend::BackendCoord,
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), plotters_backend::DrawingErrorKind<Self::ErrorType>> {
        self.set_pen_style(style);
        self.set_brush_style(fill, style.color());
        let (x, y) = center;
        self.context.draw_circle(x, y, radius as i32);
        Ok(())
    }

    fn draw_rect<S: plotters_backend::BackendStyle>(
        &mut self,
        upper_left: plotters_backend::BackendCoord,
        bottom_right: plotters_backend::BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), plotters_backend::DrawingErrorKind<Self::ErrorType>> {
        self.set_pen_style(style);
        self.set_brush_style(fill, style.color());
        let (x1, y1) = upper_left;
        let (x2, y2) = bottom_right;
        let width = x2 - x1;
        let height = y2 - y1;
        self.context.draw_rectangle(x1, y1, width, height);
        Ok(())
    }

    fn fill_polygon<
        S: plotters_backend::BackendStyle,
        I: IntoIterator<Item = plotters_backend::BackendCoord>,
    >(
        &mut self,
        vert: I,
        style: &S,
    ) -> Result<(), plotters_backend::DrawingErrorKind<Self::ErrorType>> {
        self.set_pen_style(style);
        self.set_brush_style(true, style.color());
        let points: Vec<wx::dc::Point> = vert
            .into_iter()
            .map(|(x, y)| wx::dc::Point::new(x, y))
            .collect();
        let x_offset = 0;
        let y_offset = 0;
        let fill_mode = wx::dc::PolygonFillMode::OddEven;
        self.context
            .draw_polygon(&points[..], x_offset, y_offset, fill_mode);
        Ok(())
    }

    fn draw_text<TStyle: plotters_backend::BackendTextStyle>(
        &mut self,
        text: &str,
        style: &TStyle,
        pos: plotters_backend::BackendCoord,
    ) -> Result<(), plotters_backend::DrawingErrorKind<Self::ErrorType>> {
        // this also sets the font style
        let (width, height) = self.estimate_text_size(text, style)?;
        let width = width as i32;
        let height = height as i32;
        let (x, y) = pos;

        // plotters convention is that anchor position is relative to
        // character's point of view
        let Pos { h_pos, v_pos } = style.anchor();
        let dx = match h_pos {
            HPos::Left => 0,
            HPos::Center => -width / 2,
            HPos::Right => -width,
        };
        let dy = match v_pos {
            VPos::Top => 0,
            VPos::Center => -height / 2,
            VPos::Bottom => -height,
        };
        let (dx, dy) = match style.transform() {
            FontTransform::None => (dx, dy),
            FontTransform::Rotate90 => (-dy, dx),
            FontTransform::Rotate180 => (-dx, -dy),
            FontTransform::Rotate270 => (dy, -dx),
        };

        // plotters rotates clockwise, wxwidgets rotates counterclockwise
        let angle = match style.transform() {
            FontTransform::None => None,
            FontTransform::Rotate90 => Some(-90.0),
            FontTransform::Rotate180 => Some(-180.0),
            FontTransform::Rotate270 => Some(-270.0),
        };
        if let Some(angle) = angle {
            self.context.draw_rotated_text(text, x + dx, y + dy, angle);
        } else {
            self.context.draw_text(text, x + dx, y + dy);
        }
        Ok(())
    }

    fn estimate_text_size<TStyle: plotters_backend::BackendTextStyle>(
        &self,
        text: &str,
        style: &TStyle,
    ) -> Result<(u32, u32), plotters_backend::DrawingErrorKind<Self::ErrorType>>
    {
        self.set_font_style(style).map_err(|e| {
            plotters_backend::DrawingErrorKind::FontError(Box::new(Error(e)))
        })?;
        let (width, height) = self.context.get_text_extent(text);
        Ok((width as u32, height as u32))
    }

    fn blit_bitmap(
        &mut self,
        pos: plotters_backend::BackendCoord,
        (iw, ih): (u32, u32),
        src: &[u8],
    ) -> Result<(), plotters_backend::DrawingErrorKind<Self::ErrorType>> {
        let (x, y) = pos;
        let bitmap = wx::Bitmap::from_rgba(src, iw, ih).ok_or_else(|| {
            plotters_backend::DrawingErrorKind::FontError(Box::new(Error(
                ErrorInner::CreateBitmap,
            )))
        })?;
        let transparent = false; // FIXME
        self.context.draw_bitmap(&bitmap, x, y, transparent);
        Ok(())
    }
}

/// Represents an error when drawing on a [`WxBackend`].
#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct Error(#[from] ErrorInner);

/// Error kind for `plotters_wxdragon::Error`.
#[derive(Debug, thiserror::Error)]
enum ErrorInner {
    #[error("failed to create font from plotters BackendTextStyle")]
    CreateFont,
    #[error("failed to create bitmap")]
    CreateBitmap,
}
