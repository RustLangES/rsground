import {
  LanguageServerClient,
  languageServerWithTransport,
} from "codemirror-languageserver";
import { highlightCode } from "@lezer/highlight";
import { marked } from "marked";
import { RsLspTransport } from "./transport";
import { rsgroundTheme } from "@features/editor/utils";
import { rustLanguage } from "@codemirror/lang-rust";

marked.use({
  renderer: {
    code(code) {
      if (code.lang != "rust") return false;

      let result = document.createElement("pre");

      function emit(text: string, classes: string) {
        let node = document.createTextNode(text) as Node;
        if (classes) {
          let span = document.createElement("span");
          span.appendChild(node);
          span.className = classes;
          node = span;
        }
        result.appendChild(node);
      }

      function emitBreak() {
        result.appendChild(document.createTextNode("\n"));
      }

      highlightCode(
        code.text,
        rustLanguage.parser.parse(code.text),
        rsgroundTheme,
        emit,
        emitBreak,
      );

      return result.outerHTML;
    },
  },
});

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

  return languageServerWithTransport({
    client: lsp_client,
    documentUri: "file:///home/" + file_path,
    languageId: "rust",
    allowHTMLContent: true,

    // XXX: not needed really, but
    // this function force to provide it
    transport: undefined,
    rootUri: "",
    workspaceFolders: [],
  });
}
