import { ConfigManager } from "../config"
import { UNiDDidDocument } from "./document"
import { DIDCreateRequest, DIDCreateResponse, DIDResolutionRequest, DIDResolutionResponse } from "./interfaces/did-operation"
import { didCreatePayload } from "./payload"
import { HttpClient } from "../utils/http-client"
import { UNiDNotImplementedError } from "../error"

interface UNiDDidResolverContext {
    debug?: boolean
    endpoint?: string
}

/**
 */
export class UNiDDidOperator {
    /**
     */
    private readonly debug: boolean

    /**
     */
    private readonly endpoint: string

    /**
     */
    private readonly client: HttpClient

    /**
     * @param context 
     */
    constructor(context?: UNiDDidResolverContext) {
        if ((context !== undefined) && (context.debug !== undefined)) {
            this.debug = context.debug
        } else {
            this.debug = false
        }

        if ((context !== undefined) && (context.endpoint !== undefined)) {
            this.endpoint = context.endpoint
        } else {
            this.endpoint = ConfigManager.DID_ENDPOINT_URI
        }

        this.client = HttpClient.new({
            baseURL: this.endpoint,
        }, {
            debug: this.debug,
        })
    }

    /**
     * @param params 
     * @returns
     */
    public async resolve(params: DIDResolutionRequest): Promise<UNiDDidDocument> {
        try {
            const response = await this.client.agent.get<DIDResolutionResponse>(`/api/v1/identifiers/${ params.did }`)

            return new UNiDDidDocument(response.data.didDocument)
        } catch (err) {
            throw new err
        }
    }

    /**
     * @param params 
     * @returns
     */
    public async create(params: DIDCreateRequest): Promise<UNiDDidDocument> {
        try {
            const payload  = didCreatePayload(params)
            const response = await this.client.agent.post<DIDCreateResponse>('/api/v1/operations', payload)

            return new UNiDDidDocument(response.data.didDocument)
        } catch (err) {
            throw new err
        }
    }

    /**
     */
    public async update(): Promise<void> {
        throw new UNiDNotImplementedError()
    }

    /**
     */
    public async recover(): Promise<void> {
        throw new UNiDNotImplementedError()
    }

    /**
     */
    public async deactivate(): Promise<void> {
        throw new UNiDNotImplementedError()
    }
}