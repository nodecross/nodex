import init, { encrypt, decrypt } from '../pkg/cipher_lib.js';
import {Cipher, IV_LENGTH, SALT_LENGTH} from '../original_cipher/cipher';
import crypto from 'crypto';

describe("RustCipher Test", () => {

  test("RustCipher#enc/dec - 1", () => {
    const data: string = "hello";

    const secret: string = "secret";

    const enc = encrypt(data, secret);
    const dec = decrypt(enc, secret).r_ok;

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

      const enc = encrypt(data, secret)
      const dec = decrypt(enc, secret).r_ok

      expect(dec).toEqual(data)
  })

  test('RustCipher#enc/dec - 3', () => {
    const data: string    = 'hello'
    const secret1: string = 'secret1'
    const secret2: string = 'secret2'

      expect(() => {
          const enc = encrypt(data, secret1)
          decrypt(enc, secret2)
      }).toThrow()
  })

  test('RustCipher#dec - 1', () => {
    const data: string = crypto.randomBytes((SALT_LENGTH + IV_LENGTH) - 1).toString('utf8')
    const secret: string = 'secret'

    const dec = decrypt(data, secret).r_ok
    expect(dec).toEqual("")
  })

  test('RustCipher#dec - 2', () => {
    let enc_data: string = "d21mUXlyTWtuZEVwQ2pBMXRQT2VxTlVLbGMyVzV2Qmiodr/+/GkoY8WUiz17vj8BUWJvQklTWERaaE94SldybQ=="
    
    const secret: string = "secret"

    const dec = decrypt(enc_data, secret).r_ok

    expect(dec).toEqual("Hello world!")
  });
});

describe("TSCipher Test", () => {
  test('TSCipher#enc/dec - 1', async () => {
    const data: Buffer   = Buffer.from('hello', 'utf-8')
    const secret: Buffer = Buffer.from('secret', 'utf-8')
  
    const enc = await Cipher.encrypt(data, secret)
    const dec = await Cipher.decrypt(enc, secret)
  
    expect(dec).toEqual(data)
  });
  test('TSCipher#dec - 1', async () => {
    const enc_data: string = "d21mUXlyTWtuZEVwQ2pBMXRQT2VxTlVLbGMyVzV2Qmiodr/+/GkoY8WUiz17vj8BUWJvQklTWERaaE94SldybQ==" 
    
		const enc_data_buf: Buffer = Buffer.from(enc_data, 'base64')
    const secret: string = "secret"
    const secret_buf: Buffer = Buffer.from(secret, 'utf-8')

    const dec_buf = await Cipher.decrypt(enc_data_buf, secret_buf)

    expect(dec_buf).toEqual(Buffer.from("Hello world!"))
  });
});


describe("TSCipher - RustCipher conversion", () => {
  test('RustCipher#enc -> TSCipher#dec', async () => {
    const data: string = 'hello'
    const data_buf: Buffer   = Buffer.from(data, 'utf-8')
    const secret: string = 'secret'
    const secret_buf: Buffer = Buffer.from(secret, 'utf-8')
  
    const enc = encrypt(data, secret)
    const enc_buf = Buffer.from(enc, 'base64')
    console.log(enc_buf)
    const dec = await Cipher.decrypt(enc_buf, secret_buf)
  
    expect(dec).toEqual(data_buf)
  });
  test('TSCipher#enc -> RustCipher#dec', async () => {
    const data: string = 'hello'
    const data_buf: Buffer   = Buffer.from(data, 'utf-8')
    const secret: string = 'secret'
    const secret_buf: Buffer = Buffer.from(secret, 'utf-8')
    

    const enc_buf = await Cipher.encrypt(data_buf, secret_buf)
    const enc = enc_buf.toString('base64')
    const dec = decrypt(enc, secret)
  

    expect(dec.r_ok).toEqual(data)
  });
});
