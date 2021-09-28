import async from 'async'

interface Options {
    limit?: number,
}

/**
 * @param array 
 * @param worker 
 * @param options 
 * @returns
 */
const all = async <T1, T2>(array  : Array<T1>, worker : (item: T1, index: string | number) => Promise<T2>, options?: Options): Promise<Array<T2>> => {
    return new Promise<Array<T2>>((resolve, reject) => {
        if (options === undefined) {
            options = {}
        }
        if (options.limit === undefined) {
            options.limit = 5
        }

        let results: Array<T2> = []

        async.forEachOfLimit(array, options.limit, async (item, index, next) => {
            try {
                results.push(await worker(item, index))
            } catch (error) {
                return next(error)
            }

            return next()
        }, (error) => {
            if (error) { return reject(error) }

            return resolve(results)
        })
    })
}

/**
 * @param timeout 
 * @returns
 */
const wait = (timeout: number): Promise<boolean> => {
    return new Promise<boolean>((resolve, _) => {
        setTimeout(() => {
            return resolve(true)
        }, timeout)
    })
}

/**
 */
const promise = {
    all,
    wait,
}

export {
    promise,
    Options,
}