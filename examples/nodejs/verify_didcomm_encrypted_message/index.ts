import * as os from 'os'
import * as path from 'path'
import got from 'got'

(async () => {
    const base = `unix:${ path.join(os.homedir(), '.unid/run/unid.sock') }`
    const json = await got.post([ base, '/internal/didcomm/encrypted-messages/verify' ].join(':'), {
        enableUnixSockets: true,
        json: {
            message: {
                "ciphertext": "i0ePD8tCZqEKazGrh3Pycau8azJu0HRXDCOEzjrzj7A6awPn4U2l8x6uBYXe3uSOaOEFZWQuCtVorLlkKAW4glwudgU4U0oqxbnzpIMlRncu8MJ2-ryK9fJAzdNWpUw4U8pi57obzq2_v8xp-Al5vbK5qYwxWzGeNxMqGYulZ_V0zpUYMTJssc7teis1ivq3xAOoDDvD86r5T0RehpK7FQKqUpQ3_O25kMNPJ46ar519PkmOKn0Q_m4GJoxN7hT41TgSgoMh8KLSZEzv4Pcwu24lg4h8bIRYDQU9XnnzZmgzKMP23aDR8lVBEoIEt7RzMVoI_3VS1nYkM9INrSTRvh4tamBlx36KdBPsA_K_nfF1xvhmAT-yOP1mwSKszY2oQ6SHqrP8J9id7y1n5cSGZn0XJRGTlIIC1NF2urQ2moLtg2Ro-zgyp63y4c_XUgGdNJ61Nvl5o0BxJ5EGfTpwlbGrlXZb-HY5uPTlD3UyyHVCL6oB4O3mzGC8jyejM8n3bTV0cbtygDsW5HTmw4vuZ68CVWGEVCrU37n1bSrbPzLlVsS4Z1i6hA6v1GPQtVcp5W8rAw_zDL0b4ixlEfvRu3Fi2DbKtl3YaFvNLF1hlAP79l56BoeColAa4EGKMtVryaP0xdudrZF_AuYjqVlTXA4ag_KJuCFXNG37ZDo5gecs3jNxsy5i1Ik9APamGfAevDYzQH65N9XqqurhjaQkLbU4h38UTm2E-Eu-QIIuapf6vTIa5OokzvT7HuYUfb93NFmRRGktmmE6jItHf9uUN3H_UOnNC6RwkyraQ_DeLZKe2S7fkxrUe4mdMkwR5rJry4W5I7M_5299UJLgiTeQ7AQGgY1j31h3gXFxp8XvoueADz-k7e5MIULGZ1zNWxeQAHkjqwzgCEuftujLliBtcNJ2MU_VY3CSWPib_oszY3nsx6PZs4xrfsXQ7GbYdPwTiG6kDheC2Gj1NYGpH5EIB-Zohoa9_-W8Cab61pdqnQnEdMTO6tyXm1i8jm1jPeVXufvXhwSAv8f2IvFWS7x7RUMVsegtfyqHDFMZVewRMuwMKJLZ3hlhQ6T-c9joLv8SZxjMcp3JcS_wsvo8_kUPSorUeL0hPqyt6lLVRTvlBrzsdNF7TcqIoncJEoPiAMGTgGexzVN58tLdoazqsayCUAGNUlGyeC1U7U1Vmgz-1vslHOYtDRDATIiwMLe6NIYexMg2n-rJIsup02qRTsBBWM66yiOw5QDUBBU2fEihP6xt72nhKaAtWO9T0yAmQwyxLf7TxYoEGiCfxuO2sjrnymU4Ev3m_SZfiivaFAiS-cUQvV2Id8u0R8hXAdIfJMHLt3tPnJxWy696g_eAIsp-l5pCZkPmN3tMDwTW59AGuhb314wD1SDjYtN3HX_QGTue_ybBAsggh3PgSW05PSiTNpk7uc-G0Y1n7w5NtUH1dPh6tx5YTvluoBiRlL8S5RU-n2ByaCEJIbLxMoP_PeeLZOWPaWwKR4ghdorclHJUWM9Khuc1zBP5dkn1u8xgyTfam_aQscj151mtGqnAUV7Fhkxti_T8BSlP9YxkeENhfNWQONXloH4TZewzrSW6-LhLbvc-WsnSrNY7TqYjOXeOq0VKxJO1u2D2Av14gDvxfhlAggFufstHV63FpS3Y74HayrlAW2QvdPOgY59NfPGK8CqTUPvCwOF-2XtjjsjEC4IturiPj-dlz17S1FrwS1M8zBE1ETctVO8H9liwqnYAKnz0Ak1CGUDKpofU9abNp5xH-RR3dg_1xtQPwCRPHPc7YCuSFGNOI3md2IgjCm_eRCiSy9Der3-7lPCubUARFR583JbIJlX1ZOwbIIMYGS5teIyOxWf3wqQ0_pTPrs8NUO5UZ0lvBc8lx_9hnRf1q1jMltaaRjVSfjIAgqAq-tEqZpQdmFrfJhLK4qG-u_NJgHT2ax5b-QskZ4qaBhzbFYABmULTwOMJNNGb5qmxhAthCvc0dSWiphDe6rGKtGnMhAmOu8ABIIL-XkNy9_q7qOKhYcl-zRQVY0KzdmHzBt38NZOvuDmYnvgYVvnmCZ9uDeWAtbcUOfMdx8VA5ThnihtQwfZCq8aU7ER5lLSTxwwQ76EGjXJRMUIWI_5yefFPI2optkUxNY7xM7OLJaYd-gGRXZkCKaEm5ccikqKqb7UrDV4W86sRBsZq2rbRbxExOu24evXAh79rx7Of6ejGKrpBe6KYETfN6-LuvcgcwJoRZG8QAuvgga3XftOLvXKMKehVB44NF0kQedn9_YdyiOffHRvORPiY6ILsnUJXX1viBROyzYl3rldYCihadB9NAoPRUMDtUD9kjxIvKUwSkz-elJe0TEnE3qjfqwjJ0nLyAYpvcNSbxSc9Wi_jJZeYFuKOzBpDoQ6WnV2oNz4_JCltdx84WEQjOgsMgNAeUu6PfUmgRJY7ErLVxgvosvvIkrumxxPkq4sI6B8xDTaeieXxCO0I4H1F32PyRQP_hu85rE91-119urR9SMxW8lwk2EBZlvaig8nkepLIBOSq5AL2sAy32MurxkH_IxKwnztFxNJ7egRrnlBV0yyDwG6saC6OwQICp4V-3FQL3s5NJYmhWC9stUNqW10khBJ2UGdO0MSaPI0ehhFSaOI880ip1doVCTMBkcfDrVsLRlfhXBSdgmr-gDbvIGkr3B1UKre2xu-Sm35SX-TfkFpjZvWQ1g3lMYWZkXW7iD6v8uPEzKViwGjWCzyYc71bUcIwObPh2A5PDhBSmuTw-SyaYZUxFT9HCoKko3iCoIkdc2zDISgmVIQZM_zdFMLWQV-Cl2lK6JN8oDPbnMpibz-dycD1hagvsFndDU9re4uqz3UHKnDG6QbggvQP4OM1um27NHGw9oVQ7Di8Ufc-3fDyspnQ0ezdkd3QKM3PRKdGGQYQ5vnj4HCcGUTERKbqcZRjX-BW1-REQxSbhcQ2YXd-zEjF4M0XCp8s2amWZdX41Ez5S9ApOIncnEOY44F8No86z8cE0LhGucx9r2vb0ylKjz0dodeMCdWKiefOMEUYH0MvagBXFrcAKNogHw07hd7zjTFCFANhIO-VB014_YDaVw4G1Q_9wxJuBTUPEHQ6-ukskuGYoOqz69O2SgDcme5-BMVG7mzqXv7jw6_YGcJqhaOwrx0jGEJZetTHgBcM0yRTjCz8-aWfRdUPvNBUbct6kxdMxJraWzzZQ6yUEpIvLiP-TOXIGY7qhVpYxp5fsaqr01HSwP6f5lgdOtLJ-BR2aFOuuRY4YiBThMLHUAscsLkYYRkzfj__3R7V4gmRdSg46uiJx0aEoGefHKs9hLpWPX_cUmatGRRmJf_lnKlb3YEzijF9LWxhHPBrODHMyfS5rq_0QoLKwgBOcuwHPIki8myE-OZDAo0yIzIDrlJvlJld4xv2VTuBWrW5YuoL8xdCvSjt9lVB1rg6VEYGUx0r28dyR7YYQJE_x4ulmfPvLszblHiRabv6tNB5iKUUTg0oGOE4Qe3-BdwFeP1lBPJlUX7Go0PwnSKZ1ylYYEIVChfisef6u3UIp1vd-h_JhbTJuiMP1gT49mH1uxkw0usKLEpQcaK-vlY_ZRcmEjPzHL7Oq1Rikb89BGd4sD-LgIfTiK14jdDLyjLhT7ot4jQvfEz2MDp1KHf_g5DPtlBs8katRYi23yuJ8TZhF3AFZqrkicWB-7-ZMka1F0408ypHX1U5MaKlkR7XFW3ca3eBxi7kRScP0Z5aBgIZRdkhKdNn5SnREmHd4HeK7fed0Y0p-Zjvr7J0xHTs648yoM7daCtFOHNW_kmDn3GYebKJE7E-5rt22JyHNRTTQiJGn4sXNXAZb931MjDJVtXaZZSq_RGCKQnKHOlq4UTF5ef36_bjbFFQP9r9-eOOI1VDLgcbg6XEvGQOHV8ap37dpx2B46HtlKQ58UIRWD08W2dOyv40xcMTAAGnanY2OenvPPBAIZurC8Q2o_c",
                "iv": "aIOkbAtkB2bA2O42sFwMHmam3lxzgB0c",
                "protected": "eyJ0eXAiOiJhcHBsaWNhdGlvbi9kaWRjb21tLWVuY3J5cHRlZCtqc29uIiwiZW5jIjoiWEMyMFAiLCJraWQiOiJsYUJEZl9PeENFLTF6aVdMLWJUT093RFZVMUYyNGluTk1kWkhOd1ZxUEFBIiwic2tpZCI6ImRpZDp1bmlkOnRlc3Q6RWlCcHJYcmVNaWJhNGxveWwzcHNYbTBSc0VDZHRsQ2lRSWpNOEc5QnRkUXBsQSIsImFsZyI6IkVDREgtMVBVK1hDMjBQS1cifQ",
                "recipients": [
                    {
                        "encrypted_key": "qCwu1D7AwmzBplo7J6Dah68enYC5nL8ikBlixTFrbXY",
                        "header": {
                            "alg": "ECDH-1PU+XC20PKW",
                            "epk": {
                                "crv": "X25519",
                                "kty": "OKP",
                                "x": "IQeG600y5myLvo5DUvz4BKgNSnFeJjL-ds9739VRKyA"
                            },
                            "iv": "wn8Cus_PgfRAl7ThucMINCC5c0DL6xtJ",
                            "key_ops": [],
                            "kid": "did:unid:test:EiBprXreMiba4loyl3psXm0RsECdtlCiQIjM8G9BtdQplA",
                            "tag": "HjjQUaYwDO3kA6QIclEcMA"
                        }
                    }
                ],
                "tag": "2dWzOUtNfLOgI1HHhg6Q6g"
            },
        },
    }).json()

    console.log(JSON.stringify(json, null, 4))
})()
