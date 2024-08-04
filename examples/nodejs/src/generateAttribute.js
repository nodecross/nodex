import { post } from "./lib/sock.js";

(async () => {
  const payload = {
    key_name: "test-key-name",
    value: "test-value",
  };
  const response = await post("/attributes", payload);

  console.log("The response is as follows.\n");
  console.log(response);
})();
