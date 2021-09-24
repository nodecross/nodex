import { DIDDocument, ServiceEndpoint, DidPublicKey } from "./interfaces/did-document";

/**
 */
export class UNiDDidDocument {
    /**
     */
    private _document: DIDDocument

    /**
     * @param document 
     */
    constructor(document: DIDDocument) {
        this._document = document
    }

    /**
     */
    public get document(): DIDDocument {
        return this._document
    }

    /**
     */
    public get identifier(): string {
        return this._document.id
    }

    /**
     */
    public get publicKeys(): Array<DidPublicKey> {
        return this._document.publicKey
    }

    /**
     */
    public get services(): Array<ServiceEndpoint> {
        return this._document.service
    }

    /**
     * @param keyId 
     * @returns
     */
    public getPublicKey(keyId: string): DidPublicKey {
        const ks = Object.assign([], this.publicKeys) as Array<DidPublicKey>
        const k  = ks.filter((k) => { return k.id === `#${ keyId }` }).pop()

        if (! k) {
            throw new Error()
        }

        return k
    }
}