/*
 * Minimal English-only stand-in for vue-i18n (aliased as 'vue-i18n' in
 * vite.config.ts and tsconfig.json) to keep the firmware-embedded page small.
 *
 * Supports exactly what this UI uses:
 *   - createI18n({ messages: { enUS: {...} } }) + app.use()  -> global messages
 *   - useI18n({ useScope: 'local' })  -> t() over the component's <i18n> block,
 *     falling back to global messages (the build plugin in vite.config.ts turns
 *     each <i18n> block into `Component.__i18n = <enUS messages>`)
 *   - useI18n({ useScope: 'global' }) and $t() in templates -> global messages
 * Messages are plain strings; no interpolation/pluralisation.
 */
import { getCurrentInstance, type App } from 'vue';

type Messages = Record<string, string>;

let globalMessages: Messages = {};

function translate(key: string, local?: Messages): string {
  if (local && key in local) return local[key];
  if (key in globalMessages) return globalMessages[key];
  return key;
}

export interface UseI18nOptions {
  useScope?: 'local' | 'global';
  inheritLocale?: boolean;
  [option: string]: unknown;   // accept (and ignore) other vue-i18n options
}

export function useI18n(options?: UseI18nOptions) {
  let local: Messages | undefined;
  if (options?.useScope === 'local') {
    local = (getCurrentInstance()?.type as { __i18n?: Messages } | undefined)?.__i18n;
  }
  return { t: (key: string) => translate(key, local) };
}

export interface I18nOptions {
  messages: { enUS: Messages };
  [option: string]: unknown;
}

export function createI18n(options: I18nOptions) {
  globalMessages = options.messages.enUS;
  return {
    install(app: App) {
      app.config.globalProperties.$t = (key: string) => translate(key);
    }
  };
}

declare module 'vue' {
  interface ComponentCustomProperties {
    $t: (key: string) => string;
  }
}
