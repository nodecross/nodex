import debug from 'debug'
import chalk, { Chalk } from 'chalk'
import SnakeCase from 'snakecase-keys'
import { DateTimeUtils, DateTimeTypes } from './datetime'

type Priority =
    | 'EMERG'
    | 'ALERT'
    | 'CRITICAL'
    | 'ERROR'
    | 'WARNING'
    | 'NOTICE'
    | 'INFO'
    | 'DEBUG'

class Logging {
    public constructor(options?: { requestId?: string, color?: Chalk }) {
        if (options) {
            this._color     = options.color
            this._requestId = options.requestId
        }
    }

    private _color: Chalk | undefined
    private _requestId: string | undefined
    private static readonly namespace: string = 'debug:app'

    static get colors(): Array<Chalk> {
        return [
            // chalk.black,
            chalk.red,
            chalk.green,
            chalk.yellow,
            chalk.blue,
            chalk.magenta,
            chalk.cyan,
            chalk.white,
            // chalk.gray,
            chalk.redBright,
            chalk.greenBright,
            chalk.yellowBright,
            chalk.blueBright,
            chalk.magentaBright,
            chalk.cyanBright,
            // chalk.whiteBright,
        ]
    }

    private get color(): Chalk | undefined {
        return this._color
    }

    private get requestId(): string | undefined {
        return this._requestId
    }

    private async logging(priority: Priority, message: string, ...context: Array<any>): Promise<string> {
        try {
            let time: string = new DateTimeUtils(new Date()).$toString(DateTimeTypes.iso8601)
            let data: Array<any> = []

            if (this.requestId) {
                data = [ `${ time }`, `[${ this._requestId }]`, `[${ priority }]:`, message, SnakeCase(context) ].flat()
            } else {
                data = [ `${ time }`, `[${ priority }]:`, message, SnakeCase(context) ].flat()
            }

            switch (priority) {
            case 'EMERG':
            case 'ALERT':
            case 'CRITICAL':
            case 'ERROR':
                if (this.color !== undefined) {
                    if (this.requestId) {
                        if (data.slice(4).length === 0) {
                            console.error(...data.slice(0, 3), this.color(...data.slice(3)))
                        } else if (data.slice(4).length === 1) {
                            console.error(...data.slice(0, 3), this.color(...data.slice(3, 4), JSON.stringify(data.slice(4).pop())))
                        } else {
                            console.error(...data.slice(0, 3), this.color(...data.slice(3, 4), JSON.stringify(data.slice(4))))
                        }
                    } else {
                        if (data.slice(3).length === 0) {
                            console.error(...data.slice(0, 2), this.color(...data.slice(2)))
                        } else if (data.slice(3).length === 1) {
                            console.error(...data.slice(0, 2), this.color(...data.slice(2, 3), JSON.stringify(data.slice(3).pop())))
                        } else {
                            console.error(...data.slice(0, 2), this.color(...data.slice(2, 3), JSON.stringify(data.slice(3))))
                        }
                    }
                } else {
                    console.error(...data)
                }
                break
            case 'DEBUG':
                debug(Logging.namespace)(`${time}`, ...data.slice(1))
                break
            default:
                if (this.color !== undefined) {
                    if (this.requestId) {
                        if (data.slice(4).length === 0) {
                            console.log(...data.slice(0, 3), this.color(...data.slice(3)))
                        } else if (data.slice(4).length === 1) {
                            console.log(...data.slice(0, 3), this.color(...data.slice(3, 4), JSON.stringify(data.slice(4).pop())))
                        } else {
                            console.log(...data.slice(0, 3), this.color(...data.slice(3, 4), JSON.stringify(data.slice(4))))
                        }
                    } else {
                        if (data.slice(3).length === 0) {
                            console.log(...data.slice(0, 2), this.color(...data.slice(2)))
                        } else if (data.slice(3).length === 1) {
                            console.log(...data.slice(0, 2), this.color(...data.slice(2, 3), JSON.stringify(data.slice(3).pop())))
                        } else {
                            console.log(...data.slice(0, 2), this.color(...data.slice(2, 3), JSON.stringify(data.slice(3))))
                        }
                    }
                } else {
                    console.log(...data)
                }
                break
            }

            return JSON.stringify(data)
        } catch (error) {
            return ''
        }
    }

    // POSIX: 0 - Emergency
    async emerg(message: string, ...context: Array<any>): Promise<string> {
        return this.logging('EMERG', message, ...context)
    }

    /** @deprecated use `emerg(message:)` instead. */
    async panic(message: string, ...context: Array<any>) {
        return this.emerg(message, ...context)
    }

    // POSIX: 1 - Alert
    async alert(message: string, ...context: Array<any>): Promise<string> {
        return this.logging('ALERT', message, ...context)
    }

    // POSIX: 2 - Critical
    async crit(message: string, ...context: Array<any>): Promise<string> {
        return this.logging('CRITICAL', message, ...context)
    }

    // POSIX: 3 - Error
    async err(message: string, ...context: Array<any>): Promise<string> {
        return this.logging('ERROR', message, ...context)
    }

    /** @deprecated use `err(message:)` instead. */
    async error(message: string, ...context: Array<any>) {
        return this.err(message, ...context)
    }

    // POSIX: 4 - Warning
    async warning(message: string, ...context: Array<any>): Promise<string> {
        return this.logging('WARNING', message, ...context)
    }

    /** @deprecated use `warning(message:)` instead. */
    async warn(message: string, ...context: Array<any>) {
        return this.warning(message, ...context)
    }

    // POSIX: 5 - Notice
    async notice(message: string, ...context: Array<any>): Promise<string> {
        return this.logging('NOTICE', message, ...context)
    }

    // POSIX: 6 - Informational
    async info(message: string, ...context: Array<any>): Promise<string> {
        return this.logging('INFO', message, ...context)
    }

    // POSIX: 7 - Debug
    async debug(message: string, ...context: Array<any>): Promise<string> {
        return this.logging('DEBUG', message, ...context)
    }
}

export { Logging }