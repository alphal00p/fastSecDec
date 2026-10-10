//! Write the A446 CLI fixture and native serialized model to a fresh directory.
#[path = "../../../examples/no_deformation/a446/prepare.rs"]
mod fixture;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args_os().skip(1);
    let output = arguments
        .next()
        .ok_or("usage: prepare_a446 FRESH_DIRECTORY")?;
    if arguments.next().is_some() {
        return Err("usage: prepare_a446 FRESH_DIRECTORY".into());
    }
    let report = fixture::prepare(std::path::Path::new(&output))?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn native_model_roundtrips_and_existing_output_is_preserved() {
        let temporary = tempfile::tempdir().unwrap();
        let output = temporary.path().join("a446");
        let report = super::fixture::prepare(&output).unwrap();
        assert_eq!(report["coverage_claimed"], false);
        let model = std::fs::read_to_string(output.join("standard-model.json")).unwrap();
        let native = fastsecdec::Model::from_json(&model).unwrap();
        let roundtrip: serde_json::Value =
            serde_json::from_str(&native.to_json().unwrap()).unwrap();
        assert_eq!(
            roundtrip,
            serde_json::from_str::<serde_json::Value>(&model).unwrap()
        );
        assert!(super::fixture::prepare(&output).is_err());
        assert_eq!(
            model,
            std::fs::read_to_string(output.join("standard-model.json")).unwrap()
        );
    }
}
