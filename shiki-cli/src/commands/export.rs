use anyhow::{Context, Result};
use clap::ValueEnum;
use shiki_config::Config;
use shiki_core::NotebookStore;
use std::path::Path;

use super::{find_note, get_notebook, unlock_if_encrypted};

#[derive(Clone, Copy, ValueEnum)]
pub enum ExportFormat {
    /// A single self-contained HTML file.
    Html,
    /// A single plain-Markdown bundle.
    Md,
}

impl From<ExportFormat> for shiki_core::export::Format {
    fn from(format: ExportFormat) -> Self {
        match format {
            ExportFormat::Html => shiki_core::export::Format::Html,
            ExportFormat::Md => shiki_core::export::Format::Md,
        }
    }
}

/// Exports either one explicitly-selected note or every note in `notebook`.
/// The notebook path is deliberately left unchanged when `note` is absent;
/// single-note mode reuses the same lookup/unlock contract as show/edit/etc.
pub fn run(
    store: &NotebookStore,
    config: &Config,
    notebook: &str,
    note: Option<&str>,
    out: &Path,
    format: ExportFormat,
) -> Result<()> {
    if let Some(selector) = note {
        let nb = unlock_if_encrypted(config, get_notebook(store, notebook)?)?;
        let note = find_note(&nb, selector)?;
        let title = note.frontmatter.title.clone();
        let content = shiki_core::export::render_note(&note, format.into())?;
        std::fs::write(out, content)
            .with_context(|| format!("failed to write '{}'", out.display()))?;
        println!("exported '{title}' to {}", out.display());
        return Ok(());
    }

    let nb = store.get(notebook).with_context(|| {
        format!("notebook '{notebook}' not found — see `shiki notebook list`")
    })?;
    let mut notes = nb.all_notes_recursive()?;
    notes.sort_by(|a, b| {
        a.frontmatter
            .date
            .cmp(&b.frontmatter.date)
            .then_with(|| a.frontmatter.title.cmp(&b.frontmatter.title))
    });

    let content = shiki_core::export::render(notebook, &notes, format.into());
    std::fs::write(out, content).with_context(|| format!("failed to write '{}'", out.display()))?;
    println!("exported {} notes to {}", notes.len(), out.display());
    Ok(())
}
