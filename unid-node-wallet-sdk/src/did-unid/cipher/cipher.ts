import { Runtime } from '../../runtime'

/**
 */
export const IV_LENGTH: number   = 16 // 128 Bit

/**
 */
export const SALT_LENGTH: number = 32 // 256 Bit

/**
 */
export const PASS_LENGTH: number = 32 // 256 Bit


/**
 */
export class Cipher {
    /**
     */
    private constructor() {}

    /**
     * @param data 
     * @param secret 
     * @returns
     */
    public static async encrypt(content: Buffer, secret: Buffer): Promise<Buffer> {
        const salt = await Runtime.Commons.randomBytes(SALT_LENGTH)
        const iv   = await Runtime.Commons.randomBytes(IV_LENGTH)
        const key  = await Runtime.Scrypt.kdf(secret, salt, PASS_LENGTH)

        return Buffer.concat([ salt, Runtime.AES.encrypt(content, key, iv), iv ])
    }

    /**
     * @param data 
     * @param secret 
     * @returns
     */
    public static async decrypt(content: Buffer, secret: Buffer): Promise<Buffer> {
        if (content.length < (SALT_LENGTH + IV_LENGTH)) {
            throw new Error()
        }

        const salt = content.slice(0, SALT_LENGTH)
        const iv   = content.slice(-(IV_LENGTH))
        const data = content.slice(SALT_LENGTH, -(IV_LENGTH))
        const key  = await Runtime.Scrypt.kdf(secret, salt, PASS_LENGTH)

        return Runtime.AES.decrypt(data, key, iv)
    }
}