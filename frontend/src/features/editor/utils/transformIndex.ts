import { OtOperation, OtOperationKind } from "../types";

// Ported from rust "/backend/src/colab/ot.rs"
export function transformIndex(
  idx: number,
  idx_owner: string,
  history: OtOperation[],
): number {
  for (const action of history) {
    // Ignore self actions
    if (action.owner === idx_owner) continue;

    // All deletion is left to idx, remove range
    if (action.kind === OtOperationKind.Delete && action.to <= idx) {
      idx -= action.to - action.from;
      continue;
    }

    // Insertion is left to idx, add offset
    if (action.kind === OtOperationKind.Insert && action.from <= idx) {
      idx += action.text.length;
      continue;
    }
  }

  return idx;
}
