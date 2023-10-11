import { post } from './sock.js'

(async () => {
  const json = await post('/transfer', {
    destinations: [ 'did:nodex:test:EiBprXreMiba4loyl3psXm0RsECdtlCiQIjM8G9BtdQplA' ],
    messages: [ {
      string: 'value',
      number: 1,
      boolean: true,
      array: [],
      map: {}
    }, {
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
  })

  console.log(json)
})()
