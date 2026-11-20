<script setup lang="ts">
  import { ref, watch, onUnmounted } from 'vue';
  import { RouterView, useRoute } from 'vue-router';
  import { wnbPing, RET } from '@/wnbCom';
  import { useDashboardStore } from '@/store';
  import { logoutIn3s } from '@/router';
  import menuVue from '@/components/menu.vue';
  import msgPopupVue from '@/components/msgPopup.vue';

  const dashboardStore = useDashboardStore();
  const route = useRoute();
  const menuToggle = ref<boolean>(false);
  const popAuthFail = ref<boolean>(false);

  // Close the mobile drawer after navigating
  watch(() => route.name, () => { menuToggle.value = false; });

  // Auth validity check — skipped on pages that handle auth implicitly via data polling
  const skipAuthCheckRoutes = ['statusOverview', 'statusClientInfo'];
  const interval = setInterval(async () => {
    if (skipAuthCheckRoutes.includes(route.name as string)) return;
    const resp = await wnbPing(dashboardStore.authToken);
    if (resp === RET.AUTH_FAIL) {
      popAuthFail.value = true;
      logoutIn3s();
    }
  }, 30000);

  onUnmounted(() => {
    clearInterval(interval);
  });

</script>

<template>
  <div id="layout" :class="{ active: menuToggle }">
    <header class="topbar">
      <button id="menuLink" type="button" aria-label="Menu" @click="menuToggle = !menuToggle"><span></span></button>
      <span class="brand">HaLow Bridge</span>
    </header>
    <menuVue :menu-toggle="menuToggle"></menuVue>
    <div class="scrim" v-if="menuToggle" @click="menuToggle = false"></div>
  </div>
  <div id="main">
    <RouterView></RouterView>
  </div>
  <msgPopupVue v-model:pop="popAuthFail" mode="Logout">{{$t('authTMout')}}</msgPopupVue>
</template>
