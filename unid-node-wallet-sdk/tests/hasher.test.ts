import { Hasher } from '../src/did-unid/cipher/hasher'

test('Hasher - 1', async () => {
    const data   = { a: 'hello', b: 'world' }
    const text   = JSON.stringify(data)
    const secret = 'secret123'

    const digest = Hasher.digest(
        Buffer.from(text, 'utf-8'),
        Buffer.from(secret, 'utf-8')
    )
    const verified = Hasher.verify(
        Buffer.from(text, 'utf-8'),
        Buffer.from(digest, 'hex'),
        Buffer.from(secret, 'utf-8')
    )

    expect(verified).toEqual(true)
    expect(digest).toEqual('38cf9b0d36d456eb4ca4ac6081b708e47bd5254fb55ced4ee0833b8f313af6860aa480816cb53f3d67b4642f2272793a3b7fd376455a8d536696972dea844d91')
})