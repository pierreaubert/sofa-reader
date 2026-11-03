#![cfg(feature = "sqlite")]

// Rust guideline compliant 2026-10-09
use sofa_reader::{SofaError, SofaFile, SofaWriter};
use std::process::Command;

fn assert_positions(file: &SofaFile, expected: &[[f32; 3]]) {
    let actual: Vec<_> = file
        .positions
        .iter()
        .map(|p| [p.azimuth, p.elevation, p.distance])
        .collect();
    assert_eq!(actual, expected);
}

#[test]
fn converter_round_trips_sofa_through_public_loader() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.sofa");
    let output = dir.path().join("output.hrtfdb");
    let positions = [0.0, 0.0, 1.0, 90.0, 10.0, 2.0];
    let responses = [1.0, -0.5, 0.25, 0.0, 0.125, -0.25, 0.5, -1.0];
    let mut writer = SofaWriter::new();
    writer.add_attribute_str("Conventions", "SOFA");
    writer.add_attribute_str("SOFAConventions", "SimpleFreeFieldHRIR");
    writer.add_dimension("M", 2);
    writer.add_dimension("R", 2);
    writer.add_dimension("N", 2);
    writer.add_dimension("C", 3);
    writer.add_variable_f64("Data.SamplingRate", &[]);
    writer
        .write_scalar_f64("Data.SamplingRate", 12_345.678)
        .unwrap();
    writer.add_variable_f32("SourcePosition", &["M", "C"]);
    writer.write_f32("SourcePosition", &positions).unwrap();
    writer.add_variable_f32("Data.IR", &["M", "R", "N"]);
    writer.write_f32("Data.IR", &responses).unwrap();
    writer.finish(&input).unwrap();

    let result = Command::new(env!("CARGO_BIN_EXE_sofa-to-sqlite"))
        .args([&input, &output])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let file = SofaFile::try_load(&output).unwrap();
    assert_eq!((file.num_measurements, file.ir_length), (2, 2));
    assert_eq!(file.sample_rate, 12_345.678);
    assert_eq!(file.data_sample_rate, Some(12_345.678));
    assert_eq!(file.convention, "SimpleFreeFieldHRIR");
    assert_positions(&file, &[[0.0, 0.0, 1.0], [90.0, 10.0, 2.0]]);
    assert_eq!(file.impulse_responses, responses);
    let db = rusqlite::Connection::open(&output).unwrap();
    let version: String = db
        .query_row(
            "SELECT value FROM metadata WHERE key='schema_version'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(version, "1");
}

#[test]
fn frozen_unversioned_cache_loads() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/data/legacy.hrtfdb");
    let file = SofaFile::try_load(path).unwrap();
    assert_eq!((file.num_measurements, file.ir_length), (2, 2));
    assert_eq!(file.sample_rate, 48_000.0);
    assert_eq!(file.data_sample_rate, None);
    assert_eq!(file.convention, "SimpleFreeFieldHRIR");
    assert_positions(&file, &[[0.0, 0.0, 1.0], [90.0, 10.0, 2.0]]);
    assert_eq!(
        file.impulse_responses,
        [1.0, -0.5, 0.25, 0.0, 0.125, -0.25, 0.5, -1.0]
    );
}

#[test]
fn rejects_unknown_and_malformed_schema_versions() {
    for version in ["2", "0", "garbage", ""] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("modified.hrtfdb");
        std::fs::copy(
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/legacy.hrtfdb"),
            &path,
        )
        .unwrap();
        let db = rusqlite::Connection::open(&path).unwrap();
        db.execute(
            "INSERT INTO metadata VALUES ('schema_version', ?1)",
            [version],
        )
        .unwrap();
        drop(db);
        assert!(
            matches!(SofaFile::try_load(&path), Err(SofaError::Unsupported(message)) if message.contains("schema version"))
        );
    }
}

#[test]
fn converter_fails_for_missing_input() {
    let dir = tempfile::tempdir().unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_sofa-to-sqlite"))
        .args([
            dir.path().join("missing.sofa"),
            dir.path().join("output.hrtfdb"),
        ])
        .output()
        .unwrap();
    assert!(!result.status.success());
}

#[test]
fn cache_rejects_byte_length_overflow() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("overflow.hrtfdb");
    std::fs::copy(
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/legacy.hrtfdb"),
        &path,
    )
    .unwrap();
    let db = rusqlite::Connection::open(&path).unwrap();
    db.execute(
        "UPDATE metadata SET value = '1' WHERE key = 'num_measurements'",
        [],
    )
    .unwrap();
    db.execute(
        "UPDATE metadata SET value = ?1 WHERE key = 'ir_length'",
        [(usize::MAX / 4).to_string()],
    )
    .unwrap();
    // Keep one position so validation reaches the dimension arithmetic.
    let positions = [sofa_reader::SourcePosition::new(0.0, 0.0, 1.0)];
    let blob =
        bincode::serde::encode_to_vec(positions.as_slice(), bincode::config::standard()).unwrap();
    db.execute(
        "UPDATE data SET value = ?1 WHERE key = 'positions'",
        [&blob],
    )
    .unwrap();
    drop(db);
    assert!(
        matches!(SofaFile::try_load(path), Err(SofaError::InvalidStructure(message)) if message.contains("overflow"))
    );
}
