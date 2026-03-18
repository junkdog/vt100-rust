/// Tracks which terminal rows have been modified since the last
/// [`DirtyRows::clear`]. Uses a 256-bit bitmap (`[u64; 4]`), sufficient
/// for any realistic terminal height.
#[derive(Clone, Copy, Debug, Default)]
pub struct DirtyRows {
    bits: [u64; 4],
}

impl DirtyRows {
    /// Mark a single row as dirty.
    #[inline]
    pub fn mark(&mut self, row: u16) {
        let row = row as usize;
        self.bits[row / 64] |= 1 << (row % 64);
    }

    /// Mark a contiguous range of rows `start..=end` as dirty.
    #[inline]
    pub fn mark_range(&mut self, start: u16, end: u16) {
        for row in start..=end {
            self.mark(row);
        }
    }

    /// Mark all rows as dirty.
    #[inline]
    pub fn mark_all(&mut self) {
        self.bits = [u64::MAX; 4];
    }

    /// Returns `true` if any row is dirty.
    #[inline]
    pub fn any(&self) -> bool {
        (self.bits[0] | self.bits[1] | self.bits[2] | self.bits[3]) != 0
    }

    /// Returns `true` if the given row is dirty.
    #[inline]
    pub fn is_dirty(&self, row: u16) -> bool {
        let row = row as usize;
        self.bits[row / 64] & (1 << (row % 64)) != 0
    }

    /// Reset all dirty flags to clean.
    #[inline]
    pub fn clear(&mut self) {
        self.bits = [0; 4];
    }

    /// Returns an iterator over dirty row indices (up to `max_row` exclusive).
    pub fn iter(&self, max_row: u16) -> DirtyRowIter {
        DirtyRowIter {
            bits: self.bits,
            pos: 0,
            max_row,
        }
    }
}

/// Iterator over dirty row indices.
pub struct DirtyRowIter {
    bits: [u64; 4],
    pos: u16,
    max_row: u16,
}

impl Iterator for DirtyRowIter {
    type Item = u16;

    #[inline]
    fn next(&mut self) -> Option<u16> {
        while self.pos < self.max_row {
            let word = self.pos / 64;
            let remaining = self.bits[word as usize] >> (self.pos % 64);
            if remaining == 0 {
                // skip to next word boundary
                self.pos = (word + 1) * 64;
                continue;
            }
            let offset = remaining.trailing_zeros() as u16;
            let row = self.pos + offset;
            if row >= self.max_row {
                return None;
            }
            // clear the bit so we advance past it
            self.bits[word as usize] ^= 1 << (row % 64);
            self.pos = row;
            return Some(row);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mark_and_iter() {
        let mut d = DirtyRows::default();
        d.mark(0);
        d.mark(5);
        d.mark(127);
        let rows: Vec<u16> = d.iter(256).collect();
        assert_eq!(rows, vec![0, 5, 127]);
    }

    #[test]
    fn mark_range() {
        let mut d = DirtyRows::default();
        d.mark_range(62, 66);
        let rows: Vec<u16> = d.iter(256).collect();
        assert_eq!(rows, vec![62, 63, 64, 65, 66]);
    }

    #[test]
    fn mark_all_and_clear() {
        let mut d = DirtyRows::default();
        d.mark_all();
        assert!(d.any());
        assert!(d.is_dirty(0));
        assert!(d.is_dirty(255));
        d.clear();
        assert!(!d.any());
    }

    #[test]
    fn iter_respects_max_row() {
        let mut d = DirtyRows::default();
        d.mark(10);
        d.mark(200);
        let rows: Vec<u16> = d.iter(100).collect();
        assert_eq!(rows, vec![10]);
    }
}
