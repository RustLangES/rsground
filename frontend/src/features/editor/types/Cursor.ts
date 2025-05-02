import { Range, SelectionRange } from "@codemirror/state";
import { Decoration } from "@codemirror/view";

/** Cursor from backend */
export type RsCursor = [number, number];

export interface Cursor {
  from: number;
  to: number;
}

export namespace Cursor {
  export function from(value: SelectionRange | [number, number]): Cursor {
    if (value instanceof Array) {
      return {
        from: value[0],
        to: value[1],
      };
    } else {
      return {
        from: value.from,
        to: value.to,
      };
    }
  }

  export function into_rscursor(self: Cursor): RsCursor {
    return [self.from, self.to];
  }

  export function toDecoration(
    self: Cursor,
    owner: string,
    styles: CSSModuleClasses,
  ): Range<Decoration> {
    const hue = Array(owner.length)
      .fill(0)
      .map((_, i) => i * owner.charCodeAt(i))
      .reduce((prev, acc) => ((prev + 1) * (acc + 1)) % 360, 0);

    console.log(hue);

    const common = {
      inclusive: true,
      attributes: {
        style: `--hue: ${hue}`,
      },
    };
    if (self.from === self.to) {
      return Decoration.mark({
        ...common,
        class: styles.colored_cursor,
      }).range(self.from, self.from + 1);
    }

    return Decoration.mark({
      ...common,
      class: styles.colored_selection,
    }).range(self.from, self.to);
  }
}
