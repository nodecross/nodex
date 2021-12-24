void libunid_init();
char *ciphers_hasher_digest(char *content, char *secret);
bool ciphers_hasher_verify(char *content, char *digest, char *secret);

typedef struct
{
    char *client_id;
    char *client_secret;
} UNiDConfig;

typedef struct
{
    char *client_id;
    char *client_secret;
} UNiDContext;

UNiDContext unid_init(UNiDConfig config);
void aes_init(void *encryptor, void *decryptor);
void ecdsa_init(void *kp_generator, void *signer, void *verifier);

char *unid_core_create_did(UNiDContext context);
char *unid_core_resolve_did(UNiDContext context);
char *unid_core_update_did(UNiDContext context);
char *unid_core_revoke_did(UNiDContext context);
char *unid_core_verify_credentials(UNiDContext context);
char *unid_core_verify_presentations(UNiDContext context);
char *unid_did_create_credentials(UNiDContext context);
char *unid_did_create_presentations(UNiDContext context);

char *unid_runtime_bip39_generate_mnemonic();
char *unid_utils_random_get_random_bytes(int length);
char *unid_utils_codec_base64_encode(char *content);
char *unid_utils_codec_base64_decode(char *content);
char *unid_utils_multihasher_hash(char *content);
char *unid_ciphers_signer_kp_gen();
char *unid_ciphers_signer_sign(char *message, char *secret_key);
char *unid_ciphers_signer_verify(char *message, char *signature64, char *pub_key64);
char *unid_ciphers_cipher_encrypt(char *plaintext, char *secret);
char *unid_ciphers_cipher_decrypt(char *buffered_ciphertext_base64, char *secret);
char *unid_ciphers_hasher_digest(char *content, char *secret);
int unid_ciphers_hasher_verify(char *content, char *digest, char *secret);