import { OrganizationLeaf } from './schema.org'
import { UNiDNotCompatibleError, UNiDNotUniqueError } from '../../error'
import { UNiDCredentialSubjectMetadata, UNiDVerifiableCredential, UNiDVerifiableCredentialBase, UNiDVerifiableCredentialMetadata, UNiDVerifiableCredentialOptions, UNiDVerifiablePresentation } from '.'

// WorksForCredentialV1

/**
 */
interface WorksForOrganization extends UNiDCredentialSubjectMetadata {
    '@type': 'WorksForOrganization',
    worksFor: Array<OrganizationLeaf>,
}

/**
 */
export interface WorksForCredentialV1Schema extends UNiDVerifiableCredential<
    'https://docs.getunid.io/docs/2020/credentials/worksFor',
    'WorksForCredentialV1',
    WorksForOrganization
> {}

/**
 */
export class WorksForCredentialV1 extends UNiDVerifiableCredentialBase<WorksForCredentialV1Schema> {
    /**
     * @param credential 
     * @param options 
     */
    public constructor(credentialSubject: WorksForOrganization, options?: UNiDVerifiableCredentialOptions) {
        super(options)

        this._credential = {
            '@context': [
                'https://www.w3.org/2018/credentials/v1',
                'https://docs.getunid.io/docs/2020/credentials/worksFor',
            ],
            type: [ 'VerifiableCredential', 'WorksForCredentialV1' ],
            credentialSubject: credentialSubject,
        }
    }

    /**
     * @param input 
     */
    public static isCompatible(input: any): input is WorksForCredentialV1Schema & UNiDVerifiableCredentialMetadata {
        if (typeof input !== 'object') {
            return false
        }
        if (Object.keys(input).indexOf('type') < 0) {
            return false
        }
        if (Array.isArray(input.type) !== true) {
            return false
        }
        if (Array.from(input.type).indexOf('WorksForCredentialV1') < 0) {
            return false
        }
        return true
    }

    /**
     * @param input 
     */
    public static fromObject(input: any): WorksForCredentialV1 {
        if (! WorksForCredentialV1.isCompatible(input)) {
            throw new UNiDNotCompatibleError()
        }

        return new WorksForCredentialV1(input.credentialSubject)
    }

    /**
     * @param vp 
     */
    public static select<T>(vp: UNiDVerifiablePresentation<T>): WorksForCredentialV1Schema & UNiDVerifiableCredentialMetadata | undefined {
        const selected = vp.verifiableCredential.filter((vc) => {
            return WorksForCredentialV1.isCompatible(vc)
        })

        if (1 < selected.length) {
            throw new UNiDNotUniqueError()
        }

        const select = selected.shift()

        if (select === undefined) {
            return undefined
        }
        if (! WorksForCredentialV1.isCompatible(select)) {
            return undefined
        }

        return select
    }
}