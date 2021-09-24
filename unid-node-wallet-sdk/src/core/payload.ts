import {
    DIDCreatePayload,
    DIDCreateRequest,
    DIDReplaceAction,
    DIDReplaceDeltaObject,
    DIDReplacePayload,
    DIDReplaceSuffixObject,
} from "./interfaces/did-operation";
import { Encoder } from "../utils/encoder";
import { Multihash } from "../utils/multihash";

/**
 * @param params 
 * @returns
 */
export const didCreatePayload = (params: DIDCreateRequest): DIDCreatePayload => {
    const document: DIDReplacePayload = {
        public_keys: params.publicKeys,
        service_endpoints: [],
    }
    const patch: DIDReplaceAction = {
        action: 'replace',
        document: document
    }

    const delta: DIDReplaceDeltaObject = {
        patches: [ patch ],
        update_commitment: Multihash.canonicalizeThenDoubleHashThenEncode(params.commitmentKeys.update),
    }
    const deltaBuffer = Buffer.from(JSON.stringify(delta))
    const deltaHash   = Encoder.encode(Multihash.hash(deltaBuffer))
    
    const suffixData: DIDReplaceSuffixObject = {
        delta_hash: deltaHash,
        recovery_commitment: Multihash.canonicalizeThenDoubleHashThenEncode(params.commitmentKeys.recovery)
    }
    const deltaEncodedString = Encoder.encode(deltaBuffer);
    const suffixDataEncodedString = Encoder.encode(JSON.stringify(suffixData));

    const payload: DIDCreatePayload = {
        type: 'create',
        delta: deltaEncodedString,
        suffix_data: suffixDataEncodedString,
    }

    return payload
}