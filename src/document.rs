//! The one place this crate parses a document.
//!
//! A remark is content — a text file with a metadata block and a body — and
//! this crate reads it with `fig`, which is what `fig` is for: it detects the
//! archetype (`---` YAML frontmatter, `+++` TOML, a fenced block), resolves
//! the format its content is written in, and hands back the block and the
//! prose on either side of it.
//!
//! Which format a store's documents use is not this crate's to decide, so
//! nothing here names one: [`split`] reads whichever it finds. What it will
//! not do is guess at a file with no block at all — that is a document with
//! no metadata, which is a document that is not a remark, and it comes back
//! as `None` rather than as an error.
//!
//! A caller that has already parsed its own documents never reaches this
//! module. It implements [`Fields`](crate::Fields) over its own value tree
//! and calls [`Reader::read`](crate::Reader::read), and this file is dead
//! code to it — which is the point of the seam being a trait.

use crate::error::Result;

/// A document's metadata block and its body, or `None` when it has no block.
///
/// The body is the host text outside the block: the prose under
/// frontmatter, the prose above endmatter. It is owned rather than borrowed
/// because a mid-document block has host text on *both* sides of it, and a
/// remark's body is all of it.
pub fn split(text: &str) -> Result<Option<(fig::Value, String)>> {
    let Some(kind) = fig::detect(text) else {
        return Ok(None);
    };
    let Ok(extracted) = fig::Embed::extract(text, kind) else {
        // An opening delimiter with nothing closing it. `detect` matches on
        // the open alone and says so; a caller asking for remarks in a chain
        // of a thousand documents should not have the whole answer refused
        // because one of them has a stray `---` in it.
        return Ok(None);
    };
    let meta = fig::Document::parse(extracted.content().as_bytes(), kind.inner_format())?;
    let body = match extracted.host_before().is_empty() {
        true => extracted.host_after().to_string(),
        false => format!("{}{}", extracted.host_before(), extracted.host_after()),
    };
    Ok(Some((meta.to_value()?, body)))
}
