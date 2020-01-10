use iced::Font;
use once_cell::sync::Lazy;
use font_kit::{source::SystemSource, family_name::FamilyName, properties::Properties};

pub static MONOSPACE: Lazy<Font> = Lazy::new(|| {
    let font = SystemSource::new()
            .select_best_match(&[FamilyName::Monospace], &Properties::new())
            .unwrap()
            .load()
            .unwrap();
    Font::External {
        name: Box::leak(font.full_name().into_boxed_str()),
        bytes: Box::leak(font.copy_font_data().unwrap().as_ref().clone().into_boxed_slice()),
    }
});
