import { authInfo } from "@features/auth/stores";
import { observable, untrack } from "solid-js";
import { setWsSession, setWsSessionId, wsQueue, wsSession } from "../stores";
import { BACKEND_HOST } from "@services";
import { ClientMessage, ServerMessage, ServerMessageKind, WsCallback } from "../types";

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

  onWsMessage(ServerMessageKind.UserConnected, (msg) => {
    setWsSessionId(msg.user_id);
  })
}

const wsUrl = new URL(BACKEND_HOST);
wsUrl.protocol = wsUrl.protocol === "http:" ? "ws:" : "wss:";
wsUrl.pathname = "/ws";

const ws_callbacks: Array<(msg: ServerMessage) => void> = [];

function connectWs(jwt: string) {
  const session = new WebSocket(wsUrl, [`auth.${jwt}`]);

  session.addEventListener("open", () => {
    setWsSession(session);
  });

  session.addEventListener("message", (ev) => {
    let data = JSON.parse(ev.data) as ServerMessage;
    console.debug(data)

    for (const cb of ws_callbacks) {
      cb(data)
    }
  });

  session.addEventListener("error", (ev) => {
    setWsSession(null);
    console.error("Websocket error:", ev)
  });

  session.addEventListener("close", () => {
    setWsSession(null);
  });
}

/** Register callback for websocket messages. Returns unsubscribe */
export function onWsMessage(cb: WsCallback): () => void;

/** Register callback for specific websocket messages. Returns unsubscribe */
export function onWsMessage<A extends ServerMessageKind>(action: A, cb: WsCallback<A>): () => void;
/** Register callback for specific websocket messages. Returns unsubscribe */
export function onWsMessage<A extends ServerMessageKind>(actions: A[], cb: WsCallback<A>): () => void;

export function onWsMessage(actions_or_cb: string | string[] | WsCallback, maybe_cb?: WsCallback): () => void {
  let cb: WsCallback;

  if (actions_or_cb instanceof Array) {
    cb = (msg) => {
      if (actions_or_cb.includes(msg.action)) {
        maybe_cb!(msg)
      }
    }
  } else if (typeof actions_or_cb === "string") {
    cb = (msg) => {
      if (actions_or_cb === msg.action) {
        maybe_cb!(msg)
      }
    }
  } else {
    cb = maybe_cb!;
  }

  ws_callbacks.push(cb);

  return () => {
    let idx = ws_callbacks.findIndex(v => v == cb);
    ws_callbacks.splice(idx, 1);
  }
}

export function sendMessage(msg: ClientMessage) {
  const session = untrack(wsSession);
  if (session) {
    session.send(JSON.stringify(msg));
  } else {
    wsQueue.push(msg);
  }
}
