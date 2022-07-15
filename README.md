<p align="center">
  <img src="images/unid_u_logo.svg" alt="UNiD Logo" width="160" />
</p>

# UNiD [![Release Pipeline](https://github.com/getunid/unid/actions/workflows/release-pipeline.yml/badge.svg?branch=main)](https://github.com/getunid/unid/actions/workflows/release-pipeline.yml) [![Coverage Status](https://coveralls.io/repos/github/getunid/unid/badge.svg)](https://coveralls.io/github/getunid/unid) [![Crates](https://img.shields.io/crates/v/unid.svg)](https://crates.io/crates/unid) [![Semantic Release](https://img.shields.io/badge/semantic--release-rust-B7410E?logo=semantic-release)](https://github.com/semantic-release/semantic-release)

UNiD is an open-source toolkit for building end-to-end secure messaging between all devices including IoT, micro services, and on-premise servers, using features of [decentralized identifiers (DIDs)](https://www.w3.org/TR/did-core/) as a basis of security and privacy. Build atop [DIDComm messaging protocol](https://github.com/decentralized-identity/didcomm-messaging) which works over any transports, it makes end-to-end messaging reliable, secure, and easy.

This repo where we develop libraries to be installed into your devices. It enables device's IDs and keys management with root of trust (RoT), automated provisioning, mutual authentication, and end-to-end secure messaging across network boundary.

[Explore UNiD Docs](https://docs.getunid.io/unid_edge/index.html)

## Features

- Device's IDs and keys management with RoT
- Automated provisioning without centralized servers and certificate authorities and intermediaries
- Mutual authentication and end-to-end secure messaging while ensuring confidentiality, integrity, and authenticity
- Attribute based access control and dynamic policy management
- Rust library with multiple language extensions for python and nodejs
- Mutiple OS support for Linux kernel x86(32bit) & x86-64(64bit), FreeRTOS ARM Cortex-M33(32bit)
- RoT extensions for TPM, ARM TrustZone, and Renesas SCE

## Install

- [Integration Guide](https://docs.getunid.io/integration/index.html)
- [Tutorial](https://docs.getunid.io/tutorial/ubuntu-nodejs/index.html)

## Developer's Document

- UNiD Official Website<br />
  https://www.getunid.io/

- UNiD Developers Portal<br />
  https://docs.getunid.io/

## Contribution

First off, thank you for considering making contributions. It's people like you that make UNiD better. There are many ways in which you can participate in the project, for example:

- File a bug report. Be sure to include information like what version of UNiD you are using, what your operating system and CPU is, and steps to recreate the bug.
- Suggest a new feature.

## Changelog

[CHANGELOG](CHANGELOG.md)

## Security

[SECURITY](SECURITY.md)

## License

[Apache License 2.0](LICENSE)
