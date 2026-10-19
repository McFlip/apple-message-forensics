use sha2::{Digest, Sha256};
use std::{fs, path::Path, process::Command};
use tempfile::tempdir;
use zip::ZipArchive;

fn sha256(contents: &[u8]) -> String {
    hex::encode(Sha256::digest(contents))
}

fn run_deliver(output_dir: &Path, working_dir: &Path) {
    let result = Command::new(env!("CARGO_BIN_EXE_apple-message-forensics"))
        .arg("--output")
        .arg(output_dir)
        .arg("deliver")
        .current_dir(working_dir)
        .output()
        .expect("run deliver command");

    assert!(
        result.status.success(),
        "deliver command failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

fn prepare_delivery_inputs(root: &Path, output_dir: &Path, include_cellebrite: bool) {
    let vault_path = output_dir.join("evidence/vault/vault.zip");
    fs::create_dir_all(vault_path.parent().expect("vault has parent"))
        .expect("create vault directory");
    fs::write(&vault_path, b"vault archive").expect("write vault archive");

    let report_dir = output_dir.join("report");
    fs::create_dir_all(&report_dir).expect("create report directory");
    fs::write(report_dir.join("index.html"), b"report").expect("write report");

    let bin_dir = root.join("bin");
    fs::create_dir_all(&bin_dir).expect("create binary directory");
    fs::write(bin_dir.join("reader"), b"reader binary").expect("write reader binary");
    fs::write(bin_dir.join("report-server"), b"report server binary")
        .expect("write report server binary");

    let scripts_dir = root.join("scripts");
    fs::create_dir_all(&scripts_dir).expect("create scripts directory");
    fs::write(
        scripts_dir.join("validate_hashes.ps1"),
        include_bytes!("../scripts/validate_hashes.ps1"),
    )
    .expect("write hash validation script");

    if include_cellebrite {
        let cellebrite_dir = output_dir.join("cellebrite");
        fs::create_dir_all(&cellebrite_dir).expect("create Cellebrite directory");
        let zip_contents = b"Cellebrite ZIP contents";
        fs::write(cellebrite_dir.join("fake_iPhone.zip"), zip_contents)
            .expect("write Cellebrite ZIP");
        fs::write(
            cellebrite_dir.join("fake_iPhone.zip.sha256"),
            format!("{}  fake_iPhone.zip\n", sha256(zip_contents)),
        )
        .expect("write Cellebrite hash manifest");
    }
}

fn assert_delivery_archive(delivery_dir: &Path, expect_cellebrite: bool) {
    let archive_file =
        fs::File::open(delivery_dir.join("deliverable.zip")).expect("open delivery archive");
    let mut archive = ZipArchive::new(archive_file).expect("read delivery archive");
    let names: Vec<_> = archive
        .file_names()
        .map(|name| name.expect("read delivery archive entry").into_owned())
        .collect();

    for expected in ["report.zip", "vault.zip", "validate_hashes.ps1"] {
        assert!(
            names.iter().any(|name| name == expected),
            "missing {expected}"
        );
    }
    assert!(
        names.iter().any(|name| name == "fake_iPhone.zip") == expect_cellebrite,
        "Cellebrite ZIP inclusion did not match expected condition"
    );
    assert!(
        !names.iter().any(|name| name == "deliverable.zip")
            && !names.iter().any(|name| name == "deliverable.zip.sha256"),
        "delivery archive must not contain itself or its hash manifest"
    );

    let mut script = archive
        .by_name("validate_hashes.ps1")
        .expect("find validation script in delivery archive");
    let mut script_contents = Vec::new();
    std::io::Read::read_to_end(&mut script, &mut script_contents)
        .expect("read validation script from archive");
    assert_eq!(
        script_contents,
        include_bytes!("../scripts/validate_hashes.ps1")
    );
}

#[test]
fn deliver_copies_and_verifies_cellebrite_and_hashes_the_delivery_archive() {
    let temp_dir = tempdir().expect("create temporary directory");
    let root = temp_dir.path();
    let output_dir = root.join("output");
    prepare_delivery_inputs(root, &output_dir, true);

    run_deliver(&output_dir, root);

    let delivery_dir = output_dir.join("delivery");
    let cellebrite_zip =
        fs::read(delivery_dir.join("fake_iPhone.zip")).expect("read delivered Cellebrite ZIP");
    let cellebrite_manifest = fs::read_to_string(delivery_dir.join("fake_iPhone.zip.sha256"))
        .expect("read delivered Cellebrite manifest");
    assert_eq!(
        cellebrite_manifest,
        format!("{}  fake_iPhone.zip\n", sha256(&cellebrite_zip))
    );
    assert!(
        delivery_dir.join("validate_hashes.ps1").is_file(),
        "validation script should be copied beside delivery artifacts"
    );

    let deliverable_zip =
        fs::read(delivery_dir.join("deliverable.zip")).expect("read delivery archive bytes");
    assert_eq!(
        fs::read_to_string(delivery_dir.join("deliverable.zip.sha256"))
            .expect("read delivery archive hash manifest"),
        format!("{}  deliverable.zip\n", sha256(&deliverable_zip))
    );
    assert_delivery_archive(&delivery_dir, true);
}

#[test]
fn deliver_omits_cellebrite_when_archive_and_manifest_are_not_both_present() {
    let temp_dir = tempdir().expect("create temporary directory");
    let root = temp_dir.path();
    let output_dir = root.join("output");
    prepare_delivery_inputs(root, &output_dir, false);

    run_deliver(&output_dir, root);

    let delivery_dir = output_dir.join("delivery");
    assert_delivery_archive(&delivery_dir, false);
    assert!(
        fs::read_to_string(delivery_dir.join("deliverable.zip.sha256"))
            .expect("read delivery archive hash manifest")
            .contains("  deliverable.zip\n")
    );
}
