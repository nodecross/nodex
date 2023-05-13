import { call } from './sock.js'

(async () => {
    const json = await call('post', '/internal/didcomm/encrypted-messages', {
        destinations: [ 'did:nodex:test:EiBprXreMiba4loyl3psXm0RsECdtlCiQIjM8G9BtdQplA' ],
        message: {
            string: 'value',
            number: 1,
            boolean: true,
            array: [],
            map: {}
        },
    })
    console.log(json)
})()
