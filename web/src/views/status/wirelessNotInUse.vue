<script setup lang="ts">
  import { ref, computed, reactive, onBeforeMount } from 'vue';
  // @ts-ignore
  import { useI18n } from 'vue-i18n';
  import { wifiBasicSettings, wifiBasicSettingsData } from '@/assets/commonSettings.ts';
  import { getSettings, connStat, type connStatClass, stationList, staInfoClass, RET } from '@/wnbCom';
  import { WIFI_MODE, KEY_MGMT } from '@/utils';
  import { useDashboardStore, useFormDataStore } from '@/store';
  import { isAuthenticated } from '@/router';
  import { useRouter } from 'vue-router';
  import msgPopupVue from '@/components/msgPopup.vue';

  const { t } = useI18n({
    inheritLocale: true,
    useScope: 'local'
  });
  const { t: tg } = useI18n({
    useScope: 'global'
  });

  const wifiSettingReactive = reactive({} as wifiBasicSettings);
  const connStatDisplay = reactive({} as connStatClass);
  const stationListDisplay = ref<staInfoClass[]>([]);

  const dashboardStore = useDashboardStore();
  const formDataStore = useFormDataStore();
  const router = useRouter();

  const popAuthFail = ref<boolean>(false);
  const popLoadFail = ref<boolean>(false);

  function logoutIn3s() {
    setTimeout(() => {
      isAuthenticated.value = false;
      router.push({ name: 'Login '});
    }, 3000)
  }

  async function pollConnState() {
    const resp = await connStat(dashboardStore.authToken);
    if (resp.state === RET.OK) {
      Object.assign(connStatDisplay, resp.connStat);
    } else if (resp.state === RET.AUTH_FAIL) {
      popAuthFail.value = true;
      logoutIn3s();
    } else {
      popLoadFail.value = true;
    }
  }

  async function pollStationList() {
    const resp = await stationList(dashboardStore.authToken);
    if (resp.state === RET.OK) {
      stationListDisplay.value = [...resp.staList];
    } else if (resp.state === RET.AUTH_FAIL) {
      popAuthFail.value = true;
      logoutIn3s();
    } else {
      popLoadFail.value = true;
    }
  }

  onBeforeMount(async () => {
    if ('wifiBasic' in formDataStore.dataStore) {
      Object.assign(wifiSettingReactive, formDataStore.dataStore['wifiBasic']);
    } else {
      const resp = await getSettings(wifiBasicSettingsData, dashboardStore.authToken);
      if (resp.state === RET.OK) {
        Object.assign(formDataStore.dataStore, { wifiBasic: { ...resp.settings } });
        Object.assign(wifiSettingReactive, resp.settings);
      } else if (resp.state === RET.AUTH_FAIL) {
        popAuthFail.value = true;
        logoutIn3s();
      } else {
        popLoadFail.value = true;
      }
    }
    await pollConnState();
    if (wifiSettingReactive.wifi_mode !== WIFI_MODE.STA)
      await pollStationList();
  });

  const signalStrength = computed(() => {
    return Math.min(Math.max(2 * (connStatDisplay.rssi + 100), 0), 100);
  });

  const wifiOpmode = computed(() => {
    switch (wifiSettingReactive.wifi_mode) {
    case WIFI_MODE.AP:
      return tg('wifiModeAP');
    case WIFI_MODE.STA:
      return tg('wifiModeSTA');
    case WIFI_MODE.APSTA:
      return tg('wifiModeAPSTA');
    }
  });

  const channel = computed(() => {
    const chanNum = connStatDisplay.channel;
    if (chanNum < 13) {
      let freq = chanNum * 5 + 2407;
      return chanNum + ' (' + freq +  'Mhz)';
    } else if (chanNum === 13) {
      return '13 (2484Mhz)';
    } else {
      return t('noData');
    }
  });

  const connState = computed(() => {
    switch (wifiSettingReactive.wifi_mode) {
    case WIFI_MODE.AP:
      return tg('wifiModeAP');
    case WIFI_MODE.STA:
      return connStatDisplay.connected ? t('connected') : t('disconnected');
    case WIFI_MODE.APSTA:
      return '<' + wifiSettingReactive.wifi_ssid + '> ' + (connStatDisplay.connected ? t('connected') : t('disconnected'));
    }
  });

  function stationListIP(ip: string) {
    if (ip === '0' || ip === '0.0.0.0')
      return t('noData');
    else
      return ip;
  }

</script>

<template>
  <table>
    <thead><tr><th colspan="2">{{t('WiFiInfo')}}</th></tr></thead>
    <tbody>
      <tr><td class="label">SSID</td><td class="data">{{wifiSettingReactive.wifi_ssid}}</td></tr>
      <tr v-if="wifiSettingReactive.wifi_mode === WIFI_MODE.APSTA">
        <td class="label">{{t('rssid')}}</td><td class="data">{{wifiSettingReactive.wifi_r_ssid}}</td>
      </tr>
      <tr><td class="label">{{t('opMode')}}</td><td class="data">{{wifiOpmode}}</td></tr>
      <tr>
        <td class="label">{{t('keymgmt')}}</td>
        <td class="data">{{wifiSettingReactive.wifi_keymgmt === KEY_MGMT.PSK ? 'WPA-PSK' : 'NONE'}}</td>
      </tr>
      <tr v-if="wifiSettingReactive.wifi_keymgmt === KEY_MGMT.PSK">
        <td class="label">{{t('passwd')}}</td><td class="data">{{wifiSettingReactive.wifi_passwd}}</td>
      </tr>
    </tbody>
  </table>
  <table>
    <thead><tr><th colspan="2">{{t('connInfo')}}</th></tr></thead>
    <tbody>
      <tr><td class="label">{{t('connStat')}}</td><td class="data">{{connState}}</td></tr>
      <tr><td class="label">{{t('channel')}}</td><td class="data">{{channel}}</td></tr>
      <tr v-if="connStatDisplay.connected">
        <td class="label">{{t('signal')}}</td>
        <td class="data"><progress :value="signalStrength" max="100"></progress></td>
      </tr>
    </tbody>
  </table>
  <table>
    <thead>
      <tr><th colspan="6">{{t('ClientList')}}</th></tr>
      <tr><th>AID</th><th>BSSID</th><th>IP</th><th>{{t('powerSave')}}</th><th>RSSI</th><th>EVM</th></tr>
    </thead>
    <tbody>
      <tr v-for="item in stationListDisplay">
        <td>{{item.aid}}</td>
        <td>{{item.mac}}</td>
        <td>{{stationListIP(item.ip)}}</td>
        <td>{{item.ps ? 'ON' : 'OFF'}}</td>
        <td>{{item.rssi}}</td>
        <td>{{item.evm}}</td>
      </tr>
    </tbody>
  </table>
  <msgPopupVue v-model:pop="popAuthFail" mode="Alert">{{tg('authTMout')}}</msgPopupVue>
  <msgPopupVue v-model:pop="popLoadFail" mode="Alert">{{tg('pageLoadErr')}}</msgPopupVue>
</template>

<style scoped>
  table {
    margin-top: 20px;
    width: 560px;
  }
  .label {
    width: 16%;
  }
  .data {
    width: 84%;
  }
</style>

<i18n lang="json">
  {
    "enUS": {
      "WiFiInfo": "WiFi Information",
      "opMode": "Operation Mode:",
      "keymgmt": "Encryption Mode:",
      "rssid": "Repeater SSID:",
      "passwd": "Password:",
      "connInfo": "Connection Info",
      "connStat": "Connection Status:",
      "connected": "Connected",
      "disconnected": "Disconnected",
      "signal": "Signal Strength:",
      "channel": "Channel:",
      "ClientList": "WiFi Client List",
      "powerSave": "Power Save",
      "noData": "No Data"
    },
    "zhCN": {
      "WiFiInfo": "无线信息",
      "opMode": "工作模式:",
      "keymgmt": "加密模式:",
      "rssid": "扩展SSID:",
      "passwd": "无线密码:",
      "connInfo": "连接信息",
      "connStat": "连接状态:",
      "connected": "已连接",
      "disconnected": "断开",
      "signal": "信号强度:",
      "channel": "信道:",
      "ClientList": "无线客户端列表",
      "powerSave": "节能模式",
      "noData": "无数据"   
    }
  }
</i18n>