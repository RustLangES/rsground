import { createStore } from "solid-js/store";

export type LoadingLspStore = {
  isLoading: true;
  title: string;
  message: string;
  percentage?: number;
} | {
  isLoading: false;
  title?: string;
  message?: string;
  percentage?: number;
};

export const [loadingLsp, setLoadingLsp] = createStore<LoadingLspStore>({
  isLoading: false,
});
