/*!
`tiny-skia` is a tiny [Skia](https://skia.org/) subset ported to Rust.

`tiny-skia` API is a bit unconventional.
It doesn't look like cairo, QPainter (Qt), HTML Canvas or even Skia itself.
Instead, `tiny-skia` provides a set of low-level drawing APIs
and a user should manage the world transform, clipping mask and style manually.

See the `examples/` directory for usage examples.
*/

#![no_std]
#![warn(missing_docs)]
#![warn(missing_copy_implementations)]
#![warn(missing_debug_implementations)]

#![allow(clippy::approx_constant)]
#![allow(clippy::clone_on_copy)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::collapsible_if)]
#![allow(clippy::comparison_chain)]
#![allow(clippy::enum_variant_names)]
#![allow(clippy::excessive_precision)]
#![allow(clippy::identity_op)]
#![allow(clippy::manual_range_contains)]
#![allow(clippy::needless_range_loop)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::wrong_self_convention)]

#[cfg(not(any(feature = "std", feature = "no-std-float")))]
compile_error!("You have to activate either the `std` or the `no-std-float` feature.");

#[cfg(feature = "std")]
extern crate std;

extern crate alloc;

mod alpha_runs;
mod blend_mode;
mod blitter;
mod clip;
mod color;
mod edge;
mod edge_builder;
mod edge_clipper;
mod fixed_point;
mod line_clipper;
mod math;
mod path64;
mod pipeline;
mod pixmap;
mod painter; // Keep it under `pixmap` for a better order in the docs.
mod path_geometry;
mod scan;
mod shaders;
mod wide;

pub use blend_mode::BlendMode;
pub use clip::ClipMask;
pub use color::{ALPHA_U8_TRANSPARENT, ALPHA_U8_OPAQUE, ALPHA_TRANSPARENT, ALPHA_OPAQUE};
pub use color::{Color, ColorU8, PremultipliedColor, PremultipliedColorU8};
pub use painter::{Paint, FillRule};
pub use pixmap::{Pixmap, PixmapRef, PixmapMut, BYTES_PER_PIXEL};
pub use shaders::{GradientStop, SpreadMode, FilterQuality, PixmapPaint};
pub use shaders::{Shader, LinearGradient, RadialGradient, Pattern};

pub use tiny_skia_path::{IntRect, Rect, Point, Transform};
pub use tiny_skia_path::{Path, PathSegment, PathSegmentsIter, PathBuilder};
pub use tiny_skia_path::{LineCap, LineJoin, Stroke, StrokeDash};

/// An integer length that is guarantee to be > 0
type LengthU32 = core::num::NonZeroU32;