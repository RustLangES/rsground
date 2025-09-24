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

const ignoredNotify = {
  "textDocument/didOpen": "open",
  "textDocument/didChange": "change",
};

export class RsLspTransport extends Transport {
  public async connect(): Promise<void> {
    onWsMessage(ServerMessageKind.Lsp, ({ data }) => {
      // @ts-expect-error -- resolveRes is private, but needed for performance issues
      this.transportRequestManager.resolveRes(data);
    });
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
