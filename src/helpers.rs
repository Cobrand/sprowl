//! Various functions and structs which will help you use this crate to its maximum.


use rusttype::{Font, Scale as FontScale, GlyphId};
use cgmath::Vector2;

#[derive(Clone)]
pub struct AdvancedLayoutIter<'a, 't> {
    pub (crate) font: &'a Font<'static>,
    pub (crate) chars: std::str::CharIndices<'t>,
    pub (crate) original_str: &'t str,
    pub (crate) scale: FontScale,
    pub (crate) start: Vector2<f32>,
    pub (crate) offset: Vector2<f32>,
    pub (crate) last_glyph: Option<GlyphId>,
    pub (crate) max_width: u32,
}

impl<'a, 't> AdvancedLayoutIter<'a, 't> {
    pub fn new(font: &'a Font<'static>, t: &'t str, size: f32, start: Vector2<f32>, max_width: u32) -> AdvancedLayoutIter<'a, 't> {
        AdvancedLayoutIter {
            font,
            chars: t.char_indices(),
            original_str: t,
            scale: FontScale::uniform(size),
            start,
            offset: Vector2::new(0.0, 0.0),
            last_glyph: None,
            max_width
        }
    }
}

pub struct WordPos<'t> {
    pub word: &'t str,
    pub origin: Vector2<f32>,
    pub size: Vector2<f32>,
}

impl<'a, 't> Iterator for AdvancedLayoutIter<'a, 't> {
    type Item = WordPos<'t>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.chars.as_str().len() == 0 {
            // we are trying to get a new word, but the string is now empty, so return None.
            return None;
        }
        let mut origin = self.start;
        let mut begin: Option<usize> = None;
                
        let v_metrics = self.font.v_metrics(self.scale);
        let character_height = v_metrics.ascent - v_metrics.descent;
        let mut max_i = 0;
        while let Some((i, c)) = self.chars.next() {
            max_i = i;
            if ! c.is_whitespace() && begin.is_none() {
                origin = self.start + self.offset;
                begin = Some(i);
            };
            let g = self.font.glyph(c).scaled(self.scale);
            if let Some(last) = self.last_glyph {
                self.offset.x += self.font.pair_kerning(self.scale, last, g.id());
            }
            self.last_glyph = Some(g.id());
            self.offset.x += g.h_metrics().advance_width;
            if c.is_whitespace() {
                if let Some(begin) = begin {
                    let len = self.original_str.len();
                    let end = i;
                    assert!(len >= end);
                    let word_pos = WordPos {
                        word: &self.original_str[begin..end],
                        origin,
                        size: (self.start + self.offset) + Vector2::new(0.0, character_height) - origin,
                    };
                    if c == '\n' {
                        self.offset.y += character_height + v_metrics.line_gap;
                        self.offset.x = self.start.x;
                    }
                    return Some(word_pos);
                } else if c == '\n' {
                    self.offset.y += character_height + v_metrics.line_gap;
                    self.offset.x = self.start.x;
                }
            }
            if self.offset.x >= self.max_width as f32 {
                let word_size = self.start.x + self.offset.x - origin.x;
                self.offset.y += character_height + v_metrics.line_gap;
                self.offset.x = self.start.x + word_size;
                origin = Vector2::new(self.start.x, self.offset.y);
            }
        };
        if let Some(begin) = begin {
            let len = self.original_str.len();
            let end = max_i + 1;
            assert!(len == end);
            return Some(WordPos {
                word: &self.original_str[begin..end],
                origin,
                size: (self.start + self.offset) + Vector2::new(0.0, character_height) - origin,
            })
        } else {
            None
        }
    }
}
