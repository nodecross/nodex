import { get } from './sock.js'

(async () => {
  const json = await get('/identifiers/did:nodex:test:EiD_ZSrS4E4FZruAIJnMt1KjvH1HvwCRYdnIzYpQr4vsuQ')

  console.log(json)
})()
