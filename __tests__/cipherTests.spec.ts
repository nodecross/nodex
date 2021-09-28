import init, { Cipher as CipherRust } from '../pkg/cipher_lib.js';
import crypto from 'crypto';

const IV_LENGTH: number = 16 // 128 Bit
const SALT_LENGTH: number = 32 // 256 Bit

describe("RustCipher Test", () => {

  test("RustCipher#enc/dec - 1", () => {
    const data: string = "hello";

    const secret: string = "secret";

    const enc = CipherRust.encrypt(data, secret);
    const dec = CipherRust.decrypt(enc, secret);

    expect(dec).toEqual(data);
  });

  test('RustCipher#enc/dec - 2', () => {
    const data: string = JSON.stringify({
      hello0: 'hellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohello',
      hello1: 'hellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohello',
      hello2: 'hellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohello',
      hello3: 'hellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohello',
      hello4: 'hellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohellohello',
    })
    const secret: string = 'secret'

    const enc = CipherRust.encrypt(data, secret)
    const dec = CipherRust.decrypt(enc, secret)

    expect(dec).toEqual(data)
  })

  test('RustCipher#enc/dec - 3', () => {
    const data: string = 'hello'
    const secret1: string = 'secret1'
    const secret2: string = 'secret2'

    expect(() => {
      const enc = CipherRust.encrypt(data, secret1)
      CipherRust.decrypt(enc, secret2)
    }).toThrow()
  })

  test('RustCipher#dec - 1', () => {
    const data: string = crypto.randomBytes((SALT_LENGTH + IV_LENGTH) - 1).toString('utf8')
    const secret: string = 'secret'

    expect(() => {
      CipherRust.decrypt(data, secret)
    }).toThrow()
  })

  test('RustCipher#dec - 2', () => {
    let enc_data: string = "d21mUXlyTWtuZEVwQ2pBMXRQT2VxTlVLbGMyVzV2Qmiodr/+/GkoY8WUiz17vj8BUWJvQklTWERaaE94SldybQ=="

    const secret: string = "secret"

    const dec = CipherRust.decrypt(enc_data, secret)

    expect(dec).toEqual("Hello world!")
  });
});



