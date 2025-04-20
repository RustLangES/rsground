import { ServerMessage, ServerMessageKind } from "./server";

export type WsCallback<S extends ServerMessageKind = ServerMessageKind> = (msg: ServerMessage<S>) => void;
