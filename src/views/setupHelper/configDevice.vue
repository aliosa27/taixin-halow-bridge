<script setup lang="ts">
	import { ref, onBeforeMount } from 'vue';
	// @ts-ignore
	import { useI18n } from 'vue-i18n';
	import { useRouter } from 'vue-router';
	import { isAuthenticated } from '@/router';
	import { getSettings, RET } from '@/wnbCom';
	import { lanSettingsData, wifiBasicSettingsData, lanSettings, wifiBasicSettings } from '@/assets/commonSettings';
	import { useFormDataStore, useDashboardStore, useSetupHelperStore } from '@/store';
	import setupHelperBtnVue from '@/components/setupHelperBtn.vue';
	import msgPopupVue from '@/components/msgPopup.vue';

  const { t } = useI18n({
    inheritLocale: true,
    useScope: 'local'
  });
  const { t: tg } = useI18n({
    useScope: 'global'
  });

  const router = useRouter();
  const formDataStore = useFormDataStore();
  const dashboardStore = useDashboardStore();
  const setupHelperStore = useSetupHelperStore();

  function logoutIn3s() {
    setTimeout(() => {
      isAuthenticated.value = false;
      router.push({ name: 'Login' });
    }, 3000)
  }

  const lanSettingsReactive = ref<lanSettings>({} as lanSettings);
  const wifiSettingReactive = ref<wifiBasicSettings>({} as wifiBasicSettings);

  const popAuthFail = ref<boolean>(false);
  const popLoadFail = ref<boolean>(false);

  onBeforeMount(async() => {
    // get Settings from Lan
    if ('lan' in formDataStore.dataStore) {
      Object.assign(lanSettingsReactive, formDataStore.dataStore['lan']);
    } else {
      const resp = await getSettings(lanSettingsData, dashboardStore.authToken);
      if (resp.state === RET.OK) {
        Object.assign(formDataStore.dataStore, { lan: { ...resp.settings } });
        Object.assign(lanSettingsReactive, resp.settings);
      } else if (resp.state === RET.AUTH_FAIL) {
        popAuthFail.value = true;
        logoutIn3s();
      } else {
        popLoadFail.value = true;
      }
    }
    if ('wifiBasic' in formDataStore.dataStore) {
      Object.assign(wifiSettingReactive, formDataStore.dataStore['wifiBasic']);
    } else {
      const resp = await getSettings(wifiBasicSettingsData, dashboardStore.authToken);
      if (resp.state === RET.OK) {
        Object.assign(formDataStore.dataStore, { wifiBasic: { ...resp.settings } });
        Object.assign(wifiSettingReactive, resp.settings);
      } else if (resp.state === RET.AUTH_FAIL) {
        popAuthFail.value = true;
        logoutIn3s();
      } else {
        popLoadFail.value = true;
      }
    }
  });
  
</script>

<template>
	<form>
		<fieldset>
			<legend>{{t('lan')}}</legend>
			<div class="ctrl-group">
			</div>
		</fieldset>
		<fieldset>
			<legend>{{t('wireless')}}</legend>
		</fieldset>
	</form>
	<setupHelperBtnVue :next-btn-disable="!modeSelected" @next-btn="router.push({ name: 'configDevice' })"></setupHelperBtnVue>
	<msgPopupVue v-model:pop="popAuthFail" mode="Alert">{{tg('authTMout')}}</msgPopupVue>
  <msgPopupVue v-model:pop="popLoadFail" mode="Alert">{{tg('pageLoadErr')}}</msgPopupVue>
</template>

<i18n lang="json">
	{
		"enUS": {
			"lan": "Lan Settings",
			"wireless": "Wireless Settings"
		},
		"zhCN": {

		}
	}
</i18n>