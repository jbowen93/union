/// <reference types="astro/client" />
/// <reference path="../.astro/types.d.ts" />

interface EventTarget {
  value?: string;
}

interface EnvironmentVariables {
  readonly NODE_ENV: "development" | "production" | "test";
  readonly PORT: string;
}
// Node.js environment variables types
declare module NodeJS {
  interface ProcessEnv extends EnvironmentVariables {}
}
// Vite & Astro environment variables types
interface ImportMetaEnv extends EnvironmentVariables {}
interface ImportMeta {
  readonly env: ImportMetaEnv;
}
// Cloudflare Pages/Workers
interface Env extends EnvironmentVariables {}
