import { Text } from './schema.org'
import { UNiDNotCompatibleError, UNiDNotUniqueError } from '../../error'
import { UNiDCredentialSubjectMetadata, UNiDVerifiableCredential, UNiDVerifiableCredentialBase, UNiDVerifiableCredentialMetadata, UNiDVerifiableCredentialOptions, UNiDVerifiablePresentation } from '.'

// PhoneCredentialV1

/**
 */
interface PhonePerson extends UNiDCredentialSubjectMetadata {
    '@type': 'PhonePerson',
    telephone: Text,
}

/**
 */
interface PhoneOrganization extends UNiDCredentialSubjectMetadata {
    '@type': 'PhoneOrganization',
    telephone: Text,
}

/**
 */
export interface PhoneCredentialV1Schema extends UNiDVerifiableCredential<
    'https://docs.getunid.io/docs/2020/credentials/phone',
    'PhoneCredentialV1',
    PhonePerson | PhoneOrganization
> {}

/**
 */
export class PhoneCredentialV1 extends UNiDVerifiableCredentialBase<PhoneCredentialV1Schema> {
    /**
     * @param credential 
     * @param options 
     */
    public constructor(credentialSubject: PhonePerson | PhoneOrganization, options?: UNiDVerifiableCredentialOptions) {
        super(options)

        this._credential = {
            '@context': [
                'https://www.w3.org/2018/credentials/v1',
                'https://docs.getunid.io/docs/2020/credentials/phone',
            ],
            type: [ 'VerifiableCredential', 'PhoneCredentialV1' ],
            credentialSubject: credentialSubject,
        }
    }

    /**
     * @param input 
     */
    public static isCompatible(input: any): input is PhoneCredentialV1Schema & UNiDVerifiableCredentialMetadata {
        if (typeof input !== 'object') {
            return false
        }
        if (Object.keys(input).indexOf('type') < 0) {
            return false
        }
        if (Array.isArray(input.type) !== true) {
            return false
        }
        if (Array.from(input.type).indexOf('PhoneCredentialV1') < 0) {
            return false
        }
        return true
    }

    /**
     * @param input 
     */
    public static fromObject(input: any): PhoneCredentialV1 {
        if (! PhoneCredentialV1.isCompatible(input)) {
            throw new UNiDNotCompatibleError()
        }

        return new PhoneCredentialV1(input.credentialSubject)
    }

    /**
     * @param vp 
     */
    public static select<T>(vp: UNiDVerifiablePresentation<T>): PhoneCredentialV1Schema & UNiDVerifiableCredentialMetadata | undefined {
        const selected = vp.verifiableCredential.filter((vc) => {
            return PhoneCredentialV1.isCompatible(vc)
        })

        if (1 < selected.length) {
            throw new UNiDNotUniqueError()
        }

        const select = selected.shift()

        if (select === undefined) {
            return undefined
        }
        if (! PhoneCredentialV1.isCompatible(select)) {
            return undefined
        }

        return select
    }
}