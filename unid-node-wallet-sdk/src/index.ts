import { UNiD, UNiDNetworkType } from './unid'
import { KeyRingType } from './did-unid/keyring'
import { Hasher } from './did-unid/cipher/hasher'
import { AddressCredentialV1 } from './did-unid/schemas/address'
import { AlumniOfCredentialV1 } from './did-unid/schemas/alumni-of'
import { BirthDateCredentialV1 } from './did-unid/schemas/birth-date'
import { ContactPointCredentialV1 } from './did-unid/schemas/contact-point'
import { EmailCredentialV1 } from './did-unid/schemas/email'
import { GenderCredentialV1 } from './did-unid/schemas/gender'
import { NameCredentialV1 } from './did-unid/schemas/name'
import { PhoneCredentialV1 } from './did-unid/schemas/phone'
import { QualificationCredentialV1 } from './did-unid/schemas/qualification'
import { ImageCredentialV1 } from './did-unid/schemas/image'
import { WorksForCredentialV1 } from './did-unid/schemas/works-for'
import { UNiDSDSCredentialV1 } from './did-unid/schemas/internal/unid-sds'
import { UNiDAuthCredentialV1 } from './did-unid/schemas/internal/unid-auth'
import {
    UNiDCredentialSubjectMetadata,
    UNiDVerifiableCredential,
    UNiDVerifiableCredentialBase,
    UNiDVerifiableCredentialOptions,
    UNiDVerifiableCredentialMetadata,
    UNiDVerifiablePresentation,
} from './did-unid/schemas'
import {
    UNiDNotCompatibleError,
    UNiDNotUniqueError,
} from './error'

export {
    UNiD,
    UNiDNetworkType,
    KeyRingType,
    Hasher,
    AddressCredentialV1,
    AlumniOfCredentialV1,
    BirthDateCredentialV1,
    ContactPointCredentialV1,
    EmailCredentialV1,
    GenderCredentialV1,
    NameCredentialV1,
    PhoneCredentialV1,
    QualificationCredentialV1,
    ImageCredentialV1,
    WorksForCredentialV1,
    UNiDSDSCredentialV1,
    UNiDAuthCredentialV1,
    UNiDCredentialSubjectMetadata,
    UNiDVerifiableCredential,
    UNiDVerifiableCredentialBase,
    UNiDVerifiableCredentialOptions,
    UNiDVerifiableCredentialMetadata,
    UNiDVerifiablePresentation,
    UNiDNotCompatibleError,
    UNiDNotUniqueError,
}