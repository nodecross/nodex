import * as os from 'os'
import * as path from 'path'
import got from 'got'

(async () => {
    const base = `unix:${ path.join(os.homedir(), '.unid/run/unid.sock') }`
    const json = await got.post([ base, '/internal/didcomm/plaintext-messages/verify' ].join(':'), {
        enableUnixSockets: true,
        json: {
            message: {
                "body": {
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
                    "issuanceDate": "2023-01-12T14:01:25.691539+00:00",
                    "issuer": {
                        "id": "did:unid:test:EiBprXreMiba4loyl3psXm0RsECdtlCiQIjM8G9BtdQplA"
                    },
                    "proof": {
                        "challenge": null,
                        "controller": null,
                        "created": "2023-01-12T14:01:25.691586+00:00",
                        "domain": null,
                        "jws": "eyJhbGciOiJFUzI1NksiLCJiNjQiOmZhbHNlLCJjcml0IjpbImI2NCJdfQ..UMOAMun3e3RDmBnmI_UD3jRrxxW6asVLhhqXaSsfpy5sgvoC7Eu1rdcYWA5BkmYiJTc0MRzjdUW1YkUxz41myg",
                        "proofPurpose": "authentication",
                        "type": "EcdsaSecp256k1Signature2019",
                        "verificationMethod": "did:unid:test:EiBprXreMiba4loyl3psXm0RsECdtlCiQIjM8G9BtdQplA#signingKey"
                    },
                    "type": [
                        "VerifiableCredential"
                    ]
                },
                "from": "did:unid:test:EiBprXreMiba4loyl3psXm0RsECdtlCiQIjM8G9BtdQplA",
                "id": "e3b29fd1-b3c6-4ed0-a69a-b632826e224e",
                "to": [
                    "did:unid:test:EiBprXreMiba4loyl3psXm0RsECdtlCiQIjM8G9BtdQplA"
                ],
                "typ": "application/didcomm-plain+json",
                "type": "JWM"
            },
        },
    }).json()

    console.log(JSON.stringify(json, null, 4))
})()
