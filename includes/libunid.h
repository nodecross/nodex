void libunid_init();
char* ciphers_hasher_digest(char* content, char* secret);
bool ciphers_hasher_verify(char* content, char* digest, char* secret);