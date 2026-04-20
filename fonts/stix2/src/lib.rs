extern crate font_types;

#[macro_use]
mod macros;
pub mod constants;
pub mod style;
pub mod glyphs;
pub mod symbols;
pub mod variants;
pub mod kernings;

use font_types::Glyph;

#[inline]
pub fn glyph_metrics(code: u32) -> Glyph {
    *glyphs::GLYPHS
         .get(&code)
         .unwrap_or_else(|| panic!("Unable to find glyph for code {}", code))
}
