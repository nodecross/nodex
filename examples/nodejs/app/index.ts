import { setTimeout } from "timers/promises";
import { argv } from "process";
import si from "systeminformation";
import { send } from "./handler";
import socket from "./socket";

const destination = argv[2];
if (!destination) throw new Error("Destination did is required");

(async () => {
  console.log("Start application, destination did: " + destination);

  while (true) {
    await setTimeout(5000 /* 5 seconds */);

    const { free } = await si.mem();
    const { results } = await send(destination, free);

    console.log(
      `[${new Date().toISOString()}] Sent: with ${free} freeable memory to did: ${
        results[0].destination
      }`
    );
  }
})();

process.on("SIGINT", function () {
  socket.close();
  console.log("Finish the application");
  process.exit();
});
