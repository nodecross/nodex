import { RawData } from "ws";
import got from "got";
import socket, { base } from "./socket";

/**
 * Receive and handle the freeable memory message from other DID device from the NodeX Agent API.
 */
export async function receive(data: RawData) {
  const { credentialSubject, message_id, issuer } = JSON.parse(
    data.toString()
  ).message;
  const response = {
    message_id,
  };

  console.log(
    `[${new Date().toISOString()}] Received: from device did: ${
      issuer.id
    } with ${credentialSubject?.container?.[0].free} freeable memory `
  );

  socket.send(JSON.stringify(response));
}

/**
 * Send the freeable memory to a destination's DID device using the NodeX Agent API.
 *
 * @see {@link https://docs.nodecross.io/reference/agent-api/index.html#transfer|Transfer, NodeX}
 */
export async function send(did: string, free: number): Promise<any> {
  const json = {
    destinations: [did],
    messages: [
      {
        free,
      },
    ],
    metadata: {},
  };

  return await got
    .post([base, "/transfer"].join(":"), {
      enableUnixSockets: true,
      json,
    })
    .json();
}
