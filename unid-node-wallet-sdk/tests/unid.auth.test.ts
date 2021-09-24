import { MongoClient } from 'mongodb'
import {
    UNiD,
    KeyRingType,
    AddressCredentialV1,
    NameCredentialV1,
    BirthDateCredentialV1,
} from '../src'
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

test('UNiD - AuthN Request - 1', async () => {
    const DID     = await UNiD.createDid(KeyRingType.Mnemonic)
    const request = await DID.generateAuthenticationRequest({
        required: [ 'NameCredentialV1' ],
        optional: [ 'BirthDateCredentialV1' ],
    })
    const claims = await UNiD.validateAuthenticationRequest(request)

    expect(0 <= claims.required.indexOf('NameCredentialV1')).toEqual(true)
    expect(0 <= claims.optional.indexOf('BirthDateCredentialV1')).toEqual(true)
})

test('UNiD - AuthN Response - 1', async () => {
    const DID = await UNiD.createDid(KeyRingType.Mnemonic)
    const vc1 = await DID.createCredential(
        new AddressCredentialV1({
            '@id'  : DID.getIdentifier(),
            '@type': 'AddressPerson',
            address: {
                '@type': 'PostalAddress',
            }
        })
    )
    const vc2 = await DID.createCredential(
        new NameCredentialV1({
            '@id'  : DID.getIdentifier(),
            '@type': 'NamePerson',
            name      : 'name',
            familyName: 'familyName',
            givenName : 'givenName',
        })
    )
    const vc3 = await DID.createCredential(
        new BirthDateCredentialV1({
            '@id'  : DID.getIdentifier(),
            '@type': 'BirthDatePerson',
            birthDate: '1900-01-01'
        })
    )

    const vp1 = await DID.createPresentation([ vc1, vc3 ])
    const vp2 = await DID.createPresentation([ vc2, vc3 ])
    const vp3 = await DID.createPresentation([ vc2 ])

    const response1 = await DID.generateAuthenticationResponse(vp1)
    const response2 = await DID.generateAuthenticationResponse(vp2)
    const response3 = await DID.generateAuthenticationResponse(vp3)

    await expect(async () => {
        await UNiD.validateAuthenticationResponse(response1, {
            required: [ 'NameCredentialV1' ],
            optional: [ 'BirthDateCredentialV1' ],
        })
    }).rejects.toThrow()

    const verifiedResponse1 = await UNiD.validateAuthenticationResponse(response2, {
        required: [ 'NameCredentialV1' ],
        optional: [ 'BirthDateCredentialV1' ],
    })

    const name1      = NameCredentialV1.select(verifiedResponse1.payload)
    const birthDate1 = BirthDateCredentialV1.select(verifiedResponse1.payload)

    expect(name1).not.toBeUndefined()
    expect(birthDate1).not.toBeUndefined()

    if (name1 !== undefined) {
        const verifiedName1 = await UNiD.verifyCredential(name1)

        expect(verifiedName1.isValid).toEqual(true)
        expect(verifiedName1.payload).toEqual(lodash.omit(vc2, [ 'proof' ]))
    }

    if (birthDate1 !== undefined) {
        const verifiedBirthDate1 = await UNiD.verifyCredential(birthDate1)

        expect(verifiedBirthDate1.isValid).toEqual(true)
        expect(verifiedBirthDate1.payload).toEqual(lodash.omit(vc3, [ 'proof' ]))
    }

    const verifiedResponse2 = await UNiD.validateAuthenticationResponse(response3, {
        required: [ 'NameCredentialV1' ],
        optional: [ 'BirthDateCredentialV1' ],
    })

    const name2      = NameCredentialV1.select(verifiedResponse2.payload)
    const birthDate2 = BirthDateCredentialV1.select(verifiedResponse2.payload)

    expect(name2).not.toBeUndefined()
    expect(birthDate2).toBeUndefined()

    if (name2 !== undefined) {
        const verifiedName2 = await UNiD.verifyCredential(name2)

        expect(verifiedName2.isValid).toEqual(true)
        expect(verifiedName2.payload).toEqual(lodash.omit(vc2, [ 'proof' ]))
    }
})