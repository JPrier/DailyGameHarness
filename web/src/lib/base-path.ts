export function appBasePath(): string {
  const base = import.meta.env.BASE_URL ?? '/';
  if (!base || base === '/') return '';
  return `/${base.replace(/^\/+|\/+$/g, '')}`;
}

export function withAppBase(path: string): string {
  if (/^[a-z][a-z\d+.-]*:/i.test(path)) return path;
  const base = appBasePath();
  if (!base) return path.startsWith('/') ? path : `/${path}`;
  if (path === base || path.startsWith(`${base}/`)) return path;
  return withRoutePrefix(base, path);
}

export function withRoutePrefix(routePrefix: string, path: string): string {
  const prefix = routePrefix.replace(/\/$/, '');
  const suffix = path.startsWith('/') ? path : `/${path}`;
  return `${prefix}${suffix}`;
}

export function stripContentPrefix(path: string, prefix: 'content/puzzles' | 'content/assets'): string {
  return path.replace(new RegExp(`^${prefix}/`), '');
}
