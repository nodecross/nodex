import axios, { AxiosInstance, CancelTokenSource, AxiosRequestConfig } from "axios"
import lodash from 'lodash'
import { Logging } from "./logging"

type KV = { [ key : string ]: string }

export enum HttpRequestMethod {
    GET     = 'GET',
    HEAD    = 'HEAD',
    POST    = 'POST',
    PUT     = 'PUT',
    DELETE  = 'DELETE',
    CONNECT = 'CONNECT',
    OPTIONS = 'OPTIONS',
    TRACE   = 'TRACE',
    PATCH   = 'PATCH',
}

interface HttpClientContext {
    debug?: boolean
}

export class HttpClient {
    private _timeout: number | undefined
    private _baseUri: string | undefined
    private _headers: KV

    private _logging: Logging
    private _handler: CancelTokenSource
    private _instance: AxiosInstance

    private context?: HttpClientContext

    private constructor(config?: AxiosRequestConfig, context?: HttpClientContext) {
        let cancel = axios.CancelToken

        this.context   = context
        this._headers  = {}
        this._logging  = new Logging()
        this._handler  = cancel.source()
        this._instance = (config) ? axios.create({
            ...config,
            cancelToken: this._handler.token,
        }) : axios.create({
            cancelToken: this._handler.token,
        })

        this.initialize()
    }

    private initialize() {
        // Inspect of the http request
        this._instance.interceptors.request.use((request) => {
            // set headers
            let common  = lodash.defaults<KV, KV>(this._headers, request.headers.common)
            let headers = lodash.defaults<{ [ key: string ]: KV }, { [ key: string ]: KV }>({ common: common }, request.headers)

            request.headers = headers

            // set base-uri
            if (this._baseUri !== undefined) {
                request.baseURL = this._baseUri
            }

            // set timeout
            if (this._timeout !== undefined) {
                request.timeout = this._timeout
            }

            if (this.context && this.context.debug) {
                this._logging.info('axios (request):', request)
            }

            return request
        }, (error) => {
            if (this.context && this.context.debug) {
                this._logging.err('axios (request)', error)
            }

            return Promise.reject(error)
        })

        // Inspect of the http response
        this._instance.interceptors.response.use((response) => {
            if (this.context && this.context.debug) {
                this._logging.info('axios (response):', response)
            }

            return response
        }, (error) => {
            if (this.context && this.context.debug) {
                this._logging.err('axios (response):', error)
            }

            return Promise.reject(error)
        })
    }

    public static new(config?: AxiosRequestConfig, context?: HttpClientContext): HttpClient {
        return new HttpClient(config, context)
    }

    public get agent(): AxiosInstance {
        return this._instance
    }

    public cancel() {
        this._handler.cancel()
    }

    public setBaseUri(uri: string): HttpClient {
        this._baseUri = uri

        return this
    }

    public setTimeout(timeout: number): HttpClient {
        this._timeout = timeout

        return this
    }

    public setHeaders(headers: KV): HttpClient {
        Object.keys(headers).map((k) => {
            let v  = headers[k]
            let nk = k.toLowerCase()

            delete headers[k]

            headers[nk] = v
        })

        this._headers = lodash.defaults<KV, KV>(headers, this._headers)

        return this
    }
}