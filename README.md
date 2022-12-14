# MCAN

> The M\_CAN is a CAN IP module that can be realized as a standalone
> device, as part of an ASIC or on an FPGA. It performs communication
> according to ISO11898-1:2015. It supports Classical CAN and CAN FD
> (CAN with Flexible Data-rate). Additional transceiver hardware is
> required for connection to the CAN physical layer. The message
> storage is intended to be a single or dual-ported Message RAM
> outside of the module. It is connected to the M\_CAN via the Generic
> Master Interface. Depending on the chosen integration, multiple M\_CAN
> controllers may share the same Message RAM. The Host CPU is connected
> via the 32-bit Generic Interface.[^1]

[^1]: [Bosch M\_CAN](https://www.bosch-semiconductors.com/ip-modules/can-ip-modules/m-can/)

## Repository content

This repository provides two crates:

### mcan

It contains a platform-agnostic HAL for MCAN, with support for
- classical CAN and CAN FD with bitrate switching
- message transmission using dedicated buffers, FIFO and priority queue
- message reception using dedicated buffers and two FIFOs
- message transmission cancellation
- filter settings

### mcan-core

It contains traits meant to be implemented by target HALs in order
to resolve platform-specific details

## Acknowledgement

MCAN HAL was developed by [Grepit AB](https://grepit.se) and financed
by [Volvo Cars Corporation](https://github.com/volvo-cars)

![VCC Logo](https://avatars.githubusercontent.com/u/31673679?s=100)

This project is not affiliated with `Robert Bosch GmbH` and as such
should be considered unofficial.

## Authors

- [Nils Fitinghoff](https://github.com/vccnfitingh)
- [Gabriel Górski](https://github.com/Glaeqen)

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
