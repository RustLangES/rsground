import { createSignal } from "solid-js";
import { createMutable } from "solid-js/store";
import { ClientMessage } from "../types";

export const [wsSession, setWsSession] = createSignal<WebSocket>(null)

export const [wsSessionId, setWsSessionId] = createSignal<string>(null)

export const wsQueue = createMutable<ClientMessage[]>([])
