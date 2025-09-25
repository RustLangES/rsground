import { setLoadingLsp } from "../stores";

export function updateLoadingLsp(change: {title?: string, message?: string, percentage?: number}) {
  setLoadingLsp({
    isLoading: true,
    ...change
  });
}

export function endLoadingLsp() {
  setLoadingLsp({
    isLoading: false,
  });
}
