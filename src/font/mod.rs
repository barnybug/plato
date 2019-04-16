mod harfbuzz_sys;
mod freetype_sys;

use self::harfbuzz_sys::*;
use self::freetype_sys::*;

use std::str;
use std::ptr;
use std::slice;
use std::ffi::{CString, CStr};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;
use std::collections::BTreeSet;
use std::rc::Rc;
use fnv::FnvHashMap;
use bitflags::bitflags;
use failure::{Error, Fail, format_err};
use glob::glob;
use crate::geom::Point;
use crate::framebuffer::Framebuffer;

// Font sizes in 1/64th of a point
pub const FONT_SIZES: [u32; 3] = [349, 524, 629];

pub const KEYBOARD_FONT_SIZES: [u32; 2] = [337, 843];

pub const DISPLAY_FONT_SIZE: u32 = 2516;

pub const NORMAL_STYLE: Style = Style {
    family: Family::SansSerif,
    variant: Variant::REGULAR,
    size: FONT_SIZES[1],
};

pub const KBD_CHAR: Style = Style {
    family: Family::Keyboard,
    variant: Variant::REGULAR,
    size: KEYBOARD_FONT_SIZES[1],
};

pub const KBD_LABEL: Style = Style {
    family: Family::Keyboard,
    variant: Variant::REGULAR,
    size: FONT_SIZES[0],
};

pub const DISPLAY_STYLE: Style = Style {
    family: Family::Display,
    variant: Variant::REGULAR,
    size: DISPLAY_FONT_SIZE,
};

pub const MD_TITLE: Style = Style {
    family: Family::Serif,
    variant: Variant::ITALIC,
    size: FONT_SIZES[2],
};

pub const MD_AUTHOR: Style = Style {
    family: Family::Serif,
    variant: Variant::REGULAR,
    size: FONT_SIZES[1],
};

pub const MD_YEAR: Style = NORMAL_STYLE;

pub const MD_KIND: Style = Style {
    family: Family::SansSerif,
    variant: Variant::BOLD,
    size: FONT_SIZES[0],
};

pub const MD_SIZE: Style = Style {
    family: Family::SansSerif,
    variant: Variant::REGULAR,
    size: FONT_SIZES[0],
};

#[link(name="mupdf")]
extern {
    // Extracted from mupdf via `head -n 1 generated/resources/fonts/noto/*`
    pub static _binary_DroidSansFallback_ttf: [libc::c_uchar; 3556308];
    pub static _binary_NotoEmoji_Regular_ttf: [libc::c_uchar; 418804];
    pub static _binary_NotoKufiArabic_Regular_ttf: [libc::c_uchar; 62996];
    pub static _binary_NotoNaskhArabic_Regular_ttf: [libc::c_uchar; 136084];
    pub static _binary_NotoNastaliqUrdu_Regular_ttf: [libc::c_uchar; 497204];
    pub static _binary_NotoSans_Regular_otf: [libc::c_uchar; 232644];
    pub static _binary_NotoSansAdlam_Regular_otf: [libc::c_uchar; 30084];
    pub static _binary_NotoSansAhom_Regular_otf: [libc::c_uchar; 13852];
    pub static _binary_NotoSansAnatolianHieroglyphs_Regular_otf: [libc::c_uchar; 134908];
    pub static _binary_NotoSansArabic_Regular_otf: [libc::c_uchar; 121308];
    pub static _binary_NotoSansAvestan_Regular_otf: [libc::c_uchar; 9380];
    pub static _binary_NotoSansBamum_Regular_otf: [libc::c_uchar; 104656];
    pub static _binary_NotoSansBassaVah_Regular_otf: [libc::c_uchar; 6332];
    pub static _binary_NotoSansBatak_Regular_otf: [libc::c_uchar; 11184];
    pub static _binary_NotoSansBengali_Regular_otf: [libc::c_uchar; 79944];
    pub static _binary_NotoSansBhaiksuki_Regular_otf: [libc::c_uchar; 100344];
    pub static _binary_NotoSansBrahmi_Regular_otf: [libc::c_uchar; 27528];
    pub static _binary_NotoSansBuginese_Regular_otf: [libc::c_uchar; 6312];
    pub static _binary_NotoSansBuhid_Regular_otf: [libc::c_uchar; 5100];
    pub static _binary_NotoSansCanadianAboriginal_Regular_otf: [libc::c_uchar; 38508];
    pub static _binary_NotoSansCarian_Regular_otf: [libc::c_uchar; 5684];
    pub static _binary_NotoSansChakma_Regular_otf: [libc::c_uchar; 28492];
    pub static _binary_NotoSansCham_Regular_otf: [libc::c_uchar; 21380];
    pub static _binary_NotoSansCherokee_Regular_otf: [libc::c_uchar; 56872];
    pub static _binary_NotoSansCoptic_Regular_otf: [libc::c_uchar; 21620];
    pub static _binary_NotoSansCuneiform_Regular_otf: [libc::c_uchar; 416856];
    pub static _binary_NotoSansCypriot_Regular_otf: [libc::c_uchar; 7116];
    pub static _binary_NotoSansDeseret_Regular_otf: [libc::c_uchar; 8972];
    pub static _binary_NotoSansDevanagari_Regular_otf: [libc::c_uchar; 115204];
    pub static _binary_NotoSansEgyptianHieroglyphs_Regular_otf: [libc::c_uchar; 364888];
    pub static _binary_NotoSansElbasan_Regular_otf: [libc::c_uchar; 8788];
    pub static _binary_NotoSansGlagolitic_Regular_otf: [libc::c_uchar; 17384];
    pub static _binary_NotoSansGothic_Regular_otf: [libc::c_uchar; 5572];
    pub static _binary_NotoSansHanunoo_Regular_otf: [libc::c_uchar; 6668];
    pub static _binary_NotoSansHatran_Regular_otf: [libc::c_uchar; 4388];
    pub static _binary_NotoSansImperialAramaic_Regular_otf: [libc::c_uchar; 5516];
    pub static _binary_NotoSansInscriptionalPahlavi_Regular_otf: [libc::c_uchar; 5536];
    pub static _binary_NotoSansInscriptionalParthian_Regular_otf: [libc::c_uchar; 6864];
    pub static _binary_NotoSansJavanese_Regular_ttf: [libc::c_uchar; 40468];
    pub static _binary_NotoSansKaithi_Regular_otf: [libc::c_uchar; 39768];
    pub static _binary_NotoSansKayahLi_Regular_otf: [libc::c_uchar; 7184];
    pub static _binary_NotoSansKharoshthi_Regular_otf: [libc::c_uchar; 19396];
    pub static _binary_NotoSansLepcha_Regular_otf: [libc::c_uchar; 18948];
    pub static _binary_NotoSansLimbu_Regular_otf: [libc::c_uchar; 10140];
    pub static _binary_NotoSansLinearA_Regular_otf: [libc::c_uchar; 33916];
    pub static _binary_NotoSansLinearB_Regular_otf: [libc::c_uchar; 36860];
    pub static _binary_NotoSansLisu_Regular_otf: [libc::c_uchar; 5472];
    pub static _binary_NotoSansLycian_Regular_otf: [libc::c_uchar; 4180];
    pub static _binary_NotoSansLydian_Regular_otf: [libc::c_uchar; 4164];
    pub static _binary_NotoSansMalayalam_Regular_otf: [libc::c_uchar; 48048];
    pub static _binary_NotoSansMandaic_Regular_otf: [libc::c_uchar; 13228];
    pub static _binary_NotoSansManichaean_Regular_otf: [libc::c_uchar; 16608];
    pub static _binary_NotoSansMarchen_Regular_otf: [libc::c_uchar; 63992];
    pub static _binary_NotoSansMeeteiMayek_Regular_otf: [libc::c_uchar; 12112];
    pub static _binary_NotoSansMendeKikakui_Regular_otf: [libc::c_uchar; 19800];
    pub static _binary_NotoSansMeroitic_Regular_otf: [libc::c_uchar; 20064];
    pub static _binary_NotoSansMiao_Regular_otf: [libc::c_uchar; 22908];
    pub static _binary_NotoSansMongolian_Regular_ttf: [libc::c_uchar; 135484];
    pub static _binary_NotoSansMro_Regular_otf: [libc::c_uchar; 5680];
    pub static _binary_NotoSansMultani_Regular_otf: [libc::c_uchar; 7808];
    pub static _binary_NotoSansNKo_Regular_otf: [libc::c_uchar; 13492];
    pub static _binary_NotoSansNabataean_Regular_otf: [libc::c_uchar; 6624];
    pub static _binary_NotoSansNewTaiLue_Regular_otf: [libc::c_uchar; 11240];
    pub static _binary_NotoSansNewa_Regular_otf: [libc::c_uchar; 66132];
    pub static _binary_NotoSansOgham_Regular_otf: [libc::c_uchar; 3796];
    pub static _binary_NotoSansOlChiki_Regular_otf: [libc::c_uchar; 6916];
    pub static _binary_NotoSansOldItalic_Regular_otf: [libc::c_uchar; 4716];
    pub static _binary_NotoSansOldNorthArabian_Regular_otf: [libc::c_uchar; 6276];
    pub static _binary_NotoSansOldPermic_Regular_otf: [libc::c_uchar; 8628];
    pub static _binary_NotoSansOldPersian_Regular_otf: [libc::c_uchar; 9864];
    pub static _binary_NotoSansOldSouthArabian_Regular_otf: [libc::c_uchar; 4424];
    pub static _binary_NotoSansOldTurkic_Regular_otf: [libc::c_uchar; 6992];
    pub static _binary_NotoSansOriya_Regular_ttf: [libc::c_uchar; 103684];
    pub static _binary_NotoSansOsage_Regular_otf: [libc::c_uchar; 9384];
    pub static _binary_NotoSansOsmanya_Regular_otf: [libc::c_uchar; 6864];
    pub static _binary_NotoSansPahawhHmong_Regular_otf: [libc::c_uchar; 13228];
    pub static _binary_NotoSansPalmyrene_Regular_otf: [libc::c_uchar; 8604];
    pub static _binary_NotoSansPauCinHau_Regular_otf: [libc::c_uchar; 8204];
    pub static _binary_NotoSansPhagsPa_Regular_otf: [libc::c_uchar; 24324];
    pub static _binary_NotoSansPhoenician_Regular_otf: [libc::c_uchar; 5340];
    pub static _binary_NotoSansRejang_Regular_otf: [libc::c_uchar; 6564];
    pub static _binary_NotoSansRunic_Regular_otf: [libc::c_uchar; 7304];
    pub static _binary_NotoSansSamaritan_Regular_otf: [libc::c_uchar; 9160];
    pub static _binary_NotoSansSaurashtra_Regular_otf: [libc::c_uchar; 16128];
    pub static _binary_NotoSansSharada_Regular_otf: [libc::c_uchar; 27752];
    pub static _binary_NotoSansShavian_Regular_otf: [libc::c_uchar; 5560];
    pub static _binary_NotoSansSoraSompeng_Regular_otf: [libc::c_uchar; 6388];
    pub static _binary_NotoSansSundanese_Regular_otf: [libc::c_uchar; 9416];
    pub static _binary_NotoSansSylotiNagri_Regular_otf: [libc::c_uchar; 13124];
    pub static _binary_NotoSansSymbols_Regular_otf: [libc::c_uchar; 107728];
    pub static _binary_NotoSansSymbols2_Regular_otf: [libc::c_uchar; 319912];
    pub static _binary_NotoSansSyriacEastern_Regular_ttf: [libc::c_uchar; 50164];
    pub static _binary_NotoSansSyriacEstrangela_Regular_ttf: [libc::c_uchar; 46396];
    pub static _binary_NotoSansSyriacWestern_Regular_ttf: [libc::c_uchar; 52380];
    pub static _binary_NotoSansTagalog_Regular_otf: [libc::c_uchar; 5612];
    pub static _binary_NotoSansTagbanwa_Regular_otf: [libc::c_uchar; 5800];
    pub static _binary_NotoSansTaiLe_Regular_otf: [libc::c_uchar; 8704];
    pub static _binary_NotoSansTaiTham_Regular_ttf: [libc::c_uchar; 51040];
    pub static _binary_NotoSansTaiViet_Regular_otf: [libc::c_uchar; 12420];
    pub static _binary_NotoSansThaana_Regular_ttf: [libc::c_uchar; 15284];
    pub static _binary_NotoSansTibetan_Regular_ttf: [libc::c_uchar; 422408];
    pub static _binary_NotoSansTifinagh_Regular_otf: [libc::c_uchar; 11516];
    pub static _binary_NotoSansUgaritic_Regular_otf: [libc::c_uchar; 5412];
    pub static _binary_NotoSansVai_Regular_otf: [libc::c_uchar; 24884];
    pub static _binary_NotoSansYi_Regular_otf: [libc::c_uchar; 93272];
    pub static _binary_NotoSerif_Regular_otf: [libc::c_uchar; 289080];
    pub static _binary_NotoSerifArmenian_Regular_otf: [libc::c_uchar; 13628];
    pub static _binary_NotoSerifBalinese_Regular_otf: [libc::c_uchar; 32620];
    pub static _binary_NotoSerifEthiopic_Regular_otf: [libc::c_uchar; 112600];
    pub static _binary_NotoSerifGeorgian_Regular_otf: [libc::c_uchar; 22304];
    pub static _binary_NotoSerifGujarati_Regular_otf: [libc::c_uchar; 63308];
    pub static _binary_NotoSerifGurmukhi_Regular_otf: [libc::c_uchar; 27584];
    pub static _binary_NotoSerifHebrew_Regular_otf: [libc::c_uchar; 15280];
    pub static _binary_NotoSerifKannada_Regular_otf: [libc::c_uchar; 78420];
    pub static _binary_NotoSerifKhmer_Regular_otf: [libc::c_uchar; 40688];
    pub static _binary_NotoSerifLao_Regular_otf: [libc::c_uchar; 16016];
    pub static _binary_NotoSerifMyanmar_Regular_otf: [libc::c_uchar; 137544];
    pub static _binary_NotoSerifSinhala_Regular_otf: [libc::c_uchar; 74676];
    pub static _binary_NotoSerifTamil_Regular_otf: [libc::c_uchar; 30984];
    pub static _binary_NotoSerifTelugu_Regular_ttf: [libc::c_uchar; 157544];
    pub static _binary_NotoSerifThai_Regular_otf: [libc::c_uchar; 17280];
}

pub const SLIDER_VALUE: Style = MD_SIZE;

const CATEGORY_DEPTH_LIMIT: usize = 5;

pub fn category_font_size(depth: usize) -> u32 {
    let k = (2.0 / 3.0f32).powf(CATEGORY_DEPTH_LIMIT.min(depth) as f32 /
                                CATEGORY_DEPTH_LIMIT as f32);
    (k * FONT_SIZES[1] as f32) as u32
}

pub struct FontFamily {
    pub regular: Font,
    pub italic: Font,
    pub bold: Font,
    pub bold_italic: Font,
}

pub fn family_names<P: AsRef<Path>>(search_path: P) -> Result<BTreeSet<String>, Error> {
    let opener = FontOpener::new()?;
    let end_path = Path::new("**").join("*.[ot]tf");
    let pattern_path = search_path.as_ref().join(&end_path);
    let pattern = pattern_path.to_str().unwrap_or_default();

    let mut families = BTreeSet::new();

    for path in glob(pattern)?.filter_map(Result::ok) {
        let font = opener.open(&path)?;
        if let Some(family_name) = font.family_name() {
            families.insert(family_name.to_string());
        }
    }

    Ok(families)
}

impl FontFamily {
    pub fn from_name<P: AsRef<Path>>(family_name: &str, search_path: P) -> Result<FontFamily, Error> {
        let opener = FontOpener::new()?;
        let end_path = Path::new("**").join("*.[ot]tf");
        let pattern_path = search_path.as_ref().join(&end_path);
        let pattern = pattern_path.to_str().unwrap_or_default();

        let mut styles = FnvHashMap::default();

        for path in glob(pattern)?.filter_map(Result::ok) {
            let font = opener.open(&path)?;
            if font.family_name() == Some(family_name) {
                styles.insert(font.style_name().map(String::from)
                                  .unwrap_or_else(|| "Regular".to_string()),
                              path.clone());
            }
        }

        let regular_path = styles.get("Regular")
                                 .or_else(|| styles.get("Roman"))
                                 .or_else(|| styles.get("Book"))
                                 .ok_or_else(|| format_err!("Can't find regular style."))?;
        let italic_path = styles.get("Italic")
                                .or_else(|| styles.get("Book Italic"))
                                .unwrap_or(regular_path);
        let bold_path = styles.get("Bold")
                              .or_else(|| styles.get("Semibold"))
                              .or_else(|| styles.get("SemiBold"))
                              .or_else(|| styles.get("Medium"))
                              .unwrap_or(regular_path);
        let bold_italic_path = styles.get("Bold Italic")
                                     .or_else(|| styles.get("SemiBold Italic"))
                                     .or_else(|| styles.get("Medium Italic"))
                                     .unwrap_or(italic_path);
        Ok(FontFamily {
            regular: opener.open(regular_path)?,
            italic: opener.open(italic_path)?,
            bold: opener.open(bold_path)?,
            bold_italic: opener.open(bold_italic_path)?,
        })
    }
}

pub struct Fonts {
    pub sans_serif: FontFamily,
    pub serif: FontFamily,
    pub monospace: FontFamily,
    pub keyboard: Font,
    pub display: Font,
}

impl Fonts {
    pub fn load() -> Result<Fonts, Error> {
        let opener = FontOpener::new()?;
        let mut fonts = Fonts {
            sans_serif: FontFamily {
                regular: opener.open("fonts/NotoSans-Regular.ttf")?,
                italic: opener.open("fonts/NotoSans-Italic.ttf")?,
                bold: opener.open("fonts/NotoSans-Bold.ttf")?,
                bold_italic: opener.open("fonts/NotoSans-BoldItalic.ttf")?,
            },
            serif: FontFamily {
                regular: opener.open("fonts/NotoSerif-Regular.ttf")?,
                italic: opener.open("fonts/NotoSerif-Italic.ttf")?,
                bold: opener.open("fonts/NotoSerif-Bold.ttf")?,
                bold_italic: opener.open("fonts/NotoSerif-BoldItalic.ttf")?,
            },
            monospace: FontFamily {
                regular: opener.open("fonts/SourceCodeVariable-Roman.otf")?,
                italic: opener.open("fonts/SourceCodeVariable-Italic.otf")?,
                bold: opener.open("fonts/SourceCodeVariable-Roman.otf")?,
                bold_italic: opener.open("fonts/SourceCodeVariable-Italic.otf")?,
            },
            keyboard: opener.open("fonts/VarelaRound-Regular.ttf")?,
            display: opener.open("fonts/Cormorant-Regular.ttf")?,
        };
        fonts.monospace.bold.set_variations(&["wght=600"]);
        fonts.monospace.bold_italic.set_variations(&["wght=600"]);
        Ok(fonts)
    }
}

bitflags! {
    pub struct Variant: u8 {
        const REGULAR = 0;
        const ITALIC = 1;
        const BOLD = 2;
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Family {
    SansSerif,
    Serif,
    Monospace,
    Keyboard,
    Display,
}

pub struct Style {
    family: Family,
    variant: Variant,
    pub size: u32,
}

pub fn font_from_variant(family: &mut FontFamily, variant: Variant) -> &mut Font {
    if variant.contains(Variant::ITALIC | Variant::BOLD) {
        &mut family.bold_italic
    } else if variant.contains(Variant::ITALIC) {
        &mut family.italic
    } else if variant.contains(Variant::BOLD) {
        &mut family.bold
    } else {
        &mut family.regular
    }
}

pub fn font_from_style<'a>(fonts: &'a mut Fonts, style: &Style, dpi: u16) -> &'a mut Font {
    let font = match style.family {
        Family::SansSerif => {
            let family = &mut fonts.sans_serif;
            font_from_variant(family, style.variant)
        },
        Family::Serif => {
            let family = &mut fonts.serif;
            font_from_variant(family, style.variant)
        },
        Family::Monospace => {
            let family = &mut fonts.monospace;
            font_from_variant(family, style.variant)
        },
        Family::Keyboard => &mut fonts.keyboard,
        Family::Display => &mut fonts.display,
    };
    font.set_size(style.size, dpi);
    font
}

#[inline]
unsafe fn font_data_from_script(script: HbScript) -> &'static [libc::c_uchar] {
    // Extracted from mupdf in source/fitz/noto.c
    match script {
	HB_SCRIPT_HANGUL |
	HB_SCRIPT_HIRAGANA |
	HB_SCRIPT_KATAKANA |
	HB_SCRIPT_BOPOMOFO |
	HB_SCRIPT_HAN => &_binary_DroidSansFallback_ttf,

	HB_SCRIPT_ARABIC => &_binary_NotoNaskhArabic_Regular_ttf,
	HB_SCRIPT_SYRIAC => &_binary_NotoSansSyriacWestern_Regular_ttf,
	HB_SCRIPT_MEROITIC_CURSIVE |
	HB_SCRIPT_MEROITIC_HIEROGLYPHS => &_binary_NotoSansMeroitic_Regular_otf,

	HB_SCRIPT_ADLAM => &_binary_NotoSansAdlam_Regular_otf,
	HB_SCRIPT_AHOM => &_binary_NotoSansAhom_Regular_otf,
	HB_SCRIPT_ANATOLIAN_HIEROGLYPHS => &_binary_NotoSansAnatolianHieroglyphs_Regular_otf,
	HB_SCRIPT_ARMENIAN => &_binary_NotoSerifArmenian_Regular_otf,
	HB_SCRIPT_AVESTAN => &_binary_NotoSansAvestan_Regular_otf,
	HB_SCRIPT_BALINESE => &_binary_NotoSerifBalinese_Regular_otf,
	HB_SCRIPT_BAMUM => &_binary_NotoSansBamum_Regular_otf,
	HB_SCRIPT_BASSA_VAH => &_binary_NotoSansBassaVah_Regular_otf,
	HB_SCRIPT_BATAK => &_binary_NotoSansBatak_Regular_otf,
	HB_SCRIPT_BENGALI => &_binary_NotoSansBengali_Regular_otf,
	HB_SCRIPT_BHAIKSUKI => &_binary_NotoSansBhaiksuki_Regular_otf,
	HB_SCRIPT_BRAHMI => &_binary_NotoSansBrahmi_Regular_otf,
	HB_SCRIPT_BUGINESE => &_binary_NotoSansBuginese_Regular_otf,
	HB_SCRIPT_BUHID => &_binary_NotoSansBuhid_Regular_otf,
	HB_SCRIPT_CANADIAN_SYLLABICS => &_binary_NotoSansCanadianAboriginal_Regular_otf,
	HB_SCRIPT_CARIAN => &_binary_NotoSansCarian_Regular_otf,
	HB_SCRIPT_CHAKMA => &_binary_NotoSansChakma_Regular_otf,
	HB_SCRIPT_CHAM => &_binary_NotoSansCham_Regular_otf,
	HB_SCRIPT_CHEROKEE => &_binary_NotoSansCherokee_Regular_otf,
	HB_SCRIPT_COPTIC => &_binary_NotoSansCoptic_Regular_otf,
	HB_SCRIPT_CUNEIFORM => &_binary_NotoSansCuneiform_Regular_otf,
	HB_SCRIPT_CYPRIOT => &_binary_NotoSansCypriot_Regular_otf,
	HB_SCRIPT_DESERET => &_binary_NotoSansDeseret_Regular_otf,
	HB_SCRIPT_DEVANAGARI => &_binary_NotoSansDevanagari_Regular_otf,
	HB_SCRIPT_EGYPTIAN_HIEROGLYPHS => &_binary_NotoSansEgyptianHieroglyphs_Regular_otf,
	HB_SCRIPT_ELBASAN => &_binary_NotoSansElbasan_Regular_otf,
	HB_SCRIPT_ETHIOPIC => &_binary_NotoSerifEthiopic_Regular_otf,
	HB_SCRIPT_GEORGIAN => &_binary_NotoSerifGeorgian_Regular_otf,
	HB_SCRIPT_GLAGOLITIC => &_binary_NotoSansGlagolitic_Regular_otf,
	HB_SCRIPT_GOTHIC => &_binary_NotoSansGothic_Regular_otf,
	HB_SCRIPT_GUJARATI => &_binary_NotoSerifGujarati_Regular_otf,
	HB_SCRIPT_GURMUKHI => &_binary_NotoSerifGurmukhi_Regular_otf,
	HB_SCRIPT_HANUNOO => &_binary_NotoSansHanunoo_Regular_otf,
	HB_SCRIPT_HATRAN => &_binary_NotoSansHatran_Regular_otf,
	HB_SCRIPT_HEBREW => &_binary_NotoSerifHebrew_Regular_otf,
	HB_SCRIPT_IMPERIAL_ARAMAIC => &_binary_NotoSansImperialAramaic_Regular_otf,
	HB_SCRIPT_INSCRIPTIONAL_PAHLAVI => &_binary_NotoSansInscriptionalPahlavi_Regular_otf,
	HB_SCRIPT_INSCRIPTIONAL_PARTHIAN => &_binary_NotoSansInscriptionalParthian_Regular_otf,
	HB_SCRIPT_JAVANESE => &_binary_NotoSansJavanese_Regular_ttf,
	HB_SCRIPT_KAITHI => &_binary_NotoSansKaithi_Regular_otf,
	HB_SCRIPT_KANNADA => &_binary_NotoSerifKannada_Regular_otf,
	HB_SCRIPT_KAYAH_LI => &_binary_NotoSansKayahLi_Regular_otf,
	HB_SCRIPT_KHAROSHTHI => &_binary_NotoSansKharoshthi_Regular_otf,
	HB_SCRIPT_KHMER => &_binary_NotoSerifKhmer_Regular_otf,
	HB_SCRIPT_LAO => &_binary_NotoSerifLao_Regular_otf,
	HB_SCRIPT_LEPCHA => &_binary_NotoSansLepcha_Regular_otf,
	HB_SCRIPT_LIMBU => &_binary_NotoSansLimbu_Regular_otf,
	HB_SCRIPT_LINEAR_A => &_binary_NotoSansLinearA_Regular_otf,
	HB_SCRIPT_LINEAR_B => &_binary_NotoSansLinearB_Regular_otf,
	HB_SCRIPT_LISU => &_binary_NotoSansLisu_Regular_otf,
	HB_SCRIPT_LYCIAN => &_binary_NotoSansLycian_Regular_otf,
	HB_SCRIPT_LYDIAN => &_binary_NotoSansLydian_Regular_otf,
	HB_SCRIPT_MALAYALAM => &_binary_NotoSansMalayalam_Regular_otf,
	HB_SCRIPT_MANDAIC => &_binary_NotoSansMandaic_Regular_otf,
	HB_SCRIPT_MANICHAEAN => &_binary_NotoSansManichaean_Regular_otf,
	HB_SCRIPT_MARCHEN => &_binary_NotoSansMarchen_Regular_otf,
	HB_SCRIPT_MEETEI_MAYEK => &_binary_NotoSansMeeteiMayek_Regular_otf,
	HB_SCRIPT_MENDE_KIKAKUI => &_binary_NotoSansMendeKikakui_Regular_otf,
	HB_SCRIPT_MIAO => &_binary_NotoSansMiao_Regular_otf,
	HB_SCRIPT_MONGOLIAN => &_binary_NotoSansMongolian_Regular_ttf,
	HB_SCRIPT_MRO => &_binary_NotoSansMro_Regular_otf,
	HB_SCRIPT_MULTANI => &_binary_NotoSansMultani_Regular_otf,
	HB_SCRIPT_MYANMAR => &_binary_NotoSerifMyanmar_Regular_otf,
	HB_SCRIPT_NABATAEAN => &_binary_NotoSansNabataean_Regular_otf,
	HB_SCRIPT_NEWA => &_binary_NotoSansNewa_Regular_otf,
	HB_SCRIPT_NEW_TAI_LUE => &_binary_NotoSansNewTaiLue_Regular_otf,
	HB_SCRIPT_NKO => &_binary_NotoSansNKo_Regular_otf,
	HB_SCRIPT_OGHAM => &_binary_NotoSansOgham_Regular_otf,
	HB_SCRIPT_OLD_ITALIC => &_binary_NotoSansOldItalic_Regular_otf,
	HB_SCRIPT_OLD_NORTH_ARABIAN => &_binary_NotoSansOldNorthArabian_Regular_otf,
	HB_SCRIPT_OLD_PERMIC => &_binary_NotoSansOldPermic_Regular_otf,
	HB_SCRIPT_OLD_PERSIAN => &_binary_NotoSansOldPersian_Regular_otf,
	HB_SCRIPT_OLD_SOUTH_ARABIAN => &_binary_NotoSansOldSouthArabian_Regular_otf,
	HB_SCRIPT_OLD_TURKIC => &_binary_NotoSansOldTurkic_Regular_otf,
	HB_SCRIPT_OL_CHIKI => &_binary_NotoSansOlChiki_Regular_otf,
	HB_SCRIPT_ORIYA => &_binary_NotoSansOriya_Regular_ttf,
	HB_SCRIPT_OSAGE => &_binary_NotoSansOsage_Regular_otf,
	HB_SCRIPT_OSMANYA => &_binary_NotoSansOsmanya_Regular_otf,
	HB_SCRIPT_PAHAWH_HMONG => &_binary_NotoSansPahawhHmong_Regular_otf,
	HB_SCRIPT_PALMYRENE => &_binary_NotoSansPalmyrene_Regular_otf,
	HB_SCRIPT_PAU_CIN_HAU => &_binary_NotoSansPauCinHau_Regular_otf,
	HB_SCRIPT_PHAGS_PA => &_binary_NotoSansPhagsPa_Regular_otf,
	HB_SCRIPT_PHOENICIAN => &_binary_NotoSansPhoenician_Regular_otf,
	HB_SCRIPT_REJANG => &_binary_NotoSansRejang_Regular_otf,
	HB_SCRIPT_RUNIC => &_binary_NotoSansRunic_Regular_otf,
	HB_SCRIPT_SAMARITAN => &_binary_NotoSansSamaritan_Regular_otf,
	HB_SCRIPT_SAURASHTRA => &_binary_NotoSansSaurashtra_Regular_otf,
	HB_SCRIPT_SHARADA => &_binary_NotoSansSharada_Regular_otf,
	HB_SCRIPT_SHAVIAN => &_binary_NotoSansShavian_Regular_otf,
	HB_SCRIPT_SINHALA => &_binary_NotoSerifSinhala_Regular_otf,
	HB_SCRIPT_SORA_SOMPENG => &_binary_NotoSansSoraSompeng_Regular_otf,
	HB_SCRIPT_SUNDANESE => &_binary_NotoSansSundanese_Regular_otf,
	HB_SCRIPT_SYLOTI_NAGRI => &_binary_NotoSansSylotiNagri_Regular_otf,
	HB_SCRIPT_TAGALOG => &_binary_NotoSansTagalog_Regular_otf,
	HB_SCRIPT_TAGBANWA => &_binary_NotoSansTagbanwa_Regular_otf,
	HB_SCRIPT_TAI_LE => &_binary_NotoSansTaiLe_Regular_otf,
	HB_SCRIPT_TAI_THAM => &_binary_NotoSansTaiTham_Regular_ttf,
	HB_SCRIPT_TAI_VIET => &_binary_NotoSansTaiViet_Regular_otf,
	HB_SCRIPT_TAMIL => &_binary_NotoSerifTamil_Regular_otf,
	HB_SCRIPT_TELUGU => &_binary_NotoSerifTelugu_Regular_ttf,
	HB_SCRIPT_THAANA => &_binary_NotoSansThaana_Regular_ttf,
	HB_SCRIPT_THAI => &_binary_NotoSerifThai_Regular_otf,
	HB_SCRIPT_TIBETAN => &_binary_NotoSansTibetan_Regular_ttf,
	HB_SCRIPT_TIFINAGH => &_binary_NotoSansTifinagh_Regular_otf,
	HB_SCRIPT_UGARITIC => &_binary_NotoSansUgaritic_Regular_otf,
	HB_SCRIPT_VAI => &_binary_NotoSansVai_Regular_otf,
	HB_SCRIPT_YI => &_binary_NotoSansYi_Regular_otf,
	HB_SCRIPT_BRAILLE |
	HB_SYMBOL_GEOMETRIC |
	HB_SYMBOL_ARROW |
	HB_SYMBOL_TECHNICAL |
	HB_SYMBOL_DINGBAT |
	HB_SYMBOL_GAME_CHESS |
	HB_SYMBOL_GAME_DOMINO |
	HB_SYMBOL_GAME_PLAYING_CARD => &_binary_NotoSansSymbols2_Regular_otf,
	HB_SYMBOL_EMOTICON => &_binary_NotoEmoji_Regular_ttf,
	HB_SYMBOL_GRAPHIC_FORM |
	HB_PUNCTUATION_BRACKET_CJK => &_binary_DroidSansFallback_ttf,
        _ => &_binary_NotoSansSymbols_Regular_otf,
    }
}

#[inline]
fn script_from_code(code: u32) -> HbScript {
    match code {
        0x2190 ... 0x21FF |
        0x2B00 ... 0x2B0D |
        0x2B4D ... 0x2B4F |
        0x2B5A ... 0x2B73 |
        0x2B76 ... 0x2B95 |
        0x2B98 ... 0x2BB9 |
        0x2BEC ... 0x2BEF |
        0x2900 ... 0x297F => HB_SYMBOL_ARROW,
        0x2318 | 0x231A | 0x231B |
        0x232B | 0x2324 ... 0x2328 |
        0x2394 | 0x23CE | 0x23CF |
        0x23E9 | 0x23EA | 0x23ED ... 0x23EF |
        0x23F1 ... 0x23FE |
        0x2BBD ... 0x2BBF => HB_SYMBOL_TECHNICAL,
        0x2654 ... 0x265F => HB_SYMBOL_GAME_CHESS,
        0x1F030 ... 0x1F093 => HB_SYMBOL_GAME_DOMINO,
        0x1F0A0 ... 0x1F0F5 => HB_SYMBOL_GAME_PLAYING_CARD,
        0x2500 ... 0x257F => HB_SYMBOL_GRAPHIC_FORM,
        0x25A0 ... 0x25EF |
        0x25F8 ... 0x25FF |
        0x26AA ... 0x26AC |
        0x2B12 ... 0x2B2F |
        0x2B53 ... 0x2B54 |
        0x2BC0 ... 0x2BD1 => HB_SYMBOL_GEOMETRIC,
        0x2722 ... 0x274B |
        0x274D | 0x274F | 0x2750 ... 0x2753 |
        0x2756 ... 0x2775 | 0x2794 |
        0x2798 ... 0x27AF |
        0x27B1 ... 0x27BE => HB_SYMBOL_DINGBAT,
        0x3008 ... 0x3011 |
        0x3014 ... 0x301B |
        0xFF5F ... 0xFF60 |
        0xFF62 ... 0xFF63 => HB_PUNCTUATION_BRACKET_CJK,
        0x1F600 ... 0x1F64F => HB_SYMBOL_EMOTICON,
        _ => HB_SCRIPT_UNKNOWN,
    }
}

pub struct FontLibrary(*mut FtLibrary);

pub struct FontOpener(Rc<FontLibrary>);

pub struct Font {
    lib: Rc<FontLibrary>,
    face: *mut FtFace,
    font: *mut HbFont,
    size: u32,
    dpi: u16,
    // used as truncation mark
    pub ellipsis: RenderPlan,
    // lowercase and uppercase x heights
    pub x_heights: (u32, u32),
    space_codepoint: u32,
}

impl FontOpener {
    pub fn new() -> Result<FontOpener, Error> {
        unsafe {
            let mut lib = ptr::null_mut();
            let ret = FT_Init_FreeType(&mut lib);
            if ret != FT_ERR_OK {
                Err(Error::from(FreetypeError::from(ret)))
            } else {
                Ok(FontOpener(Rc::new(FontLibrary(lib))))
            }
        }
    }

    pub fn open<P: AsRef<Path>>(&self, path: P) -> Result<Font, Error> {
        unsafe {
            let mut face = ptr::null_mut();
            let c_path = CString::new(path.as_ref().as_os_str().as_bytes()).unwrap();
            let ret = FT_New_Face((self.0).0, c_path.as_ptr(), 0, &mut face);
            if ret != FT_ERR_OK {
               return Err(Error::from(FreetypeError::from(ret)));
            }
            let font = ptr::null_mut();
            let ellipsis = RenderPlan::default();
            let x_heights = (0, 0);
            let space_codepoint = FT_Get_Char_Index(face, ' ' as libc::c_ulong);
            Ok(Font { lib: self.0.clone(), face, font,
                      size: 0, dpi: 0, ellipsis, x_heights, space_codepoint })
        }
    }

    pub fn open_memory(&self, buf: &[u8]) -> Result<Font, Error> {
        unsafe {
            let mut face = ptr::null_mut();
            let ret = FT_New_Memory_Face((self.0).0, buf.as_ptr() as *const FtByte, buf.len() as libc::c_long, 0, &mut face);
            if ret != FT_ERR_OK {
               return Err(Error::from(FreetypeError::from(ret)));
            }
            let ellipsis = RenderPlan::default();
            let font = ptr::null_mut();
            let x_heights = (0, 0);
            let space_codepoint = FT_Get_Char_Index(face, ' ' as libc::c_ulong);
            Ok(Font { lib: self.0.clone(), face, font,
                      size: 0, dpi: 0, ellipsis, x_heights, space_codepoint })
        }
    }
}

impl Font {
    pub fn family_name(&self) -> Option<&str> {
        unsafe {
            let ptr = (*self.face).family_name;
            if ptr.is_null() {
                return None;
            }
            CStr::from_ptr(ptr).to_str().ok()
        }
    }

    pub fn style_name(&self) -> Option<&str> {
        unsafe {
            let ptr = (*self.face).style_name;
            if ptr.is_null() {
                return None;
            }
            CStr::from_ptr(ptr).to_str().ok()
        }
    }

    pub fn set_size(&mut self, size: u32, dpi: u16) {
        if !self.font.is_null() && self.size == size && self.dpi == dpi {
            return;
        }

        self.size = size;
        self.dpi = dpi;

        unsafe {
            let ret = FT_Set_Char_Size(self.face, size as FtF26Dot6, 0, dpi as libc::c_uint, 0);

            if ret != FT_ERR_OK {
                return;
            }

            if self.font.is_null() {
                self.font = hb_ft_font_create(self.face, ptr::null());
            } else {
                hb_ft_font_changed(self.font);
            }

            self.ellipsis = self.plan("…", None, None);
            self.x_heights = (self.height('x'), self.height('X'));
        }
    }

    pub fn set_variations(&mut self, specs: &[&str]) {
        unsafe {
            let mut varia = ptr::null_mut();
            let ret = FT_Get_MM_Var(self.face, &mut varia);

            if ret != FT_ERR_OK {
                return;
            }

            let axes_count = (*varia).num_axis as usize;
            let mut coords = Vec::with_capacity(axes_count);

            for i in 0..(axes_count as isize) {
                let axis = ((*varia).axis).offset(i);
                coords.push((*axis).def);
            }

            for s in specs {
                let tn = s[..4].as_bytes();
                let tag = tag(tn[0], tn[1], tn[2], tn[3]);
                let value: f32 = s[5..].parse().unwrap_or_default();

                for i in 0..(axes_count as isize) {
                    let axis = ((*varia).axis).offset(i);

                    if (*axis).tag == tag as libc::c_ulong {
                        let scaled_value = ((value * 65536.0) as FtFixed).min((*axis).maximum)
                                                                         .max((*axis).minimum);
                        *coords.get_unchecked_mut(i as usize) = scaled_value;
                        break;
                    }
                }
            }

            let ret = FT_Set_Var_Design_Coordinates(self.face, coords.len() as libc::c_uint, coords.as_ptr());

            if ret == FT_ERR_OK && !self.font.is_null() {
                hb_ft_font_changed(self.font);
            }

            FT_Done_MM_Var(self.lib.0, varia);
        }
    }

    pub fn set_variations_from_name(&mut self, name: &str) -> bool {
        let mut found = false;

        unsafe {
            let mut varia = ptr::null_mut();
            let ret = FT_Get_MM_Var(self.face, &mut varia);

            if ret != FT_ERR_OK {
                return found;
            }

            let styles_count = (*varia).num_namedstyles as isize;
            let names_count = FT_Get_Sfnt_Name_Count(self.face);
            let mut sfnt_name = FtSfntName::default();

            'outer: for i in 0..styles_count {
                let style = ((*varia).namedstyle).offset(i);
                let strid = (*style).strid as libc::c_ushort;
                for j in 0..names_count {
                    let ret = FT_Get_Sfnt_Name(self.face, j, &mut sfnt_name);

                    if ret != FT_ERR_OK || sfnt_name.name_id != strid {
                        continue;
                    }

                    if sfnt_name.platform_id != TT_PLATFORM_MICROSOFT ||
                       sfnt_name.encoding_id != TT_MS_ID_UNICODE_CS ||
                       sfnt_name.language_id != TT_MS_LANGID_ENGLISH_UNITED_STATES {
                        continue;
                    }

                    let slice = slice::from_raw_parts(sfnt_name.text, sfnt_name.len as usize);
                    // We're assuming ASCII encoded as UTF_16BE
                    let vec_ascii: Vec<u8> = slice.iter().enumerate().filter_map(|x| {
                        if x.0 % 2 == 0 { None } else { Some(*x.1) }
                    }).collect();

                    if let Ok(name_str) = str::from_utf8(&vec_ascii[..]) {
                        if name.eq_ignore_ascii_case(name_str) {
                            found = true;
                            let ret = FT_Set_Var_Design_Coordinates(self.face, (*varia).num_axis, (*style).coords);
                            if ret == FT_ERR_OK && !self.font.is_null() {
                                hb_ft_font_changed(self.font);
                            }
                            break 'outer;
                        }
                    }
                }
            }

            FT_Done_MM_Var(self.lib.0, varia);
        }

        found
    }

    #[inline]
    unsafe fn patch(&mut self, txt: &str, features: &[HbFeature], render_plan: &mut RenderPlan, missing_glyphs: Vec<(usize, usize)>, buf: *mut HbBuffer) {
        let mut drift = 0;
        for (mut start, mut end) in missing_glyphs.into_iter() {
            start = (start as i32 + drift).max(0) as usize;
            end = (end as i32 + drift).max(0) as usize;
            hb_buffer_clear_contents(buf);
            let start_index = render_plan.glyphs[start].cluster;
            let end_index = render_plan.glyphs.get(end).map(|g| g.cluster)
                                       .unwrap_or_else(|| txt.len());
            let chunk = &txt[start_index..end_index];
            hb_buffer_add_utf8(buf, chunk.as_ptr() as *const libc::c_char,
                               chunk.len() as libc::c_int, 0, -1);
            hb_buffer_guess_segment_properties(buf);
            let mut script = hb_buffer_get_script(buf);
            if script == HB_SCRIPT_INVALID || script == HB_SCRIPT_UNKNOWN {
                if let Some(c) = chunk.chars().next() {
                    script = script_from_code(u32::from(c));
                }
            }
            let font_data = font_data_from_script(script);
            let mut face = ptr::null_mut();
            FT_New_Memory_Face((self.lib).0, font_data.as_ptr() as *const FtByte,
                               font_data.len() as libc::c_long, 0, &mut face);
            FT_Set_Pixel_Sizes(face, (*(*self.face).size).metrics.x_ppem as libc::c_uint, 0);
            let font = hb_ft_font_create(face, ptr::null());
            hb_shape(font, buf, features.as_ptr(), features.len() as libc::c_uint);
            let len = hb_buffer_get_length(buf) as usize;
            let info = hb_buffer_get_glyph_infos(buf, ptr::null_mut());
            let pos = hb_buffer_get_glyph_positions(buf, ptr::null_mut());
            let mut glyphs = Vec::with_capacity(len);

            for i in 0..len {
                let pos_i = &*pos.add(i);
                let info_i = &*info.add(i);
                render_plan.width += (pos_i.x_advance >> 6) as u32;
                glyphs.push(GlyphPlan {
                    codepoint: info_i.codepoint,
                    cluster: start_index + info_i.cluster as usize,
                    advance: pt!(pos_i.x_advance >> 6, pos_i.y_advance >> 6),
                    offset: pt!(pos_i.x_offset >> 6, -pos_i.y_offset >> 6),
                });
                render_plan.scripts.insert(start+i, script);
            }

            render_plan.glyphs.splice(start..end, glyphs.into_iter());
            drift += len as i32 - (end - start) as i32;

            hb_font_destroy(font);
            FT_Done_Face(face);
        }
    }

    pub fn plan(&mut self, txt: &str, max_width: Option<u32>, features: Option<&[String]>) -> RenderPlan {
        unsafe {
            let buf = hb_buffer_create();
            hb_buffer_add_utf8(buf, txt.as_ptr() as *const libc::c_char,
                               txt.len() as libc::c_int, 0, -1);

            // If the direction is RTL, the clusters are given in reverse order.
            hb_buffer_set_direction(buf, HB_DIRECTION_LTR);
            hb_buffer_guess_segment_properties(buf);

            let features_vec: Vec<HbFeature> = features.map(|ftr|
                ftr.iter().filter_map(|f| {
                    let mut feature = HbFeature::default();
                    let ret = hb_feature_from_string(f.as_ptr() as *const libc::c_char,
                                                     f.len() as libc::c_int,
                                                     &mut feature);
                    if ret == 1 {
                        Some(feature)
                    } else {
                        None
                    }
                }).collect()
            ).unwrap_or_default();

            hb_shape(self.font, buf, features_vec.as_ptr(), features_vec.len() as libc::c_uint);
 
            let len = hb_buffer_get_length(buf) as usize;
            let info = hb_buffer_get_glyph_infos(buf, ptr::null_mut());
            let pos = hb_buffer_get_glyph_positions(buf, ptr::null_mut());
            let mut render_plan = RenderPlan::default();
            let mut missing_glyphs = Vec::new();

            for i in 0..len {
                let pos_i = &*pos.add(i);
                let info_i = &*info.add(i);
                if info_i.codepoint == 0 {
                    if let Some((start, end)) = missing_glyphs.pop() {
                        if i == end {
                            missing_glyphs.push((start, end+1));
                        } else {
                            missing_glyphs.push((start, end));
                            missing_glyphs.push((i, i+1));
                        }
                    } else {
                        missing_glyphs.push((i, i+1));
                    }
                } else {
                    render_plan.width += (pos_i.x_advance >> 6) as u32;
                }
                let glyph = GlyphPlan {
                    codepoint: info_i.codepoint,
                    cluster: info_i.cluster as usize,
                    advance: pt!(pos_i.x_advance >> 6, pos_i.y_advance >> 6),
                    offset: pt!(pos_i.x_offset >> 6, -pos_i.y_offset >> 6),
                };
                render_plan.glyphs.push(glyph);
            }

            self.patch(txt, &features_vec, &mut render_plan, missing_glyphs, buf);

            hb_buffer_destroy(buf);

            if let Some(mw) = max_width {
                self.crop_right(&mut render_plan, mw);
            }

            render_plan
        }
    }

    #[inline]
    pub fn crop_right(&self, render_plan: &mut RenderPlan, max_width: u32) {
        if render_plan.width <= max_width {
            return;
        }

        render_plan.width += self.ellipsis.width;
        while let Some(gp) = render_plan.glyphs.pop() {
            render_plan.width -= gp.advance.x as u32;
            if render_plan.width <= max_width {
                break;
            }
        }

        let len = render_plan.glyphs.len();
        render_plan.scripts.retain(|&k, _| k < len);
        render_plan.glyphs.extend_from_slice(&self.ellipsis.glyphs[..]);
    }

    #[inline]
    pub fn crop_around(&self, render_plan: &mut RenderPlan, index: usize, max_width: u32) -> usize {
        if render_plan.width <= max_width {
            return 0;
        }

        let len = render_plan.glyphs.len();
        let mut width = 0;
        let mut polarity = 0;
        let mut upper_index = index;
        let mut lower_index = index as i32 - 1;

        loop {
            let next_width;
            if upper_index < len && (polarity % 2 == 0 || lower_index < 0) {
                next_width = width + render_plan.glyphs[upper_index].advance.x as u32;
                if next_width > max_width {
                    break;
                } else {
                    width = next_width;
                }
                upper_index += 1;
            } else if lower_index >= 0 && (polarity % 2 == 1 || upper_index == len) {
                next_width = width + render_plan.glyphs[lower_index as usize].advance.x as u32;
                if next_width > max_width {
                    break;
                } else {
                    width = next_width;
                }
                lower_index -= 1;
            }
            polarity += 1;
        }

        if upper_index < len {
            width += self.ellipsis.width;
            upper_index -= 1;
            while width > max_width && upper_index > (lower_index.max(0) as usize) {
                width -= render_plan.glyphs[upper_index].advance.x as u32;
                upper_index -= 1;
            }
            render_plan.glyphs.truncate(upper_index + 1);
            render_plan.glyphs.extend_from_slice(&self.ellipsis.glyphs[..]);
        }

        if lower_index >= 0 {
            width += self.ellipsis.width;
            lower_index += 1;
            while width > max_width && (lower_index as usize) < upper_index  {
                width -= render_plan.glyphs[lower_index as usize].advance.x as u32;
                lower_index += 1;
            }
            render_plan.glyphs = self.ellipsis.glyphs.iter()
                                 .chain(render_plan.glyphs[lower_index as usize..].iter()).cloned().collect();
        }

        render_plan.scripts.retain(|&k, _| k >= lower_index.max(0) as usize && k <= upper_index);
        if lower_index > 0 {
            render_plan.scripts = render_plan.scripts.drain()
                                             .map(|(k, v)| (k - lower_index as usize + 1, v)).collect();
        }
        render_plan.width = width;

        if lower_index < 0 {
            0
        } else {
            lower_index as usize
        }
    }

    pub fn cut_point(&self, render_plan: &RenderPlan, max_width: u32) -> (usize, u32) {
        let mut width = render_plan.width;
        let glyphs = &render_plan.glyphs;
        let mut i = glyphs.len() - 1;

        width -= glyphs[i].advance.x as u32;

        while i > 0 && width > max_width {
            i -= 1;
            width -= glyphs[i].advance.x as u32;
        }

        let j = i;
        let last_width = width;

        while i > 0 && glyphs[i].codepoint != self.space_codepoint {
            i -= 1;
            width -= glyphs[i].advance.x as u32;
        }

        if i == 0 {
            i = j;
            width = last_width;
        }

        (i, width)
    }

    pub fn render(&mut self, fb: &mut Framebuffer, color: u8, render_plan: &RenderPlan, origin: Point) {
        unsafe {
            let mut pos = origin;
            let mut fallback_faces = FnvHashMap::default();

            for (index, glyph) in render_plan.glyphs.iter().enumerate() {
                let face = if let Some(script) = render_plan.scripts.get(&index) {
                    *fallback_faces.entry(script).or_insert_with(|| {
                        let font_data = font_data_from_script(*script);
                        let mut face = ptr::null_mut();
                        FT_New_Memory_Face((self.lib).0, font_data.as_ptr() as *const FtByte,
                                           font_data.len() as libc::c_long, 0, &mut face);
                        FT_Set_Pixel_Sizes(face, (*(*self.face).size).metrics.x_ppem as libc::c_uint, 0);
                        face
                    })
                } else {
                    self.face
                };

                FT_Load_Glyph(face, glyph.codepoint, FT_LOAD_RENDER | FT_LOAD_NO_HINTING);

                let glyph_slot = (*face).glyph;
                let top_left = pos + glyph.offset + pt!((*glyph_slot).bitmap_left, -(*glyph_slot).bitmap_top);
                let bitmap = &(*glyph_slot).bitmap;

                for y in 0..bitmap.rows {
                    for x in 0..bitmap.width {
                        let blackness = *bitmap.buffer.offset((bitmap.pitch * y + x) as isize);
                        let alpha = blackness as f32 / 255.0;
                        let pt = top_left + pt!(x, y);
                        fb.set_blended_pixel(pt.x as u32, pt.y as u32, color, alpha);
                    }
                }

                pos += glyph.advance;
            }

            let fallback_faces: BTreeSet<*mut FtFace> = fallback_faces.into_iter().map(|(_, v)| v).collect();
            for face in fallback_faces.into_iter() {
                FT_Done_Face(face);
            }
        }
    }

    pub fn height(&self, c: char) -> u32 {
        unsafe {
            FT_Load_Char(self.face, c as libc::c_ulong, FT_LOAD_DEFAULT);
            let metrics = &((*(*self.face).glyph).metrics);
            (metrics.height >> 6) as u32
        }
    }

    pub fn em(&self) -> u16 {
        unsafe {
            (*(*self.face).size).metrics.x_ppem as u16
        }
    }

    pub fn ascender(&self) -> i32 {
        unsafe {
            (*(*self.face).size).metrics.ascender as i32 / 64
        }
    }

    pub fn descender(&self) -> i32 {
        unsafe {
            (*(*self.face).size).metrics.descender as i32 / 64
        }
    }

    pub fn line_height(&self) -> i32 {
        unsafe {
            (*(*self.face).size).metrics.height as i32 / 64
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct GlyphPlan {
    codepoint: u32,
    cluster: usize,
    offset: Point,
    advance: Point,
}

#[derive(Debug, Clone)]
pub struct RenderPlan {
    pub width: u32,
    scripts: FnvHashMap<usize, HbScript>,
    glyphs: Vec<GlyphPlan>,
}

impl Default for RenderPlan {
    fn default() -> RenderPlan {
        RenderPlan {
            width: 0,
            scripts: FnvHashMap::default(),
            glyphs: vec![],
        }
    }
}

impl RenderPlan {
    pub fn space_out(&mut self, letter_spacing: u32) {
        if letter_spacing == 0 {
            return;
        }

        if let Some((_, start)) = self.glyphs.split_last_mut() {
            let len = start.len() as u32;
            for glyph in start {
                glyph.advance.x += letter_spacing as i32;
            }
            self.width += len * letter_spacing;
        }
    }

    pub fn split_off(&mut self, index: usize, width: u32) -> RenderPlan {
        let mut next_scripts = FnvHashMap::default();
        if !self.scripts.is_empty() {
            for i in index..self.glyphs.len() {
                self.scripts.remove_entry(&i)
                    .map(|(k, v)| next_scripts.insert(k - index, v));
            }
        }
        let next_glyphs = self.glyphs.split_off(index);
        let next_width = self.width - width;
        self.width = width;
        RenderPlan {
            width: next_width,
            scripts: next_scripts,
            glyphs: next_glyphs,
        }
    }

    pub fn index_from_advance(&self, advance: i32) -> usize {
        let mut sum = 0;
        let mut index = 0;
        while index < self.glyphs.len() {
            let gad = self.glyph_advance(index);
            sum += gad;
            if sum > advance {
                if sum - advance < advance - sum + gad {
                    index += 1;
                }
                break;
            }
            index += 1;
        }
        index
    }

    pub fn total_advance(&self, index: usize) -> i32 {
        self.glyphs.iter().take(index).map(|g| g.advance.x).sum()
    }

    #[inline]
    pub fn glyph_advance(&self, index: usize) -> i32 {
        self.glyphs[index].advance.x
    }
}

impl Drop for FontLibrary {
    fn drop(&mut self) {
        unsafe { FT_Done_FreeType(self.0); }
    }
}

impl Drop for Font {
    fn drop(&mut self) {
        unsafe { 
            FT_Done_Face(self.face);
            if !self.font.is_null() {
                hb_font_destroy(self.font);
            }
        }
    }
}

#[inline]
fn tag(c1: u8, c2: u8, c3: u8, c4: u8) -> u32 {
    ((c1 as u32) << 24) | ((c2 as u32) << 16) | ((c3 as u32) << 8) | c4 as u32
}

#[derive(Fail, Debug)]
enum FreetypeError {
    #[fail(display = "Unknown error with code {}.", _0)]
    UnknownError(FtError),

    #[fail(display = "Cannot open resource.")]
    CannotOpenResource,

    #[fail(display = "Unknown file format.")]
    UnknownFileFormat,

    #[fail(display = "Broken file.")]
    InvalidFileFormat,

    #[fail(display = "Invalid FreeType version.")]
    InvalidVersion,

    #[fail(display = "Module version is too low.")]
    LowerModuleVersion,

    #[fail(display = "Invalid argument.")]
    InvalidArgument,

    #[fail(display = "Unimplemented feature.")]
    UnimplementedFeature,

    #[fail(display = "Broken table.")]
    InvalidTable,

    #[fail(display = "Broken offset within table.")]
    InvalidOffset,

    #[fail(display = "Array allocation size too large.")]
    ArrayTooLarge,

    #[fail(display = "Missing module.")]
    MissingModule,

    #[fail(display = "Missing property.")]
    MissingProperty,

    #[fail(display = "Invalid glyph index.")]
    InvalidGlyphIndex,

    #[fail(display = "Invalid character code.")]
    InvalidCharacterCode,

    #[fail(display = "Unsupported glyph image format.")]
    InvalidGlyphFormat,

    #[fail(display = "Cannot render this glyph format.")]
    CannotRenderGlyph,

    #[fail(display = "Invalid outline.")]
    InvalidOutline,

    #[fail(display = "Invalid composite glyph.")]
    InvalidComposite,

    #[fail(display = "Too many hints.")]
    TooManyHints,

    #[fail(display = "Invalid pixel size.")]
    InvalidPixelSize,

    #[fail(display = "Invalid object handle.")]
    InvalidHandle,

    #[fail(display = "Invalid library handle.")]
    InvalidLibraryHandle,

    #[fail(display = "Invalid module handle.")]
    InvalidDriverHandle,

    #[fail(display = "Invalid face handle.")]
    InvalidFaceHandle,

    #[fail(display = "Invalid size handle.")]
    InvalidSizeHandle,

    #[fail(display = "Invalid glyph slot handle.")]
    InvalidSlotHandle,

    #[fail(display = "Invalid charmap handle.")]
    InvalidCharMapHandle,

    #[fail(display = "Invalid cache manager handle.")]
    InvalidCacheHandle,

    #[fail(display = "Invalid stream handle.")]
    InvalidStreamHandle,

    #[fail(display = "Too many modules.")]
    TooManyDrivers,

    #[fail(display = "Too many extensions.")]
    TooManyExtensions,

    #[fail(display = "Out of memory.")]
    OutOfMemory,

    #[fail(display = "Unlisted object.")]
    UnlistedObject,

    #[fail(display = "Cannot open stream.")]
    CannotOpenStream,

    #[fail(display = "Invalid stream seek.")]
    InvalidStreamSeek,

    #[fail(display = "Invalid stream skip.")]
    InvalidStreamSkip,

    #[fail(display = "Invalid stream read.")]
    InvalidStreamRead,

    #[fail(display = "Invalid stream operation.")]
    InvalidStreamOperation,

    #[fail(display = "Invalid frame operation.")]
    InvalidFrameOperation,

    #[fail(display = "Nested frame access.")]
    NestedFrameAccess,

    #[fail(display = "Invalid frame read.")]
    InvalidFrameRead,

    #[fail(display = "Raster uninitialized.")]
    RasterUninitialized,

    #[fail(display = "Raster corrupted.")]
    RasterCorrupted,

    #[fail(display = "Raster overflow.")]
    RasterOverflow,

    #[fail(display = "Negative height while rastering.")]
    RasterNegativeHeight,

    #[fail(display = "Too many registered caches.")]
    TooManyCaches,

    #[fail(display = "Invalid opcode.")]
    InvalidOpcode,

    #[fail(display = "Too few arguments.")]
    TooFewArguments,

    #[fail(display = "Stack overflow.")]
    StackOverflow,

    #[fail(display = "Code overflow.")]
    CodeOverflow,

    #[fail(display = "Bad argument.")]
    BadArgument,

    #[fail(display = "Division by zero.")]
    DivideByZero,

    #[fail(display = "Invalid reference.")]
    InvalidReference,

    #[fail(display = "Found debug opcode.")]
    DebugOpCode,

    #[fail(display = "Found ENDF opcode in execution stream.")]
    ENDFInExecStream,

    #[fail(display = "Nested DEFS.")]
    NestedDEFS,

    #[fail(display = "Invalid code range.")]
    InvalidCodeRange,

    #[fail(display = "Execution context too long.")]
    ExecutionTooLong,

    #[fail(display = "Too many function definitions.")]
    TooManyFunctionDefs,

    #[fail(display = "Too many instruction definitions.")]
    TooManyInstructionDefs,

    #[fail(display = "SFNT font table missing.")]
    TableMissing,

    #[fail(display = "Horizontal header (hhea) table missing.")]
    HorizHeaderMissing,

    #[fail(display = "Locations (loca) table missing.")]
    LocationsMissing,

    #[fail(display = "Name table missing.")]
    NameTableMissing,

    #[fail(display = "Character map (cmap) table missing.")]
    CMapTableMissing,

    #[fail(display = "Horizontal metrics (hmtx) table missing.")]
    HmtxTableMissing,

    #[fail(display = "PostScript (post) table missing.")]
    PostTableMissing,

    #[fail(display = "Invalid horizontal metrics.")]
    InvalidHorizMetrics,

    #[fail(display = "Invalid character map (cmap) format.")]
    InvalidCharMapFormat,

    #[fail(display = "Invalid ppem value.")]
    InvalidPPem,

    #[fail(display = "Invalid vertical metrics.")]
    InvalidVertMetrics,

    #[fail(display = "Could not find context.")]
    CouldNotFindContext,

    #[fail(display = "Invalid PostScript (post) table format.")]
    InvalidPostTableFormat,

    #[fail(display = "Invalid PostScript (post) table.")]
    InvalidPostTable,

    #[fail(display = "Found FDEF or IDEF opcode in glyf bytecode.")]
    DEFInGlyfBytecode,

    #[fail(display = "Missing bitmap in strike.")]
    MissingBitmap,

    #[fail(display = "Opcode syntax error.")]
    SyntaxError,

    #[fail(display = "Argument stack underflow.")]
    StackUnderflow,

    #[fail(display = "Ignore.")]
    Ignore,

    #[fail(display = "No Unicode glyph name found.")]
    NoUnicodeGlyphName,

    #[fail(display = "Glyph too big for hinting.")]
    GlyphTooBig,

    #[fail(display = "`STARTFONT' field missing.")]
    MissingStartfontField,

    #[fail(display = "`FONT' field missing.")]
    MissingFontField,

    #[fail(display = "`SIZE' field missing.")]
    MissingSizeField,

    #[fail(display = "`FONTBOUNDINGBOX' field missing.")]
    MissingFontboundingboxField,

    #[fail(display = "`CHARS' field missing.")]
    MissingCharsField,

    #[fail(display = "`STARTCHAR' field missing.")]
    MissingStartcharField,

    #[fail(display = "`ENCODING' field missing.")]
    MissingEncodingField,

    #[fail(display = "`BBX' field missing.")]
    MissingBbxField,

    #[fail(display = "`BBX' too big.")]
    BbxTooBig,

    #[fail(display = "Font header corrupted or missing fields.")]
    CorruptedFontHeader,

    #[fail(display = "Font glyphs corrupted or missing fields.")]
    CorruptedFontGlyphs,
}

impl From<FtError> for FreetypeError {
    fn from(code: FtError) -> FreetypeError {
        match code {
            0x01 => FreetypeError::CannotOpenResource,
            0x02 => FreetypeError::UnknownFileFormat,
            0x03 => FreetypeError::InvalidFileFormat,
            0x04 => FreetypeError::InvalidVersion,
            0x05 => FreetypeError::LowerModuleVersion,
            0x06 => FreetypeError::InvalidArgument,
            0x07 => FreetypeError::UnimplementedFeature,
            0x08 => FreetypeError::InvalidTable,
            0x09 => FreetypeError::InvalidOffset,
            0x0A => FreetypeError::ArrayTooLarge,
            0x0B => FreetypeError::MissingModule,
            0x0C => FreetypeError::MissingProperty,
            0x10 => FreetypeError::InvalidGlyphIndex,
            0x11 => FreetypeError::InvalidCharacterCode,
            0x12 => FreetypeError::InvalidGlyphFormat,
            0x13 => FreetypeError::CannotRenderGlyph,
            0x14 => FreetypeError::InvalidOutline,
            0x15 => FreetypeError::InvalidComposite,
            0x16 => FreetypeError::TooManyHints,
            0x17 => FreetypeError::InvalidPixelSize,
            0x20 => FreetypeError::InvalidHandle,
            0x21 => FreetypeError::InvalidLibraryHandle,
            0x22 => FreetypeError::InvalidDriverHandle,
            0x23 => FreetypeError::InvalidFaceHandle,
            0x24 => FreetypeError::InvalidSizeHandle,
            0x25 => FreetypeError::InvalidSlotHandle,
            0x26 => FreetypeError::InvalidCharMapHandle,
            0x27 => FreetypeError::InvalidCacheHandle,
            0x28 => FreetypeError::InvalidStreamHandle,
            0x30 => FreetypeError::TooManyDrivers,
            0x31 => FreetypeError::TooManyExtensions,
            0x40 => FreetypeError::OutOfMemory,
            0x41 => FreetypeError::UnlistedObject,
            0x51 => FreetypeError::CannotOpenStream,
            0x52 => FreetypeError::InvalidStreamSeek,
            0x53 => FreetypeError::InvalidStreamSkip,
            0x54 => FreetypeError::InvalidStreamRead,
            0x55 => FreetypeError::InvalidStreamOperation,
            0x56 => FreetypeError::InvalidFrameOperation,
            0x57 => FreetypeError::NestedFrameAccess,
            0x58 => FreetypeError::InvalidFrameRead,
            0x60 => FreetypeError::RasterUninitialized,
            0x61 => FreetypeError::RasterCorrupted,
            0x62 => FreetypeError::RasterOverflow,
            0x63 => FreetypeError::RasterNegativeHeight,
            0x70 => FreetypeError::TooManyCaches,
            0x80 => FreetypeError::InvalidOpcode,
            0x81 => FreetypeError::TooFewArguments,
            0x82 => FreetypeError::StackOverflow,
            0x83 => FreetypeError::CodeOverflow,
            0x84 => FreetypeError::BadArgument,
            0x85 => FreetypeError::DivideByZero,
            0x86 => FreetypeError::InvalidReference,
            0x87 => FreetypeError::DebugOpCode,
            0x88 => FreetypeError::ENDFInExecStream,
            0x89 => FreetypeError::NestedDEFS,
            0x8A => FreetypeError::InvalidCodeRange,
            0x8B => FreetypeError::ExecutionTooLong,
            0x8C => FreetypeError::TooManyFunctionDefs,
            0x8D => FreetypeError::TooManyInstructionDefs,
            0x8E => FreetypeError::TableMissing,
            0x8F => FreetypeError::HorizHeaderMissing,
            0x90 => FreetypeError::LocationsMissing,
            0x91 => FreetypeError::NameTableMissing,
            0x92 => FreetypeError::CMapTableMissing,
            0x93 => FreetypeError::HmtxTableMissing,
            0x94 => FreetypeError::PostTableMissing,
            0x95 => FreetypeError::InvalidHorizMetrics,
            0x96 => FreetypeError::InvalidCharMapFormat,
            0x97 => FreetypeError::InvalidPPem,
            0x98 => FreetypeError::InvalidVertMetrics,
            0x99 => FreetypeError::CouldNotFindContext,
            0x9A => FreetypeError::InvalidPostTableFormat,
            0x9B => FreetypeError::InvalidPostTable,
            0x9C => FreetypeError::DEFInGlyfBytecode,
            0x9D => FreetypeError::MissingBitmap,
            0xA0 => FreetypeError::SyntaxError,
            0xA1 => FreetypeError::StackUnderflow,
            0xA2 => FreetypeError::Ignore,
            0xA3 => FreetypeError::NoUnicodeGlyphName,
            0xA4 => FreetypeError::GlyphTooBig,
            0xB0 => FreetypeError::MissingStartfontField,
            0xB1 => FreetypeError::MissingFontField,
            0xB2 => FreetypeError::MissingSizeField,
            0xB3 => FreetypeError::MissingFontboundingboxField,
            0xB4 => FreetypeError::MissingCharsField,
            0xB5 => FreetypeError::MissingStartcharField,
            0xB6 => FreetypeError::MissingEncodingField,
            0xB7 => FreetypeError::MissingBbxField,
            0xB8 => FreetypeError::BbxTooBig,
            0xB9 => FreetypeError::CorruptedFontHeader,
            0xBA => FreetypeError::CorruptedFontGlyphs,
            _ => FreetypeError::UnknownError(code),
        }
    }
}
