import { fetchOptions, fetchTimeOut, settingsPath } from "./protocol";
import { RET } from "./types";

async function deviceCommand(cmd: string, authToken: string): Promise<RET> {
  //const resp = await fetch(settingsPath, new fetchOptions(cmd, authToken));
  const resp = await fetchTimeOut(settingsPath, new fetchOptions(cmd, authToken));
  if (!resp)
    return RET.TIME_OUT;
  if (resp.ok)
    return RET.OK;
  else if (resp.status === 401)
    return RET.AUTH_FAIL;
  else
    return RET.FAIL;
}

/**
 * Reboot Device
 * @param  {string}       authToken AuthToken
 * @return {Promise<RET>}           RET
 */
export async function reboot(authToken: string): Promise<RET> {
  return deviceCommand('reset', authToken);
}

/**
 * Load Factory Default
 * @param  {string}       authToken AuthToken
 * @return {Promise<RET>}           RET
 */
export async function loadDef(authToken: string): Promise<RET> {
  return deviceCommand('loaddef', authToken);
}

/** Firmware extension (webui.c): start (true) or stop pairing, like the pair button / AT+PAIR. */
export async function pairing(on: boolean, authToken: string): Promise<RET> {
  return deviceCommand(on ? 'pair=1' : 'pair=0', authToken);
}
