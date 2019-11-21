// Source: rusttype/src/gpu_cache.rs (APACHE2/MIT dual licensed)

use crate::{point, vector, GlyphId, Point, PositionedGlyph, Rect, Vector};
use linked_hash_map::LinkedHashMap;

use hashbrown::{HashMap, hash_map::DefaultHashBuilder as HashBuilder};
