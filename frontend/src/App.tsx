import "./App.sass";
import "../public/fonts/inter.css"

import { Component } from "solid-js";
import { checkForAuth, interceptAuthCallback } from "@features/auth/utils";
import { interpectProjectRoutes } from "@features/colab/utils";
import { startReceivingSync } from "@features/editor/services";
import { Panels } from "@features/panels/views";
import { Sidebar } from "@features/sidebar/views";

import "@features/theme/stores"

const App: Component = () => {
  interceptAuthCallback();
  checkForAuth();
  interpectProjectRoutes();
  startReceivingSync();

  return (
    <>
      <Sidebar />
      <Panels />
    </>
  );
};

export default App;
