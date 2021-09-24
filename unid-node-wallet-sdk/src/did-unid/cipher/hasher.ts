import { Runtime } from '../../runtime'

/**
 */
interface RequestDigest {
    uri    : string,
    payload: string,
}

/**
 */
export class Hasher {
    public static readonly ALGORITHM: string = 'sha512'

    private constructor() {}

    /**
     * @param data 
     * @param secret 
     * @returns
     */
    public static digest(content: Buffer, secret: Buffer): string {
        return Runtime.HMAC.digest(content, secret, 'SHA512', 'HEX')
    }

    /**
     * @param data 
     * @param digest 
     * @param secret 
     * @returns
     */
    public static verify(content: Buffer, digest: Buffer, secret: Buffer): boolean {
        const _digest = Buffer.from(Runtime.HMAC.digest(content, secret, 'SHA512', 'HEX'), 'hex')

        return _digest.equals(digest)
    }

    /**
     * @param uri 
     * @param payload 
     * @param digest 
     * @param options 
     * @returns
     */
    public static verifyRequestDigest(uri: string, payload: string, digest: string, options: {
        clientSecret: string,
    }): boolean {
        const object: RequestDigest = {
            uri    : uri,
            payload: JSON.parse(payload),
        }
        const json = JSON.stringify(object, Object.keys(object).sort())

        return Hasher.verify(
            Buffer.from(json  , 'utf-8'),
            Buffer.from(digest, 'hex'),
            Buffer.from(options.clientSecret, 'utf-8')
        )
    }

    /**
     * 
     * @param uri 
     * @param payload 
     * @param options 
     * @returns
     */
    public static generateRequestDigest(uri: string, payload: string, options: {
        clientSecret: string,
    }): string {
        const object: RequestDigest = {
            uri    : uri,
            payload: JSON.parse(payload),
        }
        const json = JSON.stringify(object, Object.keys(object).sort())

        return Hasher.digest(
            Buffer.from(json, 'utf-8'),
            Buffer.from(options.clientSecret, 'utf-8')
        )
    }
}