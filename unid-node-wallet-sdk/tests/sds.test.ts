import { MongoClient } from 'mongodb'
import { UNiD, KeyRingType, AddressCredentialV1 } from '../src'
import lodash from 'lodash'

const client = new MongoClient('mongodb://root:password@localhost:27018', {
    useUnifiedTopology: true,
})

beforeAll(() => {
    return new Promise(async (resolve, reject) => {
        await client.connect()

        UNiD.init({
            clientId     : '718AC7F1006ECA672E1D1BE9B4666D3EEFD6C2805F9200328502853AFDFD3219',
            clientSecret : '670E362C65183C3850A8FC6E0ED26EC72FDAE67846FDCE1904F604C8E4757273',
            encryptionKey: '1AFFD4C6096D0EF4344E963612229DBF786BBC23C60611093FA9149C0E815E68',
            localStorage : client,
        })

        return resolve(true)
    })
})

afterAll(() => {
    return new Promise(async (resolve, reject) => {
        if (client.isConnected()) {
            await client.close()
        }

        return resolve(true)
    })
})

test('SDS - 1', async () => {
    const DID      = await UNiD.createDid(KeyRingType.Mnemonic)
    const signedVC = await DID.createCredential(
        new AddressCredentialV1({
            '@id'  : DID.getIdentifier(),
            '@type': 'AddressPerson',
            address: {
                '@type': 'PostalAddress',
                streetAddress: '日本橋',
            },
        })
    )

    const signedVP   = await DID.createPresentation([ signedVC ])
    const verifiedVC = await UNiD.verifyCredential(signedVC)
    const verifiedVP = await UNiD.verifyPresentation(signedVP)

    expect(verifiedVC.isValid).toEqual(true)
    expect(verifiedVP.isValid).toEqual(true)

    const filterd = AddressCredentialV1.select(verifiedVP.payload)

    expect(filterd).not.toBeUndefined()

    if (filterd) {
        const address = await UNiD.verifyCredential(filterd)

        expect(address.isValid).toEqual(true)

        await DID.postCredential(address)

        const cred = await DID.getCredential({
            type: 'AddressCredentialV1',
        })

        expect(cred).not.toBeUndefined()

        if (cred) {
            expect(cred.isValid).toEqual(true)
            expect(cred.payload).toEqual(lodash.omit(signedVC, [ 'proof' ]))
        }
    }

    const creds = await DID.getCredentials({
        type: 'AddressCredentialV1',
    })

    expect(creds.length).toEqual(1)

    creds.forEach((cred) => {
        expect(cred.isValid).toEqual(true)
        expect(cred.payload).toEqual(lodash.omit(signedVC, [ 'proof' ]))
    })
})