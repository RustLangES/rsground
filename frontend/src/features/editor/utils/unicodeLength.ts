/** Returns the number of Unicode codepoints in a string. */
export function unicodeLength(str: string): number {
  let length = 0;
  for (const _ of str) ++length;
  return length;
}
