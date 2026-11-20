// English-only; 'vue-i18n' resolves to the small stand-in in src/lite/i18n.ts.
// Per-component strings stay in each SFC's <i18n> block (only "enUS" is bundled).
import { createI18n } from "vue-i18n";
import messages from "./messages.json";

const i18n = createI18n({ messages });

export default i18n;
