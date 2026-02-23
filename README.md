# plotters-wxdragon

`plotters-wxdragon` is a backend for [Plotters], to draw plots in a GUI window
managed by [wxDragon].

[wxDragon] is a binding of wxWidgets for rust. wxWidgets is a cross-platform
GUI library toolkit for desktop applications, that uses the native GUI toolkit
on Windows, macOS, and Linux.

[Plotters] is a Rust drawing library focusing on data plotting for both WASM
and native applications.

[Plotters]: https://crates.io/crates/plotters
[wxDragon]: https://crates.io/crates/wxdragon

## Quick start

coming soon..

## How to contribute

coming soon..

## How this works

This crate implements a backend for [Plotters]. It uses the existing drawing
context of wxWidgets, and maps plotters drawing primitives to corresponding
calls fo the wxWidgets API.

See also [`plotters-backend`] for reference on implementing a backend for
plotters.

[`plotters-backend`]: https://docs.rs/plotters-backend/latest/plotters_backend/

## Roadmap

coming soon..
