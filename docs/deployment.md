# Deployment

Daily Game Harness is a static site. The default target is GitHub Pages, but the generated output can be served by any static host that can publish `web/dist`.

## GitHub Pages Default

Use GitHub Pages when you want the simplest deployment path.

Required config:

```json
{
  "site": {
    "routePrefix": "/repo-name"
  },
  "deployment": {
    "target": "github-pages",
    "githubPages": {
      "artifactPath": "web/dist",
      "environmentName": "github-pages"
    }
  }
}
```

Use an empty `routePrefix` for owner or organization pages like `https://owner.github.io/`.

Use `"/repo-name"` for project pages like `https://owner.github.io/repo-name/`.

## GitHub Pages Limitations

- No server-side code runs at request time.
- No private runtime secrets can be used by gameplay.
- Cache headers are limited compared with S3 or CloudFront.
- Static JSON puzzle files are public if a user knows or discovers their URLs.
- All routes must be generated as static files or handled by the static 404 fallback.
- Project pages require correct `site.routePrefix` handling.
- Authentication, leaderboards, paid access, and anti-cheat require a separate backend or third-party service.

These limitations are acceptable for public daily puzzle games where puzzle content can be public after deployment.

## Alternative Static Hosts

The game package contract does not change for other hosts. Only the deployment step changes.

Good options:

- S3 plus CloudFront for cache header control and custom invalidation.
- Cloudflare Pages for simple static hosting with preview URLs.
- Netlify for static hosting plus optional edge/serverless add-ons.
- Any CDN or object store that can serve HTML, JS, CSS, JSON, WASM, images, and fonts.

## Build Commands

```sh
cargo run -p daily_game_tools -- prepare-static-build
cd web
npm ci
npm run build
cd ..
cargo run -p daily_game_tools -- check-static-output --dist web/dist
```

`npm run build` uses Astro. The custom fixture build script is not part of the production build path.
