// import { WebSocket } from "ws";

// const { WebSocket } = await import("ws");
console.log(WebSocket);
const ws = new WebSocket(
  "http://127.0.0.1:8080/chats/d9b49810-a1cb-440a-9e66-c293aa61d4d9"
);

// ws.on("open", () => {
//   console.log("Loaded");
// });
// ws.on("error", (e) => {
//   console.log("Error", e);
// });
