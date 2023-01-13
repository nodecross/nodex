import * as os from 'os'
import * as path from 'path'
import got from 'got'

(async () => {
    const base = `unix:${ path.join(os.homedir(), '.unid/run/unid.sock') }`
    const json = await got.post([ base, '/transfer' ].join(':'), {
        enableUnixSockets: true,
        json: {
            destinations: [ 'did:unid:test:EiD_ZSrS4E4FZruAIJnMt1KjvH1HvwCRYdnIzYpQr4vsuQ' ],
            messages: [ {
                string: 'value',
                number: 1,
                boolean: true,
                array: [],
                map: {}
            } ],
            metadata: {
                string: 'value',
                number: 1,
                boolean: true,
                array: [],
                map: {}
            },
        },
    }).json()

    console.log(JSON.stringify(json, null, 4))
})()
