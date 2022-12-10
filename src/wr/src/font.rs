use std::{mem::ManuallyDrop, rc::Rc};

use font_loader::system_fonts;
use lazy_static::lazy_static;
use std::str;
use ttf_parser::Style;

use webrender::api::*;

use lisp_types::{
    bindings::{
        debug_print, font, font_driver, font_make_entity, font_make_object, font_metrics,
        font_property_index, font_style_to_value, frame, glyph_string, intern, Fassoc, Fcons,
        Fmake_symbol, Fnreverse, FONT_INVALID_CODE,
    },
    frame::LispFrameRef,
    globals::{
        Qbold, Qextra_bold, Qextra_light, Qiso10646_1, Qitalic, Qlight, Qmedium, Qnil, Qnormal,
        Qoblique, Qsemi_bold, Qthin, Qultra_bold, Qwr,
    },
    lisp::{ExternalPtr, LispObject},
    multibyte::LispStringRef,
    symbol::LispSymbolRef,
};

use crate::{font_db::FontDB, font_db::FontDescriptor, frame::LispFrameExt};

pub type FontRef = ExternalPtr<font>;

pub struct FontDriver(pub font_driver);
unsafe impl Sync for FontDriver {}

lazy_static! {
    pub static ref FONT_DRIVER: FontDriver = {
        let mut font_driver = font_driver::default();

        font_driver.type_ = Qwr;
        font_driver.case_sensitive = true;
        font_driver.get_cache = Some(get_cache);
        font_driver.list = Some(list);
        font_driver.match_ = Some(match_);
        font_driver.list_family = Some(list_family);
        font_driver.open_font = Some(open_font);
        font_driver.close_font = Some(close_font);
        font_driver.encode_char = Some(encode_char);
        font_driver.has_char = Some(has_char);
        font_driver.text_extents = Some(text_extents);
        font_driver.draw = Some(draw);

        FontDriver(font_driver)
    };
    static ref FONT_DB: FontDB = FontDB::new();
}

/// A newtype for objects we know are font_spec.
#[derive(Clone, Copy)]
pub struct LispFontLike(LispObject);

impl LispFontLike {
    fn aref(&self, index: font_property_index::Type) -> LispObject {
        let vl = self.0.as_vectorlike().unwrap();
        let v = unsafe { vl.as_vector_unchecked() };
        unsafe { v.get_unchecked(index as usize) }
    }

    fn get_family(&self) -> Option<String> {
        let tem = self.aref(font_property_index::FONT_FAMILY_INDEX);

        if tem.is_nil() {
            None
        } else {
            let symbol_or_string = tem.as_symbol_or_string();
            let string: LispStringRef = symbol_or_string.into();
            return Some(string.to_string().replace("-", "\\-"));
        }
    }

    fn aset(&self, index: font_property_index::Type, val: LispObject) {
        let vl = self.0.as_vectorlike().unwrap();
        let mut v = unsafe { vl.as_vector_unchecked() };
        unsafe { v.set_unchecked(index as usize, val) };
    }

    fn set_style(&self, index: font_property_index::Type, val: LispObject) {
        let value = unsafe { font_style_to_value(index, val, true) };

        self.aset(index, LispObject::from(value));
    }

    fn as_lisp_object(self) -> LispObject {
        self.0
    }
}

impl From<LispObject> for LispFontLike {
    fn from(v: LispObject) -> LispFontLike {
        LispFontLike(v)
    }
}

extern "C" fn get_cache(f: *mut frame) -> LispObject {
    let frame = LispFrameRef::new(f);
    let mut dpyinfo = frame.wr_display_info();

    dpyinfo.get_raw().name_list_element
}

extern "C" fn draw(
    _s: *mut glyph_string,
    _from: i32,
    _to: i32,
    _x: i32,
    _y: i32,
    _with_background: bool,
) -> i32 {
    0
}

extern "C" fn list(frame: *mut frame, font_spec: LispObject) -> LispObject {
    // FIXME: implment the real list in future
    match_(frame, font_spec)
}

extern "C" fn match_(_f: *mut frame, spec: LispObject) -> LispObject {
    // let now = std::time::Instant::now();
    let desc = FontDescriptor::from_font_spec(spec);
    let font_spec = LispFontLike(spec);
    let family_name = font_spec.get_family();
    // println!("match family{:?}:", family_name);
    let mut list = Qnil;

    let family_names = if let Some(family_name) = family_name {
        vec![family_name]
    } else {
        system_fonts::query_all()
    };

    for name in family_names {
        let (font_data, index) = FontDB::data_from_desc(desc.clone());

        let face = match ttf_parser::Face::parse(&font_data, index as u32) {
            Ok(f) => f,
            Err(e) => {
                eprint!("Error: {}.", e);
                std::process::exit(1);
            }
        };

        // let family_name = face
        //     .names()
        //     .into_iter()
        //     .find(|name| name.name_id == ttf_parser::name_id::FULL_NAME && name.is_unicode())
        //     .and_then(|name| name.to_string());

        let post_script_name = face
            .names()
            .into_iter()
            .find(|name| name.name_id == ttf_parser::name_id::POST_SCRIPT_NAME && name.is_unicode())
            .and_then(|name| name.to_string());

        let entity: LispFontLike = unsafe { font_make_entity() }.into();

        // set type
        entity.aset(font_property_index::FONT_TYPE_INDEX, Qwr);

        // set family
        // let family_name = family_name.unwrap_or(name);
        let family_name = FontDB::normalize_family_name(&name);
        let family_name: &str = &family_name;
        entity.aset(font_property_index::FONT_FAMILY_INDEX, unsafe {
            Fmake_symbol(LispObject::from(family_name))
        });

        let weight = face.weight().to_number();

        let weight = if weight <= 100 {
            Qthin
        } else if weight <= 200 {
            Qextra_light
        } else if weight <= 300 {
            Qlight
        } else if weight <= 400 {
            Qnormal
        } else if weight <= 500 {
            Qmedium
        } else if weight <= 600 {
            Qsemi_bold
        } else if weight <= 700 {
            Qbold
        } else if weight <= 800 {
            Qextra_bold
        } else if weight <= 900 {
            Qultra_bold
        } else {
            Qultra_bold
        };

        // unsafe { debug_print(weight) };

        // set weight
        entity.set_style(font_property_index::FONT_WEIGHT_INDEX, weight);

        let slant = match face.style() {
            Style::Normal => Qnormal,
            Style::Italic => Qitalic,
            Style::Oblique => Qoblique,
        };

        // set slant
        entity.set_style(font_property_index::FONT_SLANT_INDEX, slant);

        // set width
        entity.set_style(font_property_index::FONT_WIDTH_INDEX, (0 as usize).into());

        // set size
        entity.aset(font_property_index::FONT_SIZE_INDEX, (0 as usize).into());

        // set registry
        entity.aset(font_property_index::FONT_REGISTRY_INDEX, Qiso10646_1);

        // set postscript_name
        let post_script_name = post_script_name.unwrap();
        let post_script_name: &str = &post_script_name;
        entity.aset(font_property_index::FONT_EXTRA_INDEX, unsafe {
            Fcons(
                Fcons(
                    ":postscript-name".into(),
                    LispObject::from(post_script_name),
                ),
                Qnil,
            )
        });

        list = unsafe { Fcons(entity.as_lisp_object(), list) }
    }

    // println!("Elapsed: {}us", now.elapsed().as_micros());

    unsafe { Fnreverse(list) }
}

extern "C" fn list_family(_f: *mut frame) -> LispObject {
    let mut list = Qnil;

    let sysfonts = system_fonts::query_all();
    for family in &sysfonts {
        let f = family.replace('\'', "").trim().to_string();
        list = LispObject::cons(unsafe { intern(f.as_ptr() as *const ::libc::c_char) }, list);
    }

    list
}

#[repr(C)]
pub struct WRFont<'a> {
    // extend basic font
    pub font: font,

    pub device_pixel_ratio: f32,

    pub font_instance_key: FontInstanceKey,

    pub font_bytes: ManuallyDrop<Rc<Vec<u8>>>,

    pub face: ttf_parser::Face<'a>,
}

impl<'a> WRFont<'a> {
    pub fn glyph_for_char(&self, character: char) -> Option<u32> {
        self.face.glyph_index(character).map(|c| c.0 as u32)
    }

    pub fn get_glyph_advance_width(&self, glyph_indices: Vec<GlyphIndex>) -> Vec<Option<i32>> {
        let pixel_size = self.font.pixel_size;
        let glyph_size = pixel_size as f32 * self.device_pixel_ratio;
        let units_per_em = self.face.units_per_em();

        let scale = glyph_size / units_per_em as f32;

        glyph_indices
            .into_iter()
            .map(|i| {
                self.face
                    .glyph_hor_advance(ttf_parser::GlyphId(i as u16))
                    .map(|a| (a as f32 * scale).round() as i32)
            })
            .collect()
    }
}

pub type WRFontRef<'a> = ExternalPtr<WRFont<'a>>;

extern "C" fn open_font(frame: *mut frame, font_entity: LispObject, pixel_size: i32) -> LispObject {
    let desc = FontDescriptor::from_font_spec(font_entity);
    let font_entity: LispFontLike = font_entity.into();

    let frame: LispFrameRef = frame.into();
    let mut output = frame.wr_output();

    // pixel_size here reflects to DPR 1 for webrender display, we have scale_factor from winit.
    // while pgtk/ns/w32 reflects to actual DPR on device by setting resx/resy to display
    let pixel_size = if !output.font.is_null() {
        output.font.pixel_size as i64
    } else {
        pixel_size as i64
    };

    let device_pixel_ratio = output.device_pixel_ratio();
    let glyph_size = pixel_size as f32 * device_pixel_ratio;

    let font_object: LispFontLike = unsafe {
        font_make_object(
            vecsize!(WRFont) as i32,
            font_entity.as_lisp_object(),
            pixel_size as i32,
        )
    }
    .into();

    // set type
    font_object.aset(font_property_index::FONT_TYPE_INDEX, Qwr);

    // set name
    font_object.aset(
        font_property_index::FONT_NAME_INDEX,
        LispSymbolRef::from(font_entity.aref(font_property_index::FONT_FAMILY_INDEX)).symbol_name(),
    );

    let mut wr_font = WRFontRef::new(
        font_object
            .as_lisp_object()
            .as_font()
            .unwrap()
            .as_font_mut() as *mut WRFont,
    );
    wr_font.device_pixel_ratio = device_pixel_ratio;

    let (font_key, (font_bytes, face_index)) = output.get_or_create_font(desc);
    let bg_color = None;
    let flags = FontInstanceFlags::empty();
    let synthetic_italics = SyntheticItalics::disabled();
    let font_instance_key = output.get_or_create_font_instance(
        font_key,
        glyph_size,
        bg_color,
        flags,
        synthetic_italics,
    );
    wr_font.font_instance_key = font_instance_key;

    wr_font.font_bytes = ManuallyDrop::new(Rc::new(font_bytes.clone()));

    let font_bytes = wr_font.font_bytes.clone();
    let face = match ttf_parser::Face::parse(&font_bytes, face_index) {
        Ok(f) => f,
        Err(e) => {
            eprint!("Error: {}.", e);
            std::process::exit(1);
        }
    };
    wr_font.face = face;

    let face = &wr_font.face;

    let units_per_em = face.units_per_em();

    let underline_metrics = face.underline_metrics().unwrap();

    let ascent = face.ascender();
    let descent = face.descender();

    let average_width = face.glyph_hor_advance(ttf_parser::GlyphId(0)).unwrap();

    let scale = glyph_size / units_per_em as f32;

    wr_font.font.pixel_size = pixel_size as i32;
    wr_font.font.average_width = (average_width as f32 * scale) as i32;
    wr_font.font.ascent = (scale * ascent as f32).round() as i32;
    wr_font.font.descent = (-scale * descent as f32).round() as i32;
    wr_font.font.space_width = wr_font.font.average_width;
    wr_font.font.max_width = wr_font.font.average_width;
    wr_font.font.underline_thickness = (scale * underline_metrics.thickness as f32) as i32;
    wr_font.font.underline_position = (scale * underline_metrics.position as f32) as i32;

    wr_font.font.height = (scale * (ascent - descent) as f32).round() as i32;
    wr_font.font.baseline_offset = 0;

    wr_font.font.driver = &FONT_DRIVER.0;

    // unsafe{ debug_print(font_object.as_lisp_object())};
    font_object.as_lisp_object()
}

extern "C" fn close_font(_font: *mut font) {}

extern "C" fn encode_char(font: *mut font, c: i32) -> u32 {
    let font = WRFontRef::new(font as *mut WRFont);

    std::char::from_u32(c as u32)
        .and_then(|c| font.glyph_for_char(c))
        .unwrap_or(FONT_INVALID_CODE)
}

extern "C" fn has_char(font: LispObject, c: i32) -> i32 {
    if font.is_font_entity() {
        let desc = FontDescriptor::from_font_spec(font);
        let font_entity: LispFontLike = font.into();

        // Get postscript name form font_entity.
        let font_extra = font_entity.aref(font_property_index::FONT_EXTRA_INDEX);

        let val = unsafe { Fassoc(":postscript-name".into(), font_extra, Qnil) };

        if val.is_nil() {
            return -1;
        }

        let c = std::char::from_u32(c as u32);

        if c.is_none() {
            return 0;
        }

        let c = c.unwrap();

        let (font_data, face_index) = FontDB::data_from_desc(desc.clone());
        ttf_parser::Face::parse(&font_data, face_index)
            .ok()
            .and_then(|face| face.glyph_index(c))
            .is_some() as i32
    } else {
        let font = font.as_font().unwrap().as_font_mut();

        (encode_char(font, c) != FONT_INVALID_CODE) as i32
    }
}

#[allow(unused_variables)]
extern "C" fn text_extents(
    font: *mut font,
    code: *const u32,
    nglyphs: i32,
    metrics: *mut font_metrics,
) {
    let font = WRFontRef::new(font as *mut WRFont);

    let glyph_indices: Vec<u32> = unsafe { std::slice::from_raw_parts(code, nglyphs as usize) }
        .iter()
        .copied()
        .collect();

    let width: i32 = font
        .get_glyph_advance_width(glyph_indices.clone())
        .into_iter()
        .filter_map(|w| w)
        .sum();

    unsafe {
        (*metrics).lbearing = 0;
        (*metrics).rbearing = width as i16;
        (*metrics).width = width as i16;
        (*metrics).ascent = font.font.ascent as i16;
        (*metrics).descent = font.font.descent as i16;
    }
}
