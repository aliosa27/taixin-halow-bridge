<script setup lang="ts">
  import { ref, reactive, watch, computed, onBeforeMount } from 'vue';
  import { inputValidate, INPUT_CHECK, strObj } from '@/utils';
  import { useI18n } from 'vue-i18n';
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

  const deviceSettingData =
  ['wifi_ap_hide', 'wifi_chan',/* 'wifi_mcs', */'wifi_bssbw', 'wifi_beaconint', 'wifi_dtim_period', 'wifi_bss_max_idle',
    'wifi_chan_list'] as const;

  type deviceSettings = {
    [key in typeof deviceSettingData[number]]: string;
  }

  const settingReactive = ref<deviceSettings>({} as deviceSettings);

  const popAuthFail = ref<boolean>(false);
  const popLoadFail = ref<boolean>(false);
  const inputOK = ref<boolean>(true);

  const showInputErrMsg: strObj = reactive({
    beaconInt: '',
    dtim: '',
    bssMaxIdle: ''
  });

  onBeforeMount(async () => await loadPageData(
    dashboardStore.authToken,
    deviceSettingData,
    'wifiAdvanced',
    formDataStore.dataStore,
    settingReactive,
    popLoadFail,
    popAuthFail,
    logoutIn3s
    ));

  // HaLow (TXW8301): the device reports its channel plan as centre
  // frequencies in 100 kHz units ("8640,8660"); channel N = Nth entry.
  const chFreqs = computed(() => (settingReactive.value.wifi_chan_list || '')
    .split(',').filter((f) => f !== ''));
  const chListRender = computed(() => chFreqs.value.map((_, idx) => (idx + 1).toString()));
  function genChannelList(item: string) {
    const freq = parseInt(chFreqs.value[parseInt(item) - 1]) / 10;
    return item + ' (' + freq.toFixed(1) + 'MHz)';
  }

  //const mcsRender = ['0', '1', '2', '3', '4', '5', '6', '7'];

  const beaconInterval = computed({
    get() {
      return parseInt(settingReactive.value.wifi_beaconint);
    },
    set(newVal) {
      settingReactive.value.wifi_beaconint = newVal.toString();
    }
  });

  const dtimPeriod = computed({
    get() {
      return parseInt(settingReactive.value.wifi_dtim_period);
    },
    set(newVal) {
      settingReactive.value.wifi_dtim_period = newVal.toString();
    }
  });

  const bssMaxIdle = computed({
    get() {
      return parseInt(settingReactive.value.wifi_bss_max_idle);
    },
    set(newVal) {
      settingReactive.value.wifi_bss_max_idle = newVal.toString();
    }
  });

   watch(settingReactive.value, () => {
    var errs = 0;
    switch (inputValidate.enforceNumberInputLimit(settingReactive.value.wifi_beaconint, 50, 1000)) {
      case INPUT_CHECK.OK:
        showInputErrMsg.beaconInt = '';
        break
      case INPUT_CHECK.INVALID:
        showInputErrMsg.beaconInt = tg('invalidInput');
        errs++
        break
    }
    switch (inputValidate.enforceNumberInputLimit(settingReactive.value.wifi_dtim_period, 1, 255)) {
      case INPUT_CHECK.OK:
        showInputErrMsg.dtim = '';
        break
      case INPUT_CHECK.INVALID:
        showInputErrMsg.dtim = tg('invalidInput');
        errs++
        break
    }
    switch (inputValidate.enforceNumberInputLimit(settingReactive.value.wifi_bss_max_idle, 1, 1000)) {
      case INPUT_CHECK.OK:
        showInputErrMsg.bssMaxIdle = '';
        break
      case INPUT_CHECK.INVALID:
        showInputErrMsg.bssMaxIdle = tg('invalidInput');
        errs++
        break
    }
    inputOK.value = errs === 0;
  });

</script>

<template>
  <div class="header">
    <h1>{{t('header')}}</h1>
  </div>
  <div class="content">
    <form>
      <fieldset>
        <legend>{{t('header')}}</legend>
        <div class="ctrl-group">
          <label for="channel">{{t('channel')}}</label>
          <select id="channel" v-model="settingReactive.wifi_chan">
            <option value="0">{{t('auto')}}</option>
            <option v-for="item in chListRender" :value="item">{{genChannelList(item)}}</option>
          </select>
        </div>
        <!--
        <div class="ctrl-group">
          <label for="txmcs">{{t('txMCS')}}</label>
          <select id="txmcs" v-model="settingReactive.wifi_mcs">
            <option value="255">{{t('auto')}}</option>
            <option v-for="item in mcsRender" :value="item">{{item}}</option>
          </select>
        </div>
      -->
      <div class="ctrl-group">
        <label for="bssbw">{{t('bssbw')}}</label>
        <select id="bssbw" v-model="settingReactive.wifi_bssbw">
          <option value="1">1MHz</option>
          <option value="2">2MHz</option>
          <option value="4">4MHz</option>
          <option value="8">8MHz</option>
        </select>
      </div>
      <div class="ctrl-group">
        <label for="beaconInt">{{t('beaconInt')}}</label>
        <input id="beaconInt" type="number" v-model="beaconInterval" />
      </div>
      <div class="ctrl-group">
        <label for="dtimPeriod">{{t('dtim')}}</label>
        <input id="dtimPeriod" type="number" v-model="dtimPeriod" />
      </div>
      <div class="ctrl-group">
        <label for="bssMaxIdle">{{t('bssMaxIdle')}}</label>
        <input id="bssMaxIdle" type="number" v-model="bssMaxIdle" />
      </div>
    </fieldset>
  </form>
  <saveApplyBtnVue which-settings="wifiAdvanced" :setting-reactive="settingReactive" :input-checked="inputOK"></saveApplyBtnVue>
</div>
<msgPopupVue v-model:pop="popAuthFail" mode="Logout">{{tg('authTMout')}}</msgPopupVue>
<msgPopupVue v-model:pop="popLoadFail" mode="Alert">{{tg('pageLoadErr')}}</msgPopupVue>
</template>

<i18n lang="json">
  {
    "enUS": {
      "header": "Radio settings",
      "channel": "Channel",
      "txMCS": "Tx MCS",
      "bssbw": "Bandwidth",
      "auto": "AUTO",
      "beaconInt": "Beacon Interval",
      "dtim": "DTIM Period",
      "bssMaxIdle": "BSS Max Idle"
    },
    "zhCN": {
      "header": "无线高级配置",
      "channel": "信道",
      "txMCS": "发送MCS",
      "bssbw": "带宽",
      "auto": "自动",
      "beaconInt": "Beacon间隔",
      "dtim": "DTIM周期",
      "bssMaxIdle": "BSS Max Idle"
    }
  }
</i18n>