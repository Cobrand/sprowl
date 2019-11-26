//! Various functions and structs which will help you use this crate to its maximum.


use rusttype::{Font, Scale as FontScale, GlyphId};
use cgmath::Vector2;

pub struct AdvancedLayoutIter<'a, 't> {
    pub (crate) font: &'a Font<'static>,
    pub (crate) chars: std::str::CharIndices<'t>,
    pub (crate) scale: FontScale,
    pub (crate) start: Vector2<f32>,
    pub (crate) offset: Vector2<f32>,
    pub (crate) last_glyph: Option<GlyphId>,
    pub (crate) max_width: u32,
}

pub struct WordPos<'t> {
    pub word: &'t str,
    pub origin: Vector2<f32>,
}

impl<'a, 't> Iterator for AdvancedLayoutIter<'a, 't> {
    type Item = WordPos<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut origin = self.start;
        let mut begin: Option<usize> = None;
                
        let v_metrics = self.font.v_metrics(self.scale);
        while let Some((i, c)) = self.chars.next() {
            if c == '\n' {
                self.offset.y += v_metrics.ascent + v_metrics.descent + v_metrics.line_gap;
                self.offset.x = self.start.x;
            }
            if ! c.is_whitespace() && begin.is_none() {
                origin = self.start + self.offset;
                begin = Some(i);
            };
            let g = self.font.glyph(c).scaled(self.scale);
            if let Some(last) = self.last_glyph {
                self.offset.x += self.font.pair_kerning(self.scale, last, g.id());
            }
            self.offset.x += g.h_metrics().advance_width;
            if self.offset.x >= self.max_width as f32 {
                self.offset.y += v_metrics.ascent + v_metrics.descent + v_metrics.line_gap;
                self.offset.x = self.start.x;
            }
            self.last_glyph = Some(g.id());
            if c.is_whitespace() {
                if let Some(begin) = begin {
                    let end = i;
                    return Some(WordPos {
                        word: &self.chars.as_str()[begin..end],
                        origin,
                    })
                }
            }
        };
        None
    }
}
