// OT means for Operational Transformation
// https://en.wikipedia.org/wiki/Operational_transformation

import { wsSessionId } from "@features/ws/stores";

export type OtOperationSeq = OtOperation[];

export type OtOperation =
  | {
    kind: OtOperationKind.Insert;
    owner: string;
    from: number;
    content: string;
  }
  | {
    kind: OtOperationKind.Delete;
    owner: string;
    from: number;
    to: number;
  };

export enum OtOperationKind {
  Insert = "insert",
  Delete = "delete",
}

export namespace OtOperation {
  export function insert(from: number, content: string): OtOperation {
    return {
      kind: OtOperationKind.Insert,
      from,
      content,
      owner: wsSessionId() ?? "me"
    }
  }

  export function remove(from: number, to: number): OtOperation {
    return {
      kind: OtOperationKind.Delete,
      from,
      to,
      owner: wsSessionId() ?? "me"
    }
  }
}
