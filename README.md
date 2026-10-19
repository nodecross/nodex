<p align="center">
  <img src="images/nodex-logo.svg" alt="NodeX Logo" width="130" />
</p>

<h2 align="center">
  NodeX - Verifiable Identity and Secure Messaging for Every Device
</h2>

NodeX is a commercial platform that gives every device — IoT, edge, and on-premise servers — an identity that others can verify for themselves. Built on [decentralized identifiers (DIDs)](https://www.w3.org/TR/did-core/) with verifiable history, NodeX shifts the basis of trust from "trust the server" to "verify the record".

This repository distributes the NodeX Agent binaries through its [Releases](https://github.com/nodecross/nodex/releases) page; the source code is not published here. The agent handles device identity, provisioning, mutual verification, and secure messaging.

[Explore NodeX Docs](https://docs.nodex.inc/manual)

## Our Approach to Trust

- **Trust Anchor** — A device's identifier is derived from its own history, making the identifier itself the starting point for verification.
- **Root of Trust** — Verification runs on the device itself, without delegating it to anyone else.
- **Trustless Verification** — Every change to an identity is recorded as a signed, tamper-evident entry, verifiable by anyone regardless of where it is retrieved from.

## Features

- DID-based device identity with verifiable history
- Remote key rotation without changing the identity, with every rotation recorded in its verifiable history
- Provisioning without reliance on certificate authorities
- Mutual verification between devices and the NodeX cloud
- End-to-end encrypted messaging over [DIDComm](https://github.com/decentralized-identity/didcomm-messaging)
- Policy-based access control for messaging
- Support for on-premise and offline environments
- Supported operating systems: Linux (x86_64, aarch64), macOS (Apple silicon), and Windows (x86_64)

## Developer's Document

- NodeX Official Website<br />
  https://nodex.inc

## Support

Issues are not tracked in this repository. For bug reports, feature requests, and licensing inquiries, contact contact@nodex.inc or refer to the [NodeX Developers Portal](https://docs.nodex.inc/manual).

## License

Proprietary. Use of the NodeX Agent binaries requires a separate written license agreement with NodeX, Inc. See [LICENSE](LICENSE) for the full terms, including the treatment of third-party components. Third-party license notices are attached to each release as `THIRD_PARTY_NOTICES.md`.
