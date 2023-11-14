import * as os from "os";
import * as path from "path";
import WebSocket from "ws";
import { receive } from "./handler";

export const base = `unix:${path.join(os.homedir(), ".nodex/run/nodex.sock")}`;

const socket = new WebSocket("ws+" + base + ":/receive");
socket.on("message", receive);

export default socket;
