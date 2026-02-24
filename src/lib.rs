use plotters_backend::{
    BackendColor, DrawingBackend, FontFamily, FontStyle, FontTransform,
    text_anchor::{HPos, Pos, VPos},
};
use wxdragon::{self as wx, DeviceContext};

/// Bridge struct to allow plotters to plot on a [`wxdragon::DeviceContext`].
///
/// **FIXME** explain how to use.
///
/// **FIXME** explain which kind of [`DeviceContext`] this can work with.
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
    pub fn new(context: &'context C) -> WxBackend<'context, C> {
        context.set_background(WHITE);
        context.set_background_mode(wx::BackgroundMode::Solid);
        context.clear();
        WxBackend { context }
    }

    /// Set pen from plotters style
    fn set_pen_style<S: plotters_backend::BackendStyle>(&self, style: &S) {
        let color = convert_color(style.color());
        let width = style.stroke_width() as i32;
        // FIXME: how to get info of other styles?
        let style = wx::PenStyle::Solid;
        println!("set_pen({color:?}, {width}, {style:?})");
        self.context.set_pen(color, width, style);
    }

    /// Set brush from plotters style
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
        println!("set_brush({color:?}, {style:?})");
        self.context.set_brush(color, style);
    }

    /// Sets the font style from plotters BackendTextStyle
    fn set_font_style<TStyle: plotters_backend::BackendTextStyle>(
        &self,
        style: &TStyle,
    ) -> Result<(), ErrorInner> {
        self.context.set_text_background(WHITE); // FIXME
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
        // self.context.set_background(WHITE);
        // self.context.set_background_mode(wx::BackgroundMode::Solid);
        // self.context.clear();
        println!("ensure_prepared()");
        Ok(())
    }

    fn present(
        &mut self,
    ) -> Result<(), plotters_backend::DrawingErrorKind<Self::ErrorType>> {
        println!("present()");
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
        println!("draw_line({x1}, {y1}, {x2}, {y2})");
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
        println!("draw_lines({points:?}, {x_offset}, {y_offset})");
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
        println!("draw_circle({x}, {y}, {radius})");
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
        println!("draw_rectangle({x1}, {y1}, {width}, {height})");
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
        println!(
            "draw_polygon({points:?}, {x_offset}, {y_offset}, {fill_mode:?})"
        );
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
            println!("draw_rotated_text({text}, {x}, {y}, {angle})");
            self.context.draw_rotated_text(text, x + dx, y + dy, angle);
        } else {
            println!("draw_text({text}, {x}, {y}");
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

/// Represents an error when drawing on a [`DrawingPanel`].
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

const WHITE: wx::Colour = wx::Colour::rgb(255, 255, 255);
