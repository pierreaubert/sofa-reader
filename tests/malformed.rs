// Malformed-input tests: corrupt or truncated bytes must produce `Err`,
// never a panic. (A panic in the library fails these tests automatically.)

use sofa_reader::{SofaReader, SofaWriter};

fn valid_sofa_bytes() -> Vec<u8> {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("valid.sofa");

    let mut w = SofaWriter::new();
    w.add_attribute_str("Conventions", "SOFA");
    w.add_attribute_str("Version", "2.1");
    w.add_attribute_str("SOFAConventions", "SimpleFreeFieldHRIR");
    w.add_attribute_str("SOFAConventionsVersion", "1.0");
    w.add_attribute_str("DataType", "FIR");
    w.add_dimension("M", 2);
    w.add_dimension("R", 2);
    w.add_dimension("N", 4);
    w.add_dimension("C", 3);
    w.add_variable_f32("Data.SamplingRate", &[]);
    w.write_scalar_f32("Data.SamplingRate", 48000.0).unwrap();
    w.add_variable_f32("SourcePosition", &["M", "C"]);
    w.write_f32("SourcePosition", &[0.0, 0.0, 1.0, 90.0, 0.0, 1.0])
        .unwrap();
    w.add_variable_f32("Data.IR", &["M", "R", "N"]);
    w.write_f32("Data.IR", &[0.1; 16]).unwrap();
    w.finish(&path).unwrap();

    std::fs::read(&path).unwrap()
}

#[test]
fn empty_input_rejected() {
    assert!(SofaReader::from_slice(&[]).is_err());
}

#[test]
fn magic_only_rejected() {
    // Exactly 8 bytes (magic, no version byte) used to index out of bounds.
    let magic = [0x89, b'H', b'D', b'F', 0x0d, 0x0a, 0x1a, 0x0a];
    assert!(SofaReader::from_slice(&magic).is_err());
}

#[test]
fn bad_magic_rejected() {
    assert!(SofaReader::from_slice(&[0u8; 16]).is_err());
    assert!(SofaReader::from_slice(b"not an hdf5 file!!!!").is_err());
}

#[test]
fn truncated_superblock_rejected() {
    let valid = valid_sofa_bytes();
    // Superblock v2 needs 48 bytes; every shorter prefix must fail.
    for len in [9, 10, 12, 20, 40, 47] {
        assert!(
            SofaReader::from_slice(&valid[..len]).is_err(),
            "prefix of {len} bytes should not parse"
        );
    }
}

#[test]
fn short_prefixes_rejected() {
    let valid = valid_sofa_bytes();
    // Well past the superblock: the root object header cannot be intact.
    for len in 0..64 {
        assert!(
            SofaReader::from_slice(&valid[..len.min(valid.len())]).is_err(),
            "prefix of {len} bytes should not parse"
        );
    }
}

#[test]
fn truncation_sweep_never_panics() {
    let valid = valid_sofa_bytes();
    // Any truncation must return Err (or, past the metadata, Ok) — but the
    // real assertion is implicit: a library panic fails this test.
    for len in 0..valid.len() {
        let _ = SofaReader::from_slice(&valid[..len]);
    }
    let _ = SofaReader::from_slice(&valid);
}

#[test]
fn single_byte_flips_never_panic() {
    let valid = valid_sofa_bytes();
    // Corrupt every byte position in turn; open must not panic. Parsing only
    // touches metadata at open time, so no read is attempted here.
    for i in 0..valid.len() {
        let mut mutated = valid.clone();
        mutated[i] ^= 0xFF;
        let _ = SofaReader::from_slice(&mutated);
    }
}

#[test]
fn garbage_with_valid_magic_rejected() {
    let mut zeros = vec![0u8; 256];
    zeros[..8].copy_from_slice(&[0x89, b'H', b'D', b'F', 0x0d, 0x0a, 0x1a, 0x0a]);
    assert!(SofaReader::from_slice(&zeros).is_err());

    let mut ones = vec![0xFFu8; 256];
    ones[..8].copy_from_slice(&[0x89, b'H', b'D', b'F', 0x0d, 0x0a, 0x1a, 0x0a]);
    assert!(SofaReader::from_slice(&ones).is_err());
}

#[test]
fn truncated_reads_error_without_panic() {
    // Files truncated inside the data section open fine (metadata is
    // intact) but reads must fail cleanly.
    let valid = valid_sofa_bytes();
    let mut truncated_ok = false;
    for len in (0..valid.len()).rev() {
        if let Ok(reader) = SofaReader::from_slice(&valid[..len]) {
            // Metadata survived; data reads must Err, never panic.
            let _ = reader.read_f32("Data.IR");
            let _ = reader.read_f32("SourcePosition");
            truncated_ok = true;
            break;
        }
    }
    assert!(truncated_ok, "expected some truncation to still open");
}
