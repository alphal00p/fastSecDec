use super::*;
use fastsecdec::kernel::indexed::{ProgramArchiveCatalogue, ProgramArchiveWriter, ProgramRecipe};
use std::{
    io::{Cursor, Seek, SeekFrom},
    ops::ControlFlow,
};

fn fixture(directory: &Path) -> (Artifact, ProgramArchiveCatalogue) {
    let card = directory.join("fixed.toml");
    fs::write(&card, "[direct]\ndomain='unit_cube'\nparameters=['x']\n[[direct.terms]]\nmonomial_powers=['0']\n[[direct.terms.factors]]\npolynomial='1+x'\nexponent='-1+eps'\nsemantics='causal'\n[generation]\ncontour=true\norder=1\n[generation.evaluator]\nbackend='eager'\n").unwrap();
    let (_, fixed) = crate::generate::generate(
        &card,
        &directory.join("fixed.fsd"),
        &mut crate::display::Dashboard::new(false, false).unwrap(),
        None,
    )
    .unwrap();
    let plain = super::persistence_tests::current_kernels();
    let mut writer = ProgramArchiveWriter::new(
        Cursor::new(Vec::new()),
        blake3::hash(b"cli recipe test").to_hex().to_string(),
        [ProgramRecipe::UndeformedV1, ProgramRecipe::FixedV1],
    )
    .unwrap();
    for (recipe, kernels) in [
        (ProgramRecipe::FixedV1, fixed),
        (ProgramRecipe::UndeformedV1, plain),
    ] {
        let (bytes, catalogue) = fastsecdec::kernel::indexed::to_bytes(&kernels).unwrap();
        let mut bytes = Cursor::new(bytes);
        for record in catalogue.records {
            bytes.seek(SeekFrom::Start(record.offset)).unwrap();
            writer
                .append_record(recipe, &mut bytes, record.receipt)
                .unwrap();
        }
    }
    let (writer, catalogue) = writer.finish().unwrap();
    let staged = directory.join("recipes.dat");
    fs::write(&staged, writer.into_inner()).unwrap();
    let artifact = Artifact::from_program_archive(
        &staged,
        catalogue.clone(),
        ProgramRecipe::FixedV1,
        super::persistence_tests::provenance(),
    )
    .unwrap();
    (artifact, catalogue)
}

#[test]
fn recipe_manifest_selects_before_loading_and_supports_offline_inspection() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("bundle.fsd");
    let (artifact, catalogue) = fixture(directory.path());
    artifact.save_staged(&path).unwrap();
    let mut metadata = Artifact::load_metadata(&path).unwrap();
    assert_eq!(metadata.selected_recipe(), Some(ProgramRecipe::FixedV1));
    assert_eq!(
        metadata.kernel_summary().unwrap().orders,
        catalogue.recipe(ProgramRecipe::FixedV1).unwrap().orders
    );
    metadata.select_recipe(ProgramRecipe::UndeformedV1).unwrap();
    assert_eq!(
        metadata.kernel_summary().unwrap().orders,
        catalogue
            .recipe(ProgramRecipe::UndeformedV1)
            .unwrap()
            .orders
    );
    let data = metadata.data_path(&path).unwrap();
    let saved = fs::read(&data).unwrap();
    fs::remove_file(&data).unwrap();
    let mut offline =
        Artifact::load_metadata_with_options(&path, KernelLoadOptions { validate: true }).unwrap();
    offline.select_recipe(ProgramRecipe::UndeformedV1).unwrap();
    assert_eq!(offline.kernel_summary().unwrap().sectors, 1);
    assert_eq!(offline.selected_recipe(), Some(ProgramRecipe::UndeformedV1));
    fs::write(&data, saved).unwrap();
    for recipe in [ProgramRecipe::UndeformedV1, ProgramRecipe::FixedV1] {
        let (loaded, kernels) = Artifact::load_recipe_observed(
            &path,
            KernelLoadOptions { validate: true },
            Some(recipe),
            |metadata| {
                assert_eq!(metadata.selected_recipe(), Some(recipe));
                Ok(())
            },
            |_| ControlFlow::Continue(()),
        )
        .unwrap();
        assert_eq!(loaded.kernel_content_id, catalogue.content_id);
        assert_eq!(
            kernels.content_id(),
            catalogue.recipe(recipe).unwrap().content_id
        );
        assert_eq!(kernels.orders(), catalogue.recipe(recipe).unwrap().orders);
    }
}

#[test]
fn recipe_load_never_reads_an_unselected_corrupt_payload() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("bundle.fsd");
    let (artifact, catalogue) = fixture(directory.path());
    artifact.save_staged(&path).unwrap();
    let metadata = Artifact::load_metadata(&path).unwrap();
    let record = &catalogue
        .recipe(ProgramRecipe::UndeformedV1)
        .unwrap()
        .records[0];
    let mut data = OpenOptions::new()
        .write(true)
        .open(metadata.data_path(&path).unwrap())
        .unwrap();
    data.seek(SeekFrom::Start(record.offset)).unwrap();
    data.write_all(b"BROKEN").unwrap();
    let load = |recipe| {
        Artifact::load_recipe_observed(
            &path,
            KernelLoadOptions { validate: true },
            Some(recipe),
            |_| Ok(()),
            |_| ControlFlow::Continue(()),
        )
    };
    assert!(load(ProgramRecipe::FixedV1).is_ok());
    assert!(load(ProgramRecipe::UndeformedV1).is_err());
}

#[test]
fn failed_recipe_publication_keeps_previous_artifact_usable() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("bundle.fsd");
    let (artifact, _) = fixture(directory.path());
    artifact.save_staged(&path).unwrap();
    let metadata_path = paths(&path).unwrap().0;
    let previous = fs::read(&metadata_path).unwrap();
    fs::remove_file(artifact.staged_data.as_ref().unwrap()).unwrap();
    assert!(artifact.save_staged(&path).is_err());
    assert_eq!(fs::read(&metadata_path).unwrap(), previous);
    assert!(Artifact::load_with_options(&path, KernelLoadOptions { validate: true }).is_ok());
}
