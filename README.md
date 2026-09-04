# sofa-reader

Pure-Rust SOFA (HDF5/NetCDF4) file reader and writer for HRTF data.

## Overview

Provides reading and writing of Spatially Oriented Format for Acoustics (SOFA) files, commonly used for storing Head-Related Transfer Function (HRTF) measurements. Built entirely in Rust with no C dependencies.

## Features

- **Pure Rust**: No dependency on system HDF5/NetCDF4 libraries
- **Read & Write**: Load existing SOFA files and create new ones
- **HRTF API**: High-level types like `SofaFile`, `HrtfData`, and `SourcePosition`
- **Coordinate Conversion**: Spherical/cartesian coordinate system support
- **Nearest-Position Lookup**: Find closest HRTF measurement for a given source direction
- **SQLite Integration**: Load `.hrtfdb` SQLite databases
- **Deflate Support**: Optional zlib decompression via `flate2` (enabled by default)

## Usage

```rust
use sofa_reader::SofaReader;

let reader = SofaReader::open("hrtf.sofa")?;
let m = reader.dimension("M")?;
let ir = reader.read_f32("Data.IR")?;
```

### HRTF API

```rust
use sofa_reader::{SofaFile, SourcePosition};

let sofa = SofaFile::try_load("hrtf.sofa")?;

// Nearest HRTF for a direction (azimuth, elevation, distance)
let query = SourcePosition::new(30.0, 0.0, 1.0);
let hrtf = sofa.get_hrtf_at_position(&query).expect("no measurements");
println!("left IR length: {}", hrtf.ir_left.len());
```

### Writing

```rust
use sofa_reader::SofaWriter;

let mut w = SofaWriter::new();
w.add_attribute_str("Conventions", "SOFA");
w.add_attribute_str("SOFAConventions", "SimpleFreeFieldHRIR");
// Declare dimensions, then variables, then payloads:
w.add_dimension("M", 1);
w.add_variable_f32("Data.SamplingRate", &[]);
w.write_scalar_f32("Data.SamplingRate", 48000.0)?;
w.finish("out.sofa")?;
```

(The crate-level docs in `src/lib.rs` contain the same flow as a
compile-checked doctest, so the examples cannot rot.)

## Module Layout

- `hdf5` — Low-level HDF5/NetCDF4 file parser
- `hrtf` — High-level HRTF data structures and queries
- `error` — Error types (`SofaError`, `Result`)

## Features

| Feature  | Default | Description                            |
|----------|---------|----------------------------------------|
| `deflate`| Yes     | Enables zlib decompression via `flate2`|

## Testing

```bash
# Using just (mirrors CI)
just all

# Or directly with cargo
cargo test --all-features
```

## Development

```bash
just        # list available recipes
just all    # format check, clippy, tests, and docs
```

## License

This project is licensed under the MIT License.
