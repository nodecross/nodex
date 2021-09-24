import { EducationalOccupationalCredential } from './schema.org'
import { UNiDNotUniqueError, UNiDNotCompatibleError } from '../../error'
import { UNiDCredentialSubjectMetadata, UNiDVerifiableCredential, UNiDVerifiableCredentialBase, UNiDVerifiableCredentialMetadata, UNiDVerifiableCredentialOptions, UNiDVerifiablePresentation } from '.'

// QualificationCredentialV1

/**
 */
interface QualificationPerson extends UNiDCredentialSubjectMetadata {
    '@type': 'QualificationPerson',
    hasCredential: Array<EducationalOccupationalCredential>,
}

/**
 */
export interface QualificationCredentialV1Schema extends UNiDVerifiableCredential<
    'https://docs.getunid.io/docs/2020/credentials/qualification',
    'QualificationCredentialV1',
    QualificationPerson
> {}

/**
 */
export class QualificationCredentialV1 extends UNiDVerifiableCredentialBase<QualificationCredentialV1Schema> {
    /**
     * @param credential 
     * @param options 
     */
    public constructor(credentialSubject: QualificationPerson, options?: UNiDVerifiableCredentialOptions) {
        super(options)

        this._credential = {
            '@context': [
                'https://www.w3.org/2018/credentials/v1',
                'https://docs.getunid.io/docs/2020/credentials/qualification',
            ],
            type: [ 'VerifiableCredential', 'QualificationCredentialV1' ],
            credentialSubject: credentialSubject,
        }
    }

    /**
     * @param input 
     */
    public static isCompatible(input: any): input is QualificationCredentialV1Schema & UNiDVerifiableCredentialMetadata {
        if (typeof input !== 'object') {
            return false
        }
        if (Object.keys(input).indexOf('type') < 0) {
            return false
        }
        if (Array.isArray(input.type) !== true) {
            return false
        }
        if (Array.from(input.type).indexOf('QualificationCredentialV1') < 0) {
            return false
        }
        return true
    }

    /**
     * @param input 
     */
    public static fromObject(input: any): QualificationCredentialV1 {
        if (! QualificationCredentialV1.isCompatible(input)) {
            throw new UNiDNotCompatibleError()
        }

        return new QualificationCredentialV1(input.credentialSubject)
    }

    /**
     * @param vp 
     */
    public static select<T>(vp: UNiDVerifiablePresentation<T>): QualificationCredentialV1Schema & UNiDVerifiableCredentialMetadata | undefined {
        const selected = vp.verifiableCredential.filter((vc) => {
            return QualificationCredentialV1.isCompatible(vc)
        })

        if (1 < selected.length) {
            throw new UNiDNotUniqueError()
        }

        const select = selected.shift()

        if (select === undefined) {
            return undefined
        }
        if (! QualificationCredentialV1.isCompatible(select)) {
            return undefined
        }

        return select
    }
}