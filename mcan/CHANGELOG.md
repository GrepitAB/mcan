# Changelog

Tagging in git follows a pattern: `mcan/<version>`.

## [Unreleased]

### Added
- Add method to query the Tx FIFO/Queue full status.

## [0.6.0] - 2025-02-04

### Added
- Add method to check if bus is dominant (#51)

## [0.5.0] - 2024-03-04

### Added
- Add safe way to shutdown the bus when actively transmitting/receiving (#45)
- Add method to finalize configuration into initialization mode (#47)

### Changed
- *Breaking* Update the register mappings with svd2rust 0.30.2 and form 0.10.0 (#46)

## [0.4.0] - 2023-10-24

### Added
- Add `Can::aux::initialization_mode` (#41)

### Changed
- Fix some issues with watermark sizes for Rx FIFOs and Tx Event FIFO (#43)
- Adhere to `filter_map_bool_then` clippy lint (#42)

## [0.3.0] - 2023-04-24

### Added
- Add `Default` implementation for `OwnedInterruptSet` (#37)
- Add InterruptSet::is_empty (#36)
- Expand interrupt API (#34)

### Changed
- *Breaking:* Refine the interrupt system (#39)
- Make `OwnedInterruptSet` `must_use` (#33)

## [0.2.0] - 2022-12-15

_Initial tracked release._
