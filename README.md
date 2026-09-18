<p align="center">
  <img src="images/nodex-logo.svg" alt="NodeX Logo" width="130" />
</p>

<h2 align="center">
  NodeX - E2E Secure Messaging Platform
</h2>

NodeX is a commercial toolkit for building end-to-end secure messaging between all devices including IoT, micro services, and on-premise servers, using features of [decentralized identifiers (DIDs)](https://www.w3.org/TR/did-core/) as a basis of security and privacy. Built atop [DIDComm messaging protocol](https://github.com/decentralized-identity/didcomm-messaging) which works over any transports, it makes end-to-end messaging reliable, secure, and easy.

This repository distributes the NodeX Agent binaries through its [Releases](https://github.com/nodecross/nodex/releases) page; the source code is not published here. The agent provides device's IDs and keys management with root of trust (RoT), automated provisioning, mutual authentication, and end-to-end secure messaging across network boundary.

[Explore NodeX Docs](https://docs.nodex.inc/manual)

## Features

- Device's IDs and keys management with RoT
- Automated provisioning without centralized servers and certificate authorities and intermediaries
- Mutual authentication and end-to-end secure messaging while ensuring confidentiality, integrity, and authenticity
- Attribute based access control and dynamic policy management
- Rust library with multiple language extensions for python and nodejs
- Mutiple OS support for Linux kernel x86(32bit) & x86-64(64bit), FreeRTOS ARM Cortex-M33(32bit)
- RoT extensions for TPM, ARM TrustZone, and Renesas SCE

## Developer's Document

- NodeX Official Website<br />
  https://nodex.inc

- NodeX Developers Portal<br />
  https://docs.nodex.inc/manual

## Support

Issues are not tracked in this repository. For bug reports, feature requests, and licensing inquiries, contact contact@nodex.inc or refer to the [NodeX Developers Portal](https://docs.nodex.inc/manual).

## License

Proprietary. Use of the NodeX Agent binaries requires a separate written license agreement with NodeX, Inc. See [LICENSE](LICENSE) for the full terms, including the treatment of third-party components.
