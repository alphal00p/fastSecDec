use super::*;

fn tags(owner: &RootProgram) -> [Atom; 2] {
    [Atom::num(owner.coefficient_count()), Atom::var(owner.tag())]
}

fn alternate(owner: &RootProgram, offset: i64, version: u32) -> RootProgram {
    let (mut saved, _): (Saved, usize) =
        bincode::serde::decode_from_slice(owner.bytes(), bincode::config::standard()).unwrap();
    saved.version = version;
    if offset != 1 {
        let r = symbol!("root_scope_probe::r");
        let a = symbol!("root_scope_probe::a");
        let level = Atom::var(a) * Atom::var(r).pow(2) - Atom::num(offset);
        saved.exact = Atom::evaluator_multiple(
            &[level.clone(), level.derivative(r)],
            &[Atom::var(r), Atom::var(a)],
        )
        .build()
        .unwrap();
    }
    let bytes = bincode::serde::encode_to_vec(&saved, bincode::config::standard()).unwrap();
    RootProgram::from_bytes(&bytes).unwrap()
}

#[test]
fn selected_scope_is_nested_unwind_safe_and_never_uses_another_live_owner() {
    let owner = RootProgram::build(1).unwrap();
    // Deliberately invalid mathematical contract with the same schema. Such
    // bytes must never change resolution for an unrelated selected artifact.
    let foreign = alternate(&owner, 4, 2);
    assert_eq!(owner.tag(), foreign.tag());
    assert_ne!(owner.digest(), foreign.digest());
    let values = tags(&owner);
    let views = values.each_ref().map(Atom::as_view);
    assert!(resolve(&views).is_err());
    let callback = owner.prepare(|| {
        assert!(Arc::ptr_eq(&resolve(&views).unwrap(), &owner.0));
        foreign.prepare(|| {
            assert!(Arc::ptr_eq(&resolve(&views).unwrap(), &foreign.0));
            let mut malformed = foreign.bytes().to_vec();
            malformed.push(0);
            assert!(RootProgram::from_bytes(&malformed).is_err());
            assert!(Arc::ptr_eq(&resolve(&views).unwrap(), &foreign.0));
        });
        assert!(Arc::ptr_eq(&resolve(&views).unwrap(), &owner.0));
        let interrupted = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            foreign.prepare(|| panic!("expected preparation unwind"));
        }));
        assert!(interrupted.is_err());
        assert!(Arc::ptr_eq(&resolve(&views).unwrap(), &owner.0));
        crate::contour::functions::dynamic::numeric::real::<f64>(&views)
    });
    assert!(resolve(&views).is_err());
    // The callback holds the selected exact owner and needs no active scope
    // while sampling, including under a different concurrent preparation.
    foreign.prepare(|| assert!((callback(&[4., 0.8, 1.]) - 0.4).abs() < 1e-14));
    assert!((callback(&[4., 0.8, 1.]) - 0.4).abs() < 1e-14);
    assert_eq!(
        RootProgram::from_bytes(foreign.bytes()).unwrap().digest(),
        foreign.digest()
    );
}

#[test]
fn legacy_helper_keeps_its_original_byte_digest_tag_and_restore_path() {
    let current = RootProgram::build(1).unwrap();
    let legacy = alternate(&current, 1, 1);
    assert_eq!(
        legacy.tag().get_name(),
        format!("{TAG_PREFIX}{}", legacy.digest())
    );
    assert_ne!(current.tag(), legacy.tag());
    let values = tags(&legacy);
    let views = values.each_ref().map(Atom::as_view);
    assert!(Arc::ptr_eq(&resolve(&views).unwrap(), &legacy.0));
}

#[test]
fn semantic_helper_tags_survive_reverse_registration_in_fresh_processes() {
    let temporary = tempfile::tempdir().unwrap();
    for mode in ["ordinary", "reverse"] {
        let result = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "contour::functions::dynamic::program::tests::helper_identity_worker",
                "--nocapture",
            ])
            .env("FASTSECDEC_HELPER_IDENTITY_DIR", temporary.path())
            .env("FASTSECDEC_HELPER_IDENTITY_MODE", mode)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    let ordinary = std::fs::read_to_string(temporary.path().join("ordinary.tag")).unwrap();
    let reverse = std::fs::read_to_string(temporary.path().join("reverse.tag")).unwrap();
    assert_eq!(ordinary, reverse);
    for mode in ["ordinary", "reverse"] {
        let bytes = std::fs::read(temporary.path().join(format!("{mode}.helper"))).unwrap();
        let helper = RootProgram::from_bytes(&bytes).unwrap();
        assert_eq!(helper.tag().get_name(), ordinary);
        assert_eq!(helper.bytes(), bytes);
        assert_eq!(helper.digest(), blake3::hash(&bytes).to_hex().as_str());
        let values = tags(&helper);
        let views = values.each_ref().map(Atom::as_view);
        helper.prepare(|| assert!(Arc::ptr_eq(&resolve(&views).unwrap(), &helper.0)));
    }
}

#[test]
fn helper_identity_worker() {
    let Ok(folder) = std::env::var("FASTSECDEC_HELPER_IDENTITY_DIR") else {
        return;
    };
    let mode = std::env::var("FASTSECDEC_HELPER_IDENTITY_MODE").unwrap();
    if mode == "reverse" {
        for index in (0..4).rev() {
            let _ = symbol!(&format!(
                "fastsecdec::contour::dynamic::helper_coefficient_{index}"
            ));
        }
        let _ = symbol!("fastsecdec::contour::dynamic::helper_root");
    }
    let helper = RootProgram::build(4).unwrap();
    let folder = std::path::Path::new(&folder);
    std::fs::write(folder.join(format!("{mode}.tag")), helper.tag().get_name()).unwrap();
    std::fs::write(folder.join(format!("{mode}.helper")), helper.bytes()).unwrap();
}
