import { reactRouter } from "@react-router/dev/vite";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import tsconfigPaths from "vite-tsconfig-paths";

export default defineConfig({
  plugins: [tailwindcss(), reactRouter(), tsconfigPaths()],

  server: {
    host: "0.0.0.0",
    port: 3000,
    open: false,
  },

  build: {
    // Modern browsers only — no legacy transforms, smaller output
    target: "esnext",

    rollupOptions: {
      output: {
        // Split vendor deps into stable, cacheable chunks.
        // Chunks are keyed by node_modules package group so that
        // a change in app code never invalidates vendor caches.
        manualChunks(id) {
          if (!id.includes("node_modules")) return;

          // CodeMirror family — large, only needed on /config and /studio
          if (id.includes("codemirror") || id.includes("@codemirror")) {
            return "vendor-codemirror";
          }
          // React DOM — heavy runtime, always needed, separated for cache stability
          if (id.includes("react-dom")) {
            return "vendor-react-dom";
          }
          // React Router runtime
          if (id.includes("react-router") || id.includes("@react-router")) {
            return "vendor-router";
          }
          // Base UI headless component primitives
          if (id.includes("@base-ui")) {
            return "vendor-base-ui";
          }
          // Everything else in node_modules → vendor-misc
          return "vendor-misc";
        },
      },
    },
  },
});
