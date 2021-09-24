import { Text } from './schema.org'
import { UNiDNotCompatibleError, UNiDNotUniqueError } from '../../error'
import { UNiDCredentialSubjectMetadata, UNiDVerifiableCredential, UNiDVerifiableCredentialBase, UNiDVerifiableCredentialMetadata, UNiDVerifiableCredentialOptions, UNiDVerifiablePresentation } from '.'

// GenderCredentialV1

/**
 */
interface GenderPerson extends UNiDCredentialSubjectMetadata {
    '@type': 'GenderPerson',
    gender: Text,
}

/**
 */
export interface GenderCredentialV1Schema extends UNiDVerifiableCredential<
    'https://docs.getunid.io/docs/2020/credentials/gender',
    'GenderCredentialV1',
    GenderPerson
> {}

/**
 */
export class GenderCredentialV1 extends UNiDVerifiableCredentialBase<GenderCredentialV1Schema> {
    /**
     * @param credential 
     * @param options 
     */
    public constructor(credentialSubject: GenderPerson, options?: UNiDVerifiableCredentialOptions) {
        super(options)

        this._credential = {
            '@context': [
                'https://www.w3.org/2018/credentials/v1',
                'https://docs.getunid.io/docs/2020/credentials/gender',
            ],
            type: [ 'VerifiableCredential', 'GenderCredentialV1' ],
            credentialSubject: credentialSubject,
        }
    }

    /**
     * @param input 
     */
    public static isCompatible(input: any): input is GenderCredentialV1Schema & UNiDVerifiableCredentialMetadata {
        if (typeof input !== 'object') {
            return false
        }
        if (Object.keys(input).indexOf('type') < 0) {
            return false
        }
        if (Array.isArray(input.type) !== true) {
            return false
        }
        if (Array.from(input.type).indexOf('GenderCredentialV1') < 0) {
            return false
        }
        return true
    }

    /**
     * @param input 
     */
    public static fromObject(input: any): GenderCredentialV1 {
        if (! GenderCredentialV1.isCompatible(input)) {
            throw new UNiDNotCompatibleError()
        }

        return new GenderCredentialV1(input.credentialSubject)
    }

    /**
     * @param vp 
     */
    public static select<T>(vp: UNiDVerifiablePresentation<T>): GenderCredentialV1Schema & UNiDVerifiableCredentialMetadata | undefined {
        const selected = vp.verifiableCredential.filter((vc) => {
            return GenderCredentialV1.isCompatible(vc)
        })

        if (1 < selected.length) {
            throw new UNiDNotUniqueError()
        }

        const select = selected.shift()

        if (select === undefined) {
            return undefined
        }
        if (! GenderCredentialV1.isCompatible(select)) {
            return undefined
        }

        return select
    }
}