import { KeyPairSecp256K1, PublicKeyType } from "./key-pair";

export interface ServiceEndpoint {
    id: string
    type: string
    serviceEndpoint: string
    description?: string
}
  
export interface DidPublicKey {
    id: string
    controller: string
    type: PublicKeyType,
    publicKeyJwk: KeyPairSecp256K1
}
  
export interface Authentication {
    type: string
    publicKey: string
}

export interface DIDDocument {
    '@context': 'https://www.w3.org/ns/did/v1' | string | { [ key: string ]: string }
    id: string,
    publicKey: DidPublicKey[]
    service: ServiceEndpoint[]
    authentication: (string | DidPublicKey | Authentication)[]
}