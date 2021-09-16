<p align="center">
  <img src="images/unid_logo_github.svg" alt="logo" width="160" />
</p>

<h1 align="center">
  UNiD
</h1>

<p align="center">
  <span>
    <a href="https://github.com/semantic-release/semantic-release">
      <img src="https://img.shields.io/badge/%20%20%F0%9F%93%A6%F0%9F%9A%80-semantic--release-e10079.svg" alt="Sematic-Release" />
    </a>
  </span>
  <br />
  <span>
    Automate device provisioning with edge intelligence
  </span>
</p>

## Features

- Decentralized Identity, KMS, and PKI
- E2E(end-to-end) Secure Channel
- E2E Encrypted Communication
- Hardware Root of Trust Add-ons
- Cloud Add-ons for Real-time Data Flow and Processing

## Introduction

Hardware Root of Trust is the security foundation for an SoC, other semiconductor device or electronic system. The RoT contains the keys for cryptographic functions and is usually a part of the secure boot process providing the foundation for the software chain of trust. _UNiD_ is a set of libraries written by Rust that can leverage the RoT and decentralized identity technology to autonomously generate key pairs, register the credentials in a decentralized PKI, and build end-to-end secure channel for real-time data flow and processing. This capability releases developers from manual device provisioning.

<img src="images/e2e_secure_channel.svg" alt="e2e secure channel" />


## Overview

By abstracting every device and cloud as globally unique endpoints and building an E2E secure channel, each endpoint can send encrypted messages regardless of the network topology or routing hops.

<img src="images/iot_building_blocks.svg" alt="UNiD Overview" />

## Changelog

[CHANGELOG](CHANGELOG.md)


## License

[Apache License 2.0](LICENSE)
