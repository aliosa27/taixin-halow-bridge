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
  txMcs: number;   // firmware extension; -1 = unknown
  rxMcs: number;
  bw: number;      // BSS bandwidth, MHz
  txKB: number;    // firmware extension: bytes to / from the AP, KiB; -1 = unknown
  rxKB: number;
  staCnt: number;  // firmware extension: stations associated to our AP; -1 = not an AP / unknown
  constructor(connected: boolean, rssi: number, evm: number, channel: number, txMcs = -1, rxMcs = -1, bw = 0,
    txKB = -1, rxKB = -1, staCnt = -1) {
    this.connected = connected;
    this.rssi = rssi;
    this.evm = evm;
    this.channel = channel;
    this.txMcs = txMcs;
    this.rxMcs = rxMcs;
    this.bw = bw;
    this.txKB = txKB;
    this.rxKB = rxKB;
    this.staCnt = staCnt;
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
  txMcs: number;   // firmware extension; -1 = unknown
  rxMcs: number;
  bw: number;
  txKB: number;    // firmware extension: bytes to / from this station, KiB; -1 = unknown
  rxKB: number;
  constructor(
    aid: number, ps: boolean, mac: string, ip: string, rssi: number, evm: number, tx_snr: number, rx_snr: number,
    txMcs = -1, rxMcs = -1, bw = 0, txKB = -1, rxKB = -1
    ) {
    this.txMcs = txMcs;
    this.rxMcs = rxMcs;
    this.bw = bw;
    this.txKB = txKB;
    this.rxKB = rxKB;
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
  txMcs: number;
  rxMcs: number;
  bw: number;
  txKB: number;
  rxKB: number;
  constructor(type: 'Wired' | 'Wireless', mac: string, ip: string, signal: number, txMcs = -1, rxMcs = -1, bw = 0,
    txKB = -1, rxKB = -1) {
    this.type = type;
    this.mac = mac;
    this.ip = ip;
    this.signal = signal;
    this.txMcs = txMcs;
    this.rxMcs = rxMcs;
    this.bw = bw;
    this.txKB = txKB;
    this.rxKB = rxKB;
  }
}
