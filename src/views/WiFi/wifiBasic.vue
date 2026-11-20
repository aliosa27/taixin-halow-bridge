<script setup lang="ts">
  import { ref, reactive, watch, computed, onBeforeMount, onBeforeUnmount } from 'vue';
  import { useI18n } from 'vue-i18n';
  import { WIFI_MODE, KEY_MGMT, inputValidate, strObj, INPUT_CHECK, SELECT, wifiModeKey } from '@/utils';
  import { loadPageData, getSettings, pairing, RET, type apInfoClass } from '@/wnbCom';
  import { useFormDataStore, useDashboardStore } from '@/store';
  import { logoutIn3s } from '@/router';
  import msgPopupVue from '@/components/msgPopup.vue';
  import WiFiScanPopupVue from '@/components/WiFiScanPopup.vue';
  import saveApplyBtnVue from '@/components/saveApplyBtn.vue';

  const { t } = useI18n({
    inheritLocale: true,
    useScope: 'local'
  });
  const { t: tg } = useI18n({
    useScope: 'global'
  });

  const formDataStore = useFormDataStore();
  const dashboardStore = useDashboardStore();

  const wifiBasicSettingsData =
  ['wifi_mode', 'wifi_wnb', 'wifi_keymgmt', 'wifi_ssid', 'wifi_passwd', 'wifi_r_ssid', 'wifi_r_keymgmt', 'wifi_r_passwd', 'wifi_ap_hide'] as const;

  export type wifiBasicSettings = {
    [key in typeof wifiBasicSettingsData[number]]: string;
  }

  const settingReactive = ref<wifiBasicSettings>({} as wifiBasicSettings);

  const popAuthFail = ref<boolean>(false);
  const popLoadFail = ref<boolean>(false);
  const popWiFiScanner = ref<boolean>(false);
  const inputOK = ref<boolean>(true);
  const showInputErrMsg: strObj = reactive({
    ssid: '',
    rssid: '',
    psk: '',
    rpsk: ''
  });

  onBeforeMount(async () => await loadPageData(
    dashboardStore.authToken,
    wifiBasicSettingsData,
    'wifiBasic',
    formDataStore.dataStore,
    settingReactive,
    popLoadFail,
    popAuthFail,
    logoutIn3s
    ));

  const apHide = computed({
    get() {
      if (settingReactive.value.wifi_ap_hide === SELECT.ENABLE)
        return true;
      else
        return false;
    },
    set(newVal) {
      if (newVal === true)
        settingReactive.value.wifi_ap_hide = SELECT.ENABLE;
      else
        settingReactive.value.wifi_ap_hide = SELECT.DISABLE;
    }
  });

  // check for any input err
  watch(settingReactive.value, () => {
    var errs = 0
    switch (inputValidate.isSSID(settingReactive.value.wifi_ssid)) {
    case INPUT_CHECK.OK:
      showInputErrMsg.ssid = '';
      break;
    case INPUT_CHECK.EMPTY:
      showInputErrMsg.ssid = tg('emptyInput');
      errs++;
      break;
    case INPUT_CHECK.INVALID:
      showInputErrMsg.ssid = tg('invalidInput');
      errs++;
      break;
    }
    switch (inputValidate.isPSK(settingReactive.value.wifi_passwd)) {
    case INPUT_CHECK.OK:
      showInputErrMsg.psk = '';
      break;
    case INPUT_CHECK.EMPTY:
      if (settingReactive.value.wifi_keymgmt === KEY_MGMT.PSK || settingReactive.value.wifi_keymgmt === KEY_MGMT.SAE) {
        showInputErrMsg.psk = tg('emptyInput');
        errs++;
      } else {
        showInputErrMsg.psk = '';
      }
      break;
    case INPUT_CHECK.INVALID:
      showInputErrMsg.psk = tg('invalidInput');
      errs++;
      break;
    }
    if (settingReactive.value.wifi_mode === WIFI_MODE.APSTA) {
      switch (inputValidate.isSSID(settingReactive.value.wifi_r_ssid)) {
      case INPUT_CHECK.OK:
        showInputErrMsg.rssid = '';
        break;
      case INPUT_CHECK.EMPTY:
        showInputErrMsg.rssid = tg('emptyInput');
        errs++;
        break;
      case INPUT_CHECK.INVALID:
        showInputErrMsg.rssid = tg('invalidInput');
        errs++;
        break;
      }
      switch (inputValidate.isPSK(settingReactive.value.wifi_r_passwd)) {
      case INPUT_CHECK.OK:
        showInputErrMsg.rpsk = '';
        break;
      case INPUT_CHECK.EMPTY:
        if (settingReactive.value.wifi_r_keymgmt === KEY_MGMT.PSK) {
          showInputErrMsg.rpsk = tg('emptyInput');
          errs++;
        } else {
          showInputErrMsg.rpsk = '';
        }
        break;
      case INPUT_CHECK.INVALID:
        showInputErrMsg.rpsk = tg('invalidInput');
        errs++;
        break;
      }
    }
    inputOK.value = errs === 0;
  });

  function modeChange() {
    // Reset WPA3 encrypt, only supported in STA mode
    if (settingReactive.value.wifi_mode !== WIFI_MODE.STA)
      if (settingReactive.value.wifi_keymgmt === KEY_MGMT.OWE || settingReactive.value.wifi_keymgmt === KEY_MGMT.SAE)
        settingReactive.value.wifi_keymgmt = KEY_MGMT.NONE;
    // Autofill repeater cfg
    if (settingReactive.value.wifi_mode === WIFI_MODE.APSTA) {
      if (settingReactive.value.wifi_r_ssid.length === 0 || settingReactive.value.wifi_r_keymgmt === '0') {
        settingReactive.value.wifi_r_ssid = settingReactive.value.wifi_ssid;
        settingReactive.value.wifi_r_keymgmt = settingReactive.value.wifi_keymgmt;
        if (settingReactive.value.wifi_keymgmt === KEY_MGMT.PSK)
          settingReactive.value.wifi_r_passwd = settingReactive.value.wifi_passwd;
      }
    }
  }

  function scannerSelected(selected: apInfoClass) {
    settingReactive.value.wifi_ssid = selected.ssid;
    if (selected.encrypt === true) {
      settingReactive.value.wifi_keymgmt = KEY_MGMT.PSK;
      settingReactive.value.wifi_passwd = '';
    } else {
      settingReactive.value.wifi_keymgmt = KEY_MGMT.NONE;
    }
  }

  /* ---- Operating mode: side (wifi_mode AP/STA/APSTA) + flavour (wifi_wnb) ---- */
  type modeDef = { id: string, mode: WIFI_MODE, wnb: string };
  const modeDefs: modeDef[] = [
    { id: 'ap', mode: WIFI_MODE.AP, wnb: '0' },
    { id: 'wnbap', mode: WIFI_MODE.AP, wnb: '1' },
    { id: 'sta', mode: WIFI_MODE.STA, wnb: '0' },
    { id: 'wnbsta', mode: WIFI_MODE.STA, wnb: '1' },
    { id: 'apsta', mode: WIFI_MODE.APSTA, wnb: '0' }
  ];

  function modeIdOf(mode: string, wnb: string): string {
    if (mode === WIFI_MODE.APSTA) return 'apsta';
    return (wnb === '1' ? 'wnb' : '') + (mode === WIFI_MODE.AP ? 'ap' : 'sta');
  }

  const savedMode = computed(() => {
    const saved = formDataStore.dataStore.wifiBasic;
    return saved ? modeIdOf(saved.wifi_mode, saved.wifi_wnb) : '';
  });

  const curMode = computed({
    get() {
      return modeIdOf(settingReactive.value.wifi_mode, settingReactive.value.wifi_wnb);
    },
    set(id: string) {
      const d = modeDefs.find((m) => m.id === id);
      if (!d) return;
      settingReactive.value.wifi_mode = d.mode;
      if (settingReactive.value.wifi_wnb !== undefined && settingReactive.value.wifi_wnb !== '')
        settingReactive.value.wifi_wnb = d.wnb;
      modeChange();
    }
  });

  const modeChoices = computed(() => {
    const cfg = dashboardStore.deviceConfig;
    const savedSide = formDataStore.dataStore.wifiBasic?.wifi_mode;
    return modeDefs.filter((d) => {
      if (d.wnb === '1' && !cfg.wnbEn) return false;
      if (d.mode === WIFI_MODE.APSTA) return !cfg.noAPSTAmode && !cfg.fixWiFiMode;
      if (cfg.fixWiFiMode) return d.mode === savedSide;   // role pin decides the side
      return true;
    });
  });

  const showModeSelect = computed(() => modeChoices.value.length > 1);

  const isAPSide = computed(() => settingReactive.value.wifi_mode === WIFI_MODE.AP);

  const showScanner = computed(() => !isAPSide.value);

  const configLabel = computed(() => {
    if (settingReactive.value.wifi_mode === WIFI_MODE.APSTA)
      return t('upstream');
    return isAPSide.value ? t('apCfg') : t('staCfg');
  });

  /* ---- Pairing (firmware extension: pair=1|0, pair_state bit1 = paired) ---- */
  const PAIR_LIMIT_MS = 60000;
  const pairActive = ref<boolean>(false);
  const pairMsg = ref<string>('');
  let pairTimer = 0;
  let pairEnd = 0;

  function pairFinish(msg: string) {
    clearTimeout(pairTimer);
    pairActive.value = false;
    pairMsg.value = msg;
  }

  async function pairPoll() {
    const r = await getSettings(['pair_state'], dashboardStore.authToken);
    if (r.state === RET.AUTH_FAIL) {
      pairFinish('');
      popAuthFail.value = true;
      logoutIn3s();
      return;
    }
    const st = Number(r.settings?.pair_state ?? 0);
    if (r.state === RET.OK && !(st & 2) && Date.now() < pairEnd) {
      pairTimer = window.setTimeout(pairPoll, 2000);
      return;
    }
    await pairing(false, dashboardStore.authToken);   // paired, timed out or lost contact: stop, like releasing the button
    if (st & 2) {
      pairFinish(t('pairOk'));
      // a station takes the access point's SSID and key: show what was saved
      await loadPageData(dashboardStore.authToken, wifiBasicSettingsData, 'wifiBasic',
        formDataStore.dataStore, settingReactive, popLoadFail, popAuthFail, logoutIn3s);
    } else {
      pairFinish(t('pairNone'));
    }
  }

  async function pairToggle() {
    if (pairActive.value) {
      clearTimeout(pairTimer);
      await pairing(false, dashboardStore.authToken);
      pairFinish(t('pairStopped'));
      return;
    }
    const ret = await pairing(true, dashboardStore.authToken);
    if (ret === RET.AUTH_FAIL) {
      popAuthFail.value = true;
      logoutIn3s();
      return;
    }
    if (ret !== RET.OK) {
      pairMsg.value = t('pairFail');
      return;
    }
    pairActive.value = true;
    pairMsg.value = t('pairRunning');
    pairEnd = Date.now() + PAIR_LIMIT_MS;
    pairTimer = window.setTimeout(pairPoll, 2000);
  }

  onBeforeUnmount(() => {
    if (pairActive.value) {
      clearTimeout(pairTimer);
      pairing(false, dashboardStore.authToken);
    }
  });

</script>

<template>
  <div class="header">
    <h1>{{t('header')}}</h1>
  </div>
  <div class="content">
    <form>
      <fieldset v-if="showModeSelect">
        <legend>{{t('WiFiMode')}}</legend>
        <p class="intro">{{t('modeIntro')}}</p>
        <div class="modes">
          <label v-for="m in modeChoices" :key="m.id" class="mode" :class="{ sel: curMode === m.id }">
            <input type="radio" name="wifimode" :value="m.id" v-model="curMode" />
            <span>
              <b>{{tg(wifiModeKey(m.mode, m.wnb))}}</b>
              <small>{{t('help_' + m.id)}}</small>
            </span>
          </label>
        </div>
        <p class="note" v-if="dashboardStore.deviceConfig.fixWiFiMode">{{t('pinNote')}}</p>
        <p class="note warn" v-if="savedMode && curMode !== savedMode">{{t('rebootNote')}}</p>
      </fieldset>
      <fieldset>
        <legend>{{configLabel}}</legend>
        <div class="ctrl-group" v-if="showScanner">
          <label>{{t('WiFiScanner')}}</label>
          <input type="button" :value="t('btnScanner')" @click="popWiFiScanner = true" />
          <span class="msg-help">{{t('scannerHelpMsg')}}</span>
        </div>
        <div class="ctrl-group">
          <label for="ssid">SSID</label>
          <input type="text" id="ssid" v-model.trim="settingReactive.wifi_ssid" />
          <span class="msg-warn">{{showInputErrMsg.ssid}}</span>
        </div>
        <div class="ctrl-group" v-if="isAPSide">
          <label for="hidessid">{{t('hideSSID')}}</label>
          <input type="checkbox" id="hidessid" value="1" v-model="apHide" />
        </div>
        <div class="ctrl-group">
          <label for="keymgmt">{{t('keymgmt')}}</label>
          <select id="keymgmt" v-model="settingReactive.wifi_keymgmt">
            <option :value="KEY_MGMT.PSK">WPA-PSK</option>
            <option :value="KEY_MGMT.NONE">{{t('open')}}</option>
            <template v-if="settingReactive.wifi_mode === WIFI_MODE.STA">
              <option :value="KEY_MGMT.SAE" v-if="dashboardStore.deviceConfig.saeEn">WPA3-SAE</option>
              <option :value="KEY_MGMT.OWE" v-if="dashboardStore.deviceConfig.oweEn">WPA3-OWE</option>
            </template>
          </select>
        </div>
        <div class="ctrl-group" v-if="settingReactive.wifi_keymgmt !== KEY_MGMT.NONE">
          <label for="passphase">{{t('psk')}}</label>
          <input type="text" id="passphase" class="wide" v-model.trim="settingReactive.wifi_passwd" />
          <span class="msg-warn">{{showInputErrMsg.psk}}</span>
          <span class="msg-help">{{t('pskHelp')}}</span>
        </div>
      </fieldset>
      <fieldset>
        <legend>{{t('pairing')}}</legend>
        <p class="intro">{{t('pairIntro')}}</p>
        <div class="ctrl-group">
          <label>{{t('pairing')}}</label>
          <input type="button" :value="pairActive ? t('pairStop') : t('pairStart')" @click="pairToggle" />
          <span class="msg-help">{{pairMsg}}</span>
        </div>
      </fieldset>
      <fieldset v-if="settingReactive.wifi_mode === WIFI_MODE.APSTA">
        <legend>{{t('repeaterCfg')}}</legend>
        <div class="ctrl-group">
          <label for="rssid">SSID</label>
          <input type="text" id="rssid" v-model.trim="settingReactive.wifi_r_ssid" />
          <span class="msg-warn">{{showInputErrMsg.rssid}}</span>
        </div>
        <div class="ctrl-group">
          <label for="rhidessid">{{t('hideSSID')}}</label>
          <input type="checkbox" id="rhidessid" value="1" v-model="apHide" />
        </div>
        <div class="ctrl-group">
          <label for="rkeymgmt">{{t('keymgmt')}}</label>
          <select id="rkeymgmt" v-model="settingReactive.wifi_r_keymgmt">
            <option :value="KEY_MGMT.PSK">WPA-PSK</option>
            <option :value="KEY_MGMT.NONE">{{t('open')}}</option>
          </select>
        </div>
        <div class="ctrl-group">
          <label for="rssid_passwd">{{t('psk')}}</label>
          <input type="text" id="rssid_passwd" v-model.trim="settingReactive.wifi_r_passwd" />
          <span class="msg-warn">{{showInputErrMsg.rpsk}}</span>
        </div>
      </fieldset>
    </form>
    <saveApplyBtnVue which-settings="wifiBasic" :setting-reactive="settingReactive" :input-checked="inputOK"></saveApplyBtnVue>
  </div>
  <WiFiScanPopupVue v-model:pop="popWiFiScanner" @selected="scannerSelected"></WiFiScanPopupVue>
  <msgPopupVue v-model:pop="popAuthFail" mode="Logout">{{tg('authTMout')}}</msgPopupVue>
  <msgPopupVue v-model:pop="popLoadFail" mode="Alert">{{tg('pageLoadErr')}}</msgPopupVue>
</template>

<style scoped>
  .intro, .note { margin: 0 0 1rem; color: var(--muted); font-size: .9em; }
  .note.warn { color: var(--warn); font-weight: 500; }
  .modes {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: .75rem;
    margin-bottom: 1rem;
  }
  .mode {
    display: flex;
    gap: .6rem;
    align-items: flex-start;
    padding: .8rem .9rem;
    border: 1px solid var(--border);
    border-radius: 10px;
    cursor: pointer;
  }
  .mode.sel { border-color: var(--primary); background: var(--primary-soft); }
  .mode input { margin-top: .2em; }
  .mode b { display: block; }
  .mode small { display: block; color: var(--muted); line-height: 1.4; margin-top: .2rem; }
  @media (max-width: 600px) { .modes { grid-template-columns: 1fr; } }
</style>

<i18n lang="json">
  {
    "enUS": {
      "header": "WiFi",
      "WiFiMode": "Operating mode",
      "modeIntro": "A link needs one access point and one station of the same type: AP with Station, or WNB AP with WNB Station. Use the standard modes to work with other vendors' HaLow devices; use WNB when both ends are these bridges.",
      "help_ap": "Standard 802.11ah (Wi-Fi HaLow) access point. Use on the side connected to your main network. Any HaLow station can join, including a bridge in Station mode.",
      "help_wnbap": "Access point using Taixin's WNB bridge protocol instead of standard 802.11ah. Only bridges in WNB Station mode can connect.",
      "help_sta": "Standard 802.11ah client. Joins the access point below and bridges this device's Ethernet port onto that network. Works with any HaLow access point.",
      "help_wnbsta": "Client using Taixin's WNB bridge protocol. Connects only to a bridge in WNB Access Point mode, and can roam between WNB access points.",
      "help_apsta": "Joins an upstream access point and re-broadcasts it as a second access point to extend range.",
      "pinNote": "The access point / station side is set by the role pin (PA_8) at power-up, so only the protocol type can be changed here.",
      "rebootNote": "Saving a new mode reboots the device (about 20 seconds). If it gets its IP address by DHCP, the address may change.",
      "apCfg": "Access point",
      "staCfg": "Connect to access point",
      "upstream": "Upstream access point",
      "repeaterCfg": "Repeater access point",
      "WiFiScanner": "Find networks",
      "btnScanner": "Scan…",
      "scannerHelpMsg": "Pick an access point from a scan to fill in its SSID.",
      "keymgmt": "Security",
      "open": "Open (no encryption)",
      "psk": "Passphrase or key",
      "pskHelp": "8–32 characters, or the raw key as 64 hex digits. A key set by AT+PSK or by pairing is shown as hex.",
      "hideSSID": "Hide SSID",
      "pairing": "Pairing",
      "pairIntro": "Same as holding the pair button: start pairing here and on the other device. A station takes the access point's SSID and key. Stops after 60 seconds.",
      "pairStart": "Start pairing",
      "pairStop": "Stop pairing",
      "pairRunning": "Pairing… start pairing on the other device now.",
      "pairOk": "Paired. Settings below were reloaded.",
      "pairNone": "Pairing ended without a peer.",
      "pairStopped": "Pairing stopped.",
      "pairFail": "Could not start pairing."
    }
  }
</i18n>
