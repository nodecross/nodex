import { post } from './sock.js'

(async () => {
  const json = await post('/internal/network', {
    message: {
      "recipient_dids": ["123", "456"],
      "hub_endpoint": "https://hub.example.com",
      "heartbeat": 123,
      "trm": "true",
    }
  })

  console.log(json)
})()