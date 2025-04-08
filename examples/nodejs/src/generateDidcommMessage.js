import { post } from "./lib/sock.js";

// PLEASE WRITE destination_did, messageJson, AND operation_tag.
const destination_did =
  "did:webvh:DummyDummyDummyDummyDummyDummyDummyDummyDummyD";
const messageJson = {
  message: {
    string: "value",
    number: 1,
    boolean: true,
    array: ["foo", "bar", "baz"],
    map: { key: "value" },
  },
};

(async () => {
  const message = JSON.stringify(messageJson, null, 4);
  const response = await post("/create-didcomm-message", {
    destination_did,
    message,
  });

  console.log('\nPlease paste below to "verifyDidcommMessage.js".\n');
  console.log(response);
})();
