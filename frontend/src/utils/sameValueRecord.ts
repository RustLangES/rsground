export function sameValueRecord<K extends string | number, V>(
  keys: K[],
  value: V,
): Record<K, V> {
  return Object.fromEntries(keys.map((key) => [key, value])) as Record<K, V>;
}
