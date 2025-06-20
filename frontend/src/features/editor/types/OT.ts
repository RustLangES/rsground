// OT means for Operational Transformation
// https://en.wikipedia.org/wiki/Operational_transformation

import { OpSeq } from "frontend-wasm";

export type UserOperation = {
  user_id: string;
  /** Serialized form of `OpSeq` */
  operation: Array<number | string>;
};
