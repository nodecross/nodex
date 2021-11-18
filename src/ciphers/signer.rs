use alloc::string::{String, ToString};
use libsecp256k1::{Message, PublicKey, PublicKeyFormat, SecretKey, Signature};

const PROOF_KEY: &str = "proof";
const VM_KEY: &str = "verificationMethod";
const JWS_KEY: &str = "jws";

struct SuiteSign {
    did: String,
    key_id: String,
    secret_key64: String,
}

struct SuiteVerify {
    _did: Option<String>,
    key_id: String,
    pub_key64: String,
}

pub struct Signer {}

pub struct Jws {}

pub struct CredentialSigner {}

impl CredentialSigner {
    pub fn sign(object_json: JsValue, suite_sign_json: JsValue) -> JsValue {
        let suite_sign: SuiteSign = suite_sign_json.into_serde().unwrap();
        let obj: js_sys::Object = js_sys::Object::from(object_json.clone());
        let proof_key: JsValue = JsValue::from_str(PROOF_KEY);
        assert!(!obj.has_own_property(&proof_key));
        let created: String = Utc::now().format("%Y-%m-%dT%I:%M:%SZ").to_string();
        let jws = Jws::encode(object_json.clone(), suite_sign.secret_key64);

        let proof_serde: serde_json::Value = json!({
            "type": "EcdsaSecp256k1Signature2019",
            "proofPurpose": "authentication",
            "created": created,
            "verificationMethod": format!("{}#{}",suite_sign.did,suite_sign.key_id),
            "jws": jws,
        });
        let proof_json: JsValue = JsValue::from_serde(&proof_serde).unwrap();

        let obj_clone = object_json;
        let result1 = js_sys::Reflect::set(&obj_clone, &proof_key, &proof_json);
        assert!(result1.is_ok());

        obj_clone
    }

    pub fn verify(object_json: JsValue, suite_verify_json: JsValue) -> JsValue {
        let suite_verify: SuiteVerify = suite_verify_json.into_serde().unwrap();
        let obj: js_sys::Object = js_sys::Object::from(object_json.clone());
        let proof_key: JsValue = JsValue::from_str(PROOF_KEY);
        assert!(obj.has_own_property(&proof_key));
        let proof_json: JsValue = js_sys::Reflect::get(&object_json, &proof_key).unwrap();
        let proof_obj: js_sys::Object = js_sys::Object::from(proof_json.clone());
        assert!(!proof_obj.is_undefined());
        assert!(!proof_obj.is_null());
        let vm_key: JsValue = JsValue::from_str(VM_KEY);
        let vm_json: JsValue = js_sys::Reflect::get(&proof_json, &vm_key).unwrap();
        let vm_obj: js_sys::Object = js_sys::Object::from(vm_json);
        let vm_string: String = vm_obj.as_string().unwrap();
        let vm_array: Vec<&str> = vm_string.split('#').collect();
        let vm_key_id: &str = vm_array[1];

        assert_eq!(vm_key_id, suite_verify.key_id);
        let jws_key: JsValue = JsValue::from_str(JWS_KEY);
        let jws_json: JsValue = js_sys::Reflect::get(&proof_json, &jws_key).unwrap();
        let jws_obj: js_sys::Object = js_sys::Object::from(jws_json);
        let jws_string: String = jws_obj.as_string().unwrap();

        let obj_clone: js_sys::Object = js_sys::Object::from(object_json);
        let result1 = js_sys::Reflect::delete_property(&obj_clone, &proof_key);
        assert!(result1.is_ok());
        let payload_key: JsValue = JsValue::from_str("payload");
        let payload_json: JsValue = obj_clone.into();

        let is_valid: bool = Jws::verify(payload_json.clone(), jws_string, suite_verify.pub_key64);
        let is_valid_serde: serde_json::Value = json!({ "isValid": is_valid });
        let is_valid_json: JsValue = JsValue::from_serde(&is_valid_serde).unwrap();

        let out = is_valid_json;
        let result2 = js_sys::Reflect::set(&out, &payload_key, &payload_json);
        assert!(result2.is_ok());
        out
    }
}

impl Jws {
    pub fn encode(object: JsValue, secret_key64: String) -> String {
        let header: serde_json::Value = json!({
            "alg" : "ES256K",
            "b64" : false,
            "crit": [ "b64" ]
        });
        let header_json_string: String = serde_json::to_string(&header).unwrap();
        let header_json_str: &str = &header_json_string;
        let header_json_u8: &[u8] = header_json_str.as_bytes();
        let header64_url: String = base64_url::encode(header_json_u8);

        let object_json_string: String = js_sys::JSON::stringify(&object).unwrap().into();
        let object_json_str: &str = &object_json_string;
        let object_json_u8: &[u8] = object_json_str.as_bytes();
        let object64_url: String = base64_url::encode(object_json_u8);

        let message = format!("{}.{}", header64_url, object64_url);
        let signature64 = Signer::sign(message, secret_key64);
        let signature64_vec: Vec<u8> = base64::decode(signature64).unwrap();
        let signature64_u8: &[u8] = &signature64_vec[..];
        let signature64_url = base64_url::encode(signature64_u8);

        format!("{}..{}", header64_url, signature64_url)
    }

    pub fn verify(object: JsValue, jws: String, pub_key64: String) -> bool {
        let jws_array: Vec<&str> = jws.split('.').collect();
        let header64_url: &str = jws_array[0];
        let payload64_url: &str = jws_array[1];
        let signature64_url: &str = jws_array[2];
        let header_json_vec: Vec<u8> = base64_url::decode(header64_url).unwrap();
        let header_json_string: String = String::from_utf8(header_json_vec).unwrap();

        let header_json: serde_json::Value = serde_json::from_str(&header_json_string).unwrap();

        assert_eq!(header_json["alg"], "ES256K");
        assert!(header_json["b64"].is_boolean());
        assert!(!header_json["b64"].as_bool().unwrap());
        assert!(header_json["crit"]
            .as_array()
            .unwrap()
            .contains(&json!("b64")));

        assert_eq!(payload64_url, "");

        let object_json_string: String = js_sys::JSON::stringify(&object).unwrap().into();
        let object_json_str: &str = &object_json_string;
        let object_json_u8: &[u8] = object_json_str.as_bytes();
        let object64_url: String = base64_url::encode(object_json_u8);

        let message = format!("{}.{}", header64_url, object64_url);
        let signature64_vec: Vec<u8> = base64_url::decode(signature64_url).unwrap();
        let signature64: String = base64::encode(signature64_vec);

        Signer::verify(message, signature64, pub_key64)
    }
}

impl Signer {
    pub fn sign(_message: String, _secret: String) -> String {
        let message_u8 = message.as_bytes();
        let secret_u8 = secret.as_bytes();

        let digested = Sha256::digest(message_u8);
        let digested_u8: &[u8] = &digested.to_vec()[..];
        let digested_message = Message::parse_slice(digested_u8).unwrap();

        let secret_key_vec: Vec<u8> = base64::decode(secret_u8).unwrap();
        let secret_key_u8: &[u8] = &secret_key_vec[..];

        let secret_key_sk = SecretKey::parse_slice(secret_key_u8).unwrap();
        let sig_tuple = libsecp256k1::sign(&digested_message, &secret_key_sk);
        let sig = sig_tuple.0;
        let sig_u8 = sig.serialize();

        base64::encode(sig_u8.to_vec())
    }

    pub fn verify(_message: String, _signature64: String, _pub_key64: String) -> bool {
        let message_str: &str = &message;
        let message_u8: &[u8] = message_str.as_bytes();
        let message_u8_json = json!({ "type":"Buffer", "data":message_u8 });
        let message_u8_json_string: String = message_u8_json.to_string();
        let message_u8_json_str: &str = &message_u8_json_string;
        let message_u8_json_u8: &[u8] = message_u8_json_str.as_bytes();

        let digested = Sha256::digest(message_u8_json_u8);
        let digested_u8: &[u8] = &digested.to_vec()[..];
        let digested_message = Message::parse_slice(digested_u8).unwrap();

        let signature_vec: Vec<u8> = base64::decode(signature64.as_bytes()).unwrap();
        let signature_u8: &[u8] = &signature_vec[..];
        let sig = Signature::parse_standard_slice(signature_u8).unwrap();

        let pub_key_vec: Vec<u8> = base64::decode(pub_key64.as_bytes()).unwrap();
        let pub_key_u8: &[u8] = &pub_key_vec[..];
        let pub_key_pk = PublicKey::parse_slice(pub_key_u8, Some(PublicKeyFormat::Full)).unwrap();

        libsecp256k1::verify(&digested_message, &sig, &pub_key_pk)
    }
}

#[cfg(test)]
pub mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    const D: &str = "TFuxm1wXoGUlO+CDpJkw+9kUc8YPc1k4nisoC1y6/J4=";
    const XY: &str =
        "BNpc0uIAkafgMJBcSVJByl7ejx4rKgTDxijwM1mGMXwkZiGu2CIQ7XPa9SImgqSs2H8tQqQssYNPzNNu07tVUJI=";

    #[test]
    fn it_should_signer_sign_verify_1() {
        let data_serde: serde_json::Value = json!({
            "id" : "did:self:0x0123456789012345678901234567890123456789"
        });
        let data: &str = &data_serde.to_string();
        let signature: String = Signer::sign(data.to_string(), D.to_string());
        let verified: bool = Signer::verify(data.to_string(), signature, XY.to_string());
        assert!(verified);
    }

    fn it_should_jws_encode_verify_1() {
        let data_serde: serde_json::Value = json!({
            "test" : "ok"
        });
        let data_json: JsValue = JsValue::from_serde(&data_serde).unwrap();

        let jws: String = Jws::encode(data_json.clone(), D.to_string());
        let verified: bool = Jws::verify(data_json, jws, XY.to_string());

        assert!(verified);
    }

    fn it_should_credential_signer_sign_verify_1() {
        let suite_sign_serde: serde_json::Value = json!({
            "did" : "did:unid:test:EiBtzgWy130lNOyO3JsHkR75YFeSgU7h4p6zYvfQxrAXeA",
            "key_id" : "signingKey",
            "secret_key64" : D
        });
        let suite_verify_serde: serde_json::Value = json!({
            "key_id" : "signingKey",
            "pub_key64" : XY
        });

        let data_serde: serde_json::Value = json!({
            "test" : "ok"
        });

        let data_json: JsValue = JsValue::from_serde(&data_serde).unwrap();

        let suite_sign_json: JsValue = JsValue::from_serde(&suite_sign_serde).unwrap();
        let suite_verify_json: JsValue = JsValue::from_serde(&suite_verify_serde).unwrap();

        let document: JsValue = CredentialSigner::sign(data_json.clone(), suite_sign_json);

        let verified = CredentialSigner::verify(document, suite_verify_json);

        let obj: js_sys::Object = js_sys::Object::from(verified.clone());
        let is_valid_key: JsValue = JsValue::from_str("isValid");
        let payload_key: JsValue = JsValue::from_str("payload");
        assert!(obj.has_own_property(&is_valid_key));
        assert!(obj.has_own_property(&payload_key));

        let is_valid_json: JsValue = js_sys::Reflect::get(&verified, &is_valid_key).unwrap();
        let is_valid_json_bool = is_valid_json.as_bool();
        assert_ne!(is_valid_json_bool, None);
        let is_valid_bool: bool = is_valid_json_bool.unwrap();
        assert!(is_valid_bool);
        let payload_json: JsValue = js_sys::Reflect::get(&verified, &payload_key).unwrap();
        assert_eq!(payload_json, data_json);
    }
}
