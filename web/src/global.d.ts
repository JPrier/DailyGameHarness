declare module '*.svelte' {
  const component: any;
  export default component;
}

declare module '*.js' {
  export const createRuntime: any;
}
