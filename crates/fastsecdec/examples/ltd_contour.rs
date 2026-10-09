//! Validate and export native HEPKit inputs for the pinned LTD-paper fixtures.
//! No sector generation or integration is performed by this command.
#[path = "ltd_contour/export.rs"]
mod export;
#[path = "ltd_contour/import.rs"]
mod import;
#[path = "ltd_contour/records.rs"]
mod records;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let mut args = std::env::args().skip(1);
    let mut output = None;
    let mut selected = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => {
                output = Some(std::path::PathBuf::from(
                    args.next().ok_or("--output needs a directory")?,
                ))
            }
            "--case" => selected = Some(args.next().ok_or("--case needs an archived record name")?),
            "--help" | "-h" => {
                println!(
                    "ltd_contour [--case NAME] [--output DIRECTORY]\nValidate native diagrams/families/U/F; optionally export CLI cards. Never generates sectors."
                );
                return Ok(());
            }
            _ => return Err(format!("unknown argument {arg}").into()),
        }
    }
    let records = records::records()?;
    if selected
        .as_ref()
        .is_some_and(|name| !records.iter().any(|record| &record.name == name))
    {
        return Err("unknown or quarantined LTD record".into());
    }
    if let Some(root) = &output {
        std::fs::create_dir_all(root)?;
        std::fs::write(root.join("model.json"), records::MODEL)?;
    }
    for record in records
        .into_iter()
        .filter(|record| selected.as_ref().is_none_or(|name| name == &record.name))
    {
        let imported = import::import(&record)?;
        if let Some(root) = &output {
            export::write(root, &record, &imported)?;
        }
        let s = &imported.summary;
        println!(
            "{}: {} loops, {} ordered denominators matched, U/F terms {}/{}, F signs +{}/-{}, roundoff {:.3e}; no generation",
            record.name,
            s.loops,
            s.denominator_matches,
            s.native_u_terms,
            s.native_f_terms,
            s.f_positive_coefficients,
            s.f_negative_coefficients,
            s.momentum_roundoff_reconciliation_max
                .max(s.archived_shift_residual_max)
        );
    }
    Ok(())
}

#[cfg(test)]
#[path = "ltd_contour/tests.rs"]
mod tests;
