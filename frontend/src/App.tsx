import "./App.sass";
import "../public/fonts/inter.css"

import { Component } from "solid-js";
import { interceptAuthCallback } from "@features/auth/utils";
import { Panels } from "@features/panels/views";
import { Sidebar } from "@features/sidebar/views";
import { startWebsocket } from "@features/ws/services";

import "@features/theme/stores"

const App: Component = () => {
  interceptAuthCallback();
  startWebsocket();

  return (
    <>
      <Sidebar />
      <Panels />
    </>
  );
};

export default App;
