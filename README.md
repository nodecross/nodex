<p align="center">
  <img src="images/unid_logo_github.svg" alt="UNiD Logo" width="160" />
</p>

<h1 align="center">
  UNiD
</h1>

<p align="center">
  <a href="https://github.com/getunid/unid/actions/workflows/release-pipeline.yml" target="_blank">
    <img src="https://github.com/getunid/unid/actions/workflows/release-pipeline.yml/badge.svg?branch=main" alt="Release Pipeline" />
  </a>
  <a href="https://coveralls.io/github/getunid/unid" target="_blank">
    <img src="https://coveralls.io/repos/github/getunid/unid/badge.svg" alt="Coverage Status" />
  </a>
  <a href="https://crates.io/crates/unid" target="_blank">
    <img src="https://img.shields.io/crates/v/unid.svg" alt="UNiD" />
  </a>
  <a href="https://github.com/semantic-release/semantic-release" target="_blank">
    <img src="https://img.shields.io/badge/semantic--release-rust-B7410E?logo=semantic-release" alt="Sematic Release" />
  </a>
  <br />
  Endpoint Security Infrastructure
</p>

## Introduction

UNiD is an endpoint security infrastructure that consists of;

- **UNiD EDGE** is an embedded middleware in connected devices
- **UNiD HUB** is a message broker between connected devices and the cloud
- **UNiD Network** is a public, permissionless, decentralized identity network based on blockchain-agnostic sidtree protocol on top of bitcoin to support DIDs and DPKI (decentralized public key infrastructure)

By simply integrating UNiD EDGE into your devices, you can protect the endpoint devices, establish an end-to-end secure channel with UNiD HUB, and communicate bidirectionally with various cloud services while ensuring device authenticity, data integrity, and privacy.

<p align="center">
  <img src="images/figure2.svg" alt="e2e security infrastructure" width="80%" />
</p>

UNiD EDGE generates multiple key pairs from a hardware-derived true random number generator (TRNG) within the RoT secure processing environment, and generates a payload for registering to a blockchain-based decentralized PKI (DPKI) network to create a [decentralized identifier (DID)](https://www.w3.org/TR/did-core/) and the relevant DID document including the public key information. Anyone can obtain the corresponding device’s public key from the network to authenticate the device and verify the digitally signed data.

UNiD EDGE consists of;

- **RoT Wrapper**: This component supports TrustZone (Arm Cortex-M) and hardware security modules (supported MCUs), making it easier to use the RoT secure processing environment (SPE).
- **Key Management**: This component supports the device’s cryptographic key operations (create, read, update, delete) in the SPE.
- **Device IAM**: This component supports management of device identities, credentials, and security policies for device’s authentication and authorization.
- **E2E Secure Socket**: This component supports to establish end-to-end authenticated channels with UNiD HUB for secure bi-directional communications.

<p align="center">
  <img src="images/figure3.svg" alt="unid edge architecture" />
</p>

This identity-first, end-to-end approach can abstract the complexity of security infrastructure and introduces advanced, scalable endpoint security to connected systems. UNiD platform is designed to make the endpoint security easily available and free all developers from the heavy burden of building the complex security infrastructure for each product. For more information, see [official documentation](https://docs.getunid.io/unid_edge/index.html). 

## Quick Start

- [Integration Guide](https://docs.getunid.io/integration/index.html)
- [Tutorial](https://docs.getunid.io/tutorial/ubuntu-nodejs/index.html)

## Developer's Document

- UNiD Official Website<br />
  https://www.getunid.io/
  
- UNiD Developers Portal<br />
  https://docs.getunid.io/

## Changelog

[CHANGELOG](CHANGELOG.md)

## Security

[SECURITY](SECURITY.md)

## License

[Apache License 2.0](LICENSE)
