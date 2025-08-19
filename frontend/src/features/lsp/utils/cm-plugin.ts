import {
  LanguageServerClient,
  languageServerWithTransport,
} from "codemirror-languageserver";
import { RsLspTransport } from "./transport";

const lsp_client = new LanguageServerClient({
  transport: new RsLspTransport(),
  rootUri: "file:///home",
  workspaceFolders: [],
  languageId: "rust",

  // XXX: not needed really, but client
  // force to provide it
  documentUri: "",
});

export function rsLsp() {
  return languageServerWithTransport({
    client: lsp_client,
    documentUri: "file:///home/main.rs",

    // XXX: not needed really, but
    // this function force to provide it
    transport: undefined,
    rootUri: "",
    workspaceFolders: [],
    languageId: "",
  });
}
