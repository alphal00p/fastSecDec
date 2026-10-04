use fastsecdec::results::{ResultSectorSort, SavedIntegrationResult};

use crate::CliResult;

pub(super) fn show(
    result: &SavedIntegrationResult,
    sort: ResultSectorSort,
    json: bool,
) -> CliResult<()> {
    let order = result.sector_order(sort)?;
    let comparison = result.comparison()?;
    if json {
        // This is a view, not the persistence envelope written by encode_result.
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "result": result,
                "sector_order": order,
                "comparison": comparison,
            }))?
        );
        return Ok(());
    }
    print!("{}", result.display_with_sort(sort)?);
    Ok(())
}
