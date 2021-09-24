import { PostalAddress } from './schema.org'
import { UNiDNotCompatibleError, UNiDNotUniqueError } from '../../error'
import { UNiDCredentialSubjectMetadata, UNiDVerifiableCredential, UNiDVerifiableCredentialBase, UNiDVerifiableCredentialMetadata, UNiDVerifiableCredentialOptions, UNiDVerifiablePresentation } from '.';

// AddressCredentialV1

/**
 */
interface AddressPerson extends UNiDCredentialSubjectMetadata {
    '@type': 'AddressPerson',
    address: PostalAddress,
}

/**
 */
interface AddressOrganization extends UNiDCredentialSubjectMetadata {
    '@type': 'AddressOrganization',
    address: PostalAddress,
}

/**
 */
export interface AddressCredentialV1Schema extends UNiDVerifiableCredential<
    'https://docs.getunid.io/docs/2020/credentials/address',
    'AddressCredentialV1',
    AddressPerson | AddressOrganization
> {}

/**
 */
export class AddressCredentialV1 extends UNiDVerifiableCredentialBase<AddressCredentialV1Schema> {
    /**
     * @param credentialSubject 
     * @param options 
     */
    public constructor(credentialSubject: AddressPerson | AddressOrganization, options?: UNiDVerifiableCredentialOptions) {
        super(options)

        this._credential = {
            '@context': [
                'https://www.w3.org/2018/credentials/v1',
                'https://docs.getunid.io/docs/2020/credentials/address',
            ],
            type: [ 'VerifiableCredential', 'AddressCredentialV1' ],
            credentialSubject: credentialSubject,
        }
    }

    /**
     * @param input 
     */
    public static isCompatible(input: any): input is AddressCredentialV1Schema & UNiDVerifiableCredentialMetadata {
        if (typeof input !== 'object') {
            return false
        }
        if (Object.keys(input).indexOf('type') < 0) {
            return false
        }
        if (Array.isArray(input.type) !== true) {
            return false
        }
        if (Array.from(input.type).indexOf('AddressCredentialV1') < 0) {
            return false
        }
        return true
    }

    /**
     * @param input 
     */
    public static fromObject(input: any): AddressCredentialV1 {
        if (! AddressCredentialV1.isCompatible(input)) {
            throw new UNiDNotCompatibleError()
        }

        return new AddressCredentialV1(input.credentialSubject)
    }

    /**
     * @param vp 
     */
    public static select<T>(vp: UNiDVerifiablePresentation<T>): AddressCredentialV1Schema & UNiDVerifiableCredentialMetadata | undefined {
        const selected = vp.verifiableCredential.filter((vc) => {
            return AddressCredentialV1.isCompatible(vc)
        })

        if (1 < selected.length) {
            throw new UNiDNotUniqueError()
        }

        const select = selected.shift()

        if (select === undefined) {
            return undefined
        }
        if (! AddressCredentialV1.isCompatible(select)) {
            return undefined
        }

        return select
    }
}