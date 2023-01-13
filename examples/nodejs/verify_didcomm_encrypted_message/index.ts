import * as os from 'os'
import * as path from 'path'
import got from 'got'

(async () => {
    const base = `unix:${ path.join(os.homedir(), '.unid/run/unid.sock') }`
    const json = await got.post([ base, '/internal/didcomm/encrypted-messages/verify' ].join(':'), {
        enableUnixSockets: true,
        json: {
            message: {
                "ciphertext": "UUdX33P1VYUzBz4-_QHlKirJTbKzb7eqXPozAaWtmoqZpr61KuS1xRydIqa0No40LYkkGHcuDxmtM5oaS_mZ2DAq2gW-clzMeRW6ZSDhlJTA4tNNk0_POctjRPPcqenEEHXbUv7l5dR7QEqGZ-NZ9ArGaua1wBfZw6yGe7UThL2WJPvVR3MnAuD-aEIC0Hy4B7KBDHn8JduC8gp3NkHoZbpTEc8b-OpOXmeuRHq4fplerzihaPbuDYDkFlGG5nrycvKfBvHPsftMCS5berbyqRr2I5vJ-ZMOdvs1lUt8x0kGpdgqBiS2DWOG9J6xVkWfckNyWaiY4X6_h829yZNQZlfyqnJTmaNscNX-mgxLqyfs6utDdmRrGD4JuNqDu9oEWIR_EtoUidmo0OkrfuKOvXqG9qBP67zymk7lFwl67TVL6ZRvw7vbVT9Iv1Ep1FgIMV9gv9BP7lSaoSO9tnc_Ac8YlzV1VHKbYizDUW-jHeMm-7g4V5D3CxX6teUy13SA10zUcMO_gDFb6e17Rfb4KFrN6nNwHb2uBmiuIhNOHs9SKoJy1ADpA4bg5pkfxCQxnfjn6whRlJlzhvlfrQtHV4Lq560kfluTuwEkBJo7w11cK1vAerqz1N2igwqJ3mJhLMDxWzdomHlUa1Vtht-hAp5Tb0t8KJdFjxw7G4LOZQD0rpsltJ0ZYckZLfSG_bm60fnjfdP6FeRRlLb9LBpX7KoB0N1b7bJgK96nTG8A0I4x--75_cK-CN-Xb8xr0xg51RwsUhKREipj8wwS58SvxNd5N3BRfkShsxwTf9xgM01LAjrZdNIhIHGMlEyYiCXbdAlJ9utTO5_xGiE3Poz1wcSTDHPMLwALGLe_vss1M-x5LDlxKcFDu7Z-yIpYvmOMW6WCR-dER1j0a3SWsGldHRqPWkFy1oNKSIJd8xZoI6CfHNZ1dHySu0JZDJ70V9wMILDvCTDRohLknkGdaDMccn4KXX-5T8EFGxodyoI6Jrcmh4mENWwAvSWZad_OShum4Tmv-xluDTvWYjQM4VUBingDk-jDMttX2srWhJ5ztjSOC-PWW8u2k89L78p2gpEB0fYiTqla0t9FNFduIBscMPEs8ILboj2BZiTuoKFb54Rf8zLlfQLttOk9y6vcnt-BT6pnPNC9sJlxjM_YmqdDvI79zP_iRWsPCK0YlJgDbm4S_lsY-yYSRlpeTJrU-VMCr-LewnBWP7OCi4CbWptdeKNhRnJ5PinaI8L6OURG6fV97IcHXEFr6uNiT68HmdXOXrG5sR1g9xV_KuUqoGB5XMRzXVm_0lhvXgu113SfWpkIdyxIrgOKJXpY6sRiZ91vTiGvvFHIVYh84nilwoSEzWhfEmf1rwIEqPaJZohTmheAsHRtDDLF4BsZd0glAMRmE3JRFr69AW8Ot8pE457c886O__nXbQbFKgUY-k-J_PMEkPxQlgUT0Wba57Tdp7CmpQEgAEoxDQTRqvq5BaZIK7cp_1eyK03SbImNSeALP38p-xfz7CrsWms5K2riDScqGkevZWOBpBJDOi1lle7TTAmLjqxXXcR3FWiv6XuzvlX8LL2kzNyy87zwL_xlVbEFaC6zKdMqGGpVVrTE_4CtovafME3zCrlbi4OV2fKUrFGrFZ753RrIR9B4oWOa91KkYtAI-gHpW-NNgAhs5f0uLmp3Vp5h9APXCeSd0MmAa6YK3nsSIIwoclQMgLW_ukNqsbtfwrZXzOEDP75Eqa1WJGkqk9qcqJRyEH-RDeyBiMY8BGfQ73pGf3m2L5nCbmYItxjERGGV9bX5oFbqdeaiqQqICmRzCM5MJnQ08tKOK1uUPwwEo_D359VZh2jN-Y4E2B4ws_n2Vm5NYosYRJn0PLu3VmB5aigKejKmvQASA_4BKf3nxVZvIc_0L--PwbLKx9-ymZHqgeBBv7QFF2wtPiOUWwGwn1VCaFp6ELrh0_pE6ksi6Pz2vyvpwD48lmhX1DcEaA3b3A-wb7nlIutWVoN6DTjgEPbx3iqN83S4-5tCj_icI9v6RcxIYQS5_npK-hV31HrccY4n9Cqa40J7xSogFLiv3YLMEuQrrZ0XdUegjH2oePns6t6tJnojB-adWqEi-uIeKdLRlyUtSsSyAERlQYHnLKbNlp6uLDNMW2qVrvJDLO1BNA9PUDgcoYGShBIyiGCwcSF9XNXzNY_op5ykLrrkdgxnQZvIAq2a0CIqFwedsfqnSdqd8HI8mGYEY-quTTtqlZg3wdRK303FjF71Q_W12adskWFdbh5KKrJxVk7wrh7bjdZzFBWylquHsUchITlXz6T5K7VKhraVnEL18J1DfyvXBOMl77EPEAn73WdAxDD_0GMAbmocwttbLQpjYvnDJI4Yc7lW8mcPVvbnGDEVzvT5A-3shP3DtGRtgzorWkmvXvKWIANyxnnPdvMehlJzdLFYor7LVvGxApiVa6Tjziu3LzxrKsTrZhvg_7cmh4XqgsVYrlQc9BZPXDtoxOJseHWzlTGWIHomyU_8FJRRU8gWRxZMhZywZVm721JPZMZ9QdOgmFNCahPP_mANonGUUzRZ29KNFAUMGt3SAVWf2AHHcIY3LowkkzW4R67e6YhaTFso6qa8hjl9fMUk7S5vdOs1zNPN9-B7bkAzDENTW05F-VodE2aJJddZjwI0DlWJ1EH-JiFQmIsrPuwMBinaACsLqob9E8MmzBB32EJ8xF6iNf9kih7kH9B9JeKHATELcWYR33ghOjxCOHJQukQ2heksbgokj0N2SDWauiv4J2-JNhp1l9cAlAp_O-6SUfRxmX2qc-QA4-5xu96dQjcp0UfThzJniPp-93DR_VM8MtY-s2-wNlsSZN2HIQcxHLvyTEqolqoftno8H4Cn1X33msczICuq7XMWPDnluUHPikgSwMUMXEuu4nfz4yAgQFfcNnORsowKrZYoLMPuUXRNosPilX9j1HTW1pBz5T4dM7kOjyl67XoVU5qHyvfXpqAfKNloZapFFsy5eKp3yIc7fjPGfvGLvQrPtLMGIKGMc9uiEJCav9EXe6Rd24QWcTVbKohzDv-mV_VW9Q6tym6d1A",
                "iv": "22Z3XoSSXis9CWGskEiAyz0zwEIZe8nS",
                "protected": "eyJ0eXAiOiJhcHBsaWNhdGlvbi9kaWRjb21tLWVuY3J5cHRlZCtqc29uIiwiZW5jIjoiWEMyMFAiLCJraWQiOiJsYUJEZl9PeENFLTF6aVdMLWJUT093RFZVMUYyNGluTk1kWkhOd1ZxUEFBIiwic2tpZCI6ImRpZDp1bmlkOnRlc3Q6RWlCcHJYcmVNaWJhNGxveWwzcHNYbTBSc0VDZHRsQ2lRSWpNOEc5QnRkUXBsQSIsImFsZyI6IkVDREgtMVBVK1hDMjBQS1cifQ",
                "recipients": [
                    {
                        "encrypted_key": "F_HtG8Pb7rILOf3oXl7bpIU-Rlg06aDMlKiZFUyZmpU",
                        "header": {
                            "alg": "ECDH-1PU+XC20PKW",
                            "epk": {
                                "crv": "X25519",
                                "kty": "OKP",
                                "x": "NWB9-D9JpO69LhLZDZK-pxmi8-vkjS8K9iLDg-CiV0E"
                            },
                            "iv": "q74LMP6AA3R4-lhh4a5KfibdKWmd1IWB",
                            "key_ops": [],
                            "kid": "did:unid:test:EiBprXreMiba4loyl3psXm0RsECdtlCiQIjM8G9BtdQplA",
                            "tag": "n39K2BAxvbPCc4ocB6fEDA"
                        }
                    }
                ],
                "tag": "tIkWBKwcDqc8dqD-ImCGNA"
            },
        },
    }).json()

    console.log(JSON.stringify(json, null, 4))
})()
