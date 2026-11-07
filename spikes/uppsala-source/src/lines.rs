//! Byte offset -> (line, column) index.
//!
//! Columns follow uppsala's convention (`Document::node_column`): 1-based,
//! counted in bytes from the last `\n`. Building the index is one pass over
//! the input; each lookup is a binary search, unlike uppsala's
//! `node_line`/`node_column`, which rescan the input from the start on every
//! call.

#[derive(Debug, Clone)]
pub struct LineIndex {
    /// Byte offset of the first byte of each line.
    starts: Vec<usize>,
}

impl LineIndex {
    pub fn new(src: &str) -> Self {
        let mut starts = Vec::with_capacity(src.len() / 40 + 1);
        starts.push(0);
        for (i, b) in src.bytes().enumerate() {
            if b == b'\n' {
                starts.push(i + 1);
            }
        }
        Self { starts }
    }

    /// 1-based (line, column) of byte offset `pos`.
    pub fn line_col(&self, pos: usize) -> (usize, usize) {
        let line = self.starts.partition_point(|&s| s <= pos);
        let start = self.starts[line - 1];
        (line, pos - start + 1)
    }

    /// Byte offset of a 1-based (line, column), if it exists.
    pub fn offset(&self, line: usize, col: usize) -> Option<usize> {
        let start = *self.starts.get(line.checked_sub(1)?)?;
        Some(start + col.checked_sub(1)?)
    }

    pub fn line_count(&self) -> usize {
        self.starts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agrees_with_uppsala() {
        let xml = "<a>\n  <b x='1'/>\r\n\t<c>t</c>\n</a>";
        let doc = uppsala::parse(xml).unwrap();
        let idx = LineIndex::new(xml);
        for id in doc.descendants(doc.root()) {
            let Some(r) = doc.node_range(id) else {
                continue;
            };
            if id == doc.root() {
                continue;
            }
            assert_eq!(
                idx.line_col(r.start),
                (doc.node_line(id), doc.node_column(id))
            );
        }
    }
}
