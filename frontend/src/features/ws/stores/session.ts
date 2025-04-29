import { createSignal } from "solid-js";
import { ClientMessage } from "../types";

export const [wsSession, setWsSession] = createSignal<WebSocket>(null)

export const [wsSessionId, setWsSessionId] = createSignal<string>(null)

export let wsQueue: ClientMessage[] = [];
export const clearWsQueue = () => { wsQueue = [] }
