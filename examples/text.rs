//! Various tests of drawing rotated text and anchors
//!
//! The frame has a main drawing panel and a toolbar to choose the text
//! transform (rotation) to use. The toolbar has 4 buttons, each selecting a
//! rotation 0°, 90°, 180°, or 270°.
//!
//! Each button has a label that illustrates the applied rotation.
//!
//! When a button is pressed, it changes the global state, which holds a
//! variable indicating the chosen rotation (a variant of `FontTransform`
//! enum). The plot updates accordingly.
//!
//! The main plot shows 9 boxes, each with a text with one of the 9 possible
//! anchor positions (3 possible horizontal positions x 3 possible vertical
//! positions).

use std::cell::RefCell;
use std::rc::Rc;

// We leave the glob import of plotters so as not to change the example.
use plotters::prelude::*;
use plotters_backend::text_anchor::{HPos, Pos, VPos};
// No glob import for wxdragon to avoid conflicts, but an alias `wx`, and
// import traits as needed.
use plotters_wxdragon::WxBackend;
use wxdragon::{self as wx, WindowEvents, WxWidget};

/// Shared application state
struct State {
    /// Current text transform (rotation)
    text_transform: FontTransform,
}

struct DrawingPanel {
    panel: wx::Panel,
}

impl DrawingPanel {
    fn new(parent: &wx::Frame, state: Rc<RefCell<State>>) -> Self {
        let panel = wx::PanelBuilder::new(parent).build();
        panel.set_background_style(wx::BackgroundStyle::Paint);

        // Register the paint handler with a move closure
        panel.on_paint(move |_event| {
            let dc = wx::AutoBufferedPaintDC::new(&panel);
            let mut backend = WxBackend::new(&dc);

            let x0 = 100;
            let y0 = 100;
            let dx = 100;
            let dy = 100;
            let hpos = [HPos::Left, HPos::Center, HPos::Right];
            let vpos = [VPos::Top, VPos::Center, VPos::Bottom];

            // Get the current transform from the shared state
            let text_transform = &state.borrow().text_transform;

            // 3 x 3 grid of all possible anchor positions
            for i in 0..3 {
                for j in 0..3 {
                    let xi = x0 + 2 * i * dx;
                    let yi = y0 + 2 * j * dy;
                    backend
                        .draw_rect(
                            (xi - dx / 2, yi - dy / 2),
                            (xi + dx / 2, yi + dy / 2),
                            &BLACK,
                            false,
                        )
                        .unwrap();
                    let style =
                        TextStyle::from(("monospace", 32.0).into_font())
                            .pos(Pos::new(hpos[i as usize], vpos[j as usize]))
                            .transform(text_transform.clone());
                    backend.draw_text("M", &style, (xi, yi)).unwrap();
                }
            }

            backend.present().expect("present");
        });

        panel.on_size(move |_event| {
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

const ID_TOOL_ROTATE_0: wx::Id = wx::ID_HIGHEST + 1;
const ID_TOOL_ROTATE_90: wx::Id = wx::ID_HIGHEST + 2;
const ID_TOOL_ROTATE_180: wx::Id = wx::ID_HIGHEST + 3;
const ID_TOOL_ROTATE_270: wx::Id = wx::ID_HIGHEST + 4;

fn main() {
    let _ = wxdragon::main(|_| {
        let frame = wx::Frame::builder()
            .with_title("Text anchor position example with Plotters")
            .with_size(wx::Size::new(650, 750))
            .with_position(wx::Point::new(100, 100))
            .build();
        frame.center_on_screen();

        let state = Rc::new(RefCell::new(State {
            text_transform: FontTransform::None,
        }));

        let drawing_panel = DrawingPanel::new(&frame, state.clone());

        add_toolbar(&frame);

        // menu/toolbar events: change rotation
        {
            let drawing_panel = *drawing_panel;
            frame.on_menu(move |event| match event.get_id() {
                ID_TOOL_ROTATE_0 | ID_TOOL_ROTATE_90 | ID_TOOL_ROTATE_180
                | ID_TOOL_ROTATE_270 => {
                    state.borrow_mut().text_transform = match event.get_id() {
                        ID_TOOL_ROTATE_0 => FontTransform::None,
                        ID_TOOL_ROTATE_90 => FontTransform::Rotate90,
                        ID_TOOL_ROTATE_180 => FontTransform::Rotate180,
                        ID_TOOL_ROTATE_270 => FontTransform::Rotate270,
                        _ => unreachable!(),
                    };
                    drawing_panel.refresh(true, None);
                }
                _ => {
                    event.skip(true);
                }
            });
        }

        drawing_panel.refresh(true, None);

        frame.show(true);
    });
}

/// Creates the toolbar with rotation tools
fn add_toolbar(frame: &wx::Frame) {
    use wx::ArtClient::Toolbar;
    use wx::ArtId::{GoBack, GoDown, GoForward, GoUp};
    if let Some(toolbar) = frame
        .create_tool_bar(Some(wx::ToolBarStyle::Default), wx::ID_ANY as i32)
    {
        if let Some(new_icon) = wx::ArtProvider::get_bitmap(GoUp, Toolbar, None)
        {
            toolbar.add_tool(ID_TOOL_ROTATE_0, "0", &new_icon, "No rotation");
        }
        if let Some(new_icon) =
            wx::ArtProvider::get_bitmap(GoForward, Toolbar, None)
        {
            toolbar.add_tool(ID_TOOL_ROTATE_90, "0", &new_icon, "Rotate 90°");
        }
        if let Some(new_icon) =
            wx::ArtProvider::get_bitmap(GoDown, Toolbar, None)
        {
            toolbar.add_tool(ID_TOOL_ROTATE_180, "0", &new_icon, "Rotate 180°");
        }
        if let Some(new_icon) =
            wx::ArtProvider::get_bitmap(GoBack, Toolbar, None)
        {
            toolbar.add_tool(ID_TOOL_ROTATE_270, "0", &new_icon, "Rotate 270°");
        }
        toolbar.realize();
    }
}
