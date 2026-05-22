export function withRoutePrefix(routePrefix: string, path: string): string {
  const prefix = routePrefix.replace(/\/$/, '');
  const suffix = path.startsWith('/') ? path : `/${path}`;
  return `${prefix}${suffix}`;
}

export function stripContentPrefix(path: string, prefix: 'content/puzzles' | 'content/assets'): string {
  return path.replace(new RegExp(`^${prefix}/`), '');
}
