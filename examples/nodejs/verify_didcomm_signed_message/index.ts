import * as os from 'os'
import * as path from 'path'
import got from 'got'

(async () => {
    const base = `unix:${ path.join(os.homedir(), '.unid/run/unid.sock') }`
    const json = await got.post([ base, '/internal/didcomm/signed-messages/verify' ].join(':'), {
        enableUnixSockets: true,
        json: {
            message: {
                "payload": "eyJ0eXAiOiJhcHBsaWNhdGlvbi9kaWRjb21tLXBsYWluK2pzb24iLCJpZCI6IjEwYzlhMjg1LTZiZGQtNDE0MS04NWVjLTA3MDc1NjViMDg1ZCIsInR5cGUiOiJKV00iLCJ0byI6WyJkaWQ6dW5pZDp0ZXN0OkVpQnByWHJlTWliYTRsb3lsM3BzWG0wUnNFQ2R0bENpUUlqTThHOUJ0ZFFwbEEiXSwiZnJvbSI6ImRpZDp1bmlkOnRlc3Q6RWlCcHJYcmVNaWJhNGxveWwzcHNYbTBSc0VDZHRsQ2lRSWpNOEc5QnRkUXBsQSIsImJvZHkiOnsiQGNvbnRleHQiOlsiaHR0cHM6Ly93d3cudzMub3JnLzIwMTgvY3JlZGVudGlhbHMvdjEiXSwiY3JlZGVudGlhbFN1YmplY3QiOnsiY29udGFpbmVyIjp7ImFycmF5IjpbXSwiYm9vbGVhbiI6dHJ1ZSwibWFwIjp7fSwibnVtYmVyIjoxLCJzdHJpbmciOiJ2YWx1ZSJ9fSwiaXNzdWFuY2VEYXRlIjoiMjAyMy0wMS0xMlQxNDowMjowMS4yNDIxOTYrMDA6MDAiLCJpc3N1ZXIiOnsiaWQiOiJkaWQ6dW5pZDp0ZXN0OkVpQnByWHJlTWliYTRsb3lsM3BzWG0wUnNFQ2R0bENpUUlqTThHOUJ0ZFFwbEEifSwicHJvb2YiOnsiY2hhbGxlbmdlIjpudWxsLCJjb250cm9sbGVyIjpudWxsLCJjcmVhdGVkIjoiMjAyMy0wMS0xMlQxNDowMjowMS4yNDIyMjYrMDA6MDAiLCJkb21haW4iOm51bGwsImp3cyI6ImV5SmhiR2NpT2lKRlV6STFOa3NpTENKaU5qUWlPbVpoYkhObExDSmpjbWwwSWpwYkltSTJOQ0pkZlEuLklpWC1KbVRZdzdwb2tYZ285anlPYjBqM3RfQmQtMnd6aVpOMGYyWE1rQ000dm5NeDkyOWFQLVpTc0NnamJKVzdfc0lyZ2xxUHNBemNxNFV1cDFnUjN3IiwicHJvb2ZQdXJwb3NlIjoiYXV0aGVudGljYXRpb24iLCJ0eXBlIjoiRWNkc2FTZWNwMjU2azFTaWduYXR1cmUyMDE5IiwidmVyaWZpY2F0aW9uTWV0aG9kIjoiZGlkOnVuaWQ6dGVzdDpFaUJwclhyZU1pYmE0bG95bDNwc1htMFJzRUNkdGxDaVFJak04RzlCdGRRcGxBI3NpZ25pbmdLZXkifSwidHlwZSI6WyJWZXJpZmlhYmxlQ3JlZGVudGlhbCJdfX0",
                "signatures": [
                    {
                        "protected": "eyJ0eXAiOiJhcHBsaWNhdGlvbi9kaWRjb21tLXNpZ25lZCtqc29uIiwiYWxnIjoiRVMyNTZLIn0",
                        "signature": "b0fZEeD1vbIS1eSDILtEH3rI4j1eCGS03Gu-q70GL7Q46q20w_zYLF0ZbNdeV6cfN9uQEbyZdOqCJcXqYVnhGQ"
                    }
                ]
            },
        },
    }).json()

    console.log(JSON.stringify(json, null, 4))
})()
