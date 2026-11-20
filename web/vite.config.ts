import { fileURLToPath, URL } from 'node:url';
import { defineConfig, type Plugin } from 'vite';
import vue from '@vitejs/plugin-vue';
import svgLoader from 'vite-svg-loader';
import { viteSingleFile } from 'vite-plugin-singlefile';
import viteCompression from 'vite-plugin-compression';

const wnbLink = 'http://192.168.123.1';
const src = (p: string) => fileURLToPath(new URL('./src/' + p, import.meta.url));

/*
 * The UI ships inside the WNB firmware image (which is copied to SRAM), so
 * vue-i18n, vue-router and @vicons/utils are replaced by small stand-ins in
 * src/lite/ that implement only the API this app uses.
 *
 * This plugin turns each SFC <i18n lang="json"> block into
 * `Component.__i18n = <enUS messages>` for src/lite/i18n.ts; other locales
 * are dropped from the bundle.
 */
function i18nBlocksEnglishOnly(): Plugin {
  const isI18nBlock = (id: string) => /\?vue&type=i18n\b/.test(id);
  return {
    name: 'i18n-blocks-english-only',
    enforce: 'pre',
    configResolved(config) {
      // vite:json would otherwise try to parse the generated JS (id ends in .json).
      const json = config.plugins.find((p) => p.name === 'vite:json');
      const orig = json?.transform;
      if (json && typeof orig === 'function') {
        json.transform = function (this: unknown, code: string, id: string, ...rest: unknown[]) {
          return isI18nBlock(id) ? null : (orig as Function).call(this, code, id, ...rest);
        } as typeof orig;
      }
    },
    transform(code, id) {
      if (!isI18nBlock(id)) return null;
      const messages = (JSON.parse(code) as { enUS?: Record<string, string> }).enUS ?? {};
      return {
        code: `export default function (c) { c.__i18n = ${JSON.stringify(messages)} }`,
        map: null
      };
    }
  };
}

// https://vitejs.dev/config/
export default defineConfig({
  resolve: {
    alias: [
      { find: '@', replacement: src('') },
      { find: /^vue-i18n$/, replacement: src('lite/i18n.ts') },
      { find: /^vue-router$/, replacement: src('lite/router.ts') },
      { find: /^@vicons\/utils$/, replacement: src('lite/icon.ts') }
    ]
  },
  plugins: [
    vue(),
    i18nBlocksEnglishOnly(),
    svgLoader(),
    viteSingleFile(),
    viteCompression(),
    ],
  define: {
    // All components use <script setup>; drop the Options API from the bundle.
    __VUE_OPTIONS_API__: 'false',
    __VUE_PROD_DEVTOOLS__: 'false',
    __VUE_PROD_HYDRATION_MISMATCH_DETAILS__: 'false'
  },
  build: {
    emptyOutDir: true,
    target: 'es2015',
    minify: 'terser',
    terserOptions: {
      compress: { passes: 2 },
      format: { comments: false }
    }
  },
  server: {
    proxy: {
      '/post.action': {
        target: wnbLink,
        changeOrigin: true
      },
      '/fwupg.data': {
        target: wnbLink,
        changeOrigin: true
      },
      '/auth.action': {
        target: wnbLink,
        changeOrigin: true
      }
    }
  }
});
