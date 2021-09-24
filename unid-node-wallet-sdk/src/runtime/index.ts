import multihashes, { HashCode } from 'multihashes'
import CryptoJS from 'crypto-js'
import secp256k1 from 'secp256k1'
import * as bip32 from 'bip32'
import * as bip39 from 'bip39'
import crypto from 'crypto'
import { UNiDInvalidDataError, UNiDNotImplementedError } from '../error'

type OutputEncodingTypes =
    | 'HEX'

type HmacAlgorithmTypes =
    | 'SHA512'

export interface BIP32Interface {
    publicKey  : Buffer
    privateKey?: Buffer
}

export class Runtime {
    private constructor () {}

    public static get Multihashes() {
        return new class {
            /**
             * @param digest 
             * @param code 
             * @returns
             */
            public encode(digest: Uint8Array, code: number): Uint8Array {
                const _code = code as HashCode

                return multihashes.encode(digest, _code)
            }

            /**
             * @param bytes 
             * @returns
             */
            public decode(bytes: Uint8Array): {
                digest: Uint8Array,
                code  : number,
            } {
                return multihashes.decode(bytes)
            }
        }()
    }

    public static get SHA256() {
        return new class {
            /**
             * @param content 
             * @param encode 
             * @returns
             */
            public digest (content: Buffer, encode: OutputEncodingTypes): string {
                const buffer = CryptoJS.enc.Hex.parse(content.toString('hex'))
                const cipher = CryptoJS.SHA256(buffer)

                switch (encode) {
                    case 'HEX': {
                        return cipher.toString(CryptoJS.enc.Hex)
                    }
                    default: {
                        throw new UNiDNotImplementedError()
                    }
                }
            }
        }()
    }

    public static get Secp256k1() {
        return new class {
            /**
             * @param publicKey 
             * @param compressed 
             * @returns
             */
            public publicKeyConvert (publicKey: Uint8Array, compressed: boolean): Uint8Array {
                return secp256k1.publicKeyConvert(publicKey, compressed)
            }

            /**
             * @param message 
             * @param privateKey 
             * @returns
             */
            public async ecdsaSign (message: Uint8Array, privateKey: Uint8Array): Promise<Uint8Array> {
                return secp256k1.ecdsaSign(message, privateKey).signature
            }

            /**
             * @param signature 
             * @param message 
             * @param publicKey 
             * @returns
             */
            public async ecdsaVerify (signature: Uint8Array, message: Uint8Array, publicKey: Uint8Array): Promise<boolean> {
                return secp256k1.ecdsaVerify(signature, message, publicKey)
            }
        }()
    }

    public static get BIP39() {
        return new class {
            /**
             * @param strength 
             * @returns
             */
            public async generateMnemonic (strength?: number | undefined): Promise<string> {
                return bip39.generateMnemonic(strength)
            }

            /**
             * @param mnemonic 
             * @param password 
             * @returns
             */
            public async mnemonicToSeed (mnemonic: string, password?: string | undefined): Promise<Buffer> {
                return await bip39.mnemonicToSeed(mnemonic, password)
            }
        }()
    }

    public static get BIP32() {
        return new class {
            /**
             * @param seed 
             * @param derivationPath 
             * @returns
             */
            public getNode (seed: Buffer, derivationPath: string): BIP32Interface {
                const root = bip32.fromSeed(seed)
                const node = root.derivePath(derivationPath)

                return node
            }
        }()
    }

    public static get Commons() {
        return new class {
            /**
             * @param size 
             * @returns
             */
            public async randomBytes(size: number): Promise<Buffer> {
                const bytes = CryptoJS.lib.WordArray.random(size)
                const hex   = bytes.toString(CryptoJS.enc.Hex)

                return Buffer.from(hex, 'hex')
            }
        }()
    }

    public static get Scrypt() {
        return new class {
            /**
             * @param secret 
             * @param salt 
             * @param keylen 
             * @returns
             */
            public async kdf (secret: Buffer, salt: Buffer, keylen: number): Promise<Buffer> {
                return new Promise<Buffer>((resolve, reject) => {
                    crypto.scrypt(secret, salt, keylen, (err, key) => {
                        if (err) {
                            return reject(err)
                        }
                        return resolve(key)
                    })
                })
            }
        }()
    }

    public static get AES() {
        return new class {
            /**
             * @param content 
             * @param secret 
             * @param iv 
             * @returns
             */
            public encrypt (content: Buffer, secret: Buffer, iv: Buffer): Buffer {
                const _content = CryptoJS.enc.Hex.parse(content.toString('hex'))
                const _secret  = CryptoJS.enc.Hex.parse(secret.toString('hex'))
                const _iv      = CryptoJS.enc.Hex.parse(iv.toString('hex'))

                const cipher = CryptoJS.AES.encrypt(_content, _secret, {
                    iv  : _iv,
                    mode: CryptoJS.mode.CBC,
                });

                return Buffer.from(cipher.toString(CryptoJS.format.Hex), 'hex')
            }

            /**
             * @param content 
             * @param secret 
             * @param iv 
             * @returns
             */
            public decrypt (content: Buffer, secret: Buffer, iv: Buffer): Buffer {
                const _content = CryptoJS.enc.Hex.parse(content.toString('hex'));
                const _text    = CryptoJS.enc.Base64.stringify(_content);
                const _secret  = CryptoJS.enc.Hex.parse(secret.toString('hex'))
                const _iv      = CryptoJS.enc.Hex.parse(iv.toString('hex'))

                const cipher = CryptoJS.AES.decrypt(_text, _secret, {
                    iv  : _iv,
                    mode: CryptoJS.mode.CBC,
                })

                // WORKAROUND:
                // refs: https://github.com/brix/crypto-js/issues/158
                if (cipher.toString(CryptoJS.enc.Utf8) === '') {
                    throw new UNiDInvalidDataError()
                }

                return Buffer.from(cipher.toString(CryptoJS.enc.Hex), 'hex')
            }
        }()
    }

    public static get HMAC() {
        return new class {
            /**
             * @param content 
             * @param secret 
             * @param algorithm 
             * @param encode 
             * @returns
             */
            public digest (content: Buffer, secret: Buffer, algorithm: HmacAlgorithmTypes, encode: OutputEncodingTypes): string {
                const buffer = CryptoJS.enc.Hex.parse(content.toString('hex'))
                const key    = CryptoJS.enc.Hex.parse(secret.toString('hex'))

                switch (algorithm) {
                    case 'SHA512': {
                        const cipher = CryptoJS.HmacSHA512(buffer, key)

                        switch (encode) {
                            case 'HEX': {
                                return cipher.toString(CryptoJS.enc.Hex)
                            }
                            default: {
                                throw new UNiDNotImplementedError()
                            }
                        }
                    }
                    default: {
                        throw new UNiDNotImplementedError()
                    }
                }
            }
        }()
    }
}