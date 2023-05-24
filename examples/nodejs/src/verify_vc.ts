import { post } from './sock.js'

(async () => {
  const json = await post('/internal/verifiable-credentials/verify', {
    message: {
      "@context": [
        "https://www.w3.org/2018/credentials/v1"
      ],
      "credentialSubject": {
        "container": {
          "array": [],
          "boolean": true,
          "map": {},
          "number": 1,
          "string": "value"
        }
      },
      "issuanceDate": "2023-05-23T10:39:07.130753+00:00",
      "issuer": {
        "id": "did:nodex:test:EiB-ak3q__Y94I7suelC2h3h03YIWgKSE3YwS9cV1WQA1A"
      },
      "proof": {
        "challenge": null,
        "controller": null,
        "created": "2023-05-23T10:39:07.130753+00:00",
        "domain": null,
        "jws": "eyJhbGciOiJFUzI1NksiLCJiNjQiOmZhbHNlLCJjcml0IjpbImI2NCJdfQ..RCp1sTu409PTjJlLqhVbVHlkuY4HvtZP6ecIxCAXhSc__3_r1h9qxPVyVrapITrY1bcSOayLzX2392utgh2BzQ",
        "proofPurpose": "authentication",
        "type": "EcdsaSecp256k1Signature2019",
        "verificationMethod": "did:nodex:test:EiB-ak3q__Y94I7suelC2h3h03YIWgKSE3YwS9cV1WQA1A#signingKey"
      },
      "type": [
        "VerifiableCredential"
      ]
    },
  })

  console.log(json)
})()
