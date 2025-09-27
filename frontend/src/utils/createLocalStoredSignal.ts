import { createSignal, observable, Signal } from "solid-js";

export function createStoredSignal<T>(
  storage: Storage,
  key: string,
  _default: T,
  des: (v: string) => T,
  ser: (v: T) => string,
): Signal<T> {
  let prevValue: T = _default;
  try {
    const stored = storage.getItem(key);

    if (stored) {
      prevValue = des(stored);
    }
  } catch (e) {
    // Just log error for debug, but continue
    console.error(e);
  }

  const [value, setValue] = createSignal(prevValue);

  observable(value).subscribe((value) => {
    if (value != null) {
      storage.setItem(key, ser(value));
    } else {
      storage.removeItem(key);
    }
  });

  return [value, setValue];
}

export function createLocalStoredSignal<T>(
  key: string,
  _default: T,
  des: (v: string) => T,
  ser: (v: T) => string,
): Signal<T> {
  return createStoredSignal(window.localStorage, key, _default, des, ser);
}

export function createSessionStoredSignal<T>(
  key: string,
  _default: T,
  des: (v: string) => T,
  ser: (v: T) => string,
): Signal<T> {
  return createStoredSignal(window.sessionStorage, key, _default, des, ser);
}
