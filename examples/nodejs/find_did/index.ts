import * as os from 'os'
import * as path from 'path'
import got from 'got'

(async () => {
    const base = `unix:${ path.join(os.homedir(), '.nodex/run/nodex.sock') }`
    const json = await got.get([ base, '/identifiers/did:nodex:test:EiD_ZSrS4E4FZruAIJnMt1KjvH1HvwCRYdnIzYpQr4vsuQ' ].join(':'), {
        enableUnixSockets: true,
    }).json()

    console.log(JSON.stringify(json, null, 4))
})()
