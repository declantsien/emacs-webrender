#![feature(concat_idents)]
#![allow(non_upper_case_globals)]

#[macro_use]
extern crate lisp_types;
extern crate lisp_macros;
#[macro_use]
extern crate lisp_util;
extern crate colors;

// mod platform {
//     #[cfg(target_os = "macos")]
//     pub use crate::platform::macos::font;
//     #[cfg(any(target_os = "android", all(unix, not(target_os = "macos"))))]
//     pub use crate::platform::unix::font;
//     #[cfg(target_os = "windows")]
//     pub use crate::platform::windows::font;

//     #[cfg(target_os = "macos")]
//     pub mod macos {
//         pub mod font;
//     }
//     #[cfg(any(target_os = "android", all(unix, not(target_os = "macos"))))]
//     pub mod unix {
//         pub mod font;
// 	mod font_db;
//     }
//     #[cfg(target_os = "windows")]
//     pub mod windows {
//         pub mod font;
//     }
// }

pub mod color;
pub mod display_info;
pub mod font;
pub mod frame;
pub mod input;
pub mod output;
pub mod term;

mod cursor;
mod draw_canvas;
mod event;
mod event_loop;
mod font_db;
mod fringe;
mod future;
mod image;
mod texture;
mod util;
mod wrterm;

pub use crate::wrterm::{tip_frame, wr_display_list};

pub use crate::event_loop::wr_select;
pub use crate::wrterm::wr_get_fontset;
pub use crate::wrterm::wr_get_font;
pub use crate::wrterm::wr_get_window_desc;
pub use crate::wrterm::wr_get_display_info;
pub use crate::wrterm::wr_get_display;
pub use crate::wrterm::wr_get_baseline_offset;
pub use crate::wrterm::wr_get_pixel;
pub use crate::wrterm::wr_put_pixel;
pub use crate::wrterm::wr_can_use_native_image_api;
pub use crate::wrterm::wr_load_image;
pub use crate::wrterm::wr_transform_image;
pub use crate::wrterm::get_keysym_name;
pub use crate::wrterm::check_x_display_info;
pub use crate::wrterm::frame_set_mouse_pixel_position;
pub use crate::wrterm::image_sync_to_pixmaps;
pub use crate::wrterm::image_pixmap_draw_cross;
pub use crate::wrterm::Fx_hide_tip;
pub use crate::wrterm::Fx_create_frame;
pub use crate::wrterm::Fx_open_connection;
pub use crate::wrterm::Fxw_display_color_p;
pub use crate::wrterm::Fx_display_grayscale_p;
pub use crate::wrterm::Fxw_color_values;
pub use crate::wrterm::Fx_register_dnd_atom;
pub use crate::wrterm::Fx_change_window_property;
pub use crate::wrterm::Fx_display_color_cells;
pub use crate::wrterm::Fx_display_planes;
pub use crate::wrterm::Fx_wm_set_size_hint;
pub use crate::wrterm::Fx_display_visual_class;
pub use crate::wrterm::Fx_display_monitor_attributes_list;
pub use crate::wrterm::Fx_display_pixel_width;
pub use crate::wrterm::Fx_display_pixel_height;
pub use crate::wrterm::Fx_own_selection_internal;
pub use crate::wrterm::Fx_get_selection_internal;
pub use crate::wrterm::Fx_selection_owner_p;
pub use crate::wrterm::Fx_selection_exists_p;
pub use crate::wrterm::Fx_frame_edges;
pub use crate::wrterm::Fwr_api_capture;
pub use crate::wrterm::Fwr_api_stop_capture_sequence;

#[no_mangle]
pub extern "C" fn syms_of_ftfont() {
}
