<p align="center">
  <img src="https://i.gyazo.com/2b3ac7a80e916ed0aee482ea269d1ca7.png" alt="logo" width="160" />
</p>

<h1 align="center" style="text-align: center;">UNiD</h1>

<p align="center">
  <a href="https://github.com/semantic-release/semantic-release">
    <img src="https://img.shields.io/badge/%20%20%F0%9F%93%A6%F0%9F%9A%80-semantic--release-e10079.svg" alt="Sematic-Release" />
  </a>
</p>

<p align="center">
  <b>An easy to integrate, adaptable decentralized identity platform.</b></br>
  <span>The best gear for our partners and the modern engineers to reimagine digital experience, and progress our digital society.</span></br>
</p>

<br />

## Table of contents

- [Introduction](#introduction)
- [Installation](#installation)
- [Configuration](#configuration)
- [Tutorial](#tutorial)
- [Changelog](#changelog)
- [License](#license)

## Introduction

The NodeJS SDK offers convenience and easy-to-use javascript modules for embedding digital wallet capabilities into your application.
It enables you to operate decentralized identifiers and verifiable credentials.

## Installation
Add the `@unid/node-wallet-sdk` dependency:

```bash
npm install --save @unid/node-wallet-sdk

# OR

yarn add @unid/node-wallet-sdk
```
The NodeJS SDK uses MongoDB as a local repository to store keyRings by default. Please install and setup MongoDB in your application.

## Configuration

Configuration should happen as early as possible in your application's lifecycle.
Once you have set up a cloud agent in [UNiD Studio](https://www.getunid.io/), you will get values required for configuration.

```typescript
import { UNiD } from '@unid/node-wallet-sdk'
import { MongoClient } from 'mongodb'

const uri = 'mongodb://username:password@localhost:27017'
const mongoClient = new MongoClient(uri, {
    useUnifiedTopology: true,
})

(async () => {
    // Connect to your mongodb
    await mongoClient.connect()
    
    // Initialize UNiD instance
    UNiD.init({
        clientId     : 'client_id',
        clientSecret : 'client_secret',
        encryptionKey: 'encryption_key',
        envNetwork   : 'testnet',
        localStorage : mongoClient
    })
})()
```

| Values          | Description                                                                                                                       |
| --------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| `clientId`      | It is associated with each cloud agent for a tenant. A string consisting of 64 characters that can be retrieved from UNiD Studio. |
| `clientSecret`  | A string consisting of 64 characters that can be retrieved from UNiD Studio. It is paired with the `clientId `.                   |
| `encryptionKey` | A string used to encrypt keyRings (digital wallet) with AES-256-CBC algorithm and store them in MongoDB.                          |
| `envNetwork`    | The DPKI network to which the DID refers.                                                                                         |
| `localStorage`  | A connection instance to MongoDB that must be initialized and instantiated outside of the UNiD libraries to MongoDB.              |


## Tutorial

First create new DID in your application.

### Create DID

```typescript
import { UNiD, KeyRingType } from '@unid/node-wallet-sdk'

(async () => {
    try{
        const DID = await UNiD.createDidDocument(
            KeyRingType.Mnemonic,
            { length: 24 }
        )
        console.log('complete generating a DID:', DID.getIdentifier())
    } catch (err) {
        console.error('ERROR:', err)
    }
})()
```

For more methods and how to use them, please refer to [UNiD Documentation](https://docs.getunid.io/).

## Changelog

[CHANGELOG](CHANGELOG.md)

## License

[Apache License 2.0](LICENSE)
