import { post } from "./lib/sock.js";

(async () => {
  const payload = {
    message: "test-message",
    occurred_at: Date.now(),
  };
  const response = await post("/logs", payload);

  console.log("The response is as follows.\n");
  console.log(response);
})();
