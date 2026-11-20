<script setup lang="ts">
	import { ref, onBeforeMount, onUnmounted } from 'vue';
  import { logoutIn3s } from '@/router';
  import { useDashboardStore } from '@/store';
  import { clientList, type clientInfoClass, RET } from '@/wnbCom';
  import { useI18n } from 'vue-i18n';
  import { linkRateText } from '@/utils/halowPlan';
  import { sizeFromKB } from '@/utils';
  import msgPopupVue from '@/components/msgPopup.vue';
  import { Icon } from '@vicons/utils';
  import {
    LanOutlined,
    WifiOutlined,
    NetworkWifi1BarFilled,
    NetworkWifi2BarFilled,
    NetworkWifi3BarFilled,
    NetworkWifiFilled
  } from '@vicons/material'

  const { t } = useI18n({
    inheritLocale: true,
    useScope: 'local'
  });
  const { t: tg } = useI18n({
    useScope: 'global'
  });

  const popAuthFail = ref<boolean>(false);
  const popLoadFail = ref<boolean>(false);

  const dashboardStore = useDashboardStore();

  const clientListDisplay = ref<clientInfoClass[]>([]);

  async function pollClientList() {
    const resp = await clientList(dashboardStore.authToken);
    if (resp.state === RET.OK) {
      clientListDisplay.value = [...resp.clientList];
    } else if (resp.state === RET.AUTH_FAIL) {
      popAuthFail.value = true;
      logoutIn3s();
    } else {
      popLoadFail.value = true;
    }
  }

  onBeforeMount(async() => {
    await pollClientList();
  });

  // poll every minute
  const interval = setInterval(pollClientList, 60000);

  onUnmounted(() => {
    clearInterval(interval);
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

</script>

<template>
  <table>
    <thead>
      <tr><th>{{t('clientType')}}</th><th>{{t('macAddr')}}</th><th>{{t('ipAddr')}}</th><th>{{t('signal')}}</th><th>{{t('linkRate')}}</th><th>{{t('traffic')}}</th></tr>
    </thead>
    <tbody>
      <tr v-for="item in clientListDisplay">
        <td v-if="item.type === 'Wired'">
          <Icon><LanOutlined /></Icon>
          {{t('wired')}}
        </td>
        <td v-else>
          <Icon><WifiOutlined /></Icon>
          {{t('wireless')}}
        </td>
        <td>{{item.mac}}</td>
        <td>{{item.ip}}</td>
        <td align="center" v-if="item.type === 'Wireless'">
          <Icon v-if="signalDisplay(item.signal) === 4" color="blue"><NetworkWifiFilled /></Icon>
          <Icon v-else-if="signalDisplay(item.signal) === 3" color="green"><NetworkWifi3BarFilled /></Icon>
          <Icon v-else-if="signalDisplay(item.signal) === 2" color="orange"><NetworkWifi2BarFilled /></Icon>
          <Icon v-else-if="signalDisplay(item.signal) === 1" color="red"><NetworkWifi1BarFilled /></Icon>
          {{item.signal}} dBm
        </td>
        <td align="center" v-else>{{t('wired')}}</td>
        <td v-if="item.type === 'Wireless'">{{linkRateText(item.rxMcs, item.bw) || '–'}}</td>
        <td v-else>–</td>
        <td v-if="item.type === 'Wireless' && item.txKB >= 0">TX {{sizeFromKB(item.txKB)}} · RX {{sizeFromKB(item.rxKB)}}</td>
        <td v-else>–</td>
      </tr>
      
    </tbody>
  </table>
  <msgPopupVue v-model:pop="popAuthFail" mode="Logout">{{tg('authTMout')}}</msgPopupVue>
  <msgPopupVue v-model:pop="popLoadFail" mode="Alert">{{tg('pageLoadErr')}}</msgPopupVue>
</template>

<style scoped>
  table {
    margin-top: 20px;
  }
</style>

<i18n lang="json">
 {
  "enUS": {
    "clientType": "Client Type",
    "wireless": "Wireless",
    "wired": "Wired",
    "macAddr": "MAC Address",
    "ipAddr": "IP Address",
    "signal": "Signal",
    "linkRate": "RX rate",
    "traffic": "Traffic"
  },
  "zhCN": {
    "clientType": "客户端类型",
    "wireless": "无线",
    "wired": "有线",
    "macAddr": "MAC地址",
    "ipAddr": "IP地址",
    "signal": "信号强度"
  }
}
</i18n>