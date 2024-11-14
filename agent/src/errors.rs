use actix_web::HttpResponse;
use actix_web::{error, http::StatusCode};
use serde::Serialize;
use std::convert::From;
use thiserror::Error;

#[derive(Serialize, Clone, Copy, Debug, Error)]
pub enum AgentErrorCode {
    #[error("binary_url is required")]
    VersionNoBinaryUrl = 1001,
    #[error("path is required")]
    VersionNoPath = 1002,
    #[error("destination_did is required")]
    CreateDidCommMessageNoDestinationDid = 1003,
    #[error("message is required")]
    CreateDidCommMessageNoMessage = 1004,
    #[error("operation_tag is required")]
    CreateDidCommMessageNoOperationTag = 1005,
    #[error("cannot find public key")]
    CreateDidCommMessageNoPubKey = 1006,
    #[error("destination_did is required")]
    CreateVerifiableMessageNoDestinationDid = 1007,
    #[error("message is required")]
    CreateVerifiableMessageNoMessage = 1008,
    #[error("operation_tag is required")]
    CreateVerifiableMessageNoOperationTag = 1009,
    #[error("sender not found")]
    VerifyDidcommMessageNoSender = 1010,
    #[error("public key not found")]
    VerifyDidcommMessageNoPublicKey = 1011,
    #[error("metadata not found")]
    VerifyDidcommMessageNoMetadata = 1012,
    #[error("json error")]
    VerifyDidcommMessageJsonError = 1013,
    #[error("public key not found")]
    VerifyVerifiableMessageNoPublicKey = 1014,
    #[error("json error")]
    VerifyVerifiableMessageJsonError = 1015,
    #[error("key_name is required")]
    SendAttributeNoKeyName = 1016,
    #[error("value is required")]
    SendAttributeNoValue = 1017,
    #[error("key is required")]
    SendCustomMetricNoKey = 1018,
    #[error("occurred_at is invalid format")]
    SendCustomMetricInvalidOccurredAt = 1019,
    #[error("key is invalid")]
    SendEventNoKey = 1020,
    #[error("detail is invalid")]
    SendEventNoDetail = 1021,
    #[error("occurred_at is invalid format")]
    SendEventInvalidOccurredAt = 1022,
    #[error("Bad Request")]
    MessageActivityBadRequest = 1023,

    #[error("this message is not addressed to me")]
    VerifyDidcommMessageNotAddressedToMe = 2001,
    #[error("this message is not addressed to me")]
    VerifyVerifiableMessageNotAddressedToMe = 2002,
    #[error("Forbidden")]
    MessageActivityForbidden = 2003,

    #[error("verify failed")]
    CreateDidCommMessageVerifyFailed = 3001,
    #[error("verify failed")]
    VerifyDidcommMessageVerifyFailed = 3002,
    #[error("verify failed")]
    VerifyVerifiableMessageVerifyFailed = 3003,
    #[error("Unauthorized")]
    MessageActivityUnauthorized = 3004,

    #[error("target DID not found")]
    CreateDidCommMessageNoDid = 4001,
    #[error("target DID not found")]
    CreateVerifiableMessageNoTargetDid = 4002,
    #[error("target DID not found")]
    VerifyDidcommMessageNoTargetDid = 4003,
    #[error("identifier not found")]
    VerifyVerifiableMessageNoIdentifier = 4004,
    #[error("target DID not found")]
    VerifyVerifiableMessageNoTargetDid = 4005,
    #[error("Not Found")]
    MessageActivityNotFound = 4006,

    #[error("Internal Server Error")]
    NetworkInternal = 5001,
    #[error("Internal Server Error")]
    VersionInternal = 5002,
    #[error("Internal Server Error")]
    CreateDidcommMessageInternal = 5003,
    #[error("Internal Server Error")]
    CreateIdentifierInternal = 5004,
    #[error("Internal Server Error")]
    CreateVerifiableMessageInternal = 5005,
    #[error("Internal Server Error")]
    FindIdentifierInternal = 5006,
    #[error("Internal Server Error")]
    VerifyDidcommMessageInternal = 5007,
    #[error("Internal Server Error")]
    VerifyVerifiableMessageInternal = 5008,
    #[error("Internal Server Error")]
    SendAttributeInternal = 5009,
    #[error("Internal Server Error")]
    SendCustomMetricInternal = 5010,
    #[error("Internal Server Error")]
    SendEventInternal = 5011,
    #[error("Internal Server Error")]
    MessageActivityInternal = 5012,

    #[error("it have already been verified")]
    MessageActivityConflict = 6001,
}

#[derive(Serialize, Debug)]
pub struct AgentError {
    code: u16,
    message: String,
}

impl std::fmt::Display for AgentError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "code: {}, message: {}", self.code, self.message)
    }
}

impl AgentError {
    pub fn new(code: AgentErrorCode) -> Self {
        Self {
            code: code as u16,
            message: format!("{}", code),
        }
    }
}
impl From<&AgentError> for HttpResponse {
    fn from(error: &AgentError) -> Self {
        let code = error.code;
        if (1000..2000).contains(&code) {
            HttpResponse::BadRequest().json(error)
        } else if (2000..3000).contains(&code) {
            HttpResponse::Forbidden().json(error)
        } else if (3000..4000).contains(&code) {
            HttpResponse::Unauthorized().json(error)
        } else if (4000..5000).contains(&code) {
            HttpResponse::NotFound().json(error)
        } else if (5000..6000).contains(&code) {
            HttpResponse::InternalServerError().json(error)
        } else if (6000..6100).contains(&code) {
            HttpResponse::Conflict().json(error)
        } else {
            HttpResponse::InternalServerError().json(error)
        }
    }
}

impl From<AgentErrorCode> for AgentError {
    fn from(code: AgentErrorCode) -> Self {
        AgentError::new(code)
    }
}

impl error::ResponseError for AgentError {
    fn error_response(&self) -> HttpResponse {
        self.into()
    }

    fn status_code(&self) -> StatusCode {
        let res: HttpResponse = self.into();
        res.status()
    }
}
