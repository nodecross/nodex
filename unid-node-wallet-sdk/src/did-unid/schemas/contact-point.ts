import { ContactPoint } from './schema.org'
import { UNiDNotCompatibleError, UNiDNotUniqueError } from '../../error'
import { UNiDCredentialSubjectMetadata, UNiDVerifiableCredential, UNiDVerifiableCredentialBase, UNiDVerifiableCredentialMetadata, UNiDVerifiableCredentialOptions, UNiDVerifiablePresentation } from '.'

// ContactPointCredentialV1

/**
 */
interface ContactPointPerson extends UNiDCredentialSubjectMetadata {
    '@type': 'ContactPointPerson',
    contactPoint: ContactPoint,
}

/**
 */
interface ContactPointOrganization extends UNiDCredentialSubjectMetadata {
    '@type': 'ContactPointOrganization',
    contactPoint: ContactPoint,
}

/**
 */
export interface ContactPointCredentialV1Schema extends UNiDVerifiableCredential<
    'https://docs.getunid.io/docs/2020/credentials/contactPoint',
    'ContactPointCredentialV1',
    ContactPointPerson | ContactPointOrganization
> {}

/**
 */
export class ContactPointCredentialV1 extends UNiDVerifiableCredentialBase<ContactPointCredentialV1Schema> {
    /**
     * @param credential 
     * @param options 
     */
    public constructor(credentialSubject: ContactPointPerson | ContactPointOrganization, options?: UNiDVerifiableCredentialOptions) {
        super(options)

        this._credential = {
            '@context': [
                'https://www.w3.org/2018/credentials/v1',
                'https://docs.getunid.io/docs/2020/credentials/contactPoint',
            ],
            type: [ 'VerifiableCredential', 'ContactPointCredentialV1' ],
            credentialSubject: credentialSubject,
        }
    }

    /**
     * @param input 
     */
    public static isCompatible(input: any): input is ContactPointCredentialV1Schema & UNiDVerifiableCredentialMetadata {
        if (typeof input !== 'object') {
            return false
        }
        if (Object.keys(input).indexOf('type') < 0) {
            return false
        }
        if (Array.isArray(input.type) !== true) {
            return false
        }
        if (Array.from(input.type).indexOf('ContactPointCredentialV1') < 0) {
            return false
        }
        return true
    }

    /**
     * @param input 
     */
    public static fromObject(input: any): ContactPointCredentialV1 {
        if (! ContactPointCredentialV1.isCompatible(input)) {
            throw new UNiDNotCompatibleError()
        }

        return new ContactPointCredentialV1(input.credentialSubject)
    }

    /**
     * @param vp 
     */
    public static select<T>(vp: UNiDVerifiablePresentation<T>): ContactPointCredentialV1Schema & UNiDVerifiableCredentialMetadata | undefined {
        const selected = vp.verifiableCredential.filter((vc) => {
            return ContactPointCredentialV1.isCompatible(vc)
        })

        if (1 < selected.length) {
            throw new UNiDNotUniqueError()
        }

        const select = selected.shift()

        if (select === undefined) {
            return undefined
        }
        if (! ContactPointCredentialV1.isCompatible(select)) {
            return undefined
        }

        return select
    }
}