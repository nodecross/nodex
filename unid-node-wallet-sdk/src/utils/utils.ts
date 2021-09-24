import { UNiDNotCompatibleError } from '../error'

/**
 * @param input 
 * @returns
 */
const splitDid = (input: string): { did: string, keyId: string } => {
    const arr = input.split('#')

    if (arr.length !== 2) {
        throw new UNiDNotCompatibleError()
    }

    return {
        did  : arr[0],
        keyId: arr[1],
    }
}

/**
 * @param input 
 * @returns
 */
const isBase64 = (input: string): boolean => {
    return (Buffer.from(input, 'base64').toString('base64') === input)
}

/**
 * @param min 
 * @param max 
 * @returns
 */
const getRandomInt = (min: number, max: number): number => {
    if (max < min) {
        throw new Error()
    }

    return Math.floor(Math.random() * (max - min + 1) + min)
}

/**
 * @param input 
 * @param length 
 * @returns
 */
const trimString = (input: string | undefined, length: number): string => {
    if (input === undefined) {
        return ''
    }
    if (length < input.length) {
        return `${ input.slice(0, length) }･･`
    }
    return `${ input }`
}

/**
 * @param start 
 * @param end 
 * @returns
 */
const range = (start: number, end: number): Array<number> => {
    return Array.from({ length: (end - start) }, (_, k) => k + start);
}

/**
 * @param input 
 * @returns
 */
const numberWithComma = (input: number): string => {
    return String(input).replace(/(\d)(?=(\d\d\d)+(?!\d))/g, '$1,')
}

/**
 * @param l 
 * @param r 
 * @returns
 */
const isRequiredUpdate = (l: string, r: string): boolean => {
    const ls: Array<string> = l.split('.').map((v, _) => v.trim())
    const rs: Array<string> = r.split('.').map((v, _) => v.trim())
    const m : number = Math.max(ls.length, rs.length)

    const ln: Array<number> = Array.from({ length: m }, (_, k) => {
        return (ls[k] !== undefined && ls[k] !== '') ? ls[k] : '0'
    }).map((v, _) => parseInt(v))
    const rn: Array<number> = Array.from({ length: m }, (_, k) => {
        return (rs[k] !== undefined && rs[k] !== '') ? rs[k] : '0'
    }).map((v, _) => parseInt(v))

    const check: Array<number> = Array.from({ length: m }, (_, k) => {
        if (ln[k] < rn[k]) return -1
        if (ln[k] > rn[k]) return  1
        return 0
    })

    return (Math.min(...check) === -1)
}

const utils = {
    getRandomInt,
    trimString,
    range,
    numberWithComma,
    isRequiredUpdate,
    isBase64,
    splitDid,
}

export { utils }