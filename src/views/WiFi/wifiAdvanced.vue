<script setup lang="ts">
  import { ref, reactive, watch, computed, onBeforeMount } from 'vue';
  import { inputValidate, INPUT_CHECK, strObj } from '@/utils';
  import { type Region, PLANS, MAX_LIST, planFreqs, deviceList, inferRegion, regionsFor, mhz } from '@/utils/halowPlan';
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
    'wifi_chan_list', 'module_type'] as const;

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

  // HaLow (TXW8301): the device stores its channel list as centre frequencies
  // in 100 kHz units ("8640,8660"), and wifi_chan as a 1-based index into it
  // (0 = auto). Here the user picks region + bandwidth + channel, and the list
  // and index are derived from the region's 802.11ah plan (utils/halowPlan.ts).
  // "Custom" keeps a hand-entered list, as on older firmware.
  const listFreqs = computed(() => (settingReactive.value.wifi_chan_list || '')
    .split(',').filter((f) => f !== '').map(Number));
  const bw = computed(() => parseInt(settingReactive.value.wifi_bssbw) || 0);
  const regions = computed(() => regionsFor(parseInt(settingReactive.value.module_type)));
  const region = ref<Region>('custom');
  const selFreq = ref<number>(0);   // chosen channel's centre, 0 = auto
  const plan = computed(() => planFreqs(region.value, bw.value));
  const channelOpts = computed(() => region.value === 'custom' ? listFreqs.value : plan.value);
  const bwAllowed = (b: number) => region.value === 'custom' || planFreqs(region.value, b).length > 0;
  const regionName = (r: Region) => r === 'custom' ? t('custom') : PLANS[r].name;

  // first load: work out region and selected frequency from what the device has
  let inited = false;
  watch(() => settingReactive.value.wifi_chan_list, (v) => {
    if (inited || v === undefined) return;
    const idx = parseInt(settingReactive.value.wifi_chan) || 0;
    selFreq.value = idx > 0 ? (listFreqs.value[idx - 1] || 0) : 0;
    region.value = inferRegion(listFreqs.value, bw.value);
    inited = true;
  }, { immediate: true });

  // keep wifi_chan_list / wifi_chan in step with region, bandwidth and channel
  watch([region, bw, selFreq, listFreqs], () => {
    if (!inited) return;
    if (region.value !== 'custom') {
      const p = plan.value;
      if (p.length === 0) {     // width not allowed in this region: use the widest one that is
        settingReactive.value.wifi_bssbw = String([8, 4, 2, 1].find(bwAllowed));
        return;
      }
      if (selFreq.value && !p.includes(selFreq.value)) {   // new width: nearest channel of the new plan
        const s = selFreq.value;
        selFreq.value = p.reduce((a, b) => Math.abs(b - s) < Math.abs(a - s) ? b : a);
        return;
      }
      const list = deviceList(p, selFreq.value).join(',');
      if (settingReactive.value.wifi_chan_list !== list) settingReactive.value.wifi_chan_list = list;
    }
    const i = listFreqs.value.indexOf(selFreq.value);
    if (selFreq.value && i < 0) {
      selFreq.value = 0;
      return;
    }
    settingReactive.value.wifi_chan = String(selFreq.value ? i + 1 : 0);
  });

  const planNote = computed(() => {
    if (region.value === 'custom' || !plan.value.length) return '';
    const p = plan.value, l = listFreqs.value;
    let s = p.length + ' ' + t('channelsOf') + ' ' + bw.value + ' MHz, ' + mhz(p[0]) + '–' + mhz(p[p.length - 1]) + ' MHz.';
    if (p.length > MAX_LIST && l.length) s += ' ' + t('window') + ' ' + mhz(l[0]) + '–' + mhz(l[l.length - 1]) + ' MHz.';
    return s;
  });

  // Custom: hand-entered list, shown in MHz ("864.0, 866.0")
  const freqText = ref<string>('');
  const freqErr = ref<string>('');
  watch(listFreqs, (v) => {
    freqText.value = v.map(mhz).join(', ');
  }, { immediate: true });
  function freqChanged() {
    const parts = freqText.value.split(/[\s,;]+/).filter((f) => f !== '');
    const vals = parts.map((p) => Math.round(parseFloat(p) * 10));
    if (parts.length === 0 || parts.length > MAX_LIST || vals.some((v, i) => !/^\d{3}(\.\d)?$/.test(parts[i]) || v < 7000 || v > 9500)) {
      freqErr.value = t('freqInvalid');
      return;
    }
    freqErr.value = '';
    settingReactive.value.wifi_chan_list = vals.join(',');
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
    if (region.value === 'custom' && freqErr.value) errs++;
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
          <label for="region">{{t('region')}}</label>
          <select id="region" v-model="region">
            <option v-for="r in regions" :key="r" :value="r">{{regionName(r)}}</option>
          </select>
        </div>
        <div class="ctrl-group">
          <label for="bssbw">{{t('bssbw')}}</label>
          <select id="bssbw" v-model="settingReactive.wifi_bssbw">
            <option v-for="b in [1, 2, 4, 8]" :key="b" :value="String(b)" :disabled="!bwAllowed(b)">{{b}} MHz</option>
          </select>
        </div>
        <div class="ctrl-group" v-if="region === 'custom'">
          <label for="freqlist">{{t('freqList')}}</label>
          <input type="text" id="freqlist" class="wide" v-model.lazy="freqText" @change="freqChanged" />
          <span class="msg-warn">{{freqErr}}</span>
          <span class="msg-help">{{t('freqHelp')}}</span>
        </div>
        <div class="ctrl-group">
          <label for="channel">{{t('channel')}}</label>
          <select id="channel" v-model.number="selFreq">
            <option :value="0">{{t('auto')}}</option>
            <option v-for="(f, i) in channelOpts" :key="f" :value="f">{{i + 1}} ({{mhz(f)}} MHz)</option>
          </select>
          <span class="msg-help" v-if="planNote">{{planNote}}</span>
          <span class="msg-help">{{t('chanHelp')}}</span>
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
      "region": "Region",
      "custom": "Custom frequencies",
      "channelsOf": "channels of",
      "window": "The device holds 16; it uses the 16 around the selected channel:",
      "chanHelp": "Use the same region and bandwidth on both ends. A station only searches the channels listed here, and on Auto it searches all of them.",
      "freqList": "Frequencies (MHz)",
      "freqHelp": "Centre frequencies, comma separated, up to 16 (AT+CHAN_LIST).",
      "freqInvalid": "Enter 1 to 16 frequencies in MHz, e.g. 864.0, 866.0",
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