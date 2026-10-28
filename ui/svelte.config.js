import { vitePreprocess } from "@sveltejs/vite-plugin-svelte";

// Required for `svelte-check --tsconfig` and editor tooling. Vite itself also
// picks this up through the plugin, so the two never disagree about TS.
export default {
  preprocess: vitePreprocess(),
};
