<script setup lang="ts">
  import { computed, ref, watch, onMounted, onUnmounted } from 'vue';
  import { useI18n } from 'vue-i18n';
  import { fetchScanList, type apInfoClass, RET } from '@/wnbCom';
  import { useDashboardStore } from '@/store';
  import { Icon } from '@vicons/utils';
  import {
    WifiFindOutlined,
    ExitToAppOutlined,
    NetworkWifi1BarFilled,
    NetworkWifi2BarFilled,
    NetworkWifi3BarFilled,
    NetworkWifiFilled,
    EnhancedEncryptionOutlined,
    NoEncryptionOutlined
  } from '@vicons/material';

  const props = defineProps<{
    pop: boolean
  }>();

  const emits = defineEmits<{
    (event: 'update:pop', value: boolean): void
    (event: 'selected', value: apInfoClass): void
  }>();

  const popScanner = computed({
    get() {
      return props.pop;
    },
    set(newVal) {
      emits('update:pop', newVal);
    }
  });

  const { t } = useI18n({
    inheritLocale: true,
    useScope: 'local'
  });

  const dashboardStore = useDashboardStore();
  const scanList = ref<apInfoClass[]>([]);
  const scanning = ref<boolean>(false);

  async function scanAP() {
    scanning.value = true;
    const resp = await fetchScanList(dashboardStore.authToken);
    if (resp.state === RET.OK) {
      scanList.value = [...resp.scanList];
    }
    scanning.value = false;
  }

  function emitSelected(item: apInfoClass) {
    emits('selected', item);
    popScanner.value = false;
  }

  watch(() => props.pop, () => {
    if (props.pop === true && scanList.value.length === 0)
      scanAP();
  });

  function signalDisplay(rssi: number) {
    if(rssi > -50)
      return 4;
    else if (rssi > -55)
      return 3;
    else if (rssi > -60)
      return 2;
    else
      return 1;
  }

  const mqQuery = window.matchMedia('(min-width: 576px), (orientation: landscape)');
  const bigScreen = ref(mqQuery.matches);
  const onMqChange = (e: MediaQueryListEvent) => { bigScreen.value = e.matches; };
  onMounted(() => mqQuery.addEventListener('change', onMqChange));
  onUnmounted(() => mqQuery.removeEventListener('change', onMqChange));
  
</script>

<template>
  <Transition name="bounce">
    <div v-if="popScanner" id="modal" class="modal">
      <div class="modal-content wide">
        <div class="modal-header" style="display: flex; align-items: center; gap: 12px;">
          <div><h2>{{t('header')}}</h2></div>
          <div v-if="scanning" class="loader"></div>
        </div>
        <div class="modal-body">
          <table>
            <thead>
              <tr>
                <th v-if="bigScreen">{{t('freq')}}</th>
                <th></th>
                <th>SSID</th>
                <th v-if="bigScreen">BSSID</th>
                <th></th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="item in scanList" @click="emitSelected(item)">
                <td v-if="bigScreen">{{item.freq + 'Mhz'}}</td>
                <td align="center">
                  <Icon v-if="signalDisplay(item.signal) === 4" color="blue"><NetworkWifiFilled /></Icon>
                  <Icon v-else-if="signalDisplay(item.signal) === 3" color="green"><NetworkWifi3BarFilled /></Icon>
                  <Icon v-else-if="signalDisplay(item.signal) === 2" color="orange"><NetworkWifi2BarFilled /></Icon>
                  <Icon v-else color="red"><NetworkWifi1BarFilled /></Icon>
                </td>
                <td>{{item.ssid}}</td>
                <td v-if="bigScreen">{{item.bssid}}</td>
                <td align="center">
                  <Icon v-if="item.encrypt" color="green"><EnhancedEncryptionOutlined /></Icon>
                  <Icon v-else color="red"><NoEncryptionOutlined /></Icon>
                </td> 
              </tr>
            </tbody>
          </table>
        </div>
        <div class="modal-footer">
          <div class="btn-popup">
            <button class="btn-blue" :disabled="scanning" @click="scanAP">
              <Icon><WifiFindOutlined /></Icon>
              {{t('btnReScan')}}
            </button>
            <button @click="popScanner = false">
              <Icon><ExitToAppOutlined /></Icon>
              {{t('bntQuit')}}
            </button>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<style scoped>
  table tr:hover {
    background: var(--subtle);
    cursor: pointer;
  }
</style>

<i18n lang="json">
  {
    "enUS": {
      "header": "WiFi Scanner",
      "freq": "Frequency",
      "signal": "Signal",
      "encryption": "Encryption",
      "btnReScan": "ReScan",
      "bntQuit": "Quit"
    },
    "zhCN": {
      "header": "热点扫描",
      "freq": "频率",
      "signal": "信号强度",
      "encryption": "加密模式", 
      "btnReScan": "重新扫描",
      "bntQuit": "退出"
    }
  }
</i18n>