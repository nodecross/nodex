import { Text } from './schema.org'
import { UNiDNotCompatibleError, UNiDNotUniqueError } from '../../error'
import { UNiDCredentialSubjectMetadata, UNiDVerifiableCredential, UNiDVerifiableCredentialBase, UNiDVerifiableCredentialMetadata, UNiDVerifiableCredentialOptions, UNiDVerifiablePresentation } from '.'

// NameCredentialV1

/**
 */
interface NamePerson extends UNiDCredentialSubjectMetadata {
    '@type': 'NamePerson',
    name: Text,
    givenName: Text,
    familyName: Text,
}

/**
 */
interface NameOrganization extends UNiDCredentialSubjectMetadata {
    '@type': 'NameOrganization',
    name: Text,
    givenName: Text,
    familyName: Text,
}

/**
 */
export interface NameCredentialV1Schema extends UNiDVerifiableCredential<
    'https://docs.getunid.io/docs/2020/credentials/name',
    'NameCredentialV1',
    NamePerson | NameOrganization
> {}

/**
 */
export class NameCredentialV1 extends UNiDVerifiableCredentialBase<NameCredentialV1Schema> {
    /**
     * @param credential 
     * @param options 
     */
    public constructor(credentialSubject: NamePerson | NameOrganization, options?: UNiDVerifiableCredentialOptions) {
        super(options)

        this._credential = {
            '@context': [
                'https://www.w3.org/2018/credentials/v1',
                'https://docs.getunid.io/docs/2020/credentials/name',
            ],
            type: [ 'VerifiableCredential', 'NameCredentialV1' ],
            credentialSubject: credentialSubject,
        }
    }

    /**
     * @param input 
     */
    public static isCompatible(input: any): input is NameCredentialV1Schema & UNiDVerifiableCredentialMetadata {
        if (typeof input !== 'object') {
            return false
        }
        if (Object.keys(input).indexOf('type') < 0) {
            return false
        }
        if (Array.isArray(input.type) !== true) {
            return false
        }
        if (Array.from(input.type).indexOf('NameCredentialV1') < 0) {
            return false
        }
        return true
    }

    /**
     * @param input 
     */
    public static fromObject(input: any): NameCredentialV1 {
        if (! NameCredentialV1.isCompatible(input)) {
            throw new UNiDNotCompatibleError()
        }

        return new NameCredentialV1(input.credentialSubject)
    }

    /**
     * @param vp 
     */
    public static select<T>(vp: UNiDVerifiablePresentation<T>): NameCredentialV1Schema & UNiDVerifiableCredentialMetadata | undefined {
        const selected = vp.verifiableCredential.filter((vc) => {
            return NameCredentialV1.isCompatible(vc)
        })

        if (1 < selected.length) {
            throw new UNiDNotUniqueError()
        }

        const select = selected.shift()

        if (select === undefined) {
            return undefined
        }
        if (! NameCredentialV1.isCompatible(select)) {
            return undefined
        }

        return select
    }
}