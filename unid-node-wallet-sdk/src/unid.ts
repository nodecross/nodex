import { UNiDDidOperator, PublicKeyPurpose, UNiDDidDocument } from './core'
import { MongoDBConnector } from '@unid/wallet-sdk-mongo-connector'
import { MongoClient } from 'mongodb'
import { UNiDDid } from './did-unid/did'
import { VerifiableCredential } from './did-unid/did/credential'
import { VerifiablePresentation } from './did-unid/did/presentation'
import { UNiDInvalidSignatureError, UNiDNotCompatibleError, UNiDNotImplementedError } from "./error"
import { KeyRingType } from './did-unid/keyring'
import { MnemonicKeyring, MnemonicKeyringOptions } from './did-unid/keyring/mnemonic'
import {
    UNiDCredentialSubjectMetadata,
    UNiDExportedVerifiableCredentialMetadata,
    UNiDExportedVerifiablePresentationMetadata,
    UNiDVerifiableCredential,
    UNiDVerifiableCredentialMetadata,
    UNiDVerifiablePresentation,
    UNiDVerifiablePresentationMetadata,
    UNiDWithoutProofVerifiableCredentialMetadata,
    UNiDWithoutProofVerifiablePresentationMetadata,
    UNiDVerifiableCredentialTypes,
} from './did-unid/schemas'
import { UNiDAuthCredentialV1 } from './did-unid/schemas/internal/unid-auth'
import { promise } from './utils/promise'
import { Cipher } from './did-unid/cipher/cipher'

/**
 */
export enum UNiDNetworkType {
    Mainnet,
    Testnet
}

/**
 */
interface UNiDNetwork {
    envNetwork? : UNiDNetworkType
}

/**
 */
interface UNiDAPIClient {
    clientId    : string,
    clientSecret: string,
}

/**
 */
interface UNiDPersistentStoreExternal {
    localStorage : MongoClient,
    encryptionKey: string,
}

/**
 */
interface UNiDPersistentStoreInternal {
    connector: MongoDBConnector
}

/**
 */
interface UNiDContextExternal extends UNiDNetwork, UNiDAPIClient, UNiDPersistentStoreExternal {
}

/**
 */
export interface UNiDContextInternal extends UNiDNetwork, UNiDAPIClient, UNiDPersistentStoreInternal {
}

/**
 */
export const SIGNING_KEY_ID = 'signingKey'

/**
 */
export interface UNiDVerifyCredentialResponse<T1, T2, T3> {
    isValid : boolean,
    payload : UNiDVerifiableCredential<T1, T2, T3> & UNiDWithoutProofVerifiableCredentialMetadata,
    metadata: UNiDExportedVerifiableCredentialMetadata
    toJSON(): string
}

/**
 */
export interface UNiDVerifyPresentationResponse<T1> {
    isValid : boolean,
    payload : UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, T1>> & UNiDWithoutProofVerifiablePresentationMetadata
    metadata: UNiDExportedVerifiablePresentationMetadata,
    toJSON(): string
}

/**
 */
class UNiDKlass {
    /**
     */
    private readonly operator: UNiDDidOperator

    /**
     */
    private context?: UNiDContextInternal

    /**
     * @param context 
     */
    public constructor() {
        this.operator = new UNiDDidOperator()
    }

    /**
     * @param context 
     */
    public init(context: UNiDContextExternal) {
        const connector = new MongoDBConnector({
            client   : context.localStorage,
            encrypter: Cipher.encrypt,
            decrypter: Cipher.decrypt,
            encryptionKey: context.encryptionKey,
        })

        this.context = {
            clientId    : context.clientId,
            clientSecret: context.clientSecret,
            connector   : connector,
            envNetwork  : context.envNetwork,
        }
    }

    /**
     * @returns
     */
    private getConnector(): MongoDBConnector {
        if (this.context === undefined) {
            throw new UNiDNotImplementedError()
        }

        return this.context.connector
    }

    /**
     * @returns
     */
    private getContext(): UNiDContextInternal {
        if (this.context === undefined) {
            throw new UNiDNotImplementedError()
        }

        return this.context
    }

    /**
     * @param params 
     * @returns
     */
    public async loadDid(params: { did: string }): Promise<UNiDDid> {
        const keyring = await MnemonicKeyring.loadKeyring(this.getConnector(), params.did)

        return new UNiDDid({
            context : this.getContext(),
            keyring : keyring,
            operator: this.operator,
        })
    }

    /**
     * @param type 
     * @param options 
     */
    public async createDid(type: KeyRingType.Mnemonic, options?: MnemonicKeyringOptions): Promise<UNiDDid>
    public async createDid(type: KeyRingType, options?: MnemonicKeyringOptions): Promise<UNiDDid> {
        switch (type) {
            case KeyRingType.Mnemonic: {
                const mnemonicOptions = options as MnemonicKeyringOptions
                const keyring  = await MnemonicKeyring.createKeyring(this.getConnector(), mnemonicOptions)
                const document = await this.operator.create({
                    publicKeys: [
                        keyring.getSignKeyPair().toPublicKey(SIGNING_KEY_ID, Object.values(PublicKeyPurpose))
                    ],
                    commitmentKeys: {
                        update  : keyring.getUpdateKeyPair().toJwk(),
                        recovery: keyring.getRecoveryKeyPair().toJwk(),
                    },
                    serviceEndpoints: []
                })

                await keyring.setDid(document.identifier)

                return new UNiDDid({
                    context : this.getContext(),
                    keyring : keyring,
                    operator: this.operator,
                })
            }
            default: {
                throw new Error()
            }
        }
    }

    /**
     * @param params 
     * @returns
     */
    public async getDidDocument(params: { did: string }): Promise<UNiDDidDocument> {
        return await this.operator.resolve(params)
    }

    /**
     */
    public async updateDidDocument() {
        throw new UNiDNotImplementedError()
    }

    /**
     * @param credential 
     * @returns
     */
    public async verifyCredential<T1, T2, T3>(credential: UNiDVerifiableCredential<T1, T2, T3> & UNiDVerifiableCredentialMetadata): Promise<UNiDVerifyCredentialResponse<T1, T2, T3>> {
        if (! this.isVerifiableCredential<T1, T2, T3>(credential)) {
            throw new UNiDNotCompatibleError()
        }

        return await VerifiableCredential.verify(credential)
    }

    /**
     * @param presentation 
     * @returns
     */
    public async verifyPresentation<T1>(presentation: UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, T1>> & UNiDVerifiablePresentationMetadata): Promise<UNiDVerifyPresentationResponse<T1>> {
        if (! this.isVerifiablePresentation<T1>(presentation)) {
            throw new UNiDNotCompatibleError()
        }

        return await VerifiablePresentation.verify(presentation)
    }

    /**
     * @param input 
     * @returns
     */
    public isVerifiableCredential<T1 = string, T2 = string, T3 = UNiDCredentialSubjectMetadata>(input: any): input is UNiDVerifiableCredential<T1, T2, T3> & UNiDVerifiableCredentialMetadata {
        if (typeof(input) !== 'object') {
            return false
        }

        if ((Object.keys(input).indexOf('@context') < 0) ||
            (Object.keys(input).indexOf('type') < 0) ||
            (Object.keys(input).indexOf('credentialSubject') < 0) ||
            (Object.keys(input).indexOf('proof') < 0)
        ) {
            return false
        }

        return true
    }

    /**
     * @param input 
     * @returns
     */
    public isVerifiablePresentation<T1 = object>(input: any): input is UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, T1>> & UNiDVerifiablePresentationMetadata {
        if (typeof(input) !== 'object') {
            return false
        }

        if ((Object.keys(input).indexOf('@context') < 0) ||
            (Object.keys(input).indexOf('type') < 0) ||
            (Object.keys(input).indexOf('verifiableCredential') < 0) ||
            (Object.keys(input).indexOf('proof') < 0)
        ) {
            return false
        }

        return true
    }

    /**
     * @param payload 
     * @returns
     */
    private async validateAuthentication<T1>(payload: UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, T1>> & UNiDVerifiablePresentationMetadata) {
        const verifiedVP = await this.verifyPresentation(payload)

        if (! verifiedVP.isValid) {
            throw new UNiDInvalidSignatureError()
        }

        const selectedVC = UNiDAuthCredentialV1.select(verifiedVP.payload)

        if (selectedVC === undefined) {
            throw new UNiDNotCompatibleError()
        }

        if (! UNiDAuthCredentialV1.isCompatible(selectedVC)) {
            throw new UNiDNotCompatibleError()
        }

        const verifiedVC = await this.verifyCredential(selectedVC)

        if (! verifiedVC.isValid) {
            throw new UNiDInvalidSignatureError()
        }

        return verifiedVC.payload.credentialSubject
    }

    /**
     * @param request 
     * @returns
     */
    public async validateAuthenticationRequest<T1>(request: UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, T1>> & UNiDVerifiablePresentationMetadata) {
        const subject = await this.validateAuthentication(request)

        if (subject['@type'] !== 'AuthnRequest') {
            throw new UNiDNotCompatibleError()
        }

        return subject.claims
    }

    /**
     * @param response 
     * @returns
     */
    public async validateAuthenticationResponse<T1>(response: UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, T1>> & UNiDVerifiablePresentationMetadata, options: {
        required: Array<UNiDVerifiableCredentialTypes>,
        optional: Array<UNiDVerifiableCredentialTypes>,
    }) {
        const subject = await this.validateAuthentication(response)

        if (subject['@type'] !== 'AuthnResponse') {
            throw new UNiDNotCompatibleError()
        }

        const verifiedInnerVP = await this.verifyPresentation(subject.verifiablePresentation)

        if (! verifiedInnerVP.isValid) {
            throw new UNiDInvalidSignatureError()
        }

        // Verify: Signatures
        await promise.all(verifiedInnerVP.payload.verifiableCredential, async (item, _) => {
            if (! this.isVerifiableCredential(item)) {
                throw new UNiDNotCompatibleError()
            }

            const vc = await this.verifyCredential(item)

            if (! vc.isValid) {
                throw new UNiDInvalidSignatureError()
            }

            return vc
        })

        // Verify: Types
        options.required.forEach((type) => {
            if (verifiedInnerVP.metadata.credentialTypes.indexOf(type) < 0) {
                throw new UNiDNotCompatibleError()
            }
        })

        return verifiedInnerVP
    }
}

export const UNiD = new UNiDKlass()