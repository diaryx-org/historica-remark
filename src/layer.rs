//! Kept layers — *somebody else's chain, kept beside your own and carried
//! with it.*
//!
//! A layer is a companion history: a single-writer chain another store
//! published, addressed to this one, and kept here in the received shape —
//! the writer's documents materialised at the head, the store beside them
//! ([`Repo`]). It is kept current by fetching, so the writer's forgetting
//! destroys here what it destroyed there — the same compliance every fetch
//! owes — and it is carried onward when this store's own history leaves.
//!
//! That last part is the whole design. One reader's remarks reach every other
//! reader **without anybody's words being copied into a chain their author
//! does not control**: the author of a remark keeps writing it in their own
//! store, the store they remarked on keeps their chain as a layer and carries
//! it, and what a third reader fetches is the remark's own history, vouched
//! for by the remark's own author. The party in the middle is a courier.
//!
//! ```text
//! <root>/
//!     <name>/           one layer — a Repo
//!     <another name>/
//! ```
//!
//! Nothing in this module reads a document or knows what one means: keep,
//! list, carry, suppress. What a layer's documents *say* is
//! [`Repo::remarks`]' department, and a caller keeping layers of something
//! other than remarks never calls it.
//!
//! # Suppression
//!
//! The keeper's one editorial power, and deliberately small: a suppressed
//! path is left out of what the caller shows, and it lifts again as easily as
//! it was laid on — nothing was destroyed, so nothing has to be fetched back.
//!
//! It does not reach into the chain. The keeper is not the writer and cannot
//! redact a chain they do not own; the writer's own forgetting is the only
//! thing that destroys. (Withholding a suppressed document from the *carried*
//! copy would need the foreign chain re-derived, which this crate does not
//! do: today's carry is the chain as kept.)

use std::path::{Path, PathBuf};

use historica::store::{STORE_DIR, Source};

use crate::error::Result;

pub(crate) mod carry;
mod memory;
pub(crate) mod repo;

pub use carry::Export;
pub use repo::{
    MANIFEST_PATH, NOTES_DIR, ORIGIN_NOTE, Received, Remark, Repo, SENDER_NOTE, check_name,
};

/// Where a layer's suppressions are noted, under the notes directory: local
/// bookkeeping, not part of the chain, never carried.
pub const SUPPRESSED_NOTE: &str = "suppressed";

/// The layers kept under one root.
///
/// Cheap to construct; nothing is touched until something is asked.
#[derive(Debug, Clone)]
pub struct Layers {
    root: PathBuf,
    notes: String,
}

impl Layers {
    /// Layers under `root`, each keeping its notes under
    /// [`NOTES_DIR`](repo::NOTES_DIR).
    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            notes: repo::NOTES_DIR.to_string(),
        }
    }

    /// The same, with a notes directory of the caller's own — see
    /// [`Repo::with_notes`].
    pub fn with_notes(root: impl Into<PathBuf>, notes: impl Into<String>) -> Self {
        Self {
            root: root.into(),
            notes: notes.into(),
        }
    }

    /// The root the layers are under.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// One layer, by name. The name is checked — it is about to be a path
    /// segment — and nothing else is assumed about it.
    pub fn layer(&self, name: &str) -> Result<Repo> {
        let name = name.trim();
        repo::check_name(name)?;
        Ok(Repo::with_notes(self.root.join(name), &self.notes))
    }

    /// Every layer kept here, in name order. Empty for a root that does not
    /// exist yet.
    pub fn names(&self) -> Result<Vec<String>> {
        let mut out = Vec::new();
        let entries = match std::fs::read_dir(&self.root) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(out),
            Err(e) => return Err(e.into()),
        };
        for entry in entries {
            let Some(name) = entry?.file_name().to_str().map(str::to_string) else {
                continue;
            };
            if self.layer(&name).is_ok_and(|layer| layer.exists()) {
                out.push(name);
            }
        }
        out.sort();
        Ok(out)
    }

    /// Fetch what `source` offers into the layer called `name`, creating it
    /// on first use, and note the sender when one is given.
    ///
    /// Fetching again takes only what arrived since; a forgetting the writer
    /// made arrives as the destruction of what it forgot.
    pub fn keep<S: Source + ?Sized>(
        &self,
        name: &str,
        sender: &str,
        source: &S,
        manifest: &str,
    ) -> Result<Received> {
        let name = name.trim();
        let layer = self.layer(name)?;
        let received = layer.receive(name, source, manifest)?;
        layer.note_origin(name)?;
        layer.note_sender(sender)?;
        Ok(received)
    }

    /// The sender noted on one layer, or the layer's own name when none was.
    pub fn sender(&self, name: &str) -> String {
        self.layer(name)
            .ok()
            .and_then(|layer| layer.sender())
            .unwrap_or_else(|| name.trim().to_string())
    }

    // ── suppression ────────────────────────────────────────────────────────

    /// The suppressed paths of one layer, one per line as noted.
    pub fn suppressed(&self, name: &str) -> Result<Vec<String>> {
        let text = match std::fs::read_to_string(self.suppressed_file(name)?) {
            Ok(text) => text,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(e.into()),
        };
        Ok(text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect())
    }

    /// Note one path of a layer as suppressed — kept out of what the caller
    /// shows. Idempotent.
    pub fn suppress(&self, name: &str, path: &str) -> Result<()> {
        let path = path.trim();
        let mut noted = self.suppressed(name)?;
        if noted.iter().any(|p| p == path) {
            return Ok(());
        }
        noted.push(path.to_string());
        self.write_suppressed(name, &noted)
    }

    /// Lift the suppression on one path — shown again, from here on.
    ///
    /// Idempotent, and the inverse of [`suppress`](Self::suppress) in the
    /// only sense that matters: nothing about the chain changed either way,
    /// so nothing has to be fetched back.
    pub fn unsuppress(&self, name: &str, path: &str) -> Result<()> {
        let path = path.trim();
        let mut noted = self.suppressed(name)?;
        let before = noted.len();
        noted.retain(|p| p != path);
        if noted.len() == before {
            return Ok(());
        }
        self.write_suppressed(name, &noted)
    }

    fn suppressed_file(&self, name: &str) -> Result<PathBuf> {
        Ok(self
            .layer(name)?
            .root()
            .join(STORE_DIR)
            .join(&self.notes)
            .join(format!("{SUPPRESSED_NOTE}.txt")))
    }

    /// An empty list leaves an empty file rather than removing it: the note
    /// has been written before, and a reader of it treats absent and empty
    /// alike.
    fn write_suppressed(&self, name: &str, noted: &[String]) -> Result<()> {
        let file = self.suppressed_file(name)?;
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let text = match noted.is_empty() {
            true => String::new(),
            false => format!("{}\n", noted.join("\n")),
        };
        std::fs::write(file, text)?;
        Ok(())
    }
}
