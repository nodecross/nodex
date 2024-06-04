import { get } from "./lib/sock.js";

(async () => {
  const destination_did =
    "did:nodex:test:DummyDummyDummyDummyDummyDummyDummyDummyDummyD";
  const response = await get(`/identifiers/${destination_did}`);

  console.log(response);
})();
