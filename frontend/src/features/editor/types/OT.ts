// OT means for Operational Transformation
// https://en.wikipedia.org/wiki/Operational_transformation

import { wsSessionId } from "@features/ws/stores";

export type OtOperationSeq = OtOperation[];

export type OtOperation =
  | {
    kind: OtOperationKind.Insert;
    owner: string;
    from: number;
    text: string;
  }
  | {
    kind: OtOperationKind.Delete;
    owner: string;
    from: number;
    to: number;
  };

export enum OtOperationKind {
  Insert = "insertion",
  Delete = "deletion",
}

export namespace OtOperation {
  /** Check if both operations are equal in practical terms. Don't check owner */
  export function equal(self: OtOperation, other: OtOperation): boolean {
    return !!self && !!other && self.kind === other.kind &&
      self.from === other.from &&
      // @ts-expect-error - TS is dumb, other is Insert so has `content`
      ((self.kind == OtOperationKind.Insert && self.text === other.text) ||
        // @ts-expect-error - TS is dumb, other is Delete so has `to`
        (self.kind == OtOperationKind.Delete && self.to === other.to));
  }

  /** Check if both operations are totally equal */
  export function equalStrict(self: OtOperation, other: OtOperation): boolean {
    return self.owner === other.owner && OtOperation.equal(self, other);
  }

  export function to(self: OtOperation): number {
    if (self.kind == OtOperationKind.Insert) {
      return self.from + self.text.length;
    } else {
      return self.to;
    }
  }

  export function insert(from: number, content: string): OtOperation {
    return {
      kind: OtOperationKind.Insert,
      from,
      text: content,
      owner: wsSessionId() ?? "me",
    };
  }

  export function remove(from: number, to: number): OtOperation {
    return {
      kind: OtOperationKind.Delete,
      from,
      to,
      owner: wsSessionId() ?? "me",
    };
  }
}
