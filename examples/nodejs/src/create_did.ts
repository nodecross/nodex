import { post } from './sock.js'

(async () => {
  const json = await post('/identifiers', {})

  console.log(json)
})()
