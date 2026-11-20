import type { Ref } from 'vue';
import type { formDataStore } from '@/store';
import type { strObj } from "@/utils";
import { fetchOptions, fetchTimeOut, settingsPath, US } from "./protocol";
import { RET } from "./types";

function array2StringGet(settingList: readonly string[]): string {
  return settingList.map((e) => e + '=?').join(US);
}

function object2StringSet(settingObject: strObj): string {
  var settings2Apply = Object.keys(settingObject).map((e) => e + '=' + settingObject[e]);
  // order matter!! savecfg must be first
  //settings2Apply.push('savecfg');
  // Do apply cfg in saveApplyBtn for check settings to function
  //settings2Apply.push('applycfg');
  return settings2Apply.join(US);
}

interface settingsRet {
  settings: strObj;
  state: RET;
}

/**
 * Get WNB device settings
 * @param  {Array<string>}        settingList Settings List
 * @param  {string}               authToken     AuthToken
 * @return {Promise<settingsRet>}               Return data
 */
export async function getSettings(settingList: readonly string[], authToken: string): Promise<settingsRet> {
  //const resp = await fetch(settingsPath, new fetchOptions(array2StringGet(settingList), authToken));
  const resp = await fetchTimeOut(settingsPath, new fetchOptions(array2StringGet(settingList), authToken));
  var ret = {} as settingsRet;
  if (!resp) {
    ret.settings = {};
    ret.state = RET.TIME_OUT;
    return ret;
  }
  if (resp.ok) {
    try {
      ret.settings = await resp.json();
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
 * Common Function for getting page Data
 * @param {string}        authToken  AuthToken
 * @param {readonly   string[]} settingList   Settings List
 * @param {string}        storeName  setting store name
 * @param {formDataStore} store      Pina store formData
 * @param {Ref<any>}      dataRef    Page reactiveData Ref
 * @param {Ref<boolean>}  loadFail   Load fail Ref
 * @param {Ref<boolean>}  authFail   Auth fail Ref
 * @param {()         =>         void}        logOutFn      Logout callback function
 */
export async function loadPageData(
  authToken: string,
  settingList: readonly string[],
  storeName: string,
  store: formDataStore,
  dataRef: Ref<any>,
  loadFail: Ref<boolean>,
  authFail: Ref<boolean>,
  logOutFn: () => void
  ) {
  if (storeName in store) {
    Object.assign(dataRef.value, store[storeName]);
  } else {
    const resp = await getSettings(settingList, authToken);
    if (resp.state === RET.OK) {
      Object.assign(store, { [storeName]: { ...resp.settings }});
      Object.assign(dataRef.value, { ...resp.settings });
    } else if (resp.state === RET.AUTH_FAIL) {
      authFail.value = true;
      logOutFn();
    } else {
      loadFail.value = true;
    }
  }
}

/**
 * Apply settings to WNB device
 * @param  {strObj}               settingObject Settings Object
 * @param  {strObj}
 * @param  {string}               authToken     AuthToken
 * @return {Promise<settingsRet>}               Return data
 */
export async function applySettings(settingObject: strObj, authToken: string): Promise<settingsRet> {
  //const resp = await fetch(settingsPath, new fetchOptions(object2StringSet(settingObject), authToken));
  const resp = await fetchTimeOut(settingsPath, new fetchOptions(object2StringSet(settingObject), authToken));
  var ret = {} as settingsRet;
  if (!resp) {
    ret.state = RET.TIME_OUT;
    return ret;
  }
  if (resp.ok) {
    ret.state = RET.OK
  } else if (resp.status === 401) {
    ret.state = RET.AUTH_FAIL;
  } else {
    ret.state = RET.FAIL;
  }
  return ret;
}
