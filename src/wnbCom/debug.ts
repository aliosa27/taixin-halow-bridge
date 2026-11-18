import { fetchOptions, settingsPath, US } from "./protocol";
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

/** Firmware extension (webui.c): log text written since seq; returns the new seq. */
export async function logRead(seq: number, authToken: string): Promise<{ state: RET, seq: number, text: string }> {
  try {
    const resp = await fetch(settingsPath, new fetchOptions('logread=' + seq, authToken));
    if (resp.status === 401) return { state: RET.AUTH_FAIL, seq, text: '' };
    if (!resp.ok) return { state: RET.FAIL, seq, text: '' };
    const body = await resp.text();
    const i = body.indexOf(US);
    if (i < 0) return { state: RET.FAIL, seq, text: '' };
    return { state: RET.OK, seq: parseInt(body.slice(0, i)), text: body.slice(i + 1) };
  } catch {
    return { state: RET.TIME_OUT, seq, text: '' };
  }
}
