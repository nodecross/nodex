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

typedef struct
{
    uint8_t *ptr;
    uint32_t len;
} data_t;

UNiDContext unid_init(UNiDConfig config);

void unid_regist_handler_on_memory_alloc(void *handler);
void unid_regist_handler_on_memory_dealloc(void *handler);
void unid_regist_handler_on_debug_message(void *handler);
void unid_regist_handler_on_aes_encryptor(void *handler);
void unid_regist_handler_on_aes_decryptor(void *handler);
void unid_regist_handler_on_ecdsa_signer(void *handler);
void unid_regist_handler_on_ecdsa_verifier(void *handler);

void unid_disposer(void *ptr);

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
char *unid_ciphers_signer_sign(char *message, char *secretkey);
int unid_ciphers_signer_verify(char *message, char *signature, char *pubkey);
char *unid_ciphers_cipher_encrypt(char *plaintext, char *secret);
char *unid_ciphers_cipher_decrypt(char *ciphertext, char *secret);
char *unid_ciphers_hasher_digest(char *content, char *secret);
int unid_ciphers_hasher_verify(char *content, char *digest, char *secret);

void unid_test();
