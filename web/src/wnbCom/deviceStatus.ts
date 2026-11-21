import { WIFI_MODE } from "@/utils";
import { fetchOptions, GS, settingsPath, US } from "./protocol";
import { fetchSysInfo } from "./systemInfo";
import {
  apInfoClass,
  clientInfoClass,
  connStatClass,
  deviceConfigClass,
  dhcpLeaseClass,
  RET,
  staInfoClass
} from "./types";

interface deviceConfigRet {
  deviceConfig: deviceConfigClass;
  state: RET;
}

export async function deviceConfig(authToken: string): Promise<deviceConfigRet> {
  const resp = await fetchSysInfo('device_cfg', authToken);
  var ret = {} as deviceConfigRet;
  ret.state = resp.state;
  if (resp.state === RET.OK) {
    if (resp.textStr.length === 0) {
      // No data, put some default value in
      ret.deviceConfig = new deviceConfigClass(false, WIFI_MODE.AP, true, false, false, false, false, false, false);
      return ret;
    }
    var deviceConfig = resp.textStr.split(US);
    ret.deviceConfig = new deviceConfigClass(
      deviceConfig[0] === '1' ? true : false,
      deviceConfig[1] as WIFI_MODE,
      deviceConfig[2] === '1' ? true : false,
      deviceConfig[3] === '1' ? true : false,
      deviceConfig[4] === '1' ? true : false,
      deviceConfig[5] === '1' ? true : false,
      deviceConfig[6] === '1' ? true : false,
      deviceConfig[7] === '1' ? true : false,
      deviceConfig[8] === '1' ? true : false,
      deviceConfig[9] === '1'
      );
  }
  return ret;
}

interface connStatRet {
  connStat: connStatClass;
  state: RET;
}

/**
 * Connection status get
 * @param  {string}               authToken AuthToken
 * @return {Promise<connStatRet>}           Return data
 */
export async function connStat(authToken: string): Promise<connStatRet> {
  const resp = await fetchSysInfo('conn_state', authToken);
  var ret = {} as connStatRet;
  ret.state = resp.state;
  if (resp.state === RET.OK) {
    var stat = resp.textStr.split(US);
    ret.connStat = new connStatClass(
      stat[0] === '1' ? true : false,
      parseInt(stat[1]),
      parseInt(stat[2]),
      parseInt(stat[3]),
      stat.length > 6 ? parseInt(stat[4]) : -1,
      stat.length > 6 ? parseInt(stat[5]) : -1,
      stat.length > 6 ? parseInt(stat[6]) : 0,
      stat.length > 8 ? parseInt(stat[7]) : -1,
      stat.length > 8 ? parseInt(stat[8]) : -1,
      stat.length > 9 ? parseInt(stat[9]) : -1
      );
  }
  return ret;
}

interface stationListRet {
  staList: staInfoClass[];
  state: RET;
}

/**
 * Connected station list get
 * @param  {string}                  authToken AuthToken
 * @return {Promise<stationListRet>}           Return data
 */
export async function stationList(authToken: string): Promise<stationListRet> {
  const resp = await fetchSysInfo('stalist', authToken);
  var ret = {} as stationListRet;
  ret.state = resp.state;
  ret.staList = new Array();
  if (resp.state === RET.OK) {
    if (resp.textStr.length === 0) // Return empty array if no data
      return ret;
    var _staList = resp.textStr.split(GS);
    for (let i = 0; i < _staList.length; i++) {
      let staInfo = _staList[i].split(US);
      ret.staList[i] = new staInfoClass(
        parseInt(staInfo[0]),
        staInfo[1] === '1' ? true : false,
        staInfo[2],
        staInfo[3],
        parseInt(staInfo[4]),
        parseInt(staInfo[5]),
        parseInt(staInfo[6]),
        parseInt(staInfo[7]),
        staInfo.length > 10 ? parseInt(staInfo[8]) : -1,
        staInfo.length > 10 ? parseInt(staInfo[9]) : -1,
        staInfo.length > 10 ? parseInt(staInfo[10]) : 0,
        staInfo.length > 12 ? parseInt(staInfo[11]) : -1,
        staInfo.length > 12 ? parseInt(staInfo[12]) : -1
        );
    }
    // Do AID sort
    ret.staList.sort((a, b) => {
      if (a.aid < b.aid)
        return -1;
      if (a.aid > b.aid)
        return 1;
      return 0;
    });
  }
  return ret;
}

interface scanListRet {
  scanList: apInfoClass[];
  state: RET;
}

/**
 * Scan list of ap
 * @param  {string}               authToken AuthToken
 * @return {Promise<scanListRet>}           Return Data
 */
export async function fetchScanList(authToken: string): Promise<scanListRet> {
  const resp1 = await fetch(settingsPath, new fetchOptions('scanap', authToken));
  //const resp1 = await fetchTimeOut(settingsPath, new fetchOptions('scanap', authToken))
  var ret = {} as scanListRet;
  ret.scanList = new Array();
  if (!resp1) {
    ret.state = RET.TIME_OUT;
    return ret;
  }
  if (resp1.ok) {
    await new Promise((Tout) => setTimeout(Tout, 5000));
    const resp2 = await fetchSysInfo('scanlist', authToken);
    ret.state = resp2.state;
    if (resp2.state === RET.OK) {
      var apList: string[]
      if (resp2.textStr.length === 0)
        apList = []
      else
        var apList = resp2.textStr.split(GS);
      for (let i = 0; i < apList.length; i++) {
        let apInfo = apList[i].split(US);
        ret.scanList[i] = new apInfoClass(
          apInfo[0],
          apInfo[1],
          apInfo[2] === '1' ? true : false,
          parseInt(apInfo[3]),
          apInfo[4]
          );
      }
      // Do some sorting (do signal strenght base sorting)
      ret.scanList.sort((a, b) => {
        if (a.signal > b.signal)
          return -1;
        if (a.signal < b.signal)
          return 1;
        return 0;
      });
    }
  } else if (resp1.status === 401) {
    ret.state = RET.AUTH_FAIL;
  } else {
    ret.state = RET.FAIL;
  }
  return ret;
}

interface dhcpLeaseListRet {
  leaseList: dhcpLeaseClass[];
  state: RET;
}

/**
 * DHCP Server lease get
 * @param  {string}                    authToken authToken
 * @return {Promise<dhcpLeaseListRet>}           Return Data
 */
export async function dhcpLeaseList(authToken: string): Promise<dhcpLeaseListRet> {
  const resp = await fetchSysInfo('dhcp_leaselist=?', authToken);
  var ret = {} as dhcpLeaseListRet;
  ret.leaseList = new Array();
  ret.state = resp.state;
  if (resp.state === RET.OK) {
    if (resp.textStr.length === 0) // Return empty array if no data
      return ret;
    var leaseList = resp.textStr.split(GS); 
    for (let i = 0; i < leaseList.length; i++) {
      let lease = leaseList[i].split(US);
      ret.leaseList[i] = new dhcpLeaseClass(
        lease[0],
        lease[1]
        );
    }
  }
  return ret;
}

interface clientListRet {
  clientList: clientInfoClass[];
  state: RET;
}

export async function clientList(authToken: string) {
  const resp1 = await stationList(authToken);
  const resp2 = await dhcpLeaseList(authToken);
  var ret = {} as clientListRet;
  ret.clientList = new Array();
  if (resp1.state === RET.OK && resp2.state === RET.OK) {
    // Put stationList in
    resp1.staList.forEach((staInfo) => {
      ret.clientList.push(new clientInfoClass(
        'Wireless',
        staInfo.mac,
        staInfo.ip === '0.0.0.0' ? '-' : staInfo.ip,
        staInfo.rssi,
        staInfo.txMcs,
        staInfo.rxMcs,
        staInfo.bw,
        staInfo.txKB,
        staInfo.rxKB
        ));
    });
    // Merge dhcpClient
    resp2.leaseList.forEach((lease) => {
      // Check for duplication
      if (!resp1.staList.find(staInfo => lease.mac === staInfo.mac))
        ret.clientList.push(new clientInfoClass('Wired', lease.mac, lease.ip, 0));
    });
    ret.state = RET.OK;
  } else if (resp1.state === RET.AUTH_FAIL || resp2.state === RET.AUTH_FAIL) {
    ret.state = RET.AUTH_FAIL;
  } else {
    ret.state = RET.FAIL;
  }
  return ret;
}
