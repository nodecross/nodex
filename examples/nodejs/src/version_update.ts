import { post } from './sock.js'

(async () => {
  const json = await post('/internal/version/update', {
    message: {
      "binary_url": "https://nilth.com/nodex-agent",
      "path": "/tmp/nodex-agent",
    }
  })

  console.log(json)
})()
