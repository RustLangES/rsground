import { authInfo } from "@features/auth/stores";
import { observable, untrack } from "solid-js";
import { setWsSession, wsQueue, wsSession } from "../stores";
import { BACKEND_HOST } from "@services";
import { ClientMessage } from "../types";

export function startWebsocket() {
  observable(authInfo).subscribe((authInfo) => {
    if (authInfo?.jwt) {
      connectWs(authInfo.jwt);
    } else {
      setWsSession(null);
    }
  });

  observable(wsSession).subscribe((wsSession) => {
    if (wsSession) {
      for (const msg of wsQueue) {
        wsSession.send(JSON.stringify(msg));
      }
    }
  });
}

const wsUrl = new URL(BACKEND_HOST);
wsUrl.protocol = wsUrl.protocol === "http:" ? "ws:" : "wss:";
wsUrl.pathname = "/ws";

function connectWs(jwt: string) {
  const session = new WebSocket(wsUrl, [`auth=${jwt}`]);

  session.addEventListener("open", () => {
    setWsSession(session);
  });

  session.addEventListener("error", (ev) => {
    setWsSession(null);
    console.error("Websocket error:", ev)
  });

  session.addEventListener("close", () => {
    setWsSession(null);
  });
}

export function sendMessage(msg: ClientMessage) {
  const session = untrack(wsSession);
  if (session) {
    session.send(JSON.stringify(msg));
  } else {
    wsQueue.push(msg);
  }
}
