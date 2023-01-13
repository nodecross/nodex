import * as os from 'os'
import * as path from 'path'
import got from 'got'

(async () => {
    const base = `unix:${ path.join(os.homedir(), '.unid/run/unid.sock') }`
    const json = await got.post([ base, '/internal/verifiable-credentials/verify' ].join(':'), {
        enableUnixSockets: true,
        json: {
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
                "issuanceDate": "2023-01-12T14:02:44.193299+00:00",
                "issuer": {
                    "id": "did:unid:test:EiBprXreMiba4loyl3psXm0RsECdtlCiQIjM8G9BtdQplA"
                },
                "proof": {
                    "challenge": null,
                    "controller": null,
                    "created": "2023-01-12T14:02:44.193327+00:00",
                    "domain": null,
                    "jws": "eyJhbGciOiJFUzI1NksiLCJiNjQiOmZhbHNlLCJjcml0IjpbImI2NCJdfQ..m-49v1qZ1Dv4qMuDe9p_4RaSlNESuLL3ONnH_gcyOfZYOys7kFzP_mK5mORM1eR3dF1oojST9BPv6pbV7pWPnQ",
                    "proofPurpose": "authentication",
                    "type": "EcdsaSecp256k1Signature2019",
                    "verificationMethod": "did:unid:test:EiBprXreMiba4loyl3psXm0RsECdtlCiQIjM8G9BtdQplA#signingKey"
                },
                "type": [
                    "VerifiableCredential"
                ]
            },
        },
    }).json()

    console.log(JSON.stringify(json, null, 4))
})()
