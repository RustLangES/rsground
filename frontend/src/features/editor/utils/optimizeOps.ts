import { OtOperation, OtOperationKind } from "../types";

/// Squash operations into the smallest possible
export function optimizeOps(ops: OtOperation[]): OtOperation[] {
  let out: OtOperation[] = [];

  let idx = 0;

  while (idx < ops.length) {
    const last_op = out.pop();
    const op: OtOperation = { ...ops[idx] };

    if (last_op == null) {
      out.push(op);
      idx++;
      continue;
    }

    const last_op_to = OtOperation.to(last_op);
    const op_to = OtOperation.to(op);

    const is_from_aligned = last_op.from == op.from;
    const is_to_aligned = last_op_to == op_to;

    if (
      last_op.kind == OtOperationKind.Insert &&
      op.kind == OtOperationKind.Insert
    ) {
      if (last_op_to == op.from) {
        /// abcd|--|
        last_op.text += op.text;
        out.push(last_op);
      } else if (is_from_aligned) {
        /// |--|abcd
        op.text += last_op.text;
        out.push(op);
      } else if (op.from > last_op.from && op_to < last_op_to) {
        /// ab|--|cd
        const split_idx = op.from - last_op.from;
        const before = last_op.text.substring(0, split_idx);
        const after = last_op.text.substring(split_idx);

        last_op.text = before + op.text + after;
        out.push(last_op);
      } else {
        out.push(last_op);
        out.push(op);
      }
    } else if (
      last_op.kind == OtOperationKind.Delete &&
      op.kind == OtOperationKind.Delete
    ) {
      if (last_op.from == op.from) {
        /// abcd|--|
        last_op.to += op.to - op.from;
        out.push(last_op);
      } else if (op.to == last_op.from) {
        /// |--|abcd
        op.to = last_op.to;
        out.push(op);
      } else {
        out.push(last_op);
        out.push(op);
      }
    } else if (
      last_op.kind == OtOperationKind.Insert &&
      op.kind == OtOperationKind.Delete &&
      (
        (last_op.from <= op.from && op.from <= last_op_to) ||
        (last_op.from <= op.to && op.to <= last_op_to)
      )
    ) {
      if (is_from_aligned && op.to > last_op_to) {
        /// |abcd--|
        op.from = last_op_to;
        out.push(op);
      } else if (is_from_aligned && op.to < last_op_to) {
        /// |ab|cd
        last_op.text = last_op.text.substring(op.to - last_op.from);
        out.push(last_op);
      } else if (op.from > last_op.from) {
        /// ab|cd
        let new_content = last_op.text.substring(0, op.from - last_op.from);

        if (op.to > last_op_to) {
          /// ab|cd--|
          last_op.text = new_content;
          op.from = last_op_to;
          out.push(last_op);
          out.push(op);
        } else if (is_to_aligned) {
          /// ab|cd|
          last_op.text = new_content;
          out.push(last_op);
        } else {
          /// a|bc|d
          last_op.text = new_content +
            last_op.text.substring(op.to - last_op.from);
          out.push(last_op);
        }
      } else if (is_to_aligned && op.from < last_op.from) {
        /// |--abcd|
        op.to = last_op.from;
        out.push(op);
      } else if (op.to < last_op_to && op.from < last_op.from) {
        /// |--ab|cd
        last_op.text = last_op.text.substring(op.to - last_op.from);
        op.to = last_op.from;
        out.push(last_op);
        out.push(op);
      }
    } else {
      out.push(last_op);
      out.push(op);
    }

    idx++;
  }

  return out;
}
