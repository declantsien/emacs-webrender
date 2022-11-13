use font_kit::{
    family_name::FamilyName,
    properties::{Properties, Stretch, Style, Weight},
    source::{Source, SystemSource},
};
use fontdb::{FaceInfo, Family, Query};

pub struct FontDB {
    pub db: fontdb::Database,
}

impl FontDB {
    pub fn new() -> FontDB {
        let mut db = fontdb::Database::new();
	#[cfg(any(target_os = "macos", target_os = "windows"))]
	db.load_system_fonts();
	#[cfg(any(target_os = "android", all(unix, not(target_os = "macos"))))]
	{
	    let source = SystemSource::new();
            let font_handles = source.all_fonts().unwrap();
            for handle in font_handles {
		if let font_kit::handle::Handle::Path {
                    ref path,
                    font_index,
		} = handle
		{
                    if let Err(e) = db.load_font_file(path) {
			log::warn!("Failed to load '{}' cause {}.", path.display(), e);
                    }
		}
            }
	};


        FontDB { db }
    }

    pub fn select_family(&self, family_name: &str) -> Vec<&FaceInfo> {
	self.normalize_family_name(family_name)
	    .map(|family| {
                self.db
                    .faces()
                    .iter()
                    .filter(|f| f.family == family)
                    .collect::<Vec<&FaceInfo>>()
            })
            .unwrap_or_else(|| Vec::new())
    }

    pub fn select_postscript(&self, postscript_name: &str) -> Option<&FaceInfo> {
        self.db
            .faces()
            .iter()
            .filter(|f| f.post_script_name == postscript_name)
            .next()
    }

    pub fn query(&self, query: &Query<'_>) -> Option<&FaceInfo> {
        self.db.query(query).and_then(|id| self.db.face(id))
    }

    pub fn select_best_match(
        &self,
        family_names: &[FamilyName],
        properties: &Properties,
    ) -> Option<font_kit::font::Font> {
        let selection_result = SystemSource::new().select_best_match(family_names, &properties);
        match selection_result {
            Ok(handle) => {
                let loading_result = handle.load();
                match loading_result {
                    Ok(font) => Some(font),
                    Err(e) => {
                        log::warn!("Failed to load {:?} cause {}.", family_names, e);
                        None
                    }
                }
            }
            Err(e) => {
                log::warn!("Failed to select {:?} cause {}.", family_names, e);
                None
            }
        }
    }

    pub fn all_fonts(&self) -> Vec<&FaceInfo> {
        self.db.faces().iter().collect::<Vec<&FaceInfo>>()
    }

    pub fn all_families(&self) -> Option<Vec<String>> {
        match SystemSource::new().all_families().ok() {
            Some(families_) => {
                let mut families = vec![];
                for family in families_ {
                    families.push(family.replace('\'', "").trim().to_string());
                }
                Some(families)
            }
            _ => None,
        }
    }

    fn default_font_family(default_font_family: FamilyName) -> Option<String> {
        let family = SystemSource::new()
            .select_family_by_generic_name(&default_font_family)
            .ok()?;

        let fonts = family.fonts();

        if fonts.len() == 0 {
            return None;
        }

        let font = fonts[0].load().ok()?;

        Some(font.family_name())
    }

    pub fn normalize_family_name<'a>(&'a self, family_name: &'a str) -> Option<&'a str> {
        let family_name = match family_name.clone().to_lowercase().as_str() {
            "serif" => FamilyName::Serif,
            "sans-serif" => FamilyName::SansSerif,
	    "sans serif" => FamilyName::SansSerif,
            "monospace" => FamilyName::Monospace,
            "cursive" => FamilyName::Cursive,
            "fantasy" => FamilyName::Fantasy,
            _ => FamilyName::Title(family_name.to_string()),
        };

        let mut family_names = Vec::new();
        family_names.push(family_name.clone());
        let properties = font_kit::properties::Properties::default();
        let font = self.select_best_match(&family_names, &properties);
        let face_info = if let Some(font) = font {
	    // println!("fc font name: {:?}", font.family_name());
            let postscript_name = font.postscript_name().unwrap_or("?".to_string());
	    // println!("fc postscript_name: {:?}", postscript_name);
	    self.select_postscript(postscript_name.as_str())
        } else {
            None
        };

	let family_name = if let Some(info) = face_info {
	    // println!("ttf-parser font family: {:?}", info.family);
            Some(info.family.as_str())
        } else {
            None
        };

	family_name
    }
}
