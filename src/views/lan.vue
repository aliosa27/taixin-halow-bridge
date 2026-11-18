<script setup lang="ts">
  import { ref, reactive, watch, computed, onBeforeMount } from 'vue';
  import { useI18n } from 'vue-i18n';
  import { inputValidate, strObj, INPUT_CHECK, SELECT, WIFI_MODE } from '@/utils';
  import { loadPageData } from '@/wnbCom';
  import { useDashboardStore, useFormDataStore } from '@/store';
  import { logoutIn3s } from '@/router';
  import msgPopupVue from '@/components/msgPopup.vue';
  import saveApplyBtnVue from '@/components/saveApplyBtn.vue';

  const { t } = useI18n({
    inheritLocale: true,
    useScope: 'local'
  });
  const { t: tg } = useI18n({
    useScope: 'global'
  });

  const dashboardStore = useDashboardStore();
  const formDataStore = useFormDataStore();

  const lanSettingsData =
  ['dhcpc_en', 'dhcpd_en', 'ipaddr', 'netmask', 'gw_ip', 'dhcp_startip', 'dhcp_endip', 'dhcp_router', 'dhcp_lease_time',
    'dhcp_dns1', 'dhcp_dns2'] as const;

  export type lanSettings = {
    [key in typeof lanSettingsData[number]]: string;
  }

  const settingReactive = ref<lanSettings>({} as lanSettings);

  const popAuthFail = ref<boolean>(false);
  const popLoadFail = ref<boolean>(false);
  const inputOK = ref<boolean>(true);
  const disableIPModeSelect = ref<boolean>(false);
  const showInputErrMsg: strObj = reactive({
    ipaddr: '',
    netmask: '',
    gwip: '',
    startip: '',
    endip: '',
    router: '',
    dns1: '',
    dns2: ''
  });

  onBeforeMount(async () => {
    loadPageData(
      dashboardStore.authToken,
      lanSettingsData,
      'lan',
      formDataStore.dataStore,
      settingReactive,
      popLoadFail,
      popAuthFail,
      logoutIn3s
      );
    // Lock IP mode to DHCP if device in APSTA mode
    // should be in store as landing page is overview
    if ('wifiBasic' in formDataStore.dataStore) {
      if (formDataStore.dataStore.wifiBasic.wifi_mode === WIFI_MODE.APSTA)
        disableIPModeSelect.value = true;
    }
  });

  const dhcpc_en = computed({
    get() {
      if (disableIPModeSelect.value)
        return SELECT.ENABLE;
      else 
      return settingReactive.value.dhcpc_en;
    },
    set(newVal) {
      settingReactive.value.dhcpc_en = newVal;
    }
  });

  const leaseTime = computed({
    get() {
      return parseInt(settingReactive.value.dhcp_lease_time);
    },
    set(newVal) {
      settingReactive.value.dhcp_lease_time = newVal.toString();
    }
  });

  watch(settingReactive.value, () => {
    var errs = 0;
    switch (inputValidate.isPrivateIpv4(settingReactive.value.ipaddr)) {
    case INPUT_CHECK.OK:
      showInputErrMsg.ipaddr = '';
      break;
    case INPUT_CHECK.EMPTY:
      showInputErrMsg.ipaddr = tg('emptyInput');
      errs++;
      break;
    case INPUT_CHECK.INVALID:
      showInputErrMsg.ipaddr = tg('invalidInput');
      errs++;
      break;
    }
    switch (inputValidate.isNetmask(settingReactive.value.netmask)) {
    case INPUT_CHECK.OK:
      showInputErrMsg.netmask = '';
      break;
    case INPUT_CHECK.EMPTY:
      showInputErrMsg.netmask = tg('emptyInput');
      errs++;
      break;
    case INPUT_CHECK.INVALID:
      showInputErrMsg.netmask = tg('invalidInput');
      errs++;
      break;
    }
    switch (inputValidate.isPrivateIpv4(settingReactive.value.gw_ip)) {
    case INPUT_CHECK.OK:
      showInputErrMsg.gwip = '';
      break;
    case INPUT_CHECK.EMPTY:
      showInputErrMsg.gwip = tg('emptyInput');
      errs++;
      break;
    case INPUT_CHECK.INVALID:
      showInputErrMsg.gwip = tg('invalidInput');
      errs++;
      break;
    }
    switch (inputValidate.isPrivateIpv4(settingReactive.value.dhcp_startip)) {
    case INPUT_CHECK.OK:
      showInputErrMsg.startip = '';
      break;
    case INPUT_CHECK.EMPTY:
      showInputErrMsg.startip = tg('emptyInput');
      errs++;
      break;
    case INPUT_CHECK.INVALID:
      showInputErrMsg.startip = tg('invalidInput');
      errs++;
      break;
    }
    switch (inputValidate.isPrivateIpv4(settingReactive.value.dhcp_endip)) {
    case INPUT_CHECK.OK:
      showInputErrMsg.endip = '';
      break;
    case INPUT_CHECK.EMPTY:
      showInputErrMsg.endip = tg('emptyInput');
      errs++;
      break;
    case INPUT_CHECK.INVALID:
      showInputErrMsg.endip = tg('invalidInput');
      errs++;
      break;
    }
    switch (inputValidate.isPrivateIpv4(settingReactive.value.dhcp_router)) {
    case INPUT_CHECK.OK:
      showInputErrMsg.router = '';
      break;
    case INPUT_CHECK.EMPTY:
      showInputErrMsg.router = tg('emptyInput');
      errs++;
      break;
    case INPUT_CHECK.INVALID:
      showInputErrMsg.router = tg('invalidInput');
      errs++;
      break;
    }
    switch (inputValidate.isIpv4(settingReactive.value.dhcp_dns1)) {
    case INPUT_CHECK.OK:
      showInputErrMsg.dns1 = '';
      break;
    case INPUT_CHECK.EMPTY:
      showInputErrMsg.dns1 = tg('emptyInput');
      errs++;
      break;
    case INPUT_CHECK.INVALID:
      showInputErrMsg.dns1 = tg('invalidInput');
      errs++;
      break;
    }
    switch (inputValidate.isIpv4(settingReactive.value.dhcp_dns2)) {
    case INPUT_CHECK.OK:
      showInputErrMsg.dns2 = '';
      break;
    case INPUT_CHECK.EMPTY:
      showInputErrMsg.dns2 = tg('emptyInput');
      errs++;
      break;
    case INPUT_CHECK.INVALID:
      showInputErrMsg.dns2 = tg('invalidInput');
      errs++;
      break;
    }
    inputOK.value = errs === 0;
  });

  const showDHCPD = computed(() => {
    if (dashboardStore.deviceConfig.sysAppDHCPD && dhcpc_en.value === SELECT.DISABLE)
      return true;
    else
      return false;
  });

</script>

<template>
  <div class="header">
    <h1>{{t('header')}}</h1>
  </div>
  <div class="content">
    <form>
      <fieldset>
        <legend>{{t('ip')}}</legend>
        <div class="ctrl-group">
          <label for="dhcpClient">{{t('ipMode')}}</label>
          <select id="dhcpClient" v-model="dhcpc_en" :disabled="disableIPModeSelect">
            <option :value="SELECT.DISABLE">{{t('static')}}</option>
            <option :value="SELECT.ENABLE">DHCP</option>
          </select>
        </div>
        <div v-if="dhcpc_en === SELECT.DISABLE">
          <div class="ctrl-group">
            <label for="ipaddr">{{t('ipaddr')}}</label>
            <input type="text" id="ipaddr" v-model.trim="settingReactive.ipaddr" />
            <span class="msg-warn">{{showInputErrMsg.ipaddr}}</span>
          </div>
          <div class="ctrl-group">
            <label for="netmask">{{t('netmask')}}</label>
            <input type="text" id="netmask" v-model.trim="settingReactive.netmask" />
            <span class="msg-warn">{{showInputErrMsg.netmask}}</span>
          </div>
          <div class="ctrl-group">
            <label for="gwip">{{t('gwip')}}</label>
            <input type="text" id="gwip" v-model.trim="settingReactive.gw_ip" />
            <span class="msg-warn">{{showInputErrMsg.gwip}}</span>
          </div>
        </div>
      </fieldset>
      <fieldset v-if="showDHCPD">
        <legend>{{t('dhcpd')}}</legend>
        <div class="ctrl-group">
          <label for="dhcpServer">{{t('server')}}</label>
          <select id="dhcpServer" v-model="settingReactive.dhcpd_en">
            <option :value="SELECT.ENABLE">{{tg('enable')}}</option>
            <option :value="SELECT.DISABLE">{{tg('disable')}}</option>
          </select>
        </div>
        <div v-if="settingReactive.dhcpd_en === SELECT.ENABLE">
          <div class="ctrl-group">
            <label for="dhcpStartIP">{{t('dhcpStart')}}</label>
            <input type="text" id="dhcpStartIP" v-model.trim="settingReactive.dhcp_startip" />
            <span class="msg-warn">{{showInputErrMsg.startip}}</span>
          </div>
          <div class="ctrl-group">
            <label for="dhcpStopIP">{{t('dhcpEnd')}}</label>
            <input type="text" id="dhcpStartIP" v-model.trim="settingReactive.dhcp_endip" />
            <span class="msg-warn">{{showInputErrMsg.endip}}</span>
          </div>
          <div class="ctrl-group">
            <label for="">{{t('dhcpRouter')}}</label>
            <input type="text" id="dhcpRouter" v-model.trim="settingReactive.dhcp_router" />
            <span class="msg-warn">{{showInputErrMsg.router}}</span>
          </div>
          <div class="ctrl-group">
            <label for="dhcpLeaseTime">{{t('leaseTime')}}</label>
            <input type="number" id="dhcpLeaseTime" v-model="leaseTime" />
          </div>
          <div class="ctrl-group">
            <label for="dns1">{{t('dns1')}}</label>
            <input type="text" id="dns1" v-model.trim="settingReactive.dhcp_dns1" />
            <span class="msg-warn">{{showInputErrMsg.dns1}}</span>
          </div>
          <div class="ctrl-group">
            <label for="dns2">{{t('dns2')}}</label>
            <input type="text" id="dns2" v-model.trim="settingReactive.dhcp_dns2" />
            <span class="msg-warn">{{showInputErrMsg.dns2}}</span>
          </div>
        </div>
      </fieldset>
    </form>
    <saveApplyBtnVue which-settings="lan" :setting-reactive="settingReactive" :input-checked="inputOK"></saveApplyBtnVue>
  </div>
  <msgPopupVue v-model:pop="popAuthFail" mode="Logout">{{tg('authTMout')}}</msgPopupVue>
  <msgPopupVue v-model:pop="popLoadFail" mode="Alert">{{tg('pageLoadErr')}}</msgPopupVue>
</template>

<i18n lang="json">
  {
    "enUS": {
      "header": "LAN / IP settings",
      "ip": "Device IP",
      "ipMode": "Device IP Mode",
      "static": "Static",
      "ipaddr": "IP Address",
      "netmask": "Sub Netmask",
      "gwip": "Gateway IP",
      "dhcpd": "DHCP Server",
      "server": "Server",
      "dhcpStart": "DHCP Start Address",
      "dhcpEnd": "DHCP Stop Address",
      "dhcpRouter": "DHCP Router",
      "leaseTime": "DHCP lease time",
      "dns1": "DNS1",
      "dns2": "DNS2"
    },
    "zhCN": {
      "header": "网络配置",
      "ip": "设备IP地址",
      "ipMode": "IP模式",
      "static": "静态",
      "ipaddr": "IP地址",
      "netmask": "子网掩码",
      "gwip": "网关IP",
      "dhcpd": "DHCP服务器",
      "server": "服务器",
      "dhcpStart": "DHCP起始地址",
      "dhcpEnd": "DHCP停止地址",
      "dhcpRouter": "DHCP网关",
      "leaseTime": "DHCP租借时间",
      "dns1": "DNS服务器1",
      "dns2": "DNS服务器2"
    }
  }
</i18n>