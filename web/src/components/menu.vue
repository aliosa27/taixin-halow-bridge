<script setup lang="ts">
  import { ref, computed } from 'vue';
  import { RouterLink } from 'vue-router';
  import { isAuthenticated } from '@/router';
  import { useRouter } from 'vue-router';
  import { useDashboardStore } from '@/store';
  import { WIFI_MODE } from '@/utils';
  import { useI18n } from 'vue-i18n';
  import msgPopupVue from './msgPopup.vue';
  import { Icon } from '@vicons/utils';
  import{
    DashboardOutlined,
    SettingsEthernetOutlined,
    WifiOutlined,
    TuneOutlined,
    SettingsApplicationsOutlined,
    UpgradeOutlined,
    BugReportOutlined,
    InfoOutlined,
    LogOutOutlined
  } from '@vicons/material';

  defineProps<{
    menuToggle: boolean
  }>()

  const { t } = useI18n({
    inheritLocale: true,
    useScope: 'local'
  });

  const router = useRouter();
  const popConfirmLogout = ref<boolean>(false);
  const dashboardStore = useDashboardStore();

  const showLAN = computed(() => {
    if (dashboardStore.deviceConfig.fixWiFiMode === true && dashboardStore.deviceConfig.wifiMode === WIFI_MODE.APSTA)
      return false;
    else
      return true;
  });

  function logout() {
    isAuthenticated.value = false;
    router.push({ name: 'Login' });
    localStorage.clear();
  }

</script>

<template>
  <nav id="menu">
    <div class="menu">
      <div class="brand">HaLow Bridge<small>{{t('subtitle')}}</small></div>
      <ul class="menu-list">
        <li class="menu-item">
          <RouterLink class="menu-link menu-heading" :to="{ name: 'statusOverview' }">
            <Icon><DashboardOutlined /></Icon>
            {{t('Status')}}
          </RouterLink>
        </li>
        <li class="menu-group">{{t('network')}}</li>
        <li class="menu-item">
          <RouterLink class="menu-link" :to="{ name: 'WiFiBasic' }">
            <Icon><WifiOutlined /></Icon>
            {{t('WiFiBasic')}}
          </RouterLink>
        </li>
        <li class="menu-item" v-if="!dashboardStore.deviceConfig.noAdvWebCfg">
          <RouterLink class="menu-link" :to="{ name: 'WiFiAdvanced'}">
            <Icon><TuneOutlined /></Icon>
            {{t('WiFiAdvanced')}}
          </RouterLink>
        </li>
        <li class="menu-item" v-if="showLAN">
          <RouterLink class="menu-link" :to="{ name: 'LAN' }">
            <Icon><SettingsEthernetOutlined /></Icon>
            {{t('LAN')}}
          </RouterLink>
        </li>
        <li class="menu-group">{{t('system')}}</li>
        <li class="menu-item">
          <RouterLink class="menu-link" :to="{ name: 'DeviceManagement' }">
            <Icon><SettingsApplicationsOutlined /></Icon>
            {{t('DeviceManagement')}}
          </RouterLink>
        </li>
        <li class="menu-item">
          <RouterLink class="menu-link" :to="{ name: 'DeviceUpgrade' }">
            <Icon><UpgradeOutlined /></Icon>
            {{t('DeviceUpgrade')}}
          </RouterLink>
        </li>
        <li class="menu-item">
          <RouterLink class="menu-link" :to="{ name: 'Debug' }">
            <Icon><BugReportOutlined /></Icon>
            {{t('Debug')}}
          </RouterLink>
        </li>
        <li class="menu-item">
          <RouterLink class="menu-link" :to="{ name: 'About' }">
            <Icon><InfoOutlined /></Icon>
            {{t('About')}}
          </RouterLink>
        </li>
        <li class="menu-item menu-foot" v-if="!dashboardStore.deviceConfig.noWebAuth">
          <button type="button" @click="popConfirmLogout = true">
            <Icon><LogOutOutlined /></Icon>
            {{t('Logout')}}
          </button>
        </li>
      </ul>
    </div>
  </nav>
  <msgPopupVue mode="Confirm" v-model:pop="popConfirmLogout" @confirm-click="logout">{{t('confirmLogout')}}</msgPopupVue>
</template>

<i18n lang="json">
  {
    "enUS": {
      "subtitle": "Wireless bridge",
      "Status": "Status",
      "network": "Network",
      "system": "System",
      "LAN": "LAN / IP",
      "WiFiBasic": "WiFi",
      "WiFiAdvanced": "Radio",
      "DeviceManagement": "Management",
      "DeviceUpgrade": "Firmware",
      "Debug": "Debug",
      "About": "About",
      "Logout": "Log out",
      "confirmLogout": "Log out of the device?"
    }
  }
</i18n>
