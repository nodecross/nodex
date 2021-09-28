import { Text } from '../schema.org'
import { UNiDNotCompatibleError, UNiDNotUniqueError } from '../../../error'
import { UNiDCredentialSubjectMetadata, UNiDVerifiableCredential, UNiDVerifiableCredentialBase, UNiDVerifiableCredentialMetadata, UNiDVerifiableCredentialOptions, UNiDVerifiableCredentialTypes, UNiDVerifiablePresentation, UNiDVerifiablePresentationMetadata } from '..';
import { KeyPairSecp256K1 } from '../../../core';

// UNiDAuthCredentialV1

/**
 */
export interface UNiDAuthnClaims {
    required: Array<UNiDVerifiableCredentialTypes>,
    optional: Array<UNiDVerifiableCredentialTypes>,
}

/**
 */
export interface AuthnRequest extends UNiDCredentialSubjectMetadata {
    '@type': 'AuthnRequest',
    iss: Readonly<Text>,
    kid: Readonly<Text>,
    scope: Readonly<'did_authn'>,
    registration: Readonly<{}>,
    client_id: Readonly<Text>,
    claims: Readonly<UNiDAuthnClaims>,
    response_mode?: Readonly<'fragment' | 'form_post'>,
    response_context?: Readonly<Text>,
}

/**
 */
export interface AuthnResponse extends UNiDCredentialSubjectMetadata {
    '@type': 'AuthnResponse',
    did: Readonly<Text>,
    sub_jwk: Readonly<KeyPairSecp256K1>,
    verifiablePresentation: Readonly<UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, UNiDCredentialSubjectMetadata>> & UNiDVerifiablePresentationMetadata>,
}

/**
 */
export type UNiDAuthCredentialV1Types =
    | AuthnRequest
    | AuthnResponse

/**
 */
export interface UNiDAuthCredentialV1Schema extends UNiDVerifiableCredential<
    'https://docs.getunid.io/docs/2020/credentials/internal/unid-auth',
    'UNiDAuthCredentialV1',
    UNiDAuthCredentialV1Types
> {}

/**
 */
export class UNiDAuthCredentialV1 extends UNiDVerifiableCredentialBase<UNiDAuthCredentialV1Schema> {
    /**
     * @param credentialSubject 
     * @param options 
     */
    public constructor(credentialSubject: AuthnRequest | AuthnResponse, options?: UNiDVerifiableCredentialOptions) {
        super(options)

        this._credential = {
            '@context': [
                'https://www.w3.org/2018/credentials/v1',
                'https://docs.getunid.io/docs/2020/credentials/internal/unid-auth',
            ],
            type: [ 'VerifiableCredential', 'UNiDAuthCredentialV1' ],
            credentialSubject: credentialSubject,
        }
    }

    /**
     * @param input 
     */
    public static isCompatible(input: any): input is UNiDAuthCredentialV1Schema & UNiDVerifiableCredentialMetadata {
        if (typeof input !== 'object') {
            return false
        }
        if (Object.keys(input).indexOf('type') < 0) {
            return false
        }
        if (Array.isArray(input.type) !== true) {
            return false
        }
        if (Array.from(input.type).indexOf('UNiDAuthCredentialV1') < 0) {
            return false
        }
        return true
    }

    /**
     * @param input 
     */
    public static fromObject(input: any): UNiDAuthCredentialV1 {
        if (! UNiDAuthCredentialV1.isCompatible(input)) {
            throw new UNiDNotCompatibleError()
        }

        return new UNiDAuthCredentialV1(input.credentialSubject)
    }

    /**
     * @param vp 
     */
    public static select<T>(vp: UNiDVerifiablePresentation<T>): UNiDAuthCredentialV1Schema & UNiDVerifiableCredentialMetadata | undefined {
        const selected = vp.verifiableCredential.filter((vc) => {
            return UNiDAuthCredentialV1.isCompatible(vc)
        })

        if (1 < selected.length) {
            throw new UNiDNotUniqueError()
        }

        const select = selected.shift()

        if (select === undefined) {
            return undefined
        }
        if (! UNiDAuthCredentialV1.isCompatible(select)) {
            return undefined
        }

        return select
    }
}