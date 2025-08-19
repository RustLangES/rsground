import {
  LanguageServerClient,
  languageServerWithTransport,
} from "codemirror-languageserver";
import { RsLspTransport } from "./transport";

const lsp_client = new LanguageServerClient({
  transport: new RsLspTransport(),
  rootUri: "file:///home",
  workspaceFolders: [],

  // XXX: not needed really, but client
  // force to provide it
  documentUri: "",
  languageId: "",
});

export function rsLsp() {
  return languageServerWithTransport({
    client: lsp_client,
    documentUri: "file:///home/src/main.rs",
    languageId: "rust",

    // XXX: not needed really, but
    // this function force to provide it
    transport: undefined,
    rootUri: "",
    workspaceFolders: [],
  });
}
