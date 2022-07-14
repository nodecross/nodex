<p align="center">
  <img src="images/unid_u_logo.svg" alt="UNiD Logo" width="160" />
</p>

# UNiD [![Release Pipeline](https://github.com/getunid/unid/actions/workflows/release-pipeline.yml/badge.svg?branch=main)](https://github.com/getunid/unid/actions/workflows/release-pipeline.yml) [![Coverage Status](https://coveralls.io/repos/github/getunid/unid/badge.svg)](https://coveralls.io/github/getunid/unid) [![Crates](https://img.shields.io/crates/v/unid.svg)](https://crates.io/crates/unid) [![Semantic Release](https://img.shields.io/badge/semantic--release-rust-B7410E?logo=semantic-release)](https://github.com/semantic-release/semantic-release)

UNiD is an open-source libraries for building end-to-end secure messaging between all devices including IoT, micro services, and on-premise servers, using features of [decentralized identifiers (DIDs)](https://www.w3.org/TR/did-core/) as a basis of security and privacy. Build atop [DIDComm messaging protocol](https://github.com/decentralized-identity/didcomm-messaging) which works over any transports, it makes end-to-end messaging reliable, secure, and easy.

This repo where we develop libraries and middleware that can be installed on any devices. It enables device's identity and keys management with root of trust, automated provisioning, mutual authentication, and end-to-end encrypted message while ensuring confidentiality, integrity, and authenticity. 

[Explore UNiD Docs](https://docs.getunid.io/unid_edge/index.html)

## Features

- Device's identity, keys, and profiles protection with root of trust (RoT)
- Automated provisioning without centralized servers and certificate authorities and manual key injection process
- Mutual authentication based on digital signature and end-to-end encrypted messaging
- ABAC & Dynamic policy management 
- Libraries for multiple language - _Rust, C, NodeJS, Python_ 
- Middleware for multiple OS - _Linux kernel x86(32bit) & x86-64(64bit), FreeRTOS ARM Cortex-M33(32bit)_ 
- RoT support - _TPM, ARM TrustZone, Renesas SCE, Azure KeyVault_

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
