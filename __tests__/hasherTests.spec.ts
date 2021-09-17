import init, { Hasher as HasherRust } from '../pkg/cipher_lib.js';
import crypto from 'crypto';

describe("RustHasher Test", () => {
  test('Hasher - 1', async () => {
    const data = { a: 'hello', b: 'world' }
    const text: string = JSON.stringify(data)
    const secret: string = 'secret123'

    const digested = HasherRust.digest(
      text,
      secret
    )
    const verified = HasherRust.verify(
      text,
      digested,
      secret
    )

    expect(verified).toEqual(true)
    expect(digested).toEqual('OM+bDTbUVutMpKxggbcI5HvVJU+1XO1O4IM7jzE69oYKpICBbLU/PWe0ZC8icnk6O3/TdkVajVNmlpct6oRNkQ==')
  });
})

