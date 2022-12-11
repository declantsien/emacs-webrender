use font_loader::system_fonts;

use libc::c_int;
use lisp_types::{
    bindings::{font_property_index, xlispstrdup, AREF, SYMBOL_NAME},
    lisp::LispObject,
};
use std::path::PathBuf;
use std::str;
use std::{ffi::CStr, fs::File, io::Read};
use ttf_parser::{Style, Weight};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum FontDescriptor {
    Path {
        path: PathBuf,
        font_index: u32,
    },
    Family {
        name: String,
    },
    Properties {
        family: String,
        weight: Weight,
        style: Style,
        stretch: u32,
    },
}

impl FontDescriptor {
    pub fn from_font_spec(font_spec: LispObject) -> FontDescriptor {
        let family = unsafe {
            AREF(
                font_spec,
                font_property_index::FONT_FAMILY_INDEX.try_into().unwrap(),
            )
        };
        // unsafe{ debug_print(font_spec)};
        let family = unsafe { xlispstrdup(SYMBOL_NAME(family)) };
        let family: &CStr = unsafe { CStr::from_ptr(family) };
        let family: &str = family.to_str().unwrap();
        let family: String = family.to_owned();
        let family = family.replace(" Regular", "");
        // println!("family: {:?}", family);

        //TODO Read weight/style/stretch
        FontDescriptor::Properties {
            family,
            weight: Weight::from(400),
            style: Style::Normal,
            stretch: None.unwrap_or(5) as u32,
        }

        // let path = rsrc_path(&item["font"], aux_dir);
        // FontDescriptor::Path {
        //     path,
        //     font_index: item["font-index"].as_i64().unwrap_or(0) as u32,
        // }

        // FontDescriptor::Family {
        //     name: PLATFORM_DEFAULT_FACE_NAME.to_string(),
        // }
    }
}

pub struct FontDB {}

impl FontDB {
    pub fn new() -> FontDB {
        FontDB {}
    }

    pub fn normalize_family_name(family_name: &str) -> String {
        match family_name.clone().to_lowercase().as_str() {
            "serif" => "Times New Roman".to_string(),
            "sans-serif" => "Arial".to_string(),
            "sans serif" => "Arial".to_string(),
            "monospace" => "Courier New".to_string(),
            "cursive" => "Comic Sans MS".to_string(),
            #[cfg(target_os = "macos")]
            "fantasy" => "Papyrus".to_string(),
            #[cfg(not(target_os = "macos"))]
            "fantasy" => "Impact".to_string(),
            _ => family_name.to_string(),
        }
    }

    pub fn data_from_desc(desc: FontDescriptor) -> Option<(Vec<u8>, c_int)> {
        // println!("desc: {:?}", desc);
        match desc {
            FontDescriptor::Path {
                ref path,
                font_index,
            } => {
                let mut file = File::open(path).expect("Couldn't open font file");
                let mut bytes = vec![];
                file.read_to_end(&mut bytes)
                    .expect("failed to read font file");
                Some((bytes, font_index.try_into().unwrap()))
            }
            FontDescriptor::Family { ref name } => FontDB::data_from_name(name),
            FontDescriptor::Properties {
                ref family,
                weight,
                style,
                stretch: _,
            } => FontDB::data_from_properties(family, weight, style),
        }
    }

    pub fn data_from_name(family_name: &str) -> Option<(Vec<u8>, c_int)> {
        let family_name = Self::normalize_family_name(family_name);
        let property = system_fonts::FontPropertyBuilder::new()
            .family(&family_name)
            .build();
        system_fonts::get(&property)
    }

    pub fn data_from_properties(
        family_name: &str,
        weight: Weight,
        style: Style,
    ) -> Option<(Vec<u8>, c_int)> {
        let family_name = Self::normalize_family_name(family_name);

        let mut property = system_fonts::FontPropertyBuilder::new().family(&family_name);

        if weight.to_number() >= 700 {
            property = property.bold();
        }

        property = match style {
            Style::Normal => property,
            Style::Italic => property.italic(),
            Style::Oblique => property.oblique(),
        };

        let property = property.build();
        match system_fonts::get(&property) {
            Some(result) => Some(result),
            None => {
                eprint!(
                    "Failed loading font: {:?}, weight: {:?}, style: {:?}.",
                    family_name, weight, style
                );
                None
            }
        }
    }
}
