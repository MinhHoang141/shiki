use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use shiki_config::Config;
use shiki_core::{Note, NotebookStore};

use super::{find_note, get_notebook, unlock_if_encrypted};

fn export_dir(store: &NotebookStore, config: &Config) -> PathBuf {
    let configured = config.export.export_dir.trim();
    if configured.is_empty() {
        store.root.join("exports")
    } else {
        PathBuf::from(configured)
    }
}

/// Publishes either one explicitly-selected note or the complete notebook
/// through the same `shiki_core::publish` pipeline. An omitted output path
/// keeps the existing notebook filename, while single-note mode derives a
/// filename from the resolved note title rather than the user's selector.
pub fn run(
    store: &NotebookStore,
    config: &Config,
    notebook: &str,
    note: Option<&str>,
    out: Option<&Path>,
    theme: &str,
    cache_dir: &Path,
) -> Result<()> {
    if let Some(selector) = note {
        let nb = unlock_if_encrypted(config, get_notebook(store, notebook)?)?;
        let note = find_note(&nb, selector)?;
        let title = note.frontmatter.title.clone();
        let default_out =
            export_dir(store, config).join(format!("{}.pdf", Note::slugify(&title)));
        let out = out.unwrap_or(&default_out);
        shiki_core::publish::publish(std::slice::from_ref(&note), theme, cache_dir, out)
            .with_context(|| format!("failed to publish '{title}' to {}", out.display()))?;
        println!("published '{title}' to {}", out.display());
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
    let default_out = export_dir(store, config).join(format!("{notebook}.pdf"));
    let out = out.unwrap_or(&default_out);

    shiki_core::publish::publish(&notes, theme, cache_dir, out)
        .with_context(|| format!("failed to publish '{notebook}' to {}", out.display()))?;
    println!("published {} notes to {}", notes.len(), out.display());
    Ok(())
}
