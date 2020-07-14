//! Various functions and structs which will help you use this crate to its maximum.

use rusttype::{Font, Scale as FontScale};
use cgmath::Vector2;

use smallvec::SmallVec;

pub trait AdvancedText<'t> {
    type E;

    fn len(&self) -> usize;

    fn get(&self, begin: usize, end: usize) -> Result<&'t str, (&'t str, usize)>;
}

impl<'t> AdvancedText<'t> for &'t str {
    type E = ();

    fn len(&self) -> usize {
        str::len(self)
    }

    fn get(&self, begin: usize, end: usize) -> Result<&'t str, (&'t str, usize)> {
        Ok(str::get(self, begin..end).unwrap())
    }
}

#[derive(Clone)]
pub struct AdvancedLayout<'f, 't, T: AdvancedText<'t>> {
    pub (crate) font: &'f Font<'static>,
    pub (crate) original_str: T,
    pub (crate) scale: FontScale,
    pub (crate) start: Vector2<f32>,
    /// 0 = center, -1 = left, 1 = right
    pub (crate) align: i8,
    pub (crate) max_width: u32,

    layout: SmallVec<[WordPos<'t>; 16]>,
}

impl<'a, 't> AdvancedLayout<'a, 't, &'t str> {
    /// Compute a layout that returns word positions for a given sentence.
    ///
    /// You can specify a `max_width`, where the text will go to the next line if the total with goes 
    /// beyong `max_width`.
    ///
    /// align < 0 => left
    /// align == 0 => center
    /// align > 0 => right
    pub fn new_str(font: &'a Font<'static>, t: &'t str, size: f32, start: Vector2<f32>, align: i8, max_width: u32) -> AdvancedLayout<'a, 't, &'t str> {
        let mut l = AdvancedLayout {
            font,
            original_str: t,
            scale: FontScale::uniform(size),
            start,
            align,
            max_width,
            layout: Default::default(),
        };
        l.compute();
        l
    }

    fn line_size(&self, beg_line_word_index: usize, last_index: Option<usize>) -> f32 {
        let first_of_line = if let Some(word) = self.layout.get(beg_line_word_index) {
            word
        } else {
            return 0.0;
        };
        let last_of_line = if let Some(last_index) = last_index {
            self.layout.get(last_index)
        } else {
            self.layout.last()
        }.unwrap();

        (last_of_line.size.x + last_of_line.origin.x) - first_of_line.origin.x
    }

    fn realign(&mut self, first_line_word_index: usize, last_index: Option<usize>) {
        if self.align < 0 {
            // no need to do that if it's aligned on the left
            return;
        }
        let line_size = self.line_size(first_line_word_index, last_index);

        let max_line_size = self.max_width as f32;
        let offset = if self.align == 0 {
            (max_line_size - line_size) / 2.0
        } else {
            max_line_size - line_size
        };

        let last_index = last_index.unwrap_or(self.layout.len() - 1);
        for word in &mut self.layout[first_line_word_index..last_index+1] {
            word.origin.x += offset;
        }
    }

    fn compute(&mut self) {
        let mut char_indices = self.original_str.char_indices();

        let v_metrics = self.font.v_metrics(self.scale);
        let character_height = v_metrics.ascent - v_metrics.descent;

        // the index of the word in `layout` at the beginning of the line.
        // used to realign stuff.
        let mut beginning_line_word_index = 0;

        let mut current_word_boundaries: Option<(usize, usize)> = None;
        let mut origin = self.start;
        let mut size = Vector2::new(0.0, character_height);
        let mut last_char = None;

        while let Some((i, c)) = char_indices.next() {
            let g = self.font.glyph(c).scaled(self.scale);

            let words_in_line = self.layout.len() - beginning_line_word_index;

            let pair_kerning = last_char
                .map(|prev_char| self.font.pair_kerning(self.scale, prev_char, c))
                .unwrap_or(0.0);
            match (current_word_boundaries, c.is_whitespace()) {
                (Some((beg, end)), true) => {
                    self.layout.push(WordPos {
                        word: &self.original_str[beg..end],
                        origin,
                        size,
                    });
                    current_word_boundaries = None;
                    if self.line_size(beginning_line_word_index, None) >= self.max_width as f32 && words_in_line > 0 {
                        origin = Vector2::new( self.start.x, origin.y + character_height + v_metrics.line_gap);
                        self.layout.last_mut().unwrap().origin = origin;

                        origin.x += size.x;
                        // len() - 2 is valid because we checked earlier that there were at least 1 word (before the insert)
                        self.realign(beginning_line_word_index, Some(self.layout.len() - 2));
                        beginning_line_word_index = self.layout.len() - 1;
                        size.x = 0.0;
                    }
                    if c == '\n' {
                        // newline
                        origin.x = self.start.x;
                        origin.y += character_height + v_metrics.line_gap;
                        self.realign(beginning_line_word_index, None);
                        beginning_line_word_index = self.layout.len();
                    } else {
                        origin.x += size.x + g.h_metrics().advance_width + pair_kerning;
                    }
                    size.x = 0.0;
                },
                (None, true) => {
                    if c == '\n' {
                        // newline
                        origin.x = self.start.x;
                        origin.y += character_height + v_metrics.line_gap;
                        self.realign(beginning_line_word_index, None);
                        beginning_line_word_index = self.layout.len();
                    } else {
                        origin.x += g.h_metrics().advance_width + pair_kerning;
                    }
                },
                (Some((beg, end)), false) => {
                    current_word_boundaries = Some((beg, end + c.len_utf8()));
                    size.x += g.h_metrics().advance_width + pair_kerning;
                },
                (None, false) => {
                    current_word_boundaries = Some((i, i + c.len_utf8()));
                    size.x += g.h_metrics().advance_width + pair_kerning;
                }
            };
            last_char = Some(c);
        }
        if let Some((beg, end)) = current_word_boundaries {
            self.layout.push(WordPos {
                word: &self.original_str[beg..end],
                origin,
                size,
            });
        }
        let words_in_line = self.layout.len() - beginning_line_word_index;
        if self.line_size(beginning_line_word_index, None) >= self.max_width as f32 && words_in_line >= 2 {
            // last word is too big to fit on current line
            origin = Vector2::new( self.start.x, origin.y + character_height + v_metrics.line_gap);
            self.layout.last_mut().unwrap().origin = origin;

            // align 2nd last line
            self.realign(beginning_line_word_index, Some(self.layout.len() - 2));

            // align 1st last line
            self.realign(self.layout.len() - 1, None);
        } else {
            self.realign(beginning_line_word_index, None);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item=&WordPos<'t>> {
        self.layout.iter()
    }
}

/// Only to be consumed (it is returned by `AdvancedLayoutIter`), represents a size and a position
/// for a given word.
#[derive(Debug, Clone, Copy)]
pub struct WordPos<'t> {
    pub word: &'t str,
    pub origin: Vector2<f32>,
    pub size: Vector2<f32>,
}
