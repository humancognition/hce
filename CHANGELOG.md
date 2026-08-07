# Changelog

## 0.1.2

### Added
- Korean language pack (`ko`) with differentiated phoneme inventory
- Per-pack parser tables — each language level now has independent phoneme parsing
- Proper calendar month calculation for Open mode timestamp prefix

### Changed
- Open mode prefix changed from `KR` to `BD` to eliminate onset-coda character overlap
- Single consonants `n` and `l` removed from onset set — now coda-exclusive for unambiguous parsing
- Fixed wee-alloc dependency to remove `wee_alloc` from WASM build
### Fixed
- `Hce::new(None, Sealed/Open)` now panics instead of silently using zero-key
- `build_cipher(None, Feistel8)` now panics instead of using public fixed key
- WASM `withCipher("feistel", None)` returns error instead of using public key
- `vnuc_str_for` now uses level-specific vowel tables (previously ignored level parameter)
- Generated parser tables now come from each pack JSON, not a single shared source

### Security
- Sealed/Open modes now require a non-empty key (panic on None)
- Custom cipher factory rejects None key for Feistel mode
- WASM binding validates key presence before Feistel cipher construction

## 0.1.0

Initial release.
