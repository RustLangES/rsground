import {
  getBatchRequests,
  getNotifications,
  JSONRPCRequestData,
} from "@open-rpc/client-js/build/Request";
import { Transport } from "@open-rpc/client-js/build/transports/Transport";
import { JSONRPCError } from "@open-rpc/client-js";
import { ERR_UNKNOWN } from "@open-rpc/client-js/build/Error";

import { onWsMessage, sendMessage } from "@features/ws/services";
import { ClientMessageKind, ServerMessageKind } from "@features/ws/types";
import { ProgressParams } from "../types";
import { endLoadingLsp, updateLoadingLsp } from "../services";
import { MessageType, ShowMessageParams } from "vscode-languageserver-protocol";
import { showToast, ToastKind } from "@services/toast";

const ignoredNotify = {
  "textDocument/didOpen": "open",
  "textDocument/didChange": "change",
};

export class RsLspTransport extends Transport {
  public async connect(): Promise<void> {
    onWsMessage(ServerMessageKind.Lsp, ({ data }) => {
      if (
        "method" in data && "params" in data &&
        this.handleWorkDone(data.method as string, data.params)
      ) {
        return;
      }

      // @ts-expect-error -- resolveRes is private, but needed for performance issues
      this.transportRequestManager.resolveRes(data);
    });
  }

  private handleWorkDone(method: string, params_: unknown): boolean {
    if (method == "window/showMessage") {
      let params = params_ as ShowMessageParams;

      let kind: ToastKind = "info";

      if (params.type == MessageType.Debug) {
        kind = "debug";
      } else if (params.type == MessageType.Error) {
        kind = "error";
      } else if (params.type == MessageType.Warning) {
        kind = "warn";
      }

      showToast(kind, { text: params.message });

      return true;
    }

    if (method != "$/progress") {
      return false;
    }

    const progress = (params_ as ProgressParams).value;

    if (progress.kind == "begin") {
      updateLoadingLsp({ title: progress.title, message: "" });
    } else if (progress.kind == "report") {
      updateLoadingLsp({
        message: progress.message,
        percentage: progress.percentage,
      });
    } else if (progress.kind == "end") {
      endLoadingLsp();
    }

    return true;
  }

  public async sendData(
    data: JSONRPCRequestData,
    timeout: number | null = 5000,
  ): Promise<void> {
    const notifications = getNotifications(data);
    try {
      const sendData = this.parseData(data);

      if (Array.isArray(sendData)) {
        throw new Error("LSP Batch is not supported");
      }

      if (ignoredNotify[sendData.method]) {
        return;
      }

      let prom = this.transportRequestManager.addRequest(data, timeout);

      sendMessage(ClientMessageKind.Lsp, {
        data: sendData,
      });

      this.transportRequestManager.settlePendingRequest(notifications);

      return prom;
    } catch (err) {
      const jsonError = new JSONRPCError(
        (err as any).message,
        ERR_UNKNOWN,
        err,
      );

      this.transportRequestManager.settlePendingRequest(
        notifications,
        jsonError,
      );
      this.transportRequestManager.settlePendingRequest(
        getBatchRequests(data),
        jsonError,
      );

      return Promise.reject(jsonError);
    }
  }

  public close(): void {
    console.warn("Closing lsp transport");
  }
}
