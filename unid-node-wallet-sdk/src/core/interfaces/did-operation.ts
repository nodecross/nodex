import { DIDDocument } from "./did-document";
import { KeyPairSecp256K1, PublicKeyType } from "./key-pair";

export interface PublicKeyPayload {
    id  : string,
    type: PublicKeyType,
    jwk : KeyPairSecp256K1,
    purpose: Array<string>,
}

// ACTION: add-public-keys
export interface DIDAddPublicKeysPayload extends PublicKeyPayload {
}

export interface DIDAddPublicKeysAction {
    action     : 'add-public-keys',
    public_keys: Array<DIDAddPublicKeysPayload>
}

// ACTION: remove-public-keys
export interface DIDRemovePublicKeysAction {
    action: 'remove-public-keys',
    ids   : Array<string>,
}

// ACTION: add-services
export interface DIDAddServicesPayload {
}

export interface DIDAddServicesAction {
    action  : 'add-services',
    services: Array<DIDAddServicesPayload>,
}

// ACTION: remove-services
export interface DIDRemoveServicesAction {
    action: 'remove-services',
    ids   : Array<string>,
}

// ACTION: replace
export interface DIDReplacePayload {
    public_keys: Array<PublicKeyPayload>,
    service_endpoints: Array<string>,
}

export interface DIDReplaceAction {
    action  : 'replace',
    document: DIDReplacePayload,
}

export interface DIDReplaceDeltaObject {
    patches: Array<DIDReplaceAction>,
    update_commitment: string,
}

export interface DIDReplaceSuffixObject {
    delta_hash: string,
    recovery_commitment: string,
}

// ACTION: ietf-json-patch
export interface DIDIetfJsonPatchAction {
    action : 'replace',
    patches: Array<any>
}

export interface DIDResolutionRequest {
    did: string
}

export interface DIDResolutionResponse {
    '@context': 'https://www.w3.org/ns/did-resolution/v1',
    didDocument: DIDDocument,
    methodMetadata: {
        published: boolean,
        recoveryCommitment: string,
        updateCommitment: string,
    },
}

export interface DIDCreateRequest {
    publicKeys: Array<PublicKeyPayload>,
    commitmentKeys: {
        recovery: KeyPairSecp256K1,
        update  : KeyPairSecp256K1,
    },
    serviceEndpoints: Array<string>,
}

export interface DIDCreatePayload {
    type: 'create',
    delta: string,
    suffix_data: string,
}

export interface DIDCreateResponse {
    '@context': 'https://www.w3.org/ns/did-resolution/v1',
    didDocument: DIDDocument,
    methodMetadata: {
        published: boolean,
        recoveryCommitment: string,
        updateCommitment: string,
    },
}

export interface DIDUpdateRequest {
    // NOT IMPLEMENTED
}

export interface DIDUpdateResponse {
    // NOT IMPLEMENTED
}

export interface DIDRecoverRequest {
    // NOT IMPLEMENTED
}

export interface DIDRecoverResponse {
    // NOT IMPLEMENTED
}

export interface DIDDeactivateRequest {
    // NOT IMPLEMENTED
}

export interface DIDDeactivateResponse {
    // NOT IMPLEMENTED
}