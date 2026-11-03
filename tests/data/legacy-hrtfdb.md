# Legacy HRTF cache fixture

`legacy.hrtfdb` represents the unversioned metadata and data tables used before
schema version 1. It was generated independently with Python's `sqlite3` and
`struct` modules. The SQLite page size is 512 bytes. Its SHA-256 is
`1b15a4dfe28997e5dc4ea79e8a362e9fc6a2b615901f2d088012970df2ed504e`.

The metadata stores `SimpleFreeFieldHRIR`, sample rate 48000, two measurements,
and two samples per response. Positions are `(0, 0, 1)` and `(90, 10, 2)`.
The positions blob has a one-byte bincode vector length followed by six
little-endian f32 values. The impulse blob contains eight little-endian f32
values in measurement, receiver, sample order: `1, -0.5, 0.25, 0, 0.125,
-0.25, 0.5, -1`. There is no `schema_version` or `data_sample_rate` entry.
