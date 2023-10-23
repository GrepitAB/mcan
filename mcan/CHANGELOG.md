# Changelog

Tagging in git follows a pattern: `mcan/<version>`.

## [Unreleased]
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
