import { Connector, Id, MnemonicKeyringModel } from '@unid/wallet-sdk-base-connector'
import { Secp256k1 } from './secp256k1'
import { BIP32Interface, Runtime } from '../../runtime'

interface BIP39Context {
    seed: Buffer,
    mnemonic?: string,
}

type BIP39PhraseSize =
    | 12 // ENT = 128, CS = 4, ENT + CS = 132, MS = 12
    | 15 // ENT = 160, CS = 5, ENT + CS = 165, MS = 15
    | 18 // ENT = 192, CS = 6, ENT + CS = 198, MS = 18
    | 21 // ENT = 224, CS = 7, ENT + CS = 231, MS = 21
    | 24 // ENT = 256, CS = 8, ENT + CS = 264, MS = 24

const BIP39DefaultPhraseSize: BIP39PhraseSize = 24

export interface MnemonicKeyringOptions {
    length: BIP39PhraseSize
}

interface MnemonicKeyringContext {
    connector: Connector
    context  : BIP39Context
    sign     : Secp256k1
    update   : Secp256k1
    recovery : Secp256k1
    encrypt  : Secp256k1
}

interface SaveContextOptions {
    removeMnemonic?: boolean
}

/**
 */
export class MnemonicKeyring {
    private static readonly baseDerivationPath: string = 'm/44\'/0\'/0\'/0'

    public static readonly signDerivationPath: string     = `${ MnemonicKeyring.baseDerivationPath }/10`
    public static readonly updateDerivationPath: string   = `${ MnemonicKeyring.baseDerivationPath }/20`
    public static readonly recoveryDerivationPath: string = `${ MnemonicKeyring.baseDerivationPath }/30`
    public static readonly encryptDerivationPath: string  = `${ MnemonicKeyring.baseDerivationPath }/40`

    private connector: Connector
    private context  : BIP39Context
    private sign     : Secp256k1
    private update   : Secp256k1
    private recovery : Secp256k1
    private encrypt  : Secp256k1
    private model?   : Id<MnemonicKeyringModel>

    /**
     * @param context 
     */
    private constructor(context: MnemonicKeyringContext) {
        this.connector = context.connector
        this.context   = context.context
        this.sign      = context.sign
        this.update    = context.update
        this.recovery  = context.recovery
        this.encrypt   = context.encrypt
    }

    /**
     * @param model 
     */
    private setKeyringModel(model: Id<MnemonicKeyringModel>): void {
        this.model = model
    }

    /**
     * @param did 
     * @param options 
     * @returns
     */
    private async saveContext(did?: string, options?: SaveContextOptions): Promise<Id<MnemonicKeyringModel>> {
        let mnemonic: string | undefined = this.context.mnemonic

        if (options !== undefined) {
            if (options.removeMnemonic !== undefined) {
                if (options.removeMnemonic === true) {
                    mnemonic = undefined
                }
            }
        }

        if (this.model === undefined) {
            return await this.connector.insert({
                sign    : this.sign.toHexKeyPair(),
                update  : this.update.toHexKeyPair(),
                recovery: this.recovery.toHexKeyPair(),
                encrypt : this.encrypt.toHexKeyPair(),
                seed    : this.context.seed.toString('hex'),
                mnemonic: mnemonic,
            })
        } else {
            return await this.connector.update(this.model._id, {
                did     : did,
                sign    : this.sign.toHexKeyPair(),
                update  : this.update.toHexKeyPair(),
                recovery: this.recovery.toHexKeyPair(),
                encrypt : this.encrypt.toHexKeyPair(),
                seed    : this.context.seed.toString('hex'),
                mnemonic: mnemonic,
            })
        }
    }

    /**
     * @param did 
     */
    public async setDid(did: string): Promise<void> {
        const item = await this.connector.findByDid(did)

        if (item) {
            throw new Error()
        }

        this.setKeyringModel(await this.saveContext(did))
    }

    /**
     * @param connector 
     * @param options 
     * @returns
     */
    public static async createKeyring(connector: Connector, options?: MnemonicKeyringOptions): Promise<MnemonicKeyring> {
        const context  = await MnemonicKeyring.generateBip39Seed(options)
        const sign     = MnemonicKeyring.generateSecp256k1(context, MnemonicKeyring.signDerivationPath)
        const update   = MnemonicKeyring.generateSecp256k1(context, MnemonicKeyring.updateDerivationPath)
        const recovery = MnemonicKeyring.generateSecp256k1(context, MnemonicKeyring.recoveryDerivationPath)
        const encrypt  = MnemonicKeyring.generateSecp256k1(context, MnemonicKeyring.encryptDerivationPath)
        const instance = new MnemonicKeyring({
            connector: connector,
            context  : context,
            sign     : sign,
            update   : update,
            recovery : recovery,
            encrypt  : encrypt,
        })
        const model = await instance.saveContext()

        instance.setKeyringModel(model)

        return instance
    }

    /**
     * @param connector 
     * @param did 
     * @returns
     */
    public static async loadKeyring(connector: Connector, did: string): Promise<MnemonicKeyring> {
        const keyring = await connector.findByDid(did)

        if (! keyring) {
            throw new Error()
        }

        const context: BIP39Context = {
            mnemonic: keyring.mnemonic,
            seed    : Buffer.from(keyring.seed, 'hex'),
        }
        const sign: Secp256k1 = new Secp256k1({
            public : Buffer.from(keyring.sign.public , 'hex'),
            private: Buffer.from(keyring.sign.private, 'hex'),
        })
        const update: Secp256k1 = new Secp256k1({
            public : Buffer.from(keyring.update.public , 'hex'),
            private: Buffer.from(keyring.update.private, 'hex'),
        })
        const recovery: Secp256k1 = new Secp256k1({
            public : Buffer.from(keyring.recovery.public , 'hex'),
            private: Buffer.from(keyring.recovery.private, 'hex'),
        })
        const encrypt: Secp256k1 = new Secp256k1({
            public : Buffer.from(keyring.encrypt.public , 'hex'),
            private: Buffer.from(keyring.encrypt.private, 'hex'),
        })
        const instance = new MnemonicKeyring({
            connector: connector,
            context  : context,
            sign     : sign,
            update   : update,
            recovery : recovery,
            encrypt  : encrypt,
        })
        instance.setKeyringModel(keyring)

        return instance
    }

    /**
     * @returns
     */
    public getIdentifier(keyId?: string): string {
        if ((! this.model) || (! this.model.did)) {
            throw new Error()
        }

        if (keyId === undefined) {
            return `${ this.model.did }`
        } else {
            return `${ this.model.did }#${ keyId }`
        }
    }

    /**
     * @returns
     */
    public getSeedPhrases(): Array<string> | undefined {
        if (! this.context.mnemonic) {
            return undefined
        }

        return this.context.mnemonic.split(' ')
    }

    /**
     * @param did 
     * @param phrase 
     * @param option 
     * @returns
     */
    public async verifySeedPhrase(did: string, phrase: Array<string>, option: { isPersistent: boolean } = { isPersistent: false }): Promise<boolean> {
        const mnemonic = phrase.map((v) => { return v.trim() }).join(' ')
        const isValid  = (this.context.mnemonic === mnemonic)

        if (isValid) {
            if (option.isPersistent === false) {
                await this.saveContext(did, {
                    removeMnemonic: true,
                })

                this.context.mnemonic = undefined
            }
        }

        return isValid
    }

    /**
     * @returns
     */
    public getSignKeyPair(): Secp256k1 {
        return this.sign
    }

    /**
     * @returns
     */
    public getUpdateKeyPair(): Secp256k1 {
        return this.update
    }

    /**
     * @returns
     */
    public getRecoveryKeyPair(): Secp256k1 {
        return this.recovery
    }

    /**
     * @returns
     */
    public getEncryptKeyPair(): Secp256k1 {
        return this.encrypt
    }

    /**
     * @param context 
     * @param derivationPath 
     * @returns
     */
    public static generateSecp256k1(context: BIP39Context, derivationPath: string): Secp256k1 {
        const node = MnemonicKeyring.generateHDNodeByDerivationPath(context, derivationPath)

        return new Secp256k1({
            public : node.publicKey  || Buffer.from([]),
            private: node.privateKey || Buffer.from([]),
        })
    }

    /**
     * @param options 
     * @returns
     */
    public static async generateBip39Seed(options?: MnemonicKeyringOptions): Promise<BIP39Context> {
        const fromSize = (size: BIP39PhraseSize): number => {
            switch (size) {
                case 12: { return 128 }
                case 15: { return 160 }
                case 18: { return 192 }
                case 21: { return 224 }
                case 24: { return 256 }
                default: { throw new Error() }
            }
        }

        if (! options) {
            options = {
                length: BIP39DefaultPhraseSize
            }
        }

        const mnemonic = await Runtime.BIP39.generateMnemonic(fromSize(options.length))
        const seed     = await Runtime.BIP39.mnemonicToSeed(mnemonic)

        return {
            mnemonic: mnemonic,
            seed    : seed,
        }
    }
    
    /**
     * @param context 
     * @param derivationPath 
     * @returns
     */
    public static generateHDNodeByDerivationPath(context: BIP39Context, derivationPath: string): BIP32Interface {
        return Runtime.BIP32.getNode(context.seed, derivationPath)
    }
}