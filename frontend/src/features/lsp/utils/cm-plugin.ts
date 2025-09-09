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

export function rsLsp(file_path: string) {
  if (!file_path.endsWith(".rs")) return [];

  return [languageServerWithTransport({
    client: lsp_client,
    documentUri: "file:///home/" + file_path,
    languageId: "rust",
    allowHTMLContent: true,

    // XXX: not needed really, but
    // this function force to provide it
    transport: undefined,
    rootUri: "",
    workspaceFolders: [],
  })];
}
