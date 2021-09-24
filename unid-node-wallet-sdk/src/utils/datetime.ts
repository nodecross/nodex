import moment, { Moment } from 'moment'

/**
 */
enum DateTimeTypes {
    default = 'YYYY-MM-DDTHH:mm:ss[Z]',
    iso8601 = 'YYYY-MM-DDTHH:mm:ss.SSSSSS[Z]',
}

/**
 */
class DateTimeUtils {
    /**
     */
    private datetime: Date | undefined

    /**
     * @param input 
     */
    constructor(input: string | undefined)
    constructor(input: Moment | undefined)
    constructor(input: Date | undefined)
    constructor(input: any | undefined) {
        if (input !== undefined) {
            if (typeof(input) === 'string') {
                this.datetime = moment(input, DateTimeTypes.iso8601).toDate()
            } else if (moment.isMoment(input)) {
                this.datetime = input.toDate()
            } else {
                this.datetime = input
            }
        }
    }

    /**
     * @returns
     */
    toDate(): Date | undefined {
        return this.datetime
    }

    /**
     * @returns
     */
    $toDate(): Date {
        if (this.datetime === undefined) {
            this.datetime = new Date(0)
        }
        return this.datetime
    }

    /**
     * @param format 
     * @returns
     */
    toString(format: string | DateTimeTypes): string | undefined {
        if (this.datetime !== undefined) {
            return moment(this.datetime).utc().format(format)
        }
        return undefined
    }

    /**
     * @param format 
     * @returns
     */
    $toString(format: string | DateTimeTypes): string {
        if (this.datetime === undefined) {
            this.datetime = new Date(0)
        }
        return moment(this.datetime).utc().format(format)
    }

    /**
     * @returns
     */
    adjustMidnight(): DateTimeUtils {
        if (this.datetime !== undefined) {
            this.datetime = moment(this.datetime).startOf('day').toDate()
        }
        return this
    }

    /**
     * @returns
     */
    adjustEndOfDay(): DateTimeUtils {
        if (this.datetime !== undefined) {
            this.datetime = moment(this.datetime).endOf('day').toDate()
        }
        return this
    }

    /**
     * @returns
     */
    adjustBeginOfSecond(): DateTimeUtils {
        if (this.datetime !== undefined) {
            this.datetime = moment(this.datetime).startOf('second').toDate()
        }
        return this
    }

    /**
     * @returns
     */
    adjustEndOfSecond(): DateTimeUtils {
        if (this.datetime !== undefined) {
            this.datetime = moment(this.datetime).endOf('second').toDate()
        }
        return this
    }

    /**
     * @returns
     */
    add24Hours(): DateTimeUtils {
        if (this.datetime !== undefined) {
            this.datetime = moment(this.datetime).add(1, 'days').toDate()
        }
        return this
    }
}

export {
    DateTimeUtils, DateTimeTypes,
}