import "./App.sass";
import "../public/fonts/inter.css"

import { Component } from "solid-js";
import { checkForAuth, interceptAuthCallback } from "@features/auth/utils";
import { interceptProjectRoutes } from "@features/colab/utils";
import { startReceivingSync } from "@features/editor/services";
import { Panels } from "@features/panels/views";
import { Sidebar } from "@features/sidebar/views";

import "@features/theme/stores"
import { LoadingLsp } from "@features/lsp/views";

const App: Component = () => {
  interceptAuthCallback();
  checkForAuth();
  interceptProjectRoutes();
  startReceivingSync();

  return (
    <>
      <Sidebar />
      <Panels />
      <LoadingLsp />
    </>
  );
};

export default App;
