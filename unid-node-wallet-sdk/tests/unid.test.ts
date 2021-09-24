import { MongoClient } from 'mongodb'
import {
    UNiD,
    KeyRingType,
    AddressCredentialV1,
    PhoneCredentialV1,
    AlumniOfCredentialV1,
    BirthDateCredentialV1,
    ContactPointCredentialV1,
    EmailCredentialV1,
    GenderCredentialV1,
    ImageCredentialV1,
    NameCredentialV1,
    QualificationCredentialV1,
    WorksForCredentialV1,
} from '../src'

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

test('UNiD - 1', async () => {
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
    }
})

test('UNiD - 2', async () => {
    const text = '{"proof":{"type":"EcdsaSecp256k1Signature2019","proofPurpose":"authentication","created":"2021-01-20T18:53:20Z","verificationMethod":"did:unid:test:EiD66QsnHupVmjYLfgo4RLeAwnm7v175B52NvXQ3GY7TBw#signingKey","jws":"eyJhbGciOiJFUzI1NksiLCJiNjQiOmZhbHNlLCJjcml0IjpbImI2NCJdfQ..1bklxVH4VjfY0JvrKwgkRVRL5ElwqjBucah2vmhlGbd4BMSM2IhL9YTcwmdR9hJD_n8rXX21M4pOJRjCCSwR_w"},"id":"https://sds.getunid.io/api/v1","issuer":"did:unid:test:EiD66QsnHupVmjYLfgo4RLeAwnm7v175B52NvXQ3GY7TBw","issuanceDate":"2021-01-20T18:53:20Z","@context":["https://www.w3.org/2018/credentials/v1","https://docs.getunid.io/docs/2020/credentials/address"],"type":["VerifiableCredential","AddressCredentialV1"],"credentialSubject":{"@id":"did:unid:test:EiD66QsnHupVmjYLfgo4RLeAwnm7v175B52NvXQ3GY7TBw","@type":"AddressPerson","address":{"@type":"PostalAddress","streetAddress":"日本橋"}}}'
    const json = JSON.parse(text)

    expect(UNiD.isVerifiableCredential(json)).toEqual(true)
    expect(UNiD.isVerifiablePresentation(json)).toEqual(false)

    if (UNiD.isVerifiableCredential(json)) {
        const credential = await UNiD.verifyCredential(json)

        expect(credential.isValid).toEqual(true)
    }
})

test('UNiD - 3', async () => {
    const text = '{"proof":{"type":"EcdsaSecp256k1Signature2019","proofPurpose":"authentication","created":"2021-01-20T18:53:20Z","verificationMethod":"did:unid:test:EiD66QsnHupVmjYLfgo4RLeAwnm7v175B52NvXQ3GY7TBw#signingKey","jws":"eyJhbGciOiJFUzI1NksiLCJiNjQiOmZhbHNlLCJjcml0IjpbImI2NCJdfQ..QDAtL2i5ruunUHzOfkpO8eljglJfzsZW1CAQLLkIYGVN8TXQ6xM24lYcSSJIkm9q62EmaL2zAwrJRA81Gbaa2A"},"id":"https://sds.getunid.io/api/v1","issuer":"did:unid:test:EiD66QsnHupVmjYLfgo4RLeAwnm7v175B52NvXQ3GY7TBw","issuanceDate":"2021-01-20T18:53:20Z","@context":["https://www.w3.org/2018/credentials/v1"],"type":["VerifiablePresentation"],"verifiableCredential":[{"proof":{"type":"EcdsaSecp256k1Signature2019","proofPurpose":"authentication","created":"2021-01-20T18:53:20Z","verificationMethod":"did:unid:test:EiD66QsnHupVmjYLfgo4RLeAwnm7v175B52NvXQ3GY7TBw#signingKey","jws":"eyJhbGciOiJFUzI1NksiLCJiNjQiOmZhbHNlLCJjcml0IjpbImI2NCJdfQ..1bklxVH4VjfY0JvrKwgkRVRL5ElwqjBucah2vmhlGbd4BMSM2IhL9YTcwmdR9hJD_n8rXX21M4pOJRjCCSwR_w"},"id":"https://sds.getunid.io/api/v1","issuer":"did:unid:test:EiD66QsnHupVmjYLfgo4RLeAwnm7v175B52NvXQ3GY7TBw","issuanceDate":"2021-01-20T18:53:20Z","@context":["https://www.w3.org/2018/credentials/v1","https://docs.getunid.io/docs/2020/credentials/address"],"type":["VerifiableCredential","AddressCredentialV1"],"credentialSubject":{"@id":"did:unid:test:EiD66QsnHupVmjYLfgo4RLeAwnm7v175B52NvXQ3GY7TBw","@type":"AddressPerson","address":{"@type":"PostalAddress","streetAddress":"日本橋"}}}]}'
    const json = JSON.parse(text)

    expect(UNiD.isVerifiableCredential(json)).toEqual(false)
    expect(UNiD.isVerifiablePresentation(json)).toEqual(true)

    if (UNiD.isVerifiablePresentation(json)) {
        const presentation = await UNiD.verifyPresentation(json)

        expect(presentation.isValid).toEqual(true)

        const phone   = PhoneCredentialV1.select(presentation.payload)
        const address = AddressCredentialV1.select(presentation.payload)

        expect(phone).toBeUndefined()
        expect(address).not.toBeUndefined()

        if (address) {
            const credential = await UNiD.verifyCredential(address)

            expect(credential.isValid).toEqual(true)
        }
    }
})

test('UNiD - 4', async () => {
    const text = '{}'
    const json = JSON.parse(text)

    await expect(async () => {
        await UNiD.verifyCredential(json)
    }).rejects.toThrow('[code: 400]')
})

test('UNiD - 5', async () => {
    const text = '{}'
    const json = JSON.parse(text)

    await expect(async () => {
        await UNiD.verifyPresentation(json)
    }).rejects.toThrow('[code: 400]')
})

test('UNiD - 6', async () => {
    const DID = await UNiD.createDid(KeyRingType.Mnemonic)

    const signedVC1 = await DID.createCredential(
        new AddressCredentialV1({
            '@id'  : DID.getIdentifier(),
            '@type': 'AddressPerson',
            address: {
                '@type': 'PostalAddress',
                streetAddress: '日本橋',
            },
        })
    )

    const verifiedVC1 = await UNiD.verifyCredential(signedVC1)

    const signedVC2 = await DID.createCredential(
        new AlumniOfCredentialV1({
            '@id'  : DID.getIdentifier(),
            '@type': 'AlumniOfOrganization',
            alumniOf: [{
                '@type': 'Organization',
            }]
        })
    )

    const verifiedVC2 = await UNiD.verifyCredential(signedVC2)

    const signedVC3 = await DID.createCredential(
        new BirthDateCredentialV1({
            '@id'    : DID.getIdentifier(),
            '@type'  : 'BirthDatePerson',
            birthDate: '1900-01-01',
        })
    )

    const verifiedVC3 = await UNiD.verifyCredential(signedVC3)

    const signedVC4 = await DID.createCredential(
        new ContactPointCredentialV1({
            '@id': DID.getIdentifier(),
            '@type': 'ContactPointPerson',
            contactPoint: {
                '@type': 'PostalAddress',
            }
        })
    )

    const verifiedVC4 = await UNiD.verifyCredential(signedVC4)

    const signedVC5 = await DID.createCredential(
        new EmailCredentialV1({
            '@id'  : DID.getIdentifier(),
            '@type': 'EmailPerson',
            email  : 'username@example.com',
        })
    )

    const verifiedVC5 = await UNiD.verifyCredential(signedVC5)

    const signedVC6 = await DID.createCredential(
        new GenderCredentialV1({
            '@id'  : DID.getIdentifier(),
            '@type': 'GenderPerson',
            gender : 'Male'
        })
    )

    const verifiedVC6 = await UNiD.verifyCredential(signedVC6)

    const signedVC7 = await DID.createCredential(
        new ImageCredentialV1({
            '@id'  : DID.getIdentifier(),
            '@type': 'ImagePerson',
            image  : {
                '@type': 'Barcode',
            }
        })
    )

    const verifiedVC7 = await UNiD.verifyCredential(signedVC7)

    const signedVC8 = await DID.createCredential(
        new NameCredentialV1({
            '@id'     : DID.getIdentifier(),
            '@type'   : 'NamePerson',
            name      : 'name',
            givenName : 'givenName',
            familyName: 'familyName',
        })
    )

    const verifiedVC8 = await UNiD.verifyCredential(signedVC8)

    const signedVC9 = await DID.createCredential(
        new PhoneCredentialV1({
            '@id'    : DID.getIdentifier(),
            '@type'  : 'PhonePerson',
            telephone: '0000-0000-0000'
        })
    )

    const verifiedVC9 = await UNiD.verifyCredential(signedVC9)

    const signedVC10 = await DID.createCredential(
        new QualificationCredentialV1({
            '@id': DID.getIdentifier(),
            '@type': 'QualificationPerson',
            hasCredential: [{
                '@type': 'EducationalOccupationalCredential'
            }],
        })
    )

    const verifiedVC10 = await UNiD.verifyCredential(signedVC10)

    const signedVC11 = await DID.createCredential(
        new WorksForCredentialV1({
            '@id': DID.getIdentifier(),
            '@type': 'WorksForOrganization',
            worksFor: [{
                '@type': 'Organization',
            }]
        })
    )

    const verifiedVC11 = await UNiD.verifyCredential(signedVC11)

    const signedVP = await DID.createPresentation([
        signedVC1, signedVC2, signedVC3, signedVC4, signedVC5,
        signedVC6, signedVC7, signedVC8, signedVC9, signedVC10,
        signedVC11,
    ])

    const verifiedVP = await UNiD.verifyPresentation(signedVP)

    expect(verifiedVP.isValid).toEqual(true)
    expect(verifiedVC1.isValid).toEqual(true)
    expect(verifiedVC2.isValid).toEqual(true)
    expect(verifiedVC3.isValid).toEqual(true)
    expect(verifiedVC4.isValid).toEqual(true)
    expect(verifiedVC5.isValid).toEqual(true)
    expect(verifiedVC6.isValid).toEqual(true)
    expect(verifiedVC7.isValid).toEqual(true)
    expect(verifiedVC8.isValid).toEqual(true)
    expect(verifiedVC9.isValid).toEqual(true)
    expect(verifiedVC10.isValid).toEqual(true)
    expect(verifiedVC11.isValid).toEqual(true)

    expect(AddressCredentialV1.select(signedVP)).not.toBeUndefined()
    expect(AlumniOfCredentialV1.select(signedVP)).not.toBeUndefined()
    expect(BirthDateCredentialV1.select(signedVP)).not.toBeUndefined()
    expect(ContactPointCredentialV1.select(signedVP)).not.toBeUndefined()
    expect(EmailCredentialV1.select(signedVP)).not.toBeUndefined()
    expect(GenderCredentialV1.select(signedVP)).not.toBeUndefined()
    expect(ImageCredentialV1.select(signedVP)).not.toBeUndefined()
    expect(NameCredentialV1.select(signedVP)).not.toBeUndefined()
    expect(PhoneCredentialV1.select(signedVP)).not.toBeUndefined()
    expect(QualificationCredentialV1.select(signedVP)).not.toBeUndefined()
    expect(WorksForCredentialV1.select(signedVP)).not.toBeUndefined()
})