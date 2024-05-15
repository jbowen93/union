import preprocess from "svelte-preprocess"
import adapterStatic from "@sveltejs/adapter-static"
import { vitePreprocess } from "@sveltejs/vite-plugin-svelte"

/** @type {import('@sveltejs/kit').Config} */
export default {
  preprocess: [
    // https://kit.svelte.dev/docs/integrations#preprocessors-svelte-preprocess
    vitePreprocess(),
    preprocess({
      postcss: {
        configFilePath: "./postcss.config.cjs"
      }
    })
  ],
  kit: {
    // https://kit.svelte.dev/docs/adapter-static
    adapter: adapterStatic({
      strict: true,
      pages: "build",
      assets: "build",
      fallback: "index.html"
    }),
    /** @note `$` is a svelte path alias convention */
    alias: {
      $: "./src/",
      $styles: "./src/styles"
    }
  }
}
