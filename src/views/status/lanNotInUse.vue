<script setup lang="ts">
	import { ref, computed, reactive, onBeforeMount } from 'vue';
  // @ts-ignore
	import { useI18n } from 'vue-i18n';
	import { lanSettings, lanSettingsData } from '@/assets/commonSettings.ts';
	import { getSettings, dhcpLeaseList, dhcpLeaseClass, RET } from '@/wnbCom';
	import { SELECT } from '@/utils';
	import { useDashboardStore, useFormDataStore } from '@/store';
	import { isAuthenticated } from '@/router';
	import { useRouter } from 'vue-router';
	import msgPopupVue from '@/components/msgPopup.vue';

	const { t } = useI18n({
		inheritLocale: true,
		useScope: 'local'
	});
	const { t: tg } = useI18n({
		useScope: 'global'
	});

	const dashboardStore = useDashboardStore();
	const formDataStore = useFormDataStore();
	const router = useRouter();

	const leasedIPData = ['s_ipaddr', 's_netmask', 's_gw_ip'] as const;

	type leasedIP = {
		[key in typeof leasedIPData[number]]: string
	}

	const lanSettingsReactive = reactive({} as lanSettings);
	const leasedIPDisplay = reactive({} as leasedIP);
	const dhcpServerLeaseDisplay = ref<dhcpLeaseClass[]>([]);

	const popAuthFail = ref<boolean>(false);
	const popLoadFail = ref<boolean>(false);

	function logoutIn3s() {
		setTimeout(() => {
			isAuthenticated.value = false;
			router.push({ name: 'Login '});
		}, 3000)
	}

	async function pollLeasedIP() {
		const resp = await getSettings(leasedIPData, dashboardStore.authToken);
		if (resp.state === RET.OK) {
			Object.assign(leasedIPDisplay, resp.settings);
		} else if (resp.state === RET.AUTH_FAIL) {
			popAuthFail.value = true;
			logoutIn3s();
		} else {
			popLoadFail.value = true;
		}
	}

	async function pollServerLeaseList() {
		const resp = await dhcpLeaseList(dashboardStore.authToken);
		if (resp.state === RET.OK) {
			dhcpServerLeaseDisplay.value = [...resp.leaseList];
		} else if (resp.state === RET.AUTH_FAIL) {
			popAuthFail.value = true;
			logoutIn3s();
		} else {
			popLoadFail.value = true;
		}
	}

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
		if (lanSettingsReactive.dhcpc_en === SELECT.ENABLE)
			pollLeasedIP();
		if (lanSettingsReactive.dhcpd_en === SELECT.ENABLE)
			pollServerLeaseList();
	});

	const ipMode = computed(() => {
		if (lanSettingsReactive.dhcpc_en === SELECT.ENABLE)
			return t('ipDHCP');
		else
			return t('ipStatic');
	});

</script>

<template>
	<table>
		<thead><tr><th colspan="2">{{t('deviceIP')}}</th></tr></thead>
		<tbody v-if="lanSettingsReactive.dhcpc_en === SELECT.ENABLE">
			<tr><td class="label">{{t('ipMode')}}</td><td class="data">{{ipMode}}</td></tr>
			<tr><td class="label">{{t('ipAddr')}}</td><td class="data">{{leasedIPDisplay.s_ipaddr}}</td></tr>
			<tr><td class="label">{{t('netmask')}}</td><td class="data">{{leasedIPDisplay.s_netmask}}</td></tr>
			<tr><td class="label">{{t('gwIP')}}</td><td class="data">{{leasedIPDisplay.s_gw_ip}}</td></tr>
		</tbody>
		<tbody v-else>
			<tr><td class="label">{{t('ipMode')}}</td><td class="data">{{ipMode}}</td></tr>
			<tr><td class="label">{{t('ipAddr')}}</td><td class="data">{{lanSettingsReactive.ipaddr}}</td></tr>
			<tr><td class="label">{{t('netmask')}}</td><td class="data">{{lanSettingsReactive.netmask}}</td></tr>
			<tr><td class="label">{{t('gwIP')}}</td><td class="data">{{lanSettingsReactive.gw_ip}}</td></tr>
		</tbody>
	</table>
	<table v-if="lanSettingsReactive.dhcpd_en === SELECT.ENABLE">
		<thead>
			<tr><th colspan="2">{{t('dhcpClientList')}}</th></tr>
			<tr><th>{{t('macAddr')}}</th><th>IP</th></tr>
		</thead>
		<tbody>
			<tr v-for="item in dhcpServerLeaseDisplay">
				<td>{{item.mac}}</td>
				<td>{{item.ip}}</td>
			</tr>
		</tbody>
	</table>
	<msgPopupVue v-model:pop="popAuthFail" mode="Alert">{{tg('authTMout')}}</msgPopupVue>
	<msgPopupVue v-model:pop="popLoadFail" mode="Alert">{{tg('pageLoadErr')}}</msgPopupVue>
</template>

<style scoped>
  table {
    margin-top: 20px;
    width: 560px;
  }
  .label {
    width: 16%;
  }
  .data {
    width: 84%;
  }
</style>

<i18n lang="json">
	{
		"enUS": {
			"deviceIP": "Device IP Address",
			"ipMode": "IP Mode",
			"ipAddr": "IP Address",
			"netmask": "Sub netmask",
			"gwIP": "Gateway IP",
			"dhcpClientList": "DHCP Client List",
			"ipStatic": "Static",
			"ipDHCP": "DHCP",
			"macAddr": "MAC Address"
		},
		"zhCN": {
			"deviceIP": "设备IP地址",
			"ipMode": "IP模式",
			"ipAddr": "IP地址",
			"netmask": "子网掩码",
			"gwIP": "网关IP",
			"dhcpClientList": "DHCP客户列表",
			"ipStatic": "静态",
			"ipDHCP": "DHCP",
			"macAddr": "MAC地址"
		}
	}
</i18n>