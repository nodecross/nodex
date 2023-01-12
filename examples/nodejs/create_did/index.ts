import * as os from 'os'
import * as path from 'path'
import got from 'got'

(async () => {
    const base = `unix:${ path.join(os.homedir(), '.unid/run/unid.sock') }`
    const json = await got.post([ base, '/identifiers' ].join(':'), {
        enableUnixSockets: true,
        json: {},
    }).json()

    console.log(JSON.stringify(json, null, 4))
})()
