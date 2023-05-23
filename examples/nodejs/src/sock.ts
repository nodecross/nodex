import * as os from 'os'
import * as path from 'path'
import got from 'got'

const base = `unix:${ path.join(os.homedir(), '.nodex/run/nodex.sock') }`

const call = async (m: string, p: string, j: any) => {
  let json
  switch (m) {
    case 'get':
      json = await got.get([ base, p ].join(':'), {
        enableUnixSockets: true,
      }).json()
      break;
    case 'post':
      json = await got.post([ base, p ].join(':'), {
        enableUnixSockets: true,
        json: j,
      }).json()
      break;
    default:
      throw new Error(`Unsupported method: ${ m }`)
  }
  return JSON.stringify(json, null, 4)
}

export const get = async (p: string) => call('get', p, null)
export const post = async (p: string, j: any) => call('post', p, j)
