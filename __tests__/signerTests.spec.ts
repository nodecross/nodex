import { Signer as SignerRust, Jws as JwsRust, CredentialSigner as CredentialSignerRust } from '../pkg/cipher_lib.js';
import secp256k1 from 'secp256k1';

const KID: string = 'signingKey'
const DID: string = 'did:unid:test:EiBtzgWy130lNOyO3JsHkR75YFeSgU7h4p6zYvfQxrAXeA'
const XY: Buffer = Buffer.from('04da5cd2e20091a7e030905c495241ca5ede8f1e2b2a04c3c628f0335986317c246621aed82210ed73daf5222682a4acd87f2d42a42cb1834fccd36ed3bb555092', 'hex')
const D: Buffer = Buffer.from('4c5bb19b5c17a065253be083a49930fbd91473c60f7359389e2b280b5cbafc9e', 'hex')
const suite_sign = {
  did: "did:unid:test:EiBtzgWy130lNOyO3JsHkR75YFeSgU7h4p6zYvfQxrAXeA",
  key_id: "signingKey",
  secret_key64: D.toString('base64')
}
const suite_verify = {
  key_id: "signingKey",
  pub_key64: XY.toString('base64')
}

describe("RustSigner Test", () => {
  test('Signer - 1', () => {
    expect(secp256k1.publicKeyVerify(Uint8Array.from(XY))).toEqual(true)
    expect(secp256k1.privateKeyVerify(Uint8Array.from(D))).toEqual(true)

    const payload = {
      id: 'did:self:0x0123456789012345678901234567890123456789'
    }
    const message = JSON.stringify(payload)
    const signature = SignerRust.sign(message, D.toString('base64'))
    const verified = SignerRust.verify(message, signature, XY.toString('base64'))

    expect(verified).toEqual(true)
  })

  test('Signer - 2', () => {
    expect(secp256k1.publicKeyVerify(Uint8Array.from(XY))).toEqual(true)
    expect(secp256k1.privateKeyVerify(Uint8Array.from(D))).toEqual(true)

    const myObject = {
      test: 'ok',
    }

    const jws = JwsRust.encode(myObject, D.toString('base64'))
    const verified = JwsRust.verify(myObject, jws, XY.toString('base64'))

    expect(verified).toEqual(true)
  })

  test('Signer - 3', () => {
    expect(secp256k1.publicKeyVerify(Uint8Array.from(XY))).toEqual(true)
    expect(secp256k1.privateKeyVerify(Uint8Array.from(D))).toEqual(true)


    const myObject: { test: string } = {
      test: 'ok',
    }

    const document = CredentialSignerRust.sign(myObject, suite_sign)

    const verified = CredentialSignerRust.verify(document, suite_verify)

    expect(verified.isValid).toEqual(true)
    expect(verified.payload).toEqual({ test: 'ok' })

  })

});
