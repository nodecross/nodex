import { get } from "./lib/sock.js";

// PLEASE WRITE device_name
const device_name = "DEVICE_NAME";

(async () => {
  const response = await get(`/identifiers?device_name=${device_name}`);

  console.log("The response is as follows.\n");
  console.log(response);
})();
