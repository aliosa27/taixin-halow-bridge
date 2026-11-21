<script setup lang="ts">
  import { ref, computed } from 'vue';
  import { useI18n } from 'vue-i18n';
  import { isAuthenticated } from '@/router';
  import { useRouter } from 'vue-router';
  import { useDashboardStore, useFormDataStore } from '@/store';
  import { applySettings, RET } from '@/wnbCom';
  import { objectDiff, strObj } from '@/utils';
  import msgPopupVue from '@/components/msgPopup.vue';
  import { Icon } from '@vicons/utils';
  import { SaveAltOutlined, ClearOutlined } from '@vicons/material';

  const props = defineProps<{
    settingReactive: any,
    inputChecked: boolean,
    whichSettings: string
  }>();
  const emits = defineEmits<{
    (e: 'update:settingReactive', value: any): void
  }>();

  const settingToApply = computed({
    get() {
      return props.settingReactive;
    },
    set(newVal) {
      emits('update:settingReactive', newVal);
    }
  });

  const { t } = useI18n({
    inheritLocale: true,
    useScope: 'local'
  });
  const { t: tg } = useI18n({
    useScope: 'global'
  });

  const toRestartWiFiList =
  ['wifi_mode', 'wifi_wnb', 'wifi_keymgmt', 'wifi_ssid', 'wifi_passwd', 'wifi_r_ssid', 'wifi_r_keymgmt', 'wifi_r_passwd', 'wifi_bssbw', 'wifi_chan', 'wifi_chan_list'];
  const toConfirmApplyValue = 'web_passwd';

  const dashboardStore = useDashboardStore();
  const formDataStore = useFormDataStore();
  const router = useRouter();

  const popMsgSuccess = ref<boolean>(false);
  const popMsgFail = ref<boolean>(false);
  const popMsgAuthFail = ref<boolean>(false);
  const popInputErr = ref<boolean>(false);
  const popConfirmPasswdChange = ref<boolean>(false);

  var diffConfig: strObj;
  var toConfirmApply: boolean = false;

  function logoutIn3s() {
    setTimeout(() => {
      isAuthenticated.value = false;
      router.push({ name: 'Login' });
    }, 3000);
  }

  async function saveApply(diff: strObj) {
    const resp = await applySettings(diff, dashboardStore.authToken);
    if (resp.state === RET.OK) {
      popMsgSuccess.value = true;
      Object.assign(formDataStore.dataStore[props.whichSettings], diff);
      if (toConfirmApply) {
        // Clear the stored authToken
        localStorage.clear();
        logoutIn3s();
      }
    } else if (resp.state === RET.AUTH_FAIL) {
      popMsgAuthFail.value = true;
      logoutIn3s();
    } else {
      popMsgFail.value = true;
    }
  }

  function saveApplyAction(mode: 'saveApply' | 'reset') {
    switch(mode) {
    case 'saveApply':
      var toRestartWiFi = false;
      toConfirmApply = false;
      if (props.inputChecked) {
        diffConfig = objectDiff(formDataStore.dataStore[props.whichSettings] as unknown as strObj, settingToApply.value);
        if (dashboardStore.deviceConfig.cfgInitFlag === false) {
          Object.assign(diffConfig, { cfg_init: '1' });
          dashboardStore.deviceConfig.cfgInitFlag = true;
        }
        // Check apply setting type, to determine wherever to restart wireless
        // also check if apply confirmation is require
        Object.keys(diffConfig).forEach((key) => {
          if (toRestartWiFiList.includes(key))
            toRestartWiFi = true;
          if (key === toConfirmApplyValue)
            toConfirmApply = true;
        });
        if (toRestartWiFi)
          Object.assign(diffConfig, { savecfg: '', applycfg: '1' });
        else
          Object.assign(diffConfig, { savecfg: '', applycfg: '0' });
        if (toConfirmApply) {
          popConfirmPasswdChange.value = true;
        } else {
          saveApply(diffConfig);
        }
      } else {
        popInputErr.value = true;
      }
      break;
    case 'reset':
      Object.assign(settingToApply.value, formDataStore.dataStore[props.whichSettings]);
      break;
    }
  }

  const msgSuccess = computed(() => {
    if (toConfirmApply)
      return tg('applySuccess') + t('logout');
    else
      return tg('applySuccess');
  });

</script>

<template>
  <div class="btn-group">
    <button class="btn-blue" @click="saveApplyAction('saveApply')">
      <Icon><SaveAltOutlined /></Icon>
      {{t('saveApply')}}
    </button>
    <button @click="saveApplyAction('reset')">
      <Icon><ClearOutlined /></Icon>
      {{t('reset')}}
    </button>
  </div>
  <msgPopupVue v-model:pop="popMsgSuccess" mode="Alert">{{msgSuccess}}</msgPopupVue>
  <msgPopupVue v-model:pop="popMsgFail" mode="Alert">{{tg('applyFail')}}</msgPopupVue>
  <msgPopupVue v-model:pop="popInputErr" mode="Alert">{{t('errInForm')}}</msgPopupVue>
  <msgPopupVue v-model:pop="popMsgAuthFail" mode="Alert">{{tg('authTMout')}}</msgPopupVue>
  <msgPopupVue v-model:pop="popConfirmPasswdChange" mode="Confirm" @confirm-click="saveApply(diffConfig)">{{t('confirmPasswdChange')}}</msgPopupVue>
</template>

<i18n lang="json">
  {  
    "enUS": {
      "saveApply": "Save & apply",
      "reset": "Reset",
      "errInForm": "Some fields are invalid. Please check the form.",
      "confirmPasswdChange": "Are you sure you want to change the password?",
      "logout": " Returning to the login page…"
    },
    "zhCN": {
      "saveApply": "保存&应用",
      "reset": "恢复",
      "errInForm": "配置有误，请重新检查!!",
      "confirmPasswdChange": "你确认要修改登入密码?",
      "logout": "3秒后回到登录页面!!"
    }
  }
</i18n>