# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Expand interrupt API (#34)
- Add `Default` implementation for `OwnedInterruptSet` (#37)
- Add InterruptSet::is_empty (#36)

### Changed
- *Breaking:* Refine the interrupt system (#39)
- Make `OwnedInterruptSet` `must_use` (#33)

## [0.2.0] - 2022-12-15

This is a first actual release of the `mcan` crate.

- Release mcan/0.2.0 (#29)
