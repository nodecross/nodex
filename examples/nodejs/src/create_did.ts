import { call } from './sock.js'

(async () => {
  const json = await call('post', '/identifiers', {})

  console.log(json)
})()
