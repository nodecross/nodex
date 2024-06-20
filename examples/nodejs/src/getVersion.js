import { post } from "./lib/sock.js";

(async () => {
  const response = await post("/internal/version/get");

  console.log("The response is as follows.\n");
  console.log(response);
})();
