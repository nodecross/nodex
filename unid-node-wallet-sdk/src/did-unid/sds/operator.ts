import { Hasher } from "../cipher/hasher"
import { UNiDVerifiableCredential, UNiDVerifiablePresentation, UNiDVerifiablePresentationMetadata } from "../schemas"
import { UNiDSDSCredentialV1Types } from "../schemas/internal/unid-sds"
import { ConfigManager } from "../../config"
import { UNiDNotImplementedError } from "../../error"
import { HttpClient } from "../../utils/http-client"
import { UNiDContextInternal } from "../../unid"

interface UNiDSDSOperatorContext {
    context  : UNiDContextInternal
    debug?   : boolean
    endpoint?: string
}

/**
 */
interface SDSRequest {
    payload: string,
}

/**
 */
export interface SDSFindOperationResponsePayload {
    id                  : string,
    createdAt           : string,
    updatedAt           : string,
    context             : string,
    type                : string,
    issuerDid           : string,
    credentialSubjectDid: string,
    document            : string,
    issuanceDate        : string,
    expirationDate      : string | undefined,
}

/**
 */
interface SDSCreateRequest {
    payload: UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, UNiDSDSCredentialV1Types>> & UNiDVerifiablePresentationMetadata,
}

/**
 */
export interface SDSCreateResponse {
    payload: {
        id: string,
    },
}

/**
 */
interface SDSFindRequest {
    payload: UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, UNiDSDSCredentialV1Types>> & UNiDVerifiablePresentationMetadata,
}

/**
 */
export interface SDSFindResponse {
    payload: Array<SDSFindOperationResponsePayload>,
}

/**
 */
interface SDSFindOneRequest {
    payload: UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, UNiDSDSCredentialV1Types>> & UNiDVerifiablePresentationMetadata,
}

/**
 */
export interface SDSFindOneResponse {
    payload: SDSFindOperationResponsePayload | undefined,
}

/**
 */
interface SDSUpdateRequest {
    payload: UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, UNiDSDSCredentialV1Types>> & UNiDVerifiablePresentationMetadata,
}

/**
 */
export interface SDSUpdateResponse {
}

/**
 */
interface SDSDeleteRequest {
    payload: UNiDVerifiablePresentation<UNiDVerifiableCredential<string, string, UNiDSDSCredentialV1Types>> & UNiDVerifiablePresentationMetadata,
}

/**
 */
export interface SDSDeleteResponse {
}

/**
 */
export class UNiDSDSOperator {
    /**
     */
    private static readonly REQUEST_HEADER_KEY: string = 'X-REQUEST-DIGEST'

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
     */
    private readonly context: UNiDContextInternal

    /**
     * @param context 
     */
    constructor(context: UNiDSDSOperatorContext) {
        if (context.debug !== undefined) {
            this.debug = context.debug
        } else {
            this.debug = false
        }

        if (context.endpoint !== undefined) {
            this.endpoint = context.endpoint
        } else {
            this.endpoint = ConfigManager.SDS_ENDPOINT_URI
        }

        this.client = HttpClient.new({
            baseURL: this.endpoint,
        }, {
            debug: this.debug,
        })

        this.context = context.context
    }

    /**
     * @param request 
     * @returns
     */
    public async create(request: SDSCreateRequest): Promise<SDSCreateResponse> {
        const URI = '/api/v1/create'

        try {
            const context: SDSRequest = {
                payload: JSON.stringify(request.payload)
            }
            const digest = Hasher.generateRequestDigest(URI, context.payload, {
                clientSecret: this.context.clientSecret,
            })
            const response = await this.client.setHeaders({
                [UNiDSDSOperator.REQUEST_HEADER_KEY]: digest,
            }).agent.post<SDSCreateResponse>(URI, context)

            return response.data
        } catch (err) {
            throw err
        }
    }

    /**
     * @param request 
     * @returns
     */
    public async find(request: SDSFindRequest): Promise<SDSFindResponse> {
        const URI = '/api/v1/find'

        try {
            const context: SDSRequest = {
                payload: JSON.stringify(request.payload)
            }
            const digest = Hasher.generateRequestDigest(URI, context.payload, {
                clientSecret: this.context.clientSecret,
            })
            const response = await this.client.setHeaders({
                [UNiDSDSOperator.REQUEST_HEADER_KEY]: digest,
            }).agent.post<SDSFindResponse>(URI, context)

            return response.data
        } catch (err) {
            throw err
        }
    }

    /**
     * @param request 
     * @returns
     */
    public async findOne(request: SDSFindOneRequest): Promise<SDSFindOneResponse> {
        const URI = '/api/v1/findOne'

        try {
            const context: SDSRequest = {
                payload: JSON.stringify(request.payload)
            }
            const digest = Hasher.generateRequestDigest(URI, context.payload, {
                clientSecret: this.context.clientSecret,
            })
            const response = await this.client.setHeaders({
                [UNiDSDSOperator.REQUEST_HEADER_KEY]: digest,
            }).agent.post<SDSFindOneResponse>(URI, context)

            return response.data
        } catch (err) {
            throw err
        }
    }

    /**
     * @param request 
     */
    public async update(_: SDSUpdateRequest): Promise<SDSUpdateResponse> {
        try {
            throw new UNiDNotImplementedError()
        } catch (err) {
            throw err
        }
    }

    /**
     * @param request 
     */
    public async delete(_: SDSDeleteRequest): Promise<SDSDeleteResponse> {
        try {
            throw new UNiDNotImplementedError()
        } catch (err) {
            throw err
        }
    }
}