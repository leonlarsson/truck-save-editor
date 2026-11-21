//! Reads and writes SCS Software's SII files: the `ScsC` encrypted wrapper, binary SII and text SII.
//!
//! Bytes in, bytes out: no disk access and no UI, so this builds for WebAssembly too.
//! The format is described in `docs/sii-format.md`.
