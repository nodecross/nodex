import { post } from './sock.js'

(async () => {
  const json = await post('/internal/version/get', {})

  console.log(json)
})()
