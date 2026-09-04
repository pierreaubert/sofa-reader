# sofa-reader

Pure-Rust SOFA (HDF5/NetCDF4) file reader and writer for HRTF data.

## Overview

First-party in-house library. The standard HDF5 library is hard to build portably across platforms; this crate extracts only the subset needed for SOFA HRTF files.

## Features

- `deflate` (default) -- Deflate compression support via `flate2`

## Testing

```bash
just all  # check + check-minimal + fmt-check + clippy (-D warnings) + test + doc
```

CI (`.github/workflows/ci.yml`, ubuntu + macOS) runs the same gates plus
`--no-default-features` builds. Clippy warnings fail the build; the full
`cargo test --all-features` suite (lib + `tests/malformed.rs` +
`tests/property_tests.rs` + doctests) must pass — not just `--lib`.

## Important Notes

- This is not an upstream fork -- it is a purpose-built SOFA reader
- Only supports the HDF5/NetCDF4 features needed for SOFA files
- Used by `sotf-host` for loading HRTF data for the binaural plugin
