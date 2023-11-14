import { post } from './sock.js'

(async () => {
  const json = await post('/internal/didcomm/signed-messages', {
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
