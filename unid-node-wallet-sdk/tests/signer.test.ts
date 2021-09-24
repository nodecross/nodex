import { Signer, Jws, CredentialSigner } from '../src/did-unid/cipher/signer'
import { Secp256k1 as Context } from '../src/did-unid/keyring/secp256k1'
import secp256k1 from 'secp256k1'

const KID: string = 'signingKey'
const DID: string = 'did:unid:test:EiBtzgWy130lNOyO3JsHkR75YFeSgU7h4p6zYvfQxrAXeA'
const XY: Buffer = Buffer.from('04da5cd2e20091a7e030905c495241ca5ede8f1e2b2a04c3c628f0335986317c246621aed82210ed73daf5222682a4acd87f2d42a42cb1834fccd36ed3bb555092', 'hex')
const D : Buffer = Buffer.from('4c5bb19b5c17a065253be083a49930fbd91473c60f7359389e2b280b5cbafc9e', 'hex')

test('Signer - 1', async () => {
    expect(secp256k1.publicKeyVerify(Uint8Array.from(XY))).toEqual(true)
    expect(secp256k1.privateKeyVerify(Uint8Array.from(D))).toEqual(true)

    const context = new Context({
        public : XY,
        private: D,
    })

    expect(context.validatePoint()).toEqual(true)

    const payload = {
        id: 'did:self:0x0123456789012345678901234567890123456789'
    }
    const message   = JSON.stringify(payload)
    const signature = await Signer.sign(Buffer.from(message, 'utf-8'), context)
    const verified  = await Signer.verify(Buffer.from(message, 'utf-8'), signature, context)

    expect(verified).toEqual(true)
})

test('Signer - 2', async () => {
    expect(secp256k1.publicKeyVerify(Uint8Array.from(XY))).toEqual(true)
    expect(secp256k1.privateKeyVerify(Uint8Array.from(D))).toEqual(true)

    const context = new Context({
        public : XY,
        private: D,
    })

    expect(context.validatePoint()).toEqual(true)

    const myObject = {
        test: 'ok',
    }

    const jws      = await Jws.encode(myObject, context)
    const verified = await Jws.verify(myObject, jws, context)

    expect(verified).toEqual(true)
})

test('Signer - 3', async () => {
    expect(secp256k1.publicKeyVerify(Uint8Array.from(XY))).toEqual(true)
    expect(secp256k1.privateKeyVerify(Uint8Array.from(D))).toEqual(true)

    const context = new Context({
        public : XY,
        private: D,
    })

    expect(context.validatePoint()).toEqual(true)

    const myObject: { test: string } = {
        test: 'ok',
    }

    const document = await CredentialSigner.sign<{ test: string }>(myObject, {
        did    : DID,
        keyId  : KID,
        context: context,
    })
    const verified = await CredentialSigner.verify<{ test: string }>(document, {
        keyId  : KID,
        context: context,
    })

    expect(verified.isValid).toEqual(true)
    expect(verified.payload).toEqual({ test: 'ok' })
})

test('Signer - 4', async () => {
    expect(secp256k1.publicKeyVerify(Uint8Array.from(XY))).toEqual(true)
    expect(secp256k1.privateKeyVerify(Uint8Array.from(D))).toEqual(true)

    const context1 = new Context({
        public : XY,
        private: D,
    })
    const context2 = Context.fromJwk(context1.toJwk(/** includedPrivateKey = */ false))

    expect(context1.validatePoint()).toEqual(true)
    expect(context2.validatePoint()).toEqual(true)

    expect(context2.getPrivateKey()).toEqual(Buffer.from([
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 
        0x0, 0x0,
    ]))

    const myObject: { test: string } = {
        test: 'ok',
    }

    const document = await CredentialSigner.sign<{ test: string }>(myObject, {
        did    : DID,
        keyId  : KID,
        context: context1,
    })
    const verified = await CredentialSigner.verify<{ test: string }>(document, {
        keyId  : KID,
        context: context2,
    })

    expect(verified.isValid).toEqual(true)
    expect(verified.payload).toEqual({ test: 'ok' })
})