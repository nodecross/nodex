import base64url from 'base64url'
import lodash from 'lodash'
import { Runtime } from '../../runtime'
import { Secp256k1 as Context } from "../keyring/secp256k1";
import { DateTimeTypes, DateTimeUtils } from "../../utils/datetime";
import { utils } from "../../utils/utils";
import { UNiDNotCompatibleError } from "../../error";

interface JwsHeader {
    alg: 'ES256K',
    b64: boolean,
    crit: Array<'b64'>
}

export interface ProofContext {
    proof?: {
        type: 'EcdsaSecp256k1Signature2019' | string,
        proofPurpose: 'authentication' | string,
        created: string,
        verificationMethod: string,
        jws: string
        controller?: string,
        challenge?: string,
        domain?: string,
    }
}

/**
 */
export class CredentialSigner {
    /**
     */
    private static readonly PROOF_KEY: string = 'proof'

    /**
     */
    private constructor() {}

    /**
     * @param object 
     * @param suite 
     * @returns
     */
    public static async sign<T>(object: T, suite: {
        did    : string,
        keyId  : string,
        context: Context,
    }): Promise<T & ProofContext> {
        if (Object.keys(object).indexOf(this.PROOF_KEY) !== -1) {
            throw new Error()
        }

        const created = (new DateTimeUtils(new Date())).$toString(DateTimeTypes.default)
        const jws = await Jws.encode(object, suite.context)
        const proof: ProofContext = {
            proof: {
                type: 'EcdsaSecp256k1Signature2019',
                proofPurpose: 'authentication',
                created: created,
                verificationMethod: `${ suite.did }#${ suite.keyId }`,
                jws: jws,
            }
        }

        // Sign
        const signedObject = lodash.merge(proof, object)

        return signedObject
    }

    /**
     * @param object 
     * @param suite 
     * @returns
     */
    public static async verify<T>(object: T & ProofContext, suite: {
        keyId  : string,
        context: Context,
    }): Promise<{ payload: T, isValid: boolean }> {
        if (Object.keys(object).indexOf(this.PROOF_KEY) === -1) {
            throw new Error()
        }

        const proof = object.proof

        if (proof === undefined) {
            throw new UNiDNotCompatibleError()
        }

        const vm = utils.splitDid(proof.verificationMethod)

        if (vm.keyId !== suite.keyId) {
            throw new UNiDNotCompatibleError()
        }

        const jws     = proof.jws
        const payload = lodash.omit(object, [ this.PROOF_KEY ]) as T

        // Verify
        const isValid = await Jws.verify(payload, jws, suite.context)

        return {
            payload: payload,
            isValid: isValid,
        }
    }
}

/**
 */
export class Jws {
    /**
     */
    private constructor() {}

    /**
     * @param object 
     * @param context 
     * @returns
     */
    public static async encode(object: any, context: Context): Promise<string> {
        // Header
        const header: JwsHeader = {
            alg : 'ES256K',
            b64 : false,
            crit: [ 'b64' ]
        }
        const _header = base64url.encode(
            Buffer.from(JSON.stringify(header), 'utf-8')
        )

        // Payload
        const _payload = base64url.encode(
            Buffer.from(JSON.stringify(object), 'utf-8')
        )

        // Message
        const message = [ _header, _payload ].join('.')

        // Signature
        const signature  = await Signer.sign(Buffer.from(message, 'utf-8'), context)
        const _signature = base64url.encode(signature)

        return [ _header, '', _signature ].join('.')
    }

    /**
     * @param object 
     * @param jws 
     * @param context 
     * @returns
     */
    public static async verify(object: any, jws: string, context: Context): Promise<boolean> {
        const [ _header, __payload, _signature ] = jws.split('.')

        if ((_header == undefined) || (__payload == undefined) || (_signature == undefined)) {
            throw new Error()
        }

        // Header
        const header: JwsHeader = JSON.parse(base64url.decode(_header))

        if (header.alg !== 'ES256K') {
            throw new Error()
        }
        if (header.b64 !== false) {
            throw new Error()
        }
        if (header.crit.indexOf('b64') === -1) {
            throw new Error()
        }

        // Payload
        if (__payload !== '') {
            throw new Error()
        }

        const _payload = base64url.encode(
            Buffer.from(JSON.stringify(object), 'utf-8')
        )

        // Message
        const message = [ _header, _payload ].join('.')

        // Signature
        const signature = Buffer.from(base64url.toBuffer(_signature))

        // Verify
        return await Signer.verify(Buffer.from(message), signature, context)
    }
}

/**
 */
export class Signer {
    /**
     */
    private constructor() {}

    /**
     * @param message 
     * @param context 
     * @returns
     */
    public static async sign(message: Buffer, context: Context): Promise<Buffer> {
        const payload   = Buffer.from(JSON.stringify(message), 'utf-8')
        const digest    = Buffer.from(Runtime.SHA256.digest(payload, 'HEX'), 'hex')
        const signature = await Runtime.Secp256k1.ecdsaSign(
            Uint8Array.from(digest),
            Uint8Array.from(context.getPrivateKey()),
        )

        return Buffer.from(signature)
    }

    /**
     * @param message 
     * @param signature 
     * @param context 
     * @returns
     */
    public static async verify(message: object, signature: Buffer, context: Context): Promise<boolean> {
        const payload = Buffer.from(JSON.stringify(message), 'utf-8')
        const digest  = Buffer.from(Runtime.SHA256.digest(payload, 'HEX'), 'hex')
        const verify  = await Runtime.Secp256k1.ecdsaVerify(
            Uint8Array.from(signature),
            Uint8Array.from(digest),
            Uint8Array.from(context.getPublicKey())
        )

        return verify
    }
}