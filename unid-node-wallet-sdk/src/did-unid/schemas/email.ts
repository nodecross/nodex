import { Text } from './schema.org'
import { UNiDNotCompatibleError, UNiDNotUniqueError } from '../../error'
import { UNiDCredentialSubjectMetadata, UNiDVerifiableCredential, UNiDVerifiableCredentialBase, UNiDVerifiableCredentialMetadata, UNiDVerifiableCredentialOptions, UNiDVerifiablePresentation } from '.'

// EmailCredentialV1

/**
 */
interface EmailPerson extends UNiDCredentialSubjectMetadata {
    '@type': 'EmailPerson',
    email: Text,
}

/**
 */
interface EmailOrganization extends UNiDCredentialSubjectMetadata {
    '@type': 'EmailOrganization',
    email: Text,
}

/**
 */
export interface EmailCredentialV1Schema extends UNiDVerifiableCredential<
    'https://docs.getunid.io/docs/2020/credentials/email',
    'EmailCredentialV1',
    EmailPerson | EmailOrganization
> {}

/**
 */
export class EmailCredentialV1 extends UNiDVerifiableCredentialBase<EmailCredentialV1Schema> {
    /**
     * @param credential 
     * @param options 
     */
    public constructor(credentialSubject: EmailPerson | EmailOrganization, options?: UNiDVerifiableCredentialOptions) {
        super(options)

        this._credential = {
            '@context': [
                'https://www.w3.org/2018/credentials/v1',
                'https://docs.getunid.io/docs/2020/credentials/email',
            ],
            type: [ 'VerifiableCredential', 'EmailCredentialV1' ],
            credentialSubject: credentialSubject,
        }
    }

    /**
     * @param input 
     */
    public static isCompatible(input: any): input is EmailCredentialV1Schema & UNiDVerifiableCredentialMetadata {
        if (typeof input !== 'object') {
            return false
        }
        if (Object.keys(input).indexOf('type') < 0) {
            return false
        }
        if (Array.isArray(input.type) !== true) {
            return false
        }
        if (Array.from(input.type).indexOf('EmailCredentialV1') < 0) {
            return false
        }
        return true
    }

    /**
     * @param input 
     */
    public static fromObject(input: any): EmailCredentialV1 {
        if (! EmailCredentialV1.isCompatible(input)) {
            throw new UNiDNotCompatibleError()
        }

        return new EmailCredentialV1(input.credentialSubject)
    }

    /**
     * @param vp 
     */
    public static select<T>(vp: UNiDVerifiablePresentation<T>): EmailCredentialV1Schema & UNiDVerifiableCredentialMetadata | undefined {
        const selected = vp.verifiableCredential.filter((vc) => {
            return EmailCredentialV1.isCompatible(vc)
        })

        if (1 < selected.length) {
            throw new UNiDNotUniqueError()
        }

        const select = selected.shift()

        if (select === undefined) {
            return undefined
        }
        if (! EmailCredentialV1.isCompatible(select)) {
            return undefined
        }

        return select
    }
}