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

<p align="center">
  <img src="images/e2e_secure_channel.svg" alt="e2e secure channel" width="80%" />
</p>

## Overview

By abstracting every device and cloud as globally unique endpoints and building an E2E secure channel, each endpoint can send encrypted messages regardless of the network topology or routing hops.

<p align="center">
  <img src="images/iot_building_blocks.svg" alt="UNiD Overview" width="80%" />
</p>

---
### Required Software:

- **Wasm-pack: Build rust-generated WebAssembly and make it work with JavaScript, either in the browser or with Node.js.**

---
### Steps:
1. Install wasm-pack

```bash
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```
  for more info: https://rustwasm.github.io/wasm-pack/installer/

2. Build the rust code into wasm code

  -  For web:

```bash
wasm-pack build --target web
```

  - For nodejs:

```bash
wasm-pack build --target nodejs
```

After you build for specific target, you can import the exported classes from 'pkg/cipher_lib.js'

----
### Exported Classes and included methods:
**1. Cipher**

  **1.1 encrypt**

  encrypt given utf-8 text input with a secure utf-8 key input into a base64 encrypted string.

```jsx
  const encrypted = Cipher.encrypt("hello", "secret");
  console.log(encrypted); //gives aes encrypted base64 string
```
  **1.2 decrypt**

  decrypt given base64 encrypted string input with the correct secure utf-8 key input to get the original utf-8 text

```jsx
  const decrypted = Cipher.decrypt(encrypted, "secret"); //encrypted is base64 encrypted string from previous encryption
  console.log(decrypted); //gives the original text i.e. "hello"
```


**2. Hasher**

  **2.1 digest**

  create base64 hash string from a given utf-8 text input and a secure utf-8 key input.

```javascript
  const hashed = Hasher.digest("hello", "secret");
  console.log(hashed); //gives hmacsha512 hashed base64 string
```
  **2.2 verify**

  verify if the given base64 hash string is the correct hash output for the given pair of utf-8 text input and secure utf-8 key input.

```javascript
  const isCorrectHashed = Hasher.verify("hello", hashed, "secret"); // hashed is base64 hashed string from previous hashing
  console.log(isCorrectHashed); // returns true
```
**3. Signer**

  **3.1 sign**

  sign a given utf-8 string input with a base64 string secret key and get a base64 string ecdsa signature.

  **3.2 verify**

  verify if the given base64 string signature is the correct signature for the given pair of  utf-8 string input and a base64 string public key.
  

**4. Jws**

  **4.1 encode**
  
  encode a given object input with a base64 string secret key and get a base64 string ecdsa signature.
  
  **4.2 verify**

  verify if the given base64 string signature is the correct signature for the given pair of object input and a base64 string public key.

**5. Credential Signer**

  **5.1 sign**

  encode a given object input with signing suite object and get the signed object.

  **5.2 verify**

  verify if the given signed object is valid or not.

---

### Unit Testing
There are two types of unit test depending on the target:
1. Native test (excluding wasmbindgen's test)
```
cargo test
```
2. Wasm-pack test (only wasmbindgen's test)
```
wasm-pack test --node
```

## Changelog

[CHANGELOG](CHANGELOG.md)


## License

[Apache License 2.0](LICENSE)
