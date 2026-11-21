<script setup lang="ts">
  import { ref, computed, reactive, onBeforeMount, onUnmounted } from 'vue';
  import { useI18n } from 'vue-i18n';
  import { getSettings, connStat, RET, type connStatClass } from '@/wnbCom';
  import { WIFI_MODE, KEY_MGMT, SELECT, wifiModeKey, sizeFromKB } from '@/utils';
  import { linkRateText } from '@/utils/halowPlan';
  import { useDashboardStore } from '@/store';
  import { logoutIn3s } from '@/router';
  import msgPopupVue from '@/components/msgPopup.vue';
  import { Icon } from '@vicons/utils';
  import {
    RouterOutlined,
    SettingsEthernetOutlined,
    WifiOutlined,
    MemoryOutlined,
    LanOutlined
  } from '@vicons/material';

  const { t } = useI18n({
    inheritLocale: true,
    useScope: 'local'
  });
  const { t: tg } = useI18n({
    useScope: 'global'
  });

  const pollData = ['s_ipaddr', 's_netmask', 's_gw_ip', 'uptime', 'fwver', 'buildtime', 'mac_addr', 'dhcp_name',
    'heap_free', 'heap_total', 'skb_free', 'skb_total', 'fw_size', 'fw_max', 'eth_state',
    'dhcpc_en', 'wifi_mode', 'wifi_wnb', 'wifi_ssid', 'wifi_r_ssid', 'wifi_keymgmt'] as const;

  type poll = {
    [key in typeof pollData[number]]: string
  }
  const dashboardStore = useDashboardStore();

  const statusPoll = reactive({} as poll);

  // Firmware build time comes as C's __DATE__ __TIME__: "Nov 18 2026 18:05:12" -> "2026-11-18 18:05"
  const builtAt = computed(() => {
    const m = /^(\w{3}) +(\d+) (\d{4}) (\d\d:\d\d)/.exec(statusPoll.buildtime || '');
    if (!m) return statusPoll.buildtime || '';
    const mon = 'JanFebMarAprMayJunJulAugSepOctNovDec'.indexOf(m[1]) / 3 + 1;
    return m[3] + '-' + String(mon).padStart(2, '0') + '-' + m[2].padStart(2, '0') + ' ' + m[4];
  });
  const connStatDisplay = reactive({} as connStatClass);

  const popAuthFail = ref<boolean>(false);
  const popLoadFail = ref<boolean>(false);

  async function getPollData() {
    const resp = await getSettings(pollData, dashboardStore.authToken);
    if (resp.state === RET.OK) {
      Object.assign(statusPoll, resp.settings);
    } else if (resp.state === RET.AUTH_FAIL) {
      popAuthFail.value = true;
      logoutIn3s();
    } else {
      popLoadFail.value = true;
    }
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

  onBeforeMount(async () => {
    await getPollData();
    await pollConnState();
  });

  const interval = setInterval(async () => {
    await getPollData();
    await pollConnState();
  }, 10000);

  onUnmounted(() => {
    clearInterval(interval);
  });

  const ipMode = computed(() => {
    if (statusPoll.dhcpc_en === SELECT.ENABLE || statusPoll.wifi_mode === WIFI_MODE.APSTA)
      return t('ipDHCP');
    else
      return t('ipStatic');
  });

  const wifiOpmode = computed(() => {
    const key = wifiModeKey(statusPoll.wifi_mode, statusPoll.wifi_wnb);
    return key ? tg(key) : '';
  });

  // AP side: "connected" means at least one station is associated
  const connLabel = computed(() => {
    if (statusPoll.wifi_mode === WIFI_MODE.AP)
      return connStatDisplay.connected ? t('hasClients') : t('noClients');
    return connStatDisplay.connected ? t('connected') : t('disconnected');
  });

  const showSignal = computed(() => connStatDisplay.connected && statusPoll.wifi_mode !== WIFI_MODE.AP);

  // Memory: free bytes of total, for the Memory card
  function kb(v: string): string {
    return (parseInt(v) / 1024).toFixed(1) + ' KB';
  }
  function pct(free: string, total: string): number {
    const f = parseInt(free), tot = parseInt(total);
    return tot > 0 ? Math.round(100 * (tot - f) / tot) : 0;
  }
  const showMem = computed(() => !!statusPoll.heap_total && statusPoll.heap_total !== '0');

  // Station side: PHY rate of frames received from the AP (firmware extension in conn_state).
  // TX is not shown: the lmac has no per-station TX MCS (LMAC_IOCTL_GET_STA_TX_MCS unimplemented), it reads 0.
  const linkRate = computed(() => linkRateText(connStatDisplay.rxMcs, connStatDisplay.bw));
  const wifiTraffic = computed(() => connStatDisplay.txKB >= 0
    ? 'TX ' + sizeFromKB(connStatDisplay.txKB) + ' · RX ' + sizeFromKB(connStatDisplay.rxKB) : '');

  // Ethernet port (firmware extension eth_state): "link,speed,duplex,tx,rx,rx_err,rx_ovf", counts are frames
  const eth = computed(() => {
    const f = (statusPoll.eth_state || '').split(',').map(Number);
    if (f.length < 7 || f.some(isNaN)) return null;
    return { up: f[0] === 1, speed: f[1], full: f[2] === 1, tx: f[3], rx: f[4], err: f[5], ovf: f[6] };
  });

  // Frame counters run into the millions; keep them short so the row fits the card: 429634 -> 430k, 2058712 -> 2.06M
  function count(n: number): string {
    if (n < 10000) return String(n);
    if (n < 1e6) return Math.round(n / 1e3) + 'k';
    if (n < 1e9) return (n / 1e6).toFixed(n < 1e8 ? 2 : 0) + 'M';
    return (n / 1e9).toFixed(2) + 'G';
  }

  const signalStrength = computed(() => {
    return Math.min(Math.max(2 * (connStatDisplay.rssi + 100), 0), 100);
  });

</script>

<template>
  <div class="cards">
    <section class="card">
      <h3><Icon><WifiOutlined /></Icon>{{t('WiFiInfo')}}</h3>
      <dl class="kv">
        <dt>{{t('opMode')}}</dt><dd>{{wifiOpmode}}</dd>
        <dt>{{t('state')}}</dt><dd><span class="chip" :class="{ ok: connStatDisplay.connected }">{{connLabel}}</span></dd>
        <template v-if="connStatDisplay.staCnt >= 0">
          <dt>{{t('stations')}}</dt><dd>{{connStatDisplay.staCnt}}</dd>
        </template>
        <dt>SSID</dt><dd>{{statusPoll.wifi_ssid}}</dd>
        <template v-if="statusPoll.wifi_mode === WIFI_MODE.APSTA">
          <dt>{{t('rssid')}}</dt><dd>{{statusPoll.wifi_r_ssid}}</dd>
        </template>
        <dt>{{t('keymgmt')}}</dt><dd>{{statusPoll.wifi_keymgmt === KEY_MGMT.PSK ? 'WPA-PSK' : t('open')}}</dd>
        <template v-if="connStatDisplay.channel">
          <dt>{{t('channel')}}</dt><dd>{{connStatDisplay.channel}}</dd>
        </template>
        <template v-if="showSignal">
          <dt>{{t('signal')}}</dt>
          <dd><progress :value="signalStrength" max="100"></progress> {{connStatDisplay.rssi}} dBm</dd>
          <template v-if="linkRate">
            <dt>{{t('linkRate')}}</dt><dd>{{linkRate}}</dd>
          </template>
          <template v-if="wifiTraffic">
            <dt>{{t('traffic')}}</dt><dd>{{wifiTraffic}}</dd>
          </template>
        </template>
      </dl>
    </section>
    <section class="card" v-if="eth">
      <h3><Icon><LanOutlined /></Icon>{{t('ethernet')}}</h3>
      <dl class="kv">
        <dt>{{t('state')}}</dt><dd><span class="chip" :class="{ ok: eth.up }">{{eth.up ? t('ethUp') : t('ethDown')}}</span></dd>
        <template v-if="eth.up">
          <dt>{{t('speed')}}</dt><dd>{{eth.speed}} Mbps {{eth.full ? t('full') : t('half')}}</dd>
        </template>
        <dt>{{t('frames')}}</dt><dd :title="'TX ' + eth.tx + ' · RX ' + eth.rx">TX {{count(eth.tx)}} · RX {{count(eth.rx)}}</dd>
        <dt>{{t('errors')}}</dt><dd>{{eth.err}} RX · {{eth.ovf}} {{t('overflow')}}</dd>
      </dl>
    </section>
    <section class="card">
      <h3><Icon><SettingsEthernetOutlined /></Icon>{{t('LAN')}}</h3>
      <dl class="kv">
        <dt>{{t('ipMode')}}</dt><dd>{{ipMode}}</dd>
        <template v-if="statusPoll.dhcp_name">
          <dt>{{t('hostname')}}</dt><dd>{{statusPoll.dhcp_name}}</dd>
        </template>
        <dt>{{t('ipAddr')}}</dt><dd>{{statusPoll.s_ipaddr}}</dd>
        <dt>{{t('netmask')}}</dt><dd>{{statusPoll.s_netmask}}</dd>
        <dt>{{t('gwIP')}}</dt><dd>{{statusPoll.s_gw_ip}}</dd>
      </dl>
    </section>
    <section class="card">
      <h3><Icon><RouterOutlined /></Icon>{{t('deviceInfo')}}</h3>
      <dl class="kv">
        <dt>{{t('fwVer')}}</dt><dd>{{statusPoll.fwver}}</dd>
        <template v-if="builtAt"><dt>{{t('built')}}</dt><dd>{{builtAt}}</dd></template>
        <dt>{{t('uptime')}}</dt><dd>{{statusPoll.uptime}}</dd>
        <dt>{{t('macAddr')}}</dt><dd>{{statusPoll.mac_addr}}</dd>
      </dl>
    </section>
    <section class="card" v-if="showMem">
      <h3><Icon><MemoryOutlined /></Icon>{{t('memory')}}</h3>
      <dl class="kv">
        <dt>{{t('heap')}}</dt>
        <dd>{{kb(statusPoll.heap_free)}} {{t('freeOf')}} {{kb(statusPoll.heap_total)}}</dd>
        <dd class="bar"><progress :value="pct(statusPoll.heap_free, statusPoll.heap_total)" max="100"></progress></dd>
        <dt>{{t('skb')}}</dt>
        <dd>{{kb(statusPoll.skb_free)}} {{t('freeOf')}} {{kb(statusPoll.skb_total)}}</dd>
        <dd class="bar"><progress :value="pct(statusPoll.skb_free, statusPoll.skb_total)" max="100"></progress></dd>
        <template v-if="statusPoll.fw_max && statusPoll.fw_max !== '0'">
          <dt>{{t('flash')}}</dt>
          <dd>{{kb(String(parseInt(statusPoll.fw_max) - parseInt(statusPoll.fw_size)))}} {{t('freeOf')}} {{kb(statusPoll.fw_max)}}</dd>
          <dd class="bar"><progress :value="pct(String(parseInt(statusPoll.fw_max) - parseInt(statusPoll.fw_size)), statusPoll.fw_max)" max="100"></progress></dd>
        </template>
      </dl>
      <p class="hint">{{t('memHint')}}</p>
    </section>
  </div>
  <msgPopupVue v-model:pop="popAuthFail" mode="Logout">{{tg('authTMout')}}</msgPopupVue>
  <msgPopupVue v-model:pop="popLoadFail" mode="Alert">{{tg('pageLoadErr')}}</msgPopupVue>
</template>

<i18n lang="json">
  {
    "enUS": {
      "deviceInfo": "Device",
      "fwVer": "Firmware",
      "built": "Built",
      "uptime": "Uptime",
      "macAddr": "MAC address",
      "LAN": "Network",
      "ipMode": "IP mode",
      "hostname": "Host name",
      "ipAddr": "IP address",
      "netmask": "Netmask",
      "gwIP": "Gateway",
      "WiFiInfo": "Wireless",
      "opMode": "Mode",
      "state": "Link",
      "rssid": "Repeater SSID",
      "keymgmt": "Security",
      "open": "Open",
      "channel": "Channel",
      "ipStatic": "Static",
      "ipDHCP": "DHCP",
      "connected": "Connected",
      "disconnected": "Not connected",
      "hasClients": "Stations connected",
      "noClients": "No stations",
      "stations": "Stations",
      "signal": "Signal",
      "linkRate": "RX rate",
      "traffic": "Traffic",
      "ethernet": "Ethernet",
      "ethUp": "Up",
      "ethDown": "No cable",
      "speed": "Speed",
      "full": "full duplex",
      "half": "half duplex",
      "frames": "Frames",
      "errors": "Errors",
      "overflow": "overflow",
      "memory": "Memory",
      "heap": "System heap",
      "skb": "Packet buffers",
      "flash": "Firmware slot",
      "freeOf": "free of",
      "memHint": "Bars show the share in use. Refreshes every 10 s."
    }
  }
</i18n>

<style scoped>
  .kv .bar { grid-column: 1 / -1; text-align: left; margin-top: -.2rem; }
  .kv .bar progress { max-width: none; }
  .hint { margin: .8rem 0 0; font-size: .82em; color: var(--muted); }
</style>
