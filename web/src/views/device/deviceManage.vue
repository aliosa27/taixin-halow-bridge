<script setup lang="ts">
  import { ref, reactive, watch, computed, onBeforeMount } from 'vue';
  import { useI18n } from 'vue-i18n';
  import { useDashboardStore, useFormDataStore } from '@/store';
  import { logoutIn3s } from '@/router';
  import { reboot, loadDef, loadPageData, RET } from '@/wnbCom';
  import { inputValidate, INPUT_CHECK, strObj } from '@/utils';
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

  const deviceSettingData = ['hostname', 'web_passwd', 'mac_addr'] as const;

  type deviceSettings = {
    [key in typeof deviceSettingData[number]]: string;
  }

  const settingReactive = ref<deviceSettings>({} as deviceSettings);

  // DHCP host name as sys_dhcp_hostname() (main.c) builds it: filtered name, '-', MAC bytes 3..5
  const dhcpName = computed(() => {
    const name = (settingReactive.value.hostname || '').replace(/[^A-Za-z0-9-]/g, '').replace(/^-+/, '') || 'tx';
    const mac = (settingReactive.value.mac_addr || '').toLowerCase().split(':');
    return name + '-' + (mac.length === 6 ? mac.slice(3).join('') : 'xxxxxx');
  });

  onBeforeMount(async () => loadPageData(
    dashboardStore.authToken,
    deviceSettingData,
    'deviceManage',
    formDataStore.dataStore,
    settingReactive,
    popLoadFail,
    popAuthFail,
    logoutIn3s
    ));

  const popAuthFail = ref<boolean>(false);
  const popLoadFail = ref<boolean>(false);
  const popConfirmReboot = ref<boolean>(false);
  const popConfirmLoaddef = ref<boolean>(false);
  const popActionSuccess = ref<boolean>(false);
  const popActionFail = ref<boolean>(false);
  const inputOK = ref<boolean>(true);
  const showInputErrMsg: strObj = reactive({
    hostname: '',
    macaddr: '',
    passwd: ''
  });

  async function doAction(mode: 'loadDef' | 'Reboot') {
    if (mode === 'Reboot') 
      var resp = await reboot(dashboardStore.authToken);
    else
      var resp = await loadDef(dashboardStore.authToken);

    if (resp === RET.OK) {
      popActionSuccess.value = true;
      if (mode === 'loadDef')
        for (var setting in formDataStore.dataStore) delete formDataStore.dataStore[setting];
          logoutIn3s();
      } else if (resp === RET.AUTH_FAIL) {
        popAuthFail.value = true;
        logoutIn3s();
      } else {
        popActionFail.value = true;
      }
    }

    watch(settingReactive.value, () => {
      var errs = 0;
      switch (inputValidate.isHostname(settingReactive.value.hostname)) {
      case INPUT_CHECK.OK:
        showInputErrMsg.hostname = '';
        break;
      case INPUT_CHECK.EMPTY:
        errs++;
        showInputErrMsg.hostname = tg('emptyInput');
        break;
      case INPUT_CHECK.INVALID:
        errs++;
        showInputErrMsg.hostname = tg('invalidInput');
        break;
      }
      switch (inputValidate.isMacAddr(settingReactive.value.mac_addr)) {
      case INPUT_CHECK.OK:
        showInputErrMsg.macaddr = '';
        break;
      case INPUT_CHECK.EMPTY:
        errs++;
        showInputErrMsg.macaddr = tg('emptyInput');
        break;
      case INPUT_CHECK.INVALID:
        errs++;
        showInputErrMsg.macaddr = tg('invalidInput');
        break;
      }
      switch(inputValidate.isWebPasswd(settingReactive.value.web_passwd)) {
      case INPUT_CHECK.OK:
        showInputErrMsg.passwd = '';
        break;
      case INPUT_CHECK.EMPTY:
        errs++;
        showInputErrMsg.passwd = tg('emptyInput');
        break;
      case INPUT_CHECK.INVALID:
        errs++;
        showInputErrMsg.passwd = tg('invalidInput');
        break;
      }
      if (errs > 0)
        inputOK.value = false;
      else
        inputOK.value = true;
    });

  // Prevent button in form reflesh page
    function onSubmit() {}

  </script>

  <template>
    <div class="header">
      <h1>{{t('header')}}</h1>
    </div>
    <div class="content">
      <form v-on:submit.prevent="onSubmit">
        <fieldset>
          <legend>{{t('deviceCtrl')}}</legend>
          <div class="ctrl-group">
            <label for="reboot">{{t('reboot')}}</label>
            <button id="reboot" class="btn-orange" @click="popConfirmReboot = true">{{t('btnReboot')}}</button>
          </div>
          <div class="ctrl-group">
            <label for="loaddef">{{t('loadDef')}}</label>
            <button id="loaddef" class="btn-red" @click="popConfirmLoaddef = true">{{t('btnLoadDef')}}</button>
          </div>
        </fieldset>
        <fieldset>
          <legend>{{t('deviceSettings')}}</legend>
          <div class="ctrl-group">
            <label for="hostname">{{t('hostname')}}</label>
            <input type="text" id="hostname" maxlength="7" v-model.trim="settingReactive.hostname" />
            <span class="msg-warn">{{showInputErrMsg.hostname}}</span>
            <span class="msg-help">{{(settingReactive.hostname || '').length}} / 7 {{t('chars')}}</span>
          </div>
          <div class="ctrl-group">
            <label for="dhcpname">{{t('dhcpName')}}</label>
            <input type="text" id="dhcpname" :value="dhcpName" readonly />
            <span class="msg-help">{{t('dhcpNameHelp')}}</span>
          </div>
          <div class="ctrl-group">
            <label for="macaddr">{{t('macAddr')}}</label>
            <input type="text" id="macaddr" v-model.trim="settingReactive.mac_addr" />
            <span class="msg-warn">{{showInputErrMsg.macaddr}}</span>
          </div>
          <div class="ctrl-group">
            <label for="webpasswd">{{t('webpasswd')}}</label>
            <input type="text" id="webpasswd" v-model.trim="settingReactive.web_passwd" />
            <span class="msg-warn">{{showInputErrMsg.passwd}}</span>
          </div>
        </fieldset>
      </form>
      <saveApplyBtnVue which-settings="deviceManage" v-model:setting-reactive="settingReactive" :input-checked="inputOK"></saveApplyBtnVue>
    </div>
    <msgPopupVue v-model:pop="popAuthFail" mode="Logout">{{tg('authTMout')}}</msgPopupVue>
    <msgPopupVue v-model:pop="popLoadFail" mode="Alert">{{tg('pageLoadErr')}}</msgPopupVue>
    <msgPopupVue v-model:pop="popConfirmReboot" mode="Confirm" @confirm-click="doAction('Reboot')">{{t('confirmReboot')}}</msgPopupVue>
    <msgPopupVue v-model:pop="popConfirmLoaddef" mode="Confirm" @confirm-click="doAction('loadDef')">{{t('confirmLoadDef')}}</msgPopupVue>
    <msgPopupVue v-model:pop="popActionSuccess" mode="Logout">{{t('actionSuccess')}}</msgPopupVue>
    <msgPopupVue v-model:pop="popActionFail" mode="Alert">{{t('actionFail')}}</msgPopupVue>
  </template>

  <i18n lang="json">
    {
      "enUS": {
        "header": "Device Management",
        "deviceCtrl": "Device control",
        "deviceSettings": "Device settings",
        "reboot": "Reboot Device",
        "loadDef": "Load Factory Default",
        "hostname": "Device name",
        "chars": "characters",
        "dhcpName": "Network host name",
        "dhcpNameHelp": "Sent to your DHCP server: the device name (letters, digits and '-') plus the last 6 hex digits of the MAC address.",
        "macAddr": "MAC address",
        "webpasswd": "Login password",
        "confirmLoadDef": "Are you sure you want to reset the device to factory defaults?",
        "confirmReboot": "Are you sure you want to reboot the device?",
        "actionSuccess": "Operation successful, Logout in 3s",
        "actionFail": "Operation fail, please try again!!",
        "btnReboot": "Reboot",
        "btnLoadDef": "Load Default"
      },
      "zhCN": {
        "header": "设备管理",
        "deviceCtrl": "设备控制",
        "deviceSettings": "设备设置",
        "reboot": "重启设备",
        "loadDef": "恢复出厂配置",
        "hostname": "设备名称",
        "macAddr": "设备MAC地址",
        "webpasswd": "登录密码",
        "confirmLoadDef": "您确认要恢复出厂配置?",
        "confirmReboot": "您确认要重启设备?",
        "actionSuccess": "执行成功, 3秒后回到登录页面",
        "actionFail": "执行失败，请重试!!",
        "btnReboot": "重启",
        "btnLoadDef": "恢复"  
      }
    }
  </i18n>