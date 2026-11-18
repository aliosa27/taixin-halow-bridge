<script setup lang="ts">
	import { reactive, ref } from 'vue';
	// @ts-ignore
	import { useI18n } from 'vue-i18n';
	import { useRouter } from 'vue-router';
	import { WIFI_MODE } from '@/utils';
	import { useSetupHelperStore } from '@/store';
	import { Icon, IconConfigProvider } from '@vicons/utils';
	import accessPointIcon from '@/assets/accessPoint.vue';
	import repeaterIcon from '@/assets/repeater.vue';
	import wifi2lanIcon from '@/assets/wifi2Lan.vue';
	import setupHelperBtnVue from '@/components/setupHelperBtn.vue';

	const { t } = useI18n({
		inheritLocale: true,
		useScope: 'local'
	});

	const router = useRouter();
	const setupHelperStore = useSetupHelperStore();

	const modeSelected = ref<boolean>(false);
	const modeSelect = reactive({
		AP: false,
		APSTA: false,
		STA: false
	});

	function modeClicked(mode: WIFI_MODE) {
		modeSelect.AP = false;
		modeSelect.APSTA = false;
		modeSelect.STA = false;
		modeSelected.value = true;
		switch(mode) {
			case WIFI_MODE.NONE:
				break;
			case WIFI_MODE.STA:
				setupHelperStore.deviceOpMode = WIFI_MODE.STA;
				modeSelect.STA = true;
				break;
			case WIFI_MODE.AP:
				setupHelperStore.deviceOpMode = WIFI_MODE.AP;
				modeSelect.AP = true;
				break;
			case WIFI_MODE.APSTA:
				setupHelperStore.deviceOpMode = WIFI_MODE.APSTA;
				modeSelect.APSTA = true;
				break;
		}
	}

</script>

<template>
	<IconConfigProvider size="60" color="blue">
		<div class="g">
			<div class="u" @click="modeClicked(WIFI_MODE.AP)" :class="{ selected: modeSelect.AP}">
				<div class="icon">
					<Icon><accessPointIcon /></Icon>
				</div>
				<h3 class="text">{{t('accessPoint')}}</h3>
				<span class="text">{{t('apHelper')}}</span>
			</div>
			<div class="u" @click="modeClicked(WIFI_MODE.APSTA)" :class="{ selected: modeSelect.APSTA }">
				<div class="icon">
					<Icon><repeaterIcon /></Icon>
				</div>
				<h3 class="text">{{t('repeater')}}</h3>
				<span class="text">{{t('repeaterHelper')}}</span>
			</div>
			<div class="u" @click="modeClicked(WIFI_MODE.STA)" :class="{ selected: modeSelect.STA }">
				<div class="icon">
					<Icon><wifi2lanIcon /></Icon>
				</div>
				<h3 class="text">{{t('wifi2Lan')}}</h3>
				<span class="text">{{t('wifi2LanHelper')}}</span>
			</div>
		</div>
	</IconConfigProvider>
	<setupHelperBtnVue back-btn-disable :next-btn-disable="!modeSelected" @next-btn="router.push({ name: 'configDevice' })"></setupHelperBtnVue>
</template>

<style scoped>
	.g {
		display: flex;
		flex-flow: row wrap;
		align-content: flex-start;
		margin-top: 20px;
	}
	.g .u {
		display: inline-block;
		letter-spacing: normal;
		word-spacing: normal;
		vertical-align: top;
		text-rendering: auto;
		width: 30%;
		border: 2px solid #0F80FF;
		border-radius: 10px;
		margin-right: 20px;
	}
	.g .u.selected,
	.g .u:hover {
		background-image: linear-gradient(transparent, rgba(0,0,0, 0.05) 40%, rgba(0,0,0, 0.10));
	}
	.icon {
		width: 50%;
		margin-left: 35%;
		margin-top: 10px;
		margin-bottom: 10px;
	}
	.text {
		text-align: center;
	}
</style>

<i18n lang="json">
	{
		"enUS": {
			"accessPoint": "Access Point",
			"apHelper": "Connect to an router through an Ethernet cable to extend the wireless coverage.",
			"repeater": "WiFi Repeater",
			"repeaterHelper": "Connect to an existing Wireless network to extend the WiFi coverage, can also provide network to none WiFi device through Ethernet",
			"wifi2Lan": "Wireless to Lan",
			"wifi2LanHelper": "Provide network connection to none wireless device throught Ethernet cable, without extending Wireless network"
		},
		"zhCN": {
			"accessPoint": "有线转无线",
			"apHelper": "通过有线网络扩展WiFi覆盖",
			"repeater": "无线扩展器",
			"repeaterHelper": "通过连接已有的WiFi网络扩展WiFi覆盖，也可以使用有线提供网络给非无线设备提供连接",
			"wifi2Lan": "无线转有线",
			"wifi2LanHelper": "给不支持无线的设备提供网络连接，不扩展WiFi"
		}
	}
</i18n>