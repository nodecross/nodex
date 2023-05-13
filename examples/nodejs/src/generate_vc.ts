import { call } from './sock.js'

(async () => {
    const json = await call('post', '/internal/verifiable-credentials', {
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
