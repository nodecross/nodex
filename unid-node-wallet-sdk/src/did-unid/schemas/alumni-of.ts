import { OrganizationLeaf } from './schema.org'
import { UNiDNotCompatibleError, UNiDNotUniqueError } from '../../error'
import { UNiDCredentialSubjectMetadata, UNiDVerifiableCredential, UNiDVerifiableCredentialBase, UNiDVerifiableCredentialMetadata, UNiDVerifiableCredentialOptions, UNiDVerifiablePresentation } from '.'

// AlumniOfCredentialV1

/**
 */
interface AlumniOfOrganization extends UNiDCredentialSubjectMetadata {
    '@type': 'AlumniOfOrganization',
    alumniOf: Array<OrganizationLeaf>,
}

/**
 */
export interface AlumniOfCredentialV1Schema extends UNiDVerifiableCredential<
    'https://docs.getunid.io/docs/2020/credentials/alumniOf',
    'AlumniOfCredentialV1',
    AlumniOfOrganization
> {}

/**
 */
export class AlumniOfCredentialV1 extends UNiDVerifiableCredentialBase<AlumniOfCredentialV1Schema> {
    /**
     * @param credential 
     * @param options 
     */
    public constructor(credentialSubject: AlumniOfOrganization, options?: UNiDVerifiableCredentialOptions) {
        super(options)

        this._credential = {
            '@context': [
                'https://www.w3.org/2018/credentials/v1',
                'https://docs.getunid.io/docs/2020/credentials/alumniOf',
            ],
            type: [ 'VerifiableCredential', 'AlumniOfCredentialV1' ],
            credentialSubject: credentialSubject,
        }
    }

    /**
     * @param input 
     */
    public static isCompatible(input: any): input is AlumniOfCredentialV1Schema & UNiDVerifiableCredentialMetadata {
        if (typeof input !== 'object') {
            return false
        }
        if (Object.keys(input).indexOf('type') < 0) {
            return false
        }
        if (Array.isArray(input.type) !== true) {
            return false
        }
        if (Array.from(input.type).indexOf('AlumniOfCredentialV1') < 0) {
            return false
        }
        return true
    }

    /**
     * @param input 
     */
    public static fromObject(input: any): AlumniOfCredentialV1 {
        if (! AlumniOfCredentialV1.isCompatible(input)) {
            throw new UNiDNotCompatibleError()
        }

        return new AlumniOfCredentialV1(input.credentialSubject)
    }

    /**
     * @param vp 
     */
    public static select<T>(vp: UNiDVerifiablePresentation<T>): AlumniOfCredentialV1Schema & UNiDVerifiableCredentialMetadata | undefined {
        const selected = vp.verifiableCredential.filter((vc) => {
            return AlumniOfCredentialV1.isCompatible(vc)
        })

        if (1 < selected.length) {
            throw new UNiDNotUniqueError()
        }

        const select = selected.shift()

        if (select === undefined) {
            return undefined
        }
        if (! AlumniOfCredentialV1.isCompatible(select)) {
            return undefined
        }

        return select
    }
}