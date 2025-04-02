/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_BACKEND_HOST: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
