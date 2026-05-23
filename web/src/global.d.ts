declare module '*.svelte' {
  const component: any;
  export default component;
}

declare module '*.js' {
  export const createRuntime: any;
}

interface ImportMetaEnv {
  readonly BASE_URL: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
