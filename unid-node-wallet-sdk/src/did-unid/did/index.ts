import { MnemonicKeyring } from '../keyring/mnemonic'
import { UNiDDidOperator } from '../../core'
import { VerifiableCredential } from './credential'
import {
    VC_ID,
    UNiDVerifiableCredentialBase,
    UNiDVerifiableCredentialMetadata,
    UNiDVerifiableCredential,
    UNiDVerifiablePresentationV1,
    UNiDVerifiableCredentialTypes,
    UNiDCredentialSubjectMetadata,
    UNiDVerifiablePresentation,
    UNiDVerifiablePresentationMetadata,
} from '../schemas'
import { DateTimeTypes, DateTimeUtils } from '../../utils/datetime'
import { UNiDInvalidDataError, UNiDInvalidSignatureError, UNiDNotCompatibleError, UNiDNotUniqueError } from '../../error'
import { VerifiablePresentation } from './presentation'
import { SIGNING_KEY_ID, UNiDContextInternal, UNiDVerifyCredentialResponse } from '../../unid'
import { Cipher } from '../cipher/cipher'
import { UNiDSDSCredentialV1, UNiDSDSCredentialV1Types } from '../schemas/internal/unid-sds'
import { UNiDSDSOperator, SDSCreateResponse, SDSFindOperationResponsePayload } from '../sds/operator'
import { UNiD } from '../../unid'
import { promise } from '../../utils/promise'
import { utils } from '../../utils/utils'
import { Secp256k1 } from '../keyring/secp256k1'
import { UNiDAuthCredentialV1 } from '../schemas/internal/unid-auth'

/**
 */
interface UNiDDidContext {
    context : UNiDContextInternal
    keyring : MnemonicKeyring
    operator: UNiDDidOperator
}

/**
 */
interface UNiDFindOneQuery {
    // [[ REQUIRED ]]
    type: UNiDVerifiableCredentialTypes | string,

    // [[ OPTIONAL ]]
    issuerDid?     : string,
    issuanceDate?  : { begin?: Date, end?: Date },
    expirationDate?: { begin?: Date, end?: Date },
}

/**
 */
interface UNiDFindQuery extends UNiDFindOneQuery {
    limit?: number,
    page? : number,
}

/**
 */
export type Weaken<T, K extends keyof T> = {
    [P in keyof T]: P extends K ? any : T[P]
}

/**
 */
export class UNiDDid {
    /**
     */
    private readonly keyring : MnemonicKeyring

    /**
     */
    private readonly operator: UNiDDidOperator

    /**
     */
    private readonly context: UNiDContextInternal

    /**
     * @param context 
     */
    constructor(context: UNiDDidContext) {
        this.keyring  = context.keyring
        this.operator = context.operator
        this.context  = context.context
    }

    /**
     * @returns
     */
    protected getKeyPairs(): {
        sign    : Secp256k1,
        update  : Secp256k1,
        recovery: Secp256k1,
        encrypt : Secp256k1
    } {
        return {
            sign    : this.keyring.getSignKeyPair(),
            update  : this.keyring.getUpdateKeyPair(),
            recovery: this.keyring.getRecoveryKeyPair(),
            encrypt : this.keyring.getEncryptKeyPair(),
        }
    }

    /**
     * @returns
     */
    public getSeedPhrase(): Array<string> | undefined {
        return this.keyring.getSeedPhrases()
    }

    /**
     * @param phrase 
     * @param option 
     * @returns
     */
    public async verifySeedPhrase(phrase: Array<string>, option: { isPersistent: boolean } = { isPersistent: false }): Promise<boolean> {
        return await this.keyring.verifySeedPhrase(this.getIdentifier(), phrase, option)
    }

    /**
     * @returns
     */
    public getIdentifier(): string {
        return this.keyring.getIdentifier()
    }

    /**
     * @returns
     */
    public async getDidDocument() {
        return await this.operator.resolve({
            did: this.getIdentifier(),
        })
    }

    /**
     * Create: Verifiable Credential
     * 
     * @param credential 
     * @returns
     */
    public async createCredential<T>(credential: UNiDVerifiableCredentialBase<T>) {
        const iss = (new DateTimeUtils(credential.issuanceDate)).$toString(DateTimeTypes.default)
        const exp = (new DateTimeUtils(credential.expirationDate)).toString(DateTimeTypes.default)

        const data = credential.getVerifiableCredential({
            id    : VC_ID,
            issuer: this.getIdentifier(),
            issuanceDate: iss,
        })

        if (exp !== undefined) {
            data.expirationDate = exp
        }

        const verifiableCredential = new VerifiableCredential(data)

        return await verifiableCredential.sign({
            did    : this.keyring.getIdentifier(),
            keyId  : SIGNING_KEY_ID,
            context: this.keyring.getSignKeyPair(),
        })
    }

    /**
     * Create: Verifiable Presentation
     * 
     * @param credentials 
     * @returns
     */
    public async createPresentation(credentials: Array<UNiDVerifiableCredential<string, string, UNiDCredentialSubjectMetadata> & UNiDVerifiableCredentialMetadata>) {
        const types: Array<string> = []

        credentials.forEach((credential) => {
            credential.type.forEach((type) => {
                // [TODO]: 'VerifiableCredential' should be a constant
                if (type !== 'VerifiableCredential') {
                    types.push(type)
                }
            })
        })

        const duplicated = types.filter((type, _, self) => {
            return self.indexOf(type) !== self.lastIndexOf(type)
        })

        if (0 < duplicated.length) {
            throw new UNiDNotUniqueError()
        }

        const presentation = new UNiDVerifiablePresentationV1(credentials)

        const iss = (new DateTimeUtils(presentation.issuanceDate)).$toString(DateTimeTypes.default)
        const exp = (new DateTimeUtils(presentation.expirationDate)).toString(DateTimeTypes.default)

        const data = presentation.getVerifiablePresentation({
            id    : VC_ID,
            issuer: this.getIdentifier(),
            issuanceDate: iss,
        })

        if (exp !== undefined) {
            data.expirationDate = exp
        }

        const verifiablePresentation = new VerifiablePresentation(data)

        return await verifiablePresentation.sign({
            did    : this.keyring.getIdentifier(),
            keyId  : SIGNING_KEY_ID,
            context: this.keyring.getSignKeyPair(),
        })
    }

    /**
     * To: SDS
     * 
     * @param credential 
     * @returns
     */
    public async postCredential<T1, T2, T3>(credential: UNiDVerifyCredentialResponse<T1, T2, T3>): Promise<SDSCreateResponse> {
        const operator = new UNiDSDSOperator({
            context: this.context,
        })

        const data: Buffer   = Buffer.from(credential.toJSON(), 'utf-8')
        const secret: Buffer = this.keyring.getEncryptKeyPair().getPrivateKey()

        const metadata   = credential.metadata
        const encrypted  = (await Cipher.encrypt(data, secret)).toString('base64')
        const issuance   = (new DateTimeUtils(metadata.issuanceDate)).$toString(DateTimeTypes.iso8601)
        const expiration = (new DateTimeUtils(metadata.expirationDate)).toString(DateTimeTypes.iso8601)

        const payload = (await this.createPresentation([
            await this.createCredential(
                new UNiDSDSCredentialV1({
                    '@id'    : this.getIdentifier(),
                    '@type'  : 'CreateOperation',
                    clientId : this.context.clientId,
                    payload  : encrypted,
                    context  : metadata['@context'],
                    type     : metadata.type,
                    issuerDid: metadata.issuerDid,
                    credentialSubjectDid: metadata.credentialSubjectDid,
                    issuanceDate  : issuance,
                    expirationDate: expiration,
                })
            )
        ])) as UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, UNiDSDSCredentialV1Types>> & UNiDVerifiablePresentationMetadata

        return await operator.create({ payload: payload })
    }

    /**
     * From: SDS
     * 
     * @param query 
     * @returns
     */
    public async getCredential(query: UNiDFindOneQuery): Promise<UNiDVerifyCredentialResponse<string, string, UNiDCredentialSubjectMetadata> | undefined> {
        const operator = new UNiDSDSOperator({
            context: this.context,
        })

        let issuanceDate  : { begin?: string, end?: string } | undefined = undefined
        let expirationDate: { begin?: string, end?: string } | undefined = undefined

        if (query.issuanceDate) {
            issuanceDate = {
                begin: (new DateTimeUtils(query.issuanceDate.begin)).toString(DateTimeTypes.iso8601),
                end  : (new DateTimeUtils(query.issuanceDate.end)).toString(DateTimeTypes.iso8601),
            }
        }
        if (query.expirationDate) {
            expirationDate = {
                begin: (new DateTimeUtils(query.expirationDate.begin)).toString(DateTimeTypes.iso8601),
                end  : (new DateTimeUtils(query.expirationDate.end)).toString(DateTimeTypes.iso8601),
            }
        }

        const payload = (await this.createPresentation([
            await this.createCredential(
                new UNiDSDSCredentialV1({
                    '@id'   : this.getIdentifier(),
                    '@type' : 'FindOneOperation',
                    clientId: this.context.clientId,
                    // REQUIRED
                    type                : query.type,
                    credentialSubjectDid: this.getIdentifier(),
                    // OPTIONAL
                    issuerDid     : query.issuerDid,
                    issuanceDate  : issuanceDate,
                    expirationDate: expirationDate,
                })
            )
        ])) as UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, UNiDSDSCredentialV1Types>> & UNiDVerifiablePresentationMetadata

        const response = await operator.findOne({ payload: payload })

        if (response.payload === undefined) {
            return undefined
        }

        return this.decryptCredential(response.payload.document)
    }

    /**
     * From: SDS
     * 
     * @param query 
     * @returns
     */
    public async getCredentials(query: UNiDFindQuery): Promise<Array<UNiDVerifyCredentialResponse<string, string, UNiDCredentialSubjectMetadata>>> {
        const operator = new UNiDSDSOperator({
            context: this.context,
        })

        let issuanceDate  : { begin?: string, end?: string } | undefined = undefined
        let expirationDate: { begin?: string, end?: string } | undefined = undefined

        if (query.issuanceDate) {
            issuanceDate = {
                begin: (new DateTimeUtils(query.issuanceDate.begin)).toString(DateTimeTypes.iso8601),
                end  : (new DateTimeUtils(query.issuanceDate.end)).toString(DateTimeTypes.iso8601),
            }
        }
        if (query.expirationDate) {
            expirationDate = {
                begin: (new DateTimeUtils(query.expirationDate.begin)).toString(DateTimeTypes.iso8601),
                end  : (new DateTimeUtils(query.expirationDate.end)).toString(DateTimeTypes.iso8601),
            }
        }

        const payload = (await this.createPresentation([
            await this.createCredential(
                new UNiDSDSCredentialV1({
                    '@id'   : this.getIdentifier(),
                    '@type' : 'FindOperation',
                    clientId: this.context.clientId,
                    // METADATA/REQUIRED
                    type                : query.type,
                    credentialSubjectDid: this.getIdentifier(),
                    // METADATA/OPTIONAL
                    issuerDid     : query.issuerDid,
                    issuanceDate  : issuanceDate,
                    expirationDate: expirationDate,
                    // OPTIONS
                    limit: query.limit,
                    page : query.page,
                })
            )
        ])) as UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, UNiDSDSCredentialV1Types>> & UNiDVerifiablePresentationMetadata
        
        const response = await operator.find({ payload: payload })
        const verified = await promise.all<SDSFindOperationResponsePayload, UNiDVerifyCredentialResponse<string, string, UNiDCredentialSubjectMetadata>>(response.payload, async (item, _) => {
            return this.decryptCredential(item.document)
        })

        return verified
    }

    /**
     * @param params 
     * @returns
     */
    public async generateAuthenticationRequest(options: {
        required: Array<UNiDVerifiableCredentialTypes>,
        optional: Array<UNiDVerifiableCredentialTypes>,
    }) {
        const vc = await this.createCredential(
                new UNiDAuthCredentialV1({
                '@id'  : this.getIdentifier(),
                '@type': 'AuthnRequest',
                iss: this.getIdentifier(),
                kid: this.keyring.getIdentifier(SIGNING_KEY_ID),
                client_id: this.getIdentifier(),
                registration: {},
                scope: 'did_authn',
                claims: {
                    required: options.required,
                    optional: options.optional,
                },
            })
        )

        return await this.createPresentation([ vc ])
    }

    /**
     * @param verifiablePresentation 
     * @returns
     */
    public async generateAuthenticationResponse(verifiablePresentation: UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, UNiDCredentialSubjectMetadata>> & UNiDVerifiablePresentationMetadata) {
        const vc = await this.createCredential(
            new UNiDAuthCredentialV1({
                '@id': this.getIdentifier(),
                '@type': 'AuthnResponse',
                did: this.getIdentifier(),
                sub_jwk: this.keyring.getSignKeyPair().toJwk(),
                verifiablePresentation: verifiablePresentation,
            })
        )

        return await this.createPresentation([ vc ])
    }

    /**
     * @param encryptedCredential 
     * @returns
     */
    private async decryptCredential(encryptedCredential: string): Promise<UNiDVerifyCredentialResponse<string, string, UNiDCredentialSubjectMetadata>> {
        if (! utils.isBase64(encryptedCredential)) {
            throw new UNiDInvalidDataError()
        }

        const data: Buffer   = Buffer.from(encryptedCredential, 'base64')
        const secret: Buffer = this.keyring.getEncryptKeyPair().getPrivateKey()

        const decrypted = (await Cipher.decrypt(data, secret)).toString('utf-8')
        const object    = JSON.parse(decrypted)

        if (! UNiD.isVerifiableCredential(object)) {
            throw new UNiDNotCompatibleError()
        }

        const verified = await UNiD.verifyCredential(object)

        if (! verified.isValid) {
            throw new UNiDInvalidSignatureError()
        }

        return verified
    }
}