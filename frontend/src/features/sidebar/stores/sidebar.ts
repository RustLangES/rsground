import { createSignal } from "solid-js";

// Close sidebar when it uses more than 1/3 of screen
const defaultOpened = window.innerWidth > 750;

export const [isSidebarOpen, setIsSidebarOpen] = createSignal(defaultOpened);
