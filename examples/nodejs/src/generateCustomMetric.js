import { post } from "./lib/sock.js";

(async () => {
  const payload = {
    key: "test-key",
    value: 10.52,
    occurred_at: String(Math.floor(Date.now() / 1000)),
  };
  const response = await post("/custom_metrics", payload);

  console.log("The response is as follows.\n");
  console.log(response);
})();
