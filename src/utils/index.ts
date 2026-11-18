
/**
 * Basic string object
 */
export interface strObj {
  [key: string]: string
};

/**
 * Select switch
 */
export const enum SELECT {
  ENABLE = '1',
  DISABLE = '0'
}

/**
 * Wireless operation mode
 */
export const enum WIFI_MODE {
  NONE = '0',
  STA = '1',
  AP = '2',
  APSTA = '3'
}

/**
 * Global i18n key for the operating mode name. wnb is the firmware's
 * 'wifi_wnb' setting ('1' = WNB flavour of AP/STA).
 */
export function wifiModeKey(mode: string, wnb?: string): string {
  const w = wnb === '1' ? 'WNB' : '';
  switch (mode) {
  case WIFI_MODE.AP: return 'wifiMode' + w + 'AP';
  case WIFI_MODE.STA: return 'wifiMode' + w + 'STA';
  case WIFI_MODE.APSTA: return 'wifiModeAPSTA';
  }
  return '';
}

/**
 * Wireless keymgmt
 */
export const enum KEY_MGMT {
  DISABLE = '0',
  PSK = '2',
  NONE = '4',
  SAE = '1024',
  OWE = '4194304'
}

/**
 * inputValidate Result
 */
export const enum INPUT_CHECK {
  OK,
  EMPTY,
  INVALID
}

/* ------------------------------------------------- */
/* Web Cookies utils */
/* No longer in use */

/**
 * Set Cookie value and lifespan
 * @param {String} cName  Cookie name
 * @param {String} cValue Cookie value
 * @param {number} exMin  Lifespan in minute
 */
/*
function setCookie(cName: string, cValue: string, exMin: number) {
  const date = new Date();
  date.setTime(date.getTime() + (exMin * 60 * 1000));
  document.cookie = cName + '=' + cValue + ';' + 'expires=' + date.toUTCString() + ';path=/';
}
*/

/**
 * Get Cookie value
 * @param  {String} cName Cookie name
 * @return {String}       Cookie value
 */
/*
function getCookie(cName: string): string {
  let name = cName + '=';
  let cookieArray = document.cookie.split(';');
  for (let i = 0; i < cookieArray.length; i++) {
    let cookie = cookieArray[i];
    while (cookie.charAt(0) == ' ') {
      cookie = cookie.substring(1);
    }
    if (cookie.indexOf(name) == 0) {
      return cookie.substring(name.length, cookie.length);
    }
  }
  return '';
}
*/

/**
 * Delete Cookie
 * @param {String} cName Cookie name
 */
/*
function deleteCookie(cName: string) {
  document.cookie = cName + '=;' + 'expires=Thu, 01 Jan 1970 00:00:00 UTC; path=/;';
}

export const cookieUtils = {
  setCookie,
  getCookie,
  deleteCookie
}
*/

/* ------------------------------------------------- */
/* Object manipulation utils */

/**
 * Copy object1 to object2, ignore orphans
 * @param {Object} obj1 object1
 * @param {Object} obj2 object2
 */
/* Not in use
function objectCopy(obj1: strObj, obj2: strObj) {
  Object.keys(obj1).forEach((key) => {
    if (key in obj2)
      obj2[key] = obj1[key];
  });
}
*/

/**
 * Merge array of objects
 * @param  {Array<Object>} objArray Array of Objects
 * @return {Object}                 merged object
 */
/* Not in use
function objectMerge(objArray: Array<strObj>): strObj {
  var merged = {};
  objArray.forEach((item) => {
    merged = { ...merged, ...item}
  });
  return merged;
}
*/

/**
 * Return the keys/values pair from object2 that are not equal object1 
 * @param  {Object} obj1 object1
 * @param  {Object} obj2 object2
 * @return {Object}      diff object
 */
export function objectDiff(obj1: strObj, obj2: strObj): strObj {
  return Object.keys(obj2).reduce((diff, key) => {
    if (obj1[key] === obj2[key])
      return diff;
    return {
      ...diff,
      [key]: obj2[key]
    }
  }, {});
}

/*
export const objectUtils = {
  objectCopy,
  objectMerge,
  objectDiff
}
*/

/* ------------------------------------------------- */
/* Input value validation */


function isSSID(ssid: string): INPUT_CHECK {
  if (ssid.length < 1)
    return INPUT_CHECK.EMPTY;
  else if (ssid.length > 32)
    return INPUT_CHECK.INVALID;
  else
    return INPUT_CHECK.OK;
}

const enNumRegex = new RegExp('^[\x20-\x7E]{8,32}$');

function isPSK(psk: string): INPUT_CHECK {
  if (psk.length < 1)
    return INPUT_CHECK.EMPTY;
  else if (!enNumRegex.test(psk))
    return INPUT_CHECK.INVALID;
  else
    return INPUT_CHECK.OK;
}

const Ipv4Regex = new RegExp('^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$');

function isIpv4(ip: string): INPUT_CHECK {
  if (ip.length < 1)
    return INPUT_CHECK.EMPTY;
  else if (!Ipv4Regex.test(ip))
    return INPUT_CHECK.INVALID;
  else
    return INPUT_CHECK.OK;
}

const IPv4RFC1918Regex = new RegExp('^(127(?:\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){3}$)|(10(?:\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){3}$)|(192\.168(?:\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){2}$)|(172\.(?:1[6-9]|2\d|3[0-1])(?:\.(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)){2}$)');

function isPrivateIpv4(ip: string): INPUT_CHECK {
  if (ip.length < 1)
    return INPUT_CHECK.EMPTY;
  else if (!IPv4RFC1918Regex.test(ip))
    return INPUT_CHECK.INVALID;
  else
    return INPUT_CHECK.OK;
}

const IPv4MaskRegex = new RegExp('^(((0|128|192|224|240|248|252|254).0.0.0)|(255.(0|128|192|224|240|248|252|254).0.0)|(255.255.(0|128|192|224|240|248|252|254).0)|(255.255.255.(0|128|192|224|240|248|252|254)))$');

function isNetmask(ip: string): INPUT_CHECK {
  if (ip.length < 1)
    return INPUT_CHECK.EMPTY;
  else if (!IPv4MaskRegex.test(ip))
    return INPUT_CHECK.INVALID;
  else
    return INPUT_CHECK.OK;
}

const macAddrRegex = new RegExp('^[a-fA-F0-9]{2}(:[a-fA-F0-9]{2}){5}$');

function isMacAddr(mac: string): INPUT_CHECK {
  if (mac.length < 1)
    return INPUT_CHECK.EMPTY;
  else if (!macAddrRegex.test(mac))
    return INPUT_CHECK.INVALID;
  else
    return INPUT_CHECK.OK;
}


function isHostname(hostname: string): INPUT_CHECK {
  if (hostname.length < 1)
    return INPUT_CHECK.EMPTY;
  if (hostname.length > 7)  // TXW8301 WNB: sys_cfgs.devname[8] incl. NUL
    return INPUT_CHECK.INVALID;
  // same rule as webui.c: letters, digits, '-' and '.'
  return /^[A-Za-z0-9.-]+$/.test(hostname) ? INPUT_CHECK.OK : INPUT_CHECK.INVALID;
}

// Web login password: 1-32 printable ASCII characters (webui.c, PASSWD_MAX_LEN)
function isWebPasswd(passwd: string): INPUT_CHECK {
  if (passwd.length < 1)
    return INPUT_CHECK.EMPTY;
  return /^[ -~]{1,32}$/.test(passwd) ? INPUT_CHECK.OK : INPUT_CHECK.INVALID;
}

function enforceNumberInputLimit(inputField: string, min: number, max: number): INPUT_CHECK {
  if (parseInt(inputField) > max || parseInt(inputField) < min)
    return INPUT_CHECK.INVALID;
  else
    return INPUT_CHECK.OK
}


export const inputValidate = {
  isSSID,
  isPSK,
  isIpv4,
  isPrivateIpv4,
  isNetmask,
  isMacAddr,
  isHostname,
  isWebPasswd,
  enforceNumberInputLimit
}

/**
 * encode svg icon to inline it in html
 * @param  {string} svg svg icon in raw string
 * @return {string}     encoded string
 */
export function encodeSvg(svg: string): string {
    return svg.replace(/</g, '%3C').replace(/>/g, '%3E').replace(/"/g, "'").replace(/#/g, '%23');
}