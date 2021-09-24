import crypto from 'crypto'
import CryptoJS from 'crypto-js'

test('utils/Multihash - 1', async () => {
    const message = 'hello'
    const digest  = '2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824'

    const cipher1 = crypto.createHash('sha256')
    const cipher2 = CryptoJS.SHA256(message)

    cipher1.update(message)

    expect(digest).toEqual(cipher1.digest('hex'))
    expect(digest).toEqual(cipher2.toString(CryptoJS.enc.Hex))
})

test('utils/Multihash - 2', async () => {
    const message = Buffer.from([ 0x00, 0x01, 0x02, 0x03, 0x04, 0x05 ])
    const digest  = '17e88db187afd62c16e5debf3e6527cd006bc012bc90b51a810cd80c2d511f43'

    const cipher1 = crypto.createHash('sha256')
    const cipher2 = CryptoJS.SHA256(CryptoJS.enc.Hex.parse(message.toString('hex')))

    cipher1.update(message)

    expect(digest).toEqual(cipher1.digest('hex'))
    expect(digest).toEqual(cipher2.toString(CryptoJS.enc.Hex))
})