// Re-export language-neutral types and functions from site-gen.
pub use site_gen::dialog::{parse_dialog, slugify, DialogLine, Gender};
pub use site_gen::language::{assign_voices, parse_character_genders, Language, Voice};

pub mod french;
pub mod spanish;
