import { post } from './sock.js'

(async () => {
  const json = await post('/internal/network', {
    message: {}
  })

  console.log(json)
})()