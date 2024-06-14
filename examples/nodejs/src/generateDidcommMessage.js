import { post } from "./lib/sock.js";

(async () => {
  const destination_did =
    "did:nodex:test:DummyDummyDummyDummyDummyDummyDummyDummyDummyD";
  const messageJson = {
    message: {
      string: "value",
      number: 1,
      boolean: true,
      array: ["foo", "bar", "baz"],
      map: { key: "value" },
    },
  };
  const message = JSON.stringify(messageJson, null, 4);

  const response = await post("/create-didcomm-message", {
    destination_did,
    message,
    operation_tag: "test-operation-tag",
  });

  console.log('\nPlease paste below to "verifyDidcommMessage.js".\n');
  console.log(response);
})();
