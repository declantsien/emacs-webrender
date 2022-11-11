use std::{cell::RefCell, mem::MaybeUninit, rc::Rc, sync::Arc};

use euclid::default::Size2D;
use gleam::gl::{self, Gl};
use log::warn;
use std::{
    ops::{Deref, DerefMut},
    ptr,
};
use surfman::Connection;
use surfman::GLApi;
use surfman::SurfaceType;
use webrender_surfman::WebrenderSurfman;
use winit::{
    self,
    dpi::PhysicalSize,
    window::{CursorIcon, Window},
};

#[cfg(not(any(target_os = "macos", windows)))]
use winit::platform::unix::WindowBuilderExtUnix;

use webrender::{self, api::units::*, api::*, RenderApi, Renderer, Transaction};

use lisp_types::{
    bindings::{wr_output, Emacs_Cursor},
    frame::LispFrameRef,
};

use crate::event_loop::WrEventLoop;

use super::texture::TextureResourceManager;
use super::util::HandyDandyRectBuilder;
use super::{cursor::emacs_to_winit_cursor, display_info::DisplayInfoRef};
use super::{cursor::winit_to_emacs_cursor, font::FontData, font::FontRef};

#[cfg(all(feature = "wayland", not(any(target_os = "macos", windows))))]
use lisp_types::{bindings::globals, multibyte::LispStringRef};

pub struct Output {
    // Extend `wr_output` struct defined in `wrterm.h`
    pub output: wr_output,

    pub font: FontRef,
    pub fontset: i32,

    pub render_api: RenderApi,
    pub document_id: DocumentId,
    pipeline_id: PipelineId,
    epoch: Epoch,

    display_list_builder: Option<DisplayListBuilder>,
    previous_frame_image: Option<ImageKey>,

    pub background_color: ColorF,
    pub cursor_color: ColorF,
    pub cursor_foreground_color: ColorF,

    // The drop order is important here.

    // Need to dropped before window context
    texture_resources: Rc<RefCell<TextureResourceManager>>,

    // Need to droppend before window context
    renderer: Renderer,

    window: winit::window::Window,
    webrender_surfman: WebrenderSurfman,
    gl: Rc<dyn gl::Gl>,

    frame: LispFrameRef,
}

impl Output {
    pub fn build(event_loop: &mut WrEventLoop, frame: LispFrameRef) -> Self {
        let window_builder = winit::window::WindowBuilder::new()
            .with_visible(true)
            .with_maximized(true);

        #[cfg(all(feature = "wayland", not(any(target_os = "macos", windows))))]
        let window_builder = {
            let invocation_name: LispStringRef = unsafe { globals.Vinvocation_name.into() };
            let invocation_name = invocation_name.to_utf8();
            window_builder.with_name(invocation_name, "")
        };

        let window = window_builder.build(&event_loop.el()).unwrap();
        let window_id = window.id();

        // Initialize surfman
        let connection =
            Connection::from_winit_window(&window).expect("Failed to create connection");
        let adapter = connection
            .create_adapter()
            .expect("Failed to create adapter");
        let native_widget = connection
            .create_native_widget_from_winit_window(&window)
            .expect("Failed to create native widget");
        let surface_type = SurfaceType::Widget { native_widget };
        let webrender_surfman = WebrenderSurfman::create(&connection, &adapter, surface_type)
            .expect("Failed to create WR surfman");

        // Get GL bindings
        let gl = match webrender_surfman.connection().gl_api() {
            GLApi::GL => unsafe { gl::GlFns::load_with(|s| webrender_surfman.get_proc_address(s)) },
            GLApi::GLES => unsafe {
                gl::GlesFns::load_with(|s| webrender_surfman.get_proc_address(s))
            },
        };

        let gl = gl::ErrorCheckingGl::wrap(gl);

        println!("OpenGL version {}", gl.get_string(gl::VERSION));
        let device_pixel_ratio = window.scale_factor() as f32;
        println!("Device pixel ratio: {}", device_pixel_ratio);

        // Make sure the gl context is made current.
        webrender_surfman.make_gl_context_current().unwrap();
        debug_assert_eq!(gl.get_error(), gleam::gl::NO_ERROR,);

        let webrender_opts = webrender::WebRenderOptions {
            clear_color: ColorF::new(1.0, 1.0, 1.0, 1.0),
            ..webrender::WebRenderOptions::default()
        };

        let notifier = Box::new(Notifier::new(event_loop.create_proxy()));
        let (mut renderer, sender) =
            webrender::create_webrender_instance(gl.clone(), notifier, webrender_opts, None)
                .unwrap();

        let texture_resources = Rc::new(RefCell::new(TextureResourceManager::new(
            gl.clone(),
            sender.create_api(),
        )));

        let external_image_handler = texture_resources.borrow_mut().new_external_image_handler();

        renderer.set_external_image_handler(external_image_handler);

        let epoch = Epoch(0);
        let pipeline_id = PipelineId(0, 0);

        let mut api = sender.create_api();
        let device_size = {
            let size = window.inner_size();
            DeviceIntSize::new(size.width as i32, size.height as i32)
        };
        let document_id = api.add_document(device_size);

        let mut output = Self {
            output: wr_output::default(),
            window,
            font: FontRef::new(ptr::null_mut()),
            fontset: 0,
            render_api: api,
            document_id,
            pipeline_id,
            gl,
            epoch,
            display_list_builder: None,
            previous_frame_image: None,
            background_color: ColorF::WHITE,
            cursor_color: ColorF::BLACK,
            cursor_foreground_color: ColorF::WHITE,
            renderer,
            webrender_surfman,
            texture_resources,
            frame,
        };

        Self::build_mouse_cursors(&mut output);

        output
    }

    fn copy_framebuffer_to_texture(&self, device_rect: DeviceIntRect) -> ImageKey {
        let mut origin = device_rect.min;

        let device_size = self.get_deivce_size();

        if !self.renderer.device.surface_origin_is_top_left() {
            origin.y = device_size.height - origin.y - device_rect.height();
        }

        let fb_rect = FramebufferIntRect::from_origin_and_size(
            FramebufferIntPoint::from_untyped(origin.to_untyped()),
            FramebufferIntSize::from_untyped(device_rect.size().to_untyped()),
        );

        let need_flip = !self.renderer.device.surface_origin_is_top_left();

        let (image_key, texture_id) = self.texture_resources.borrow_mut().new_image(
            self.document_id,
            fb_rect.size(),
            need_flip,
        );

        let gl = &self.gl;
        gl.bind_texture(gl::TEXTURE_2D, texture_id);

        gl.copy_tex_sub_image_2d(
            gl::TEXTURE_2D,
            0,
            0,
            0,
            fb_rect.min.x,
            fb_rect.min.y,
            fb_rect.size().width,
            fb_rect.size().height,
        );

        gl.bind_texture(gl::TEXTURE_2D, 0);

        image_key
    }

    fn get_size(window: &Window) -> LayoutSize {
        let physical_size = window.inner_size();
        let device_size = LayoutSize::new(physical_size.width as f32, physical_size.height as f32);
        device_size
    }

    fn new_builder(&mut self, image: Option<(ImageKey, LayoutRect)>) -> DisplayListBuilder {
        let pipeline_id = self.pipeline_id;

        let layout_size = Self::get_size(&self.get_window());
        let mut builder = DisplayListBuilder::new(pipeline_id);
        builder.begin();

        if let Some((image_key, image_rect)) = image {
            let space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);

            let bounds = (0, 0).by(layout_size.width as i32, layout_size.height as i32);

            builder.push_image(
                &CommonItemProperties::new(bounds, space_and_clip),
                image_rect,
                ImageRendering::Auto,
                AlphaType::PremultipliedAlpha,
                image_key,
                ColorF::WHITE,
            );
        }

        builder
    }

    pub fn show_window(&self) {
        self.get_window().set_visible(true);
    }
    pub fn hide_window(&self) {
        self.get_window().set_visible(false);
    }

    pub fn maximize(&self) {
        self.get_window().set_maximized(true);
    }

    pub fn set_title(&self, title: &str) {
        self.get_window().set_title(title);
    }

    pub fn set_display_info(&mut self, mut dpyinfo: DisplayInfoRef) {
        self.output.display_info = dpyinfo.get_raw().as_mut();
    }

    pub fn get_frame(&self) -> LispFrameRef {
        self.frame
    }

    pub fn display_info(&self) -> DisplayInfoRef {
        DisplayInfoRef::new(self.output.display_info as *mut _)
    }

    pub fn device_pixel_ratio(&self) -> f32 {
        self.window.scale_factor() as f32
    }

    pub fn get_inner_size(&self) -> PhysicalSize<u32> {
        self.get_window().inner_size()
    }

    fn get_deivce_size(&self) -> DeviceIntSize {
        let size = self.get_window().inner_size();
        DeviceIntSize::new(size.width as i32, size.height as i32)
    }

    pub fn display<F>(&mut self, f: F)
    where
        F: Fn(&mut DisplayListBuilder, SpaceAndClipInfo),
    {
        if self.display_list_builder.is_none() {
            let layout_size = Self::get_size(&self.get_window());

            let image_and_pos = self
                .previous_frame_image
                .map(|image_key| (image_key, LayoutRect::from_size(layout_size)));

            self.display_list_builder = Some(self.new_builder(image_and_pos));
        }

        let pipeline_id = PipelineId(0, 0);

        if let Some(builder) = &mut self.display_list_builder {
            let space_and_clip = SpaceAndClipInfo::root_scroll(pipeline_id);

            f(builder, space_and_clip);
        }
    }

    fn ensure_context_is_current(&mut self) {
        // Make sure the gl context is made current.
        self.webrender_surfman.make_gl_context_current().unwrap();
        debug_assert_eq!(self.gl.get_error(), gleam::gl::NO_ERROR,);
    }

    pub fn flush(&mut self) {
        let builder = std::mem::replace(&mut self.display_list_builder, None);

        if let Some(mut builder) = builder {
            let layout_size = Self::get_size(&self.get_window());

            let epoch = Epoch(0);
            let mut txn = Transaction::new();

            txn.set_display_list(epoch, None, layout_size.to_f32(), builder.end());
            txn.set_root_pipeline(self.pipeline_id);
            txn.generate_frame(0, RenderReasons::NONE);

            self.display_list_builder = None;

            self.render_api.send_transaction(self.document_id, txn);

            self.render_api.flush_scene_builder();

            let device_size = self.get_deivce_size();

            // Bind the webrender framebuffer
            let framebuffer_object = self
                .webrender_surfman
                .context_surface_info()
                .unwrap_or(None)
                .map(|info| info.framebuffer_object)
                .unwrap_or(0);
            self.gl
                .bind_framebuffer(gleam::gl::FRAMEBUFFER, framebuffer_object);

            self.renderer.update();

            self.ensure_context_is_current();

            self.renderer.render(device_size, 0).unwrap();
            let _ = self.renderer.flush_pipeline_info();

            self.texture_resources.borrow_mut().clear();

            let image_key = self.copy_framebuffer_to_texture(DeviceIntRect::from_size(device_size));
            self.previous_frame_image = Some(image_key);

            // Perform the page flip. This will likely block for a while.
            if let Err(err) = self.webrender_surfman.present() {
                warn!("Failed to present surface: {:?}", err);
            }
        }
    }

    pub fn get_previous_frame(&self) -> Option<ImageKey> {
        self.previous_frame_image
    }

    pub fn clear_display_list_builder(&mut self) {
        let _ = std::mem::replace(&mut self.display_list_builder, None);
    }

    pub fn add_font_instance_by_data(
        &mut self,
        data: FontData,
        pixel_size: i32,
    ) -> FontInstanceKey {
        let font_key = self.add_font(data);
        self.add_font_instance(font_key, pixel_size)
    }

    pub fn add_font_instance(&mut self, font_key: FontKey, pixel_size: i32) -> FontInstanceKey {
        let mut txn = Transaction::new();

        let font_instance_key = self.render_api.generate_font_instance_key();

        txn.add_font_instance(
            font_instance_key,
            font_key,
            pixel_size as f32,
            None,
            None,
            vec![],
        );

        self.render_api.send_transaction(self.document_id, txn);
        font_instance_key
    }

    pub fn add_font(&mut self, data: FontData) -> FontKey {
        let font_key = self.render_api.generate_font_key();
        let mut txn = Transaction::new();
        match data {
            FontData::Raw(ref bytes, index) => txn.add_raw_font(font_key, bytes.clone(), index),
            FontData::Native(native_font) => txn.add_native_font(font_key, native_font),
        }
        self.render_api.send_transaction(self.document_id, txn);

        font_key
    }

    pub fn get_color_bits(&self) -> u8 {
        // let descriptor = self.device.context_descriptor(&ctx);
        // let descriptor_attributes = self.device.context_descriptor_attributes(&descriptor);
        // let gl_version = descriptor_attributes.version;
        // also https://github.com/rust-windowing/glutin/blob/v0.29.1/glutin/src/platform_impl/macos/mod.rs#L105
        //TODO check surfman context attributes
        24
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }

    fn build_mouse_cursors(output: &mut Output) {
        output.output.text_cursor = winit_to_emacs_cursor(CursorIcon::Text);
        output.output.nontext_cursor = winit_to_emacs_cursor(CursorIcon::Arrow);
        output.output.modeline_cursor = winit_to_emacs_cursor(CursorIcon::Hand);
        output.output.hand_cursor = winit_to_emacs_cursor(CursorIcon::Hand);
        output.output.hourglass_cursor = winit_to_emacs_cursor(CursorIcon::Progress);

        output.output.horizontal_drag_cursor = winit_to_emacs_cursor(CursorIcon::ColResize);
        output.output.vertical_drag_cursor = winit_to_emacs_cursor(CursorIcon::RowResize);

        output.output.left_edge_cursor = winit_to_emacs_cursor(CursorIcon::WResize);
        output.output.right_edge_cursor = winit_to_emacs_cursor(CursorIcon::EResize);
        output.output.top_edge_cursor = winit_to_emacs_cursor(CursorIcon::NResize);
        output.output.bottom_edge_cursor = winit_to_emacs_cursor(CursorIcon::SResize);

        output.output.top_left_corner_cursor = winit_to_emacs_cursor(CursorIcon::NwResize);
        output.output.top_right_corner_cursor = winit_to_emacs_cursor(CursorIcon::NeResize);

        output.output.bottom_left_corner_cursor = winit_to_emacs_cursor(CursorIcon::SwResize);
        output.output.bottom_right_corner_cursor = winit_to_emacs_cursor(CursorIcon::SeResize);
    }

    pub fn set_mouse_cursor(&self, cursor: Emacs_Cursor) {
        let cursor = emacs_to_winit_cursor(cursor);

        self.get_window().set_cursor_icon(cursor)
    }

    pub fn add_image(&mut self, width: i32, height: i32, image_data: Arc<Vec<u8>>) -> ImageKey {
        let image_key = self.render_api.generate_image_key();

        self.update_image(image_key, width, height, image_data);

        image_key
    }

    pub fn update_image(
        &mut self,
        image_key: ImageKey,
        width: i32,
        height: i32,
        image_data: Arc<Vec<u8>>,
    ) {
        let mut txn = Transaction::new();

        txn.add_image(
            image_key,
            ImageDescriptor::new(
                width,
                height,
                ImageFormat::RGBA8,
                ImageDescriptorFlags::empty(),
            ),
            ImageData::Raw(image_data),
            None,
        );

        self.render_api.send_transaction(self.document_id, txn);
    }

    pub fn delete_image(&mut self, image_key: ImageKey) {
        let mut txn = Transaction::new();

        txn.delete_image(image_key);

        self.render_api.send_transaction(self.document_id, txn);
    }

    pub fn resize(&mut self, size: &PhysicalSize<u32>) {
        let device_size = DeviceIntSize::new(size.width as i32, size.height as i32);

        let device_rect =
            DeviceIntRect::from_origin_and_size(DeviceIntPoint::new(0, 0), device_size);

        let mut txn = Transaction::new();
        txn.set_document_view(device_rect);
        self.render_api.send_transaction(self.document_id, txn);

        self.webrender_surfman
            .resize(Size2D::new(size.width as i32, size.height as i32))
            .unwrap();
    }
}

#[derive(PartialEq)]
#[repr(transparent)]
pub struct OutputRef(*mut Output);

impl Copy for OutputRef {}

// Derive fails for this type so do it manually
impl Clone for OutputRef {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

impl OutputRef {
    pub const fn new(p: *mut Output) -> Self {
        Self(p)
    }

    pub fn as_mut(&mut self) -> *mut wr_output {
        self.0 as *mut wr_output
    }

    pub fn as_rust_ptr(&mut self) -> *mut Output {
        self.0 as *mut Output
    }
}

impl Deref for OutputRef {
    type Target = Output;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0 }
    }
}

impl DerefMut for OutputRef {
    fn deref_mut(&mut self) -> &mut Output {
        unsafe { &mut *self.0 }
    }
}

impl From<*mut wr_output> for OutputRef {
    fn from(o: *mut wr_output) -> Self {
        Self::new(o as *mut Output)
    }
}

struct Notifier {
    events_proxy: winit::event_loop::EventLoopProxy<(i32)>,
}

impl Notifier {
    fn new(events_proxy: winit::event_loop::EventLoopProxy<(i32)>) -> Notifier {
        Notifier { events_proxy }
    }
}

impl RenderNotifier for Notifier {
    fn clone(&self) -> Box<dyn RenderNotifier> {
        Box::new(Notifier {
            events_proxy: self.events_proxy.clone(),
        })
    }

    fn wake_up(&self, _composite_needed: bool) {
        // #[cfg(not(target_os = "android"))]
        // let _ = self.events_proxy.send_event((1));
    }

    fn new_frame_ready(&self, _: DocumentId, _scrolled: bool, composite_needed: bool) {
        self.wake_up(composite_needed);
    }
}