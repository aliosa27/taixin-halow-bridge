import { authPath, fetchOptions, fetchTimeOut, settingsPath } from "./protocol";
import { RET } from "./types";

/**
 * Authorization Return data
 */
interface authorizationRet {
  authToken: string;
  state: RET;
}

/**
 * Device Authorization
 * @param  {string}                    passwd Auth Password
 * @return {Promise<authorizationRet>}        Return data
 */
export async function authorization(passwd: string): Promise<authorizationRet> {
  const encoder = new TextEncoder();
  var encodedPasswd = "";
  (encoder.encode(passwd) as unknown as Array<number>).forEach(e => encodedPasswd += e.toString(16))
  //const resp = await fetch(authPath, new fetchOptions(encodedPasswd, ''));
  const resp = await fetchTimeOut(authPath, new fetchOptions(encodedPasswd, ''));
  var ret = {} as authorizationRet;
  if (!resp) {
    ret.state = RET.TIME_OUT;
    return ret;
  }
  if (resp.ok) {
    try {
      ret.authToken = await resp.text();
      ret.state = RET.OK;
    } catch {
      ret.state = RET.FAIL;
    }
  } else if (resp.status === 401) {
    ret.state = RET.AUTH_FAIL;
  } else {
    ret.state = RET.FAIL;
  }
  return ret;
}

/**
 * Send a empty POST request to ping device
 * @param  {string}       authToken AuthToken
 * @return {Promise<RET>}           Response Status
 */
export async function wnbPing(authToken: string): Promise<RET> {
  //const resp = await fetch(settingsPath, new fetchOptions('', authToken));
  const resp = await fetchTimeOut(settingsPath, new fetchOptions('', authToken));
  if (!resp)
    return RET.TIME_OUT;
  if (resp.ok)
    return RET.OK;
  else if (resp.status === 401)
    return RET.AUTH_FAIL;
  else
    return RET.FAIL;
}
