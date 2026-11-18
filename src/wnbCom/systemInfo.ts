import { fetchOptions, fetchTimeOut, settingsPath } from "./protocol";
import { RET } from "./types";

interface sysInfoRet {
  textStr: string;
  state: RET;
}

export async function fetchSysInfo(cmd: string, authToken: string): Promise<sysInfoRet> {
  //const resp = await fetch(settingsPath, new fetchOptions(cmd, authToken));
  const resp = await fetchTimeOut(settingsPath, new fetchOptions(cmd, authToken));
  var ret = {} as sysInfoRet;
  if (!resp) {
    ret.textStr = '';
    ret.state = RET.TIME_OUT;
    return ret;
  }
  if (resp.ok) {
    try  {
      ret.textStr = await resp.text();
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
