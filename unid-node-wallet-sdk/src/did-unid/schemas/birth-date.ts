import { Date } from './schema.org'
import { UNiDNotCompatibleError, UNiDNotUniqueError } from '../../error'
import { UNiDCredentialSubjectMetadata, UNiDVerifiableCredential, UNiDVerifiableCredentialBase, UNiDVerifiableCredentialMetadata, UNiDVerifiableCredentialOptions, UNiDVerifiablePresentation } from '.'

// BirthDateCredentialV1

/**
 */
interface BirthDatePerson extends UNiDCredentialSubjectMetadata {
    '@type': 'BirthDatePerson',
    birthDate: Date,
}

/**
 */
export interface BirthDateCredentialV1Schema extends UNiDVerifiableCredential<
    'https://docs.getunid.io/docs/2020/credentials/birthDate',
    'BirthDateCredentialV1',
    BirthDatePerson
> {}

/**
 */
export class BirthDateCredentialV1 extends UNiDVerifiableCredentialBase<BirthDateCredentialV1Schema> {
    /**
     * @param credential 
     * @param options 
     */
    public constructor(credentialSubject: BirthDatePerson, options?: UNiDVerifiableCredentialOptions) {
        super(options)

        this._credential = {
            '@context': [
                'https://www.w3.org/2018/credentials/v1',
                'https://docs.getunid.io/docs/2020/credentials/birthDate',
            ],
            type: [ 'VerifiableCredential', 'BirthDateCredentialV1' ],
            credentialSubject: credentialSubject,
        }
    }

    /**
     * @param input 
     */
    public static isCompatible(input: any): input is BirthDateCredentialV1Schema & UNiDVerifiableCredentialMetadata {
        if (typeof input !== 'object') {
            return false
        }
        if (Object.keys(input).indexOf('type') < 0) {
            return false
        }
        if (Array.isArray(input.type) !== true) {
            return false
        }
        if (Array.from(input.type).indexOf('BirthDateCredentialV1') < 0) {
            return false
        }
        return true
    }

    /**
     * @param input 
     */
    public static fromObject(input: any): BirthDateCredentialV1 {
        if (! BirthDateCredentialV1.isCompatible(input)) {
            throw new UNiDNotCompatibleError()
        }

        return new BirthDateCredentialV1(input.credentialSubject)
    }

    /**
     * @param vp 
     */
    public static select<T>(vp: UNiDVerifiablePresentation<T>): BirthDateCredentialV1Schema & UNiDVerifiableCredentialMetadata | undefined {
        const selected = vp.verifiableCredential.filter((vc) => {
            return BirthDateCredentialV1.isCompatible(vc)
        })

        if (1 < selected.length) {
            throw new UNiDNotUniqueError()
        }

        const select = selected.shift()

        if (select === undefined) {
            return undefined
        }
        if (! BirthDateCredentialV1.isCompatible(select)) {
            return undefined
        }

        return select
    }
}