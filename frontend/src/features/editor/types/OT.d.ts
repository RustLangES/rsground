// OT means for Operational Transformation
// https://en.wikipedia.org/wiki/Operational_transformation

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
