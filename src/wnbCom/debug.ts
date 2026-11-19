import { fetchOptions, settingsPath } from "./protocol";
import { RET } from "./types";

/** Firmware extension (webui.c): run one AT command, reply text as atcmd_printf() produced it. */
export async function atCommand(cmd: string, authToken: string): Promise<{ state: RET, text: string }> {
  const ctrl = new AbortController();
  const timer = setTimeout(() => ctrl.abort(), 10000);   // device waits up to 4 s for OK/ERROR
  try {
    const resp = await fetch(settingsPath, { ...new fetchOptions('atcmd=' + cmd, authToken), signal: ctrl.signal });
    if (resp.status === 401) return { state: RET.AUTH_FAIL, text: '' };
    if (!resp.ok) return { state: RET.FAIL, text: '' };
    return { state: RET.OK, text: await resp.text() };
  } catch {
    return { state: RET.TIME_OUT, text: '' };
  } finally {
    clearTimeout(timer);
  }
}
