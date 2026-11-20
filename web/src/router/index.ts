import { ref } from 'vue';
import { createRouter, createWebHistory, RouteRecordRaw } from 'vue-router';
import notFoundVue from '@/views/notFound.vue';
import loginVue from '@/views/login.vue';
import dashboardVue from '@/views/dashboard.vue';
import statusVue from '@/views/status/status.vue';
import statusOverviewVue from '@/views/status/overview.vue';
import clientInfoVue from '@/views/status/clientInfo.vue';
import lanVue from '@/views/lan.vue';
import wifiBasicVue from '@/views/WiFi/wifiBasic.vue';
import wifiAdvancedVue from '@/views/WiFi/wifiAdvanced.vue';
import deviceManageVue from '@/views/device/deviceManage.vue';
import deviceUpgradeVue from '@/views/device/deviceUpgrade.vue';
import debugVue from '@/views/debug.vue';
import aboutVue from '@/views/about.vue';
//import setupHelperVue from '@/views/setupHelper/setupHelper.vue';
//import modeSelectVue from '@/views/setupHelper/modeSelect.vue';
//import configDeviceVue from '@/views/setupHelper/configDevice.vue';
//import confirmConfigVue from '@/views/setupHelper/confirmConfig.vue';

// Set this to false to force login
export const isAuthenticated = ref<boolean>(false);

export function logoutIn3s() {
  setTimeout(() => {
    isAuthenticated.value = false;
    router.push({ name: 'Login' });
  }, 3000);
}

const routes: Readonly<RouteRecordRaw[]> = [{
  path: '/',
  name: 'Dashboard',
  component: dashboardVue,
  children: [{
    path: 'status',
    name: 'Status',
    component: statusVue,
    children: [{
      path: 'overview',
      name: 'statusOverview',
      component: statusOverviewVue
    }, {
      path: 'client-info',
      name: 'statusClientInfo',
      component: clientInfoVue
    }]
  }, /*{
    path: 'setup-helper',
    name: 'setupHelper',
    component: setupHelperVue,
    children: [{
      path: 'mode-select',
      name: 'modeSelect',
      component: modeSelectVue
    }, {
      path: 'config-device',
      name: 'configDevice',
      component: configDeviceVue
    }, {
      path: 'confirm-setup',
      name: 'confirmSetup',
      component: confirmConfigVue
    }]
  },*/ {
    path: 'lan',
    name: 'LAN',
    component: lanVue
  }, {
    path: 'wifi-basic',
    name: 'WiFiBasic',
    component: wifiBasicVue
  }, {
    path: 'wifi-advanced',
    name: 'WiFiAdvanced',
    component: wifiAdvancedVue
  }, {
    path: 'device-management',
    name: 'DeviceManagement',
    component: deviceManageVue
  }, {
    path: 'device-upgrade',
    name: 'DeviceUpgrade',
    component: deviceUpgradeVue
  }, {
    path: 'debug',
    name: 'Debug',
    component: debugVue
  }, {
    path: 'about',
    name: 'About',
    component: aboutVue
  }]
}, {
  path: '/login',
  name: 'Login',
  component: loginVue
}, { // Catch all if not found route
  path: '/:pathMatch(.*)*',
  name: 'notFound',
  component: notFoundVue
}];

const router = createRouter({
  history: createWebHistory(),
  routes
});

router.beforeEach(async (to) => {
  if (!isAuthenticated.value && to.name !== 'Login') {
    return { name: 'Login' };
  }
});

export default router;

