import { Text, DateTime, Number } from '../schema.org'
import { UNiDNotCompatibleError, UNiDNotUniqueError } from '../../../error'
import { UNiDCredentialSubjectMetadata, UNiDVerifiableCredential, UNiDVerifiableCredentialBase, UNiDVerifiableCredentialMetadata, UNiDVerifiableCredentialOptions, UNiDVerifiablePresentation } from '..';

// UNiDSDSCredentialV1

/**
 */
interface DateTimeQuery {
    begin?: Readonly<DateTime>,
    end?  : Readonly<DateTime>,
}

/**
 */
interface CreateOperation extends UNiDCredentialSubjectMetadata {
    '@type' : 'CreateOperation',
    clientId: Readonly<Text>,

    // [[ PAYLOAD ]]
    payload: Readonly<Text>,

    // [[ METADATA ]]
    context             : Readonly<Text>,
    type                : Readonly<Text>,
    issuerDid           : Readonly<Text>,
    credentialSubjectDid: Readonly<Text>,
    issuanceDate        : Readonly<DateTime>,
    expirationDate?     : Readonly<DateTime>,
}

/**
 */
interface FindOneOperation extends UNiDCredentialSubjectMetadata {
    '@type' : 'FindOneOperation',
    clientId: Readonly<Text>,

    // [[ ID ]]
    id?: Readonly<Text>,

    // [[ METADATA - REQUIRED ]]
    type                : Readonly<Text>,
    credentialSubjectDid: Readonly<Text>,

    // [[ METADATA - OPTIONAL ]]
    context?       : Readonly<Text>,
    issuerDid?     : Readonly<Text>,
    issuanceDate?  : Readonly<DateTimeQuery>,
    expirationDate?: Readonly<DateTimeQuery>,
}

/**
 */
interface FindOperation extends UNiDCredentialSubjectMetadata {
    '@type' : 'FindOperation',
    clientId: Readonly<Text>,

    // [[ ID ]]
    id?: Readonly<Text>,

    // [[ METADATA - REQUIRED ]]
    type                : Readonly<Text>,
    credentialSubjectDid: Readonly<Text>,

    // [[ METADATA - OPTIONAL ]]
    context?       : Readonly<Text>,
    issuerDid?     : Readonly<Text>,
    issuanceDate?  : Readonly<DateTimeQuery>,
    expirationDate?: Readonly<DateTimeQuery>,

    // [[ FIND OPTION ]]
    page? : Readonly<Number>,
    limit?: Readonly<Number>,
}

/**
 */
interface UpdateOperation extends UNiDCredentialSubjectMetadata {
    '@type' : 'UpdateOperation',
    clientId: Readonly<Text>,

    // [[ ID ]]
    id: Readonly<Text>,

    // [[ PAYLOAD ]]
    payload?: Readonly<Text>,

    // [[ METADATA ]]
    context?             : Readonly<Text>,
    type?                : Readonly<Text>,
    issuerDid?           : Readonly<Text>,
    credentialSubjectDid?: Readonly<Text>,
    issuanceDate?        : Readonly<DateTime>,
    expirationDate?      : Readonly<DateTime>,
}

/**
 */
interface DeleteOperation extends UNiDCredentialSubjectMetadata {
    '@type' : 'DeleteOperation',
    clientId: Readonly<Text>,

    // [[ ID ]]
    id: Readonly<Text>,
}

/**
 */
export type UNiDSDSCredentialV1Types =
    | CreateOperation
    | FindOneOperation
    | FindOperation
    | UpdateOperation
    | DeleteOperation

/**
 */
export interface UNiDSDSCredentialV1Schema extends UNiDVerifiableCredential<
    'https://docs.getunid.io/docs/2020/credentials/internal/unid-sds',
    'UNiDSDSCredentialV1',
    UNiDSDSCredentialV1Types
> {}

/**
 */
export class UNiDSDSCredentialV1 extends UNiDVerifiableCredentialBase<UNiDSDSCredentialV1Schema> {
    /**
     * @param credentialSubject 
     * @param options 
     */
    public constructor(credentialSubject: CreateOperation | FindOneOperation | FindOperation | UpdateOperation | DeleteOperation, options?: UNiDVerifiableCredentialOptions) {
        super(options)

        this._credential = {
            '@context': [
                'https://www.w3.org/2018/credentials/v1',
                'https://docs.getunid.io/docs/2020/credentials/internal/unid-sds',
            ],
            type: [ 'VerifiableCredential', 'UNiDSDSCredentialV1' ],
            credentialSubject: credentialSubject,
        }
    }

    /**
     * @param input 
     */
    public static isCompatible(input: any): input is UNiDSDSCredentialV1Schema & UNiDVerifiableCredentialMetadata {
        if (typeof input !== 'object') {
            return false
        }
        if (Object.keys(input).indexOf('type') < 0) {
            return false
        }
        if (Array.isArray(input.type) !== true) {
            return false
        }
        if (Array.from(input.type).indexOf('UNiDSDSCredentialV1') < 0) {
            return false
        }
        return true
    }

    /**
     * @param input 
     */
    public static fromObject(input: any): UNiDSDSCredentialV1 {
        if (! UNiDSDSCredentialV1.isCompatible(input)) {
            throw new UNiDNotCompatibleError()
        }

        return new UNiDSDSCredentialV1(input.credentialSubject)
    }

    /**
     * @param vp 
     */
    public static select<T>(vp: UNiDVerifiablePresentation<T>): UNiDSDSCredentialV1Schema & UNiDVerifiableCredentialMetadata | undefined {
        const selected = vp.verifiableCredential.filter((vc) => {
            return UNiDSDSCredentialV1.isCompatible(vc)
        })

        if (1 < selected.length) {
            throw new UNiDNotUniqueError()
        }

        const select = selected.shift()

        if (select === undefined) {
            return undefined
        }
        if (! UNiDSDSCredentialV1.isCompatible(select)) {
            return undefined
        }

        return select
    }
}