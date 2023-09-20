import WebSocket from 'ws';
import { base } from "./sock.js";

const URL = 'ws+' + base + ':/receive';
console.log("connecting to " + URL);
const socket = new WebSocket(URL);

console.log("socket connected");
socket.on('open', () => {
    console.log("socket opened");
})

socket.on('message', (data) => {
    console.log("socket received: " + data);
})

// close the socket after 30 seconds
setTimeout(() => {
    console.log("closing socket");
    socket.close();
}, 30000);