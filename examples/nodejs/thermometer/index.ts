import * as os from "os";
import * as path from "path";
import WebSocket from "ws";
import got from "got";
import { setTimeout } from "timers/promises";
import { argv } from "process";

export const base = `unix:${path.join(os.homedir(), ".nodex/run/nodex.sock")}`;

const URL = "ws+" + base + ":/receive";
const socket = new WebSocket(URL);

const destination = argv[2];
if (!destination) throw new Error("Destination did is required");

socket.on("message", (data) => {
  const { credentialSubject, message_id, issuer } = JSON.parse(
    data.toString()
  ).message;
  const response = {
    message_id,
  };
  console.log(
    `[${new Date().toISOString()}] Received: from device did: ${
      issuer.id
    } cpu temperature with ${
      credentialSubject?.container?.[0].temperature
    } celsius degrees`
  );
  socket.send(JSON.stringify(response));
});

async () => {
  console.log("Start thermometer example. Connect to " + URL);

  while (true) {
    await setTimeout(5000 /* 5 seconds */);

    console.log(
      `[${new Date().toISOString()}] Sent: with temperature ${1} celsius degrees to did: ${destination}`
    );
  }
};

process.on("SIGINT", function () {
  socket.close();
  console.log("Finish the thermometer example");
  process.exit();
});
