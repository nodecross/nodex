import init, { encrypt, decrypt } from './pkg/cipher_lib.js';

async function run() {
  await init();
  let abc = encrypt("Hello world!", "secret");
  console.log(abc);

  let cba = decrypt( "TJWAkEKJ7hAatpmR+CwiJfOJYt/kwKBSzRSm7qhGs760rqPWa5LX9b9oMIQpz+uS0hAZxKYhrDk7fr6zh58yxQ==", "secret");
  console.log(cba);
}
run();