import { UNiDDidOperator } from "../src/core/operator"

test('UNiDDidOperator#resolve', async () => {
    const did = 'did:unid:test:EiAJ1Ybh8D43hV_VOvwG8S4Mrscm_qp6GAvdW7jkSG5Yfw'

    const op  = new UNiDDidOperator()
    const res = await op.resolve({ did: did })

    expect(did).toEqual(res.identifier)
})