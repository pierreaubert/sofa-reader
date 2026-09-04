// Regression tests over `tests/data/minimal.sofa`, a writer-produced fixture
// that the reference HDF5/netCDF tools read cleanly (`h5dump` datasets and
// attributes verify; `ncdump -h` lists all dimensions/variables).
//
// To regenerate after an intentional writer change, run this once (e.g. as a
// scratch example) and re-validate with `h5dump`/`ncdump` before committing:
// ```rust,no_run
// let mut w = sofa_reader::SofaWriter::new();
// w.add_attribute_str("Conventions", "SOFA");
// w.add_attribute_str("Version", "2.1");
// w.add_attribute_str("SOFAConventions", "SimpleFreeFieldHRIR");
// w.add_attribute_str("SOFAConventionsVersion", "1.0");
// w.add_attribute_str("DataType", "FIR");
// w.add_dimension("M", 3);
// w.add_dimension("R", 2);
// w.add_dimension("N", 8);
// w.add_dimension("C", 3);
// w.add_variable_f32("Data.SamplingRate", &[]);
// w.write_scalar_f32("Data.SamplingRate", 44100.0).unwrap();
// w.add_variable_f32("SourcePosition", &["M", "C"]);
// w.write_f32(
//     "SourcePosition",
//     &[0.0, 0.0, 1.0, 90.0, 0.0, 1.0, 180.0, 0.0, 1.0],
// )
// .unwrap();
// w.add_variable_f32("Data.IR", &["M", "R", "N"]);
// let ir: Vec<f32> = (0..3 * 2 * 8).map(|i| i as f32 * 0.125).collect();
// w.write_f32("Data.IR", &ir).unwrap();
// w.finish("tests/data/minimal.sofa").unwrap();
// ```

use sofa_reader::{SofaFile, SofaReader, SourcePosition};

const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/minimal.sofa");

#[test]
fn golden_dimensions_and_attributes() {
    let r = SofaReader::open(FIXTURE).unwrap();
    assert_eq!(r.dimension("M").unwrap(), 3);
    assert_eq!(r.dimension("R").unwrap(), 2);
    assert_eq!(r.dimension("N").unwrap(), 8);
    assert_eq!(r.dimension("C").unwrap(), 3);
    assert_eq!(r.attribute_string("Conventions").unwrap(), "SOFA");
    assert_eq!(
        r.attribute_string("SOFAConventions").unwrap(),
        "SimpleFreeFieldHRIR"
    );
    assert_eq!(r.attribute_string("DataType").unwrap(), "FIR");
}

#[test]
fn golden_values() {
    let r = SofaReader::open(FIXTURE).unwrap();
    assert_eq!(r.read_scalar_f32("Data.SamplingRate").unwrap(), 44100.0);

    let pos = r.read_f32("SourcePosition").unwrap();
    assert_eq!(pos.len(), 9);
    assert_eq!(&pos[0..3], &[0.0, 0.0, 1.0]);
    assert_eq!(&pos[3..6], &[90.0, 0.0, 1.0]);

    // IR sample i holds i * 0.125, row-major over [M, R, N].
    let ir = r.read_f32("Data.IR").unwrap();
    assert_eq!(ir.len(), 3 * 2 * 8);
    assert_eq!(ir[0], 0.0);
    assert_eq!(ir[8], 1.0);
    assert_eq!(ir[47], 47.0 * 0.125);
}

#[test]
fn golden_strict_load() {
    let sofa = SofaFile::strict_load(FIXTURE).unwrap();
    assert_eq!(sofa.num_measurements, 3);
    assert_eq!(sofa.ir_length, 8);
    assert_eq!(sofa.sample_rate, 44100.0);
    assert_eq!(sofa.convention, "SimpleFreeFieldHRIR");
    assert_eq!(sofa.positions.len(), 3);

    let (_, left, right) = sofa.get_hrtf_slices(2).unwrap();
    assert_eq!(left.len(), 8);
    assert_eq!(left[0], 32.0 * 0.125);
    assert_eq!(right[0], 40.0 * 0.125);

    let nearest = sofa.get_hrtf_at_position(&SourcePosition::new(95.0, 0.0, 1.0));
    assert!(nearest.is_some());
}
