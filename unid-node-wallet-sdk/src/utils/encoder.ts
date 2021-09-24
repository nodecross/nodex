import base64url from 'base64url'

/**
 */
export class Encoder {
    /**
     * @param content 
     * @returns
     */
    public static encode (content: Buffer | string): string {
        return base64url.encode(content)
    }

    /**
     * @param encodedContent 
     * @returns
     */
    public static decodeAsBuffer (encodedContent: string): Buffer {
        Encoder.validateBase64UrlString(encodedContent)

        return base64url.toBuffer(encodedContent)
    }

    /**
     * @param encodedContent 
     * @returns
     */
    public static decodeAsString (encodedContent: string): string {
        return Encoder.decodeBase64UrlAsString(encodedContent)
    }

    /**
     * @param input 
     * @returns
     */
    public static decodeBase64UrlAsString (input: string): string {
        Encoder.validateBase64UrlString(input)

        return base64url.decode(input)
    }

    /**
     * @param input 
     */
    private static validateBase64UrlString (input: any) {
        if (typeof input !== 'string') {
            throw new Error()
        }

        const isBase64UrlString = Encoder.isBase64UrlString(input)

        if (! isBase64UrlString) {
            throw new Error()
        }
    }

    /**
     * @param input 
     * @returns
     */
    public static isBase64UrlString (input: string): boolean {
        return (new RegExp('^[A-Za-z0-9_-]+$')).test(input)
    }
}