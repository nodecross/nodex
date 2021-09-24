import { UNiDExportedVerifiablePresentationMetadata, UNiDVerifiableCredential, UNiDVerifiablePresentation, UNiDVerifiablePresentationContext, UNiDVerifiablePresentationMetadata, UNiDWithoutProofVerifiablePresentationMetadata } from "../schemas"
import { DateTimeUtils } from "../../utils/datetime"
import { CredentialSigner, ProofContext } from "../cipher/signer"
import { Secp256k1 } from "../keyring/secp256k1"
import { SIGNING_KEY_ID, UNiD } from '../../unid'
import { utils } from "../../utils/utils"

/**
 */
class VerifyContainer<T1> {
    private _object : UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, T1>> & UNiDVerifiablePresentationMetadata
    private _payload: UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, T1>> & UNiDWithoutProofVerifiablePresentationMetadata
    private _isValid: boolean

    constructor(
        object: UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, T1>> & UNiDVerifiablePresentationMetadata,
        validated: { payload: UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, T1>> & UNiDWithoutProofVerifiablePresentationMetadata, isValid: boolean }
    ) {
        this._object  = object
        this._payload = validated.payload
        this._isValid = validated.isValid
    }

    /**
     */
    public toJSON(): string {
        return JSON.stringify(this._object)
    }

    /**
     */
    public get isValid(): boolean {
        return this._isValid
    }

    /**
     */
    public get payload(): UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, T1>> & UNiDWithoutProofVerifiablePresentationMetadata {
        return this._payload
    }

    /**
     */
    public get metadata(): UNiDVerifiablePresentationContext & UNiDExportedVerifiablePresentationMetadata {
        const credentialTypes: Array<string> = this._payload.verifiableCredential
            .map((vc) => {
                return vc.type
            })
            .reduce((left, right) => {
                right.forEach((type) => {
                    // [TODO]: 'VerifiableCredential' should be a constant
                    if (type !== 'VerifiableCredential') {
                        if (left.indexOf(type) < 0) {
                            left.push(type)
                        }
                    }
                })
                return left
            }, [])

        const issuanceDate   = (new DateTimeUtils(this._payload.issuanceDate)).$toDate()
        const expirationDate = (new DateTimeUtils(this._payload.expirationDate)).toDate()

        const meta: UNiDVerifiablePresentationContext & UNiDExportedVerifiablePresentationMetadata = {
            '@context': this.payload["@context"],
            type      : this.payload.type,
            id        : this._payload.id,
            issuerDid : this._payload.issuer,
            issuanceDate   : issuanceDate,
            expirationDate : expirationDate,
            credentialTypes: credentialTypes,
        }

        return meta
    }
}

/**
 */
export class VerifiablePresentation<T1> {
    /**
     */
    private presentation: T1

    /**
     * @param presentation 
     */
    constructor(presentation: T1) {
        this.presentation = presentation
    }

    /**
     */
    public getVerifiablePresentation(): T1 {
        return this.presentation
    }

    /**
     * @param suite 
     * @returns
     */
    public async sign(suite: { did: string, keyId: string, context: Secp256k1 }): Promise<T1 & ProofContext> {
        return await CredentialSigner.sign(this.presentation, {
            did    : suite.did,
            keyId  : suite.keyId,
            context: suite.context,
        })
    }

    /**
     * @param presentation 
     * @returns
     */
    public static async verify<T1>(presentation: UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, T1>> & UNiDVerifiablePresentationMetadata): Promise<VerifyContainer<T1>> {
        if (presentation.proof === undefined) {
            throw new Error()
        }

        const vm  = utils.splitDid(presentation.proof.verificationMethod)
        const did = await UNiD.getDidDocument({
            did: vm.did,
        })

        const validated = await CredentialSigner.verify(presentation, {
            keyId  : SIGNING_KEY_ID,
            context: Secp256k1.fromJwk(did.getPublicKey(SIGNING_KEY_ID).publicKeyJwk),
        })

        return new VerifyContainer(presentation, validated)
    }

    /**
     * @param vcs 
     * @param checker 
     * @returns
     */
    public static filter<T1>(vcs: Array<any>, checker: (input: any) => boolean): Array<T1> {
        return vcs.filter((vc) => {
            return checker(vc)
        }) as Array<T1>
    }
}