import { createStore } from "solid-js/store";

// All rendered nodes
export const [outputPanel, setOutputPanel] = createStore<Array<HTMLElement>>([]);

/** Unprocessed part of malformed ansi code */
export let outputCarrier: string = "";

export function setOutputCarrier(carrier: string) {
  outputCarrier = carrier;
}
