// Site base URL (set by Rspress from rspress.config.ts `base`), with a
// trailing slash. Used to fetch files from `docs/public/` at runtime.
const rawBase: string =
    typeof import.meta !== 'undefined'
        ? // eslint-disable-next-line @typescript-eslint/no-explicit-any
          (import.meta as any).env?.BASE_URL || '/imferno/'
        : '/imferno/';

export const base = rawBase.endsWith('/') ? rawBase : rawBase + '/';
