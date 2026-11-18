import { WIFI_MODE } from "@/utils";

/**
 * WNBCom Return status
 */
export const enum RET {
  OK, FAIL, AUTH_FAIL, TIME_OUT
}

/**
 * Device config class
 */
export class deviceConfigClass {
  cfgInitFlag: boolean;
  wifiMode: WIFI_MODE;
  sysAppDHCPD: boolean;
  fixWiFiMode: boolean;
  noAdvWebCfg: boolean;
  noAPSTAmode: boolean;
  noWebAuth: boolean;
  saeEn: boolean;
  oweEn: boolean;
  wnbEn: boolean;   // firmware extension: WNB (Taixin bridge protocol) modes available
  constructor (cfgInitFlag: boolean,
    wifiMode: WIFI_MODE,
    sysAppDHCPD: boolean,
    fixWiFiMode: boolean,
    noAdvWebCfg: boolean,
    noAPSTAmode: boolean,
    noWebAuth: boolean,
    saeEn: boolean,
    oweEn: boolean,
    wnbEn: boolean = false
    ) {
    this.cfgInitFlag = cfgInitFlag;
    this.wifiMode = wifiMode;
    this.sysAppDHCPD = sysAppDHCPD;
    this.fixWiFiMode = fixWiFiMode;
    this.noAdvWebCfg = noAdvWebCfg;
    this.noAPSTAmode = noAPSTAmode;
    this.noWebAuth = noWebAuth;
    this.saeEn = saeEn;
    this.oweEn = oweEn;
    this.wnbEn = wnbEn;
  }
}

/**
 * Connection status class
 */
export class connStatClass {
  connected: boolean;
  rssi: number;
  evm: number;
  channel: number;
  constructor(connected: boolean, rssi: number, evm: number, channel: number) {
    this.connected = connected;
    this.rssi = rssi;
    this.evm = evm;
    this.channel = channel;
  }
}

export class staInfoClass {
  aid: number;
  ps: boolean;
  mac: string;
  ip: string;
  rssi: number;
  evm: number;
  tx_snr: number;
  rx_snr: number;
  constructor(
    aid: number, ps: boolean, mac: string, ip: string, rssi: number, evm: number, tx_snr: number, rx_snr: number
    ) {
    this.aid = aid;
    this.ps = ps;
    this.mac = mac;
    this.ip = ip;
    this.rssi = rssi;
    this.evm = evm;
    this.tx_snr = tx_snr;
    this.rx_snr = rx_snr;
  }
}

export class apInfoClass {
  bssid: string;
  ssid: string;
  encrypt: boolean;
  signal: number;
  freq: string;
  constructor(bssid: string, ssid: string, encrypt: boolean, signal: number, freq: string) {
    this.bssid = bssid;
    this.ssid = ssid;
    this.encrypt = encrypt;
    this.signal = signal;
    this.freq = freq;
  }
}

export class dhcpLeaseClass {
  mac: string;
  ip: string;
  constructor(mac: string, ip: string) {
    this.mac = mac;
    this.ip = ip;
  }
}

export class clientInfoClass {
  type: 'Wired' | 'Wireless';
  mac: string
  ip: string
  signal: number;
  constructor(type: 'Wired' | 'Wireless', mac: string, ip: string, signal: number) {
    this.type = type;
    this.mac = mac;
    this.ip = ip;
    this.signal = signal;
  }
}
