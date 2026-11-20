<script setup lang="ts">
	import { ref, onBeforeMount } from 'vue';
	import { useI18n } from 'vue-i18n';
	import { useDashboardStore } from '@/store';
	import { logoutIn3s } from '@/router';
	import { firmwareUpgrade, getSettings, reboot, RET } from '@/wnbCom';
	import { fetchLatest, downloadImage, isNewer, CHANGELOG_URL, type Release } from '@/utils/update';
	import msgPopupVue from '@/components/msgPopup.vue';
	import { Icon } from '@vicons/utils';
	import { FileUploadOutlined, SystemUpdateAltOutlined } from '@vicons/material';

	const { t } = useI18n({
		inheritLocale: true,
		useScope: 'local'
	});
	const { t: tg } = useI18n({
		useScope: 'global'
	});

	const dashboardStore = useDashboardStore();

	const popAuthFail = ref<boolean>(false);
	const popConfirmUpload = ref<boolean>(false);
	const popConfirmOnline = ref<boolean>(false);
	const popNoFile = ref<boolean>(false);
	const popInvalidFile = ref<boolean>(false);
	const popUploadFail = ref<boolean>(false);
	const popUploadSuccess = ref<boolean>(false);
	const progressDisplay = ref<number>(0);
	const file = ref();

	// Online update (utils/update.ts): '' | checking | latest | available | downloading | uploading | net | verify
	const installed = ref<string>('');
	const upd = ref<string>('');
	const rel = ref<Release | null>(null);
	const dlProgress = ref<number>(0);

	onBeforeMount(async () => {
		const resp = await getSettings(['fwver'], dashboardStore.authToken);
		if (resp.state === RET.OK) installed.value = resp.settings.fwver;
	});

	async function checkUpdate() {
		upd.value = 'checking';
		try {
			rel.value = await fetchLatest();
			upd.value = isNewer(rel.value.version, installed.value) ? 'available' : 'latest';
		} catch {
			upd.value = 'net';
		}
	}

	async function installUpdate() {
		if (!rel.value) return;
		upd.value = 'downloading';
		try {
			const img = await downloadImage(rel.value, (p) => { dlProgress.value = p; });
			upd.value = 'uploading';
			startUpload(img);
		} catch (e) {
			upd.value = (e as Error).message === 'verify' ? 'verify' : 'net';
		}
	}

  function reloadPage() {
  	setTimeout(() => {
  		location.reload();
  	}, 5000);
  }

	function uploadStatus(status: RET) {
		if (status === RET.OK) {
			popUploadSuccess.value = true;
			reboot(dashboardStore.authToken);
			reloadPage();
		} else if (status === RET.AUTH_FAIL) {
			popAuthFail.value = true;
			logoutIn3s();
		} else {
			popUploadFail.value = true;
			if (upd.value === 'uploading') upd.value = 'available';
		}
	}

	function startUpload(img: Blob) {
		const resp = firmwareUpgrade(dashboardStore.authToken, img);
		resp.status = uploadStatus;
		resp.progress = (progress: number) => {
			progressDisplay.value = progress;
		}
	}

	function uploadFirmware() {
		if (file.value.files[0]) {
			if (!file.value.files[0].name.toLowerCase().endsWith('.bin')) {
				popInvalidFile.value = true;
				return;
			}
			startUpload(file.value.files[0]);
		} else {
			popNoFile.value = true;
		}
	}

</script>

<template>
	<div class="header">
		<h1>{{t('header')}}</h1>
	</div>
	<div class="content">
		<form @submit.prevent>
			<fieldset>
				<legend>{{t('online')}}</legend>
				<div class="ctrl-group">
					<label>{{t('installed')}}</label>
					<span id="fwInstalled">{{installed}}</span>
				</div>
				<div class="ctrl-group" v-if="rel && upd !== 'checking' && upd !== 'net'">
					<label>{{t('latest')}}</label>
					<span id="fwLatest">{{rel.version}} ({{rel.date}}) · <a :href="CHANGELOG_URL" target="_blank" rel="noopener">{{t('changelog')}}</a></span>
				</div>
				<p class="upd-msg" id="updMsg" v-if="upd && upd !== 'available'">{{t('st_' + upd)}}</p>
				<p class="upd-msg" v-if="rel && upd === 'available'">{{rel.notes}}</p>
				<div class="ctrl-group" v-if="upd === 'downloading'">
					<label for="download">{{t('download')}}</label>
					<progress id="download" :value="dlProgress" max="100"></progress>
				</div>
			</fieldset>
		</form>
		<div class="btn-group">
			<button id="checkUpd" type="button" :disabled="upd === 'checking' || upd === 'downloading' || upd === 'uploading'" @click="checkUpdate">{{t('check')}}</button>
			<button id="installUpd" type="button" class="btn-blue" v-if="upd === 'available'" @click="popConfirmOnline = true">
				<Icon><SystemUpdateAltOutlined /></Icon>
				{{t('install')}}
			</button>
		</div>
		<form>
			<fieldset>
				<legend>{{t('uploadFW')}}</legend>
				<div class="ctrl-group">
					<label for="fileSelect">{{t('selectFile')}}</label>
					<input type="file" id="fileSelect" accept=".bin" ref="file" />
				</div>
				<div class="ctrl-group" v-if="progressDisplay > 0">
					<label for="upgrade">{{t('progress')}}</label>
					<progress id="upgrade" :value="progressDisplay" max="100"></progress>
				</div>
			</fieldset>
		</form>
		<div class="btn-group">
			<button id="upload" class="btn-blue" @click="popConfirmUpload = true">
				<Icon><FileUploadOutlined /></Icon>
				{{t('btnUpload')}}
			</button>
		</div>
	</div>
	<msgPopupVue mode="Logout" v-model:pop="popAuthFail">{{tg('authTMout')}}</msgPopupVue>
	<msgPopupVue mode="Confirm" v-model:pop="popConfirmUpload" @confirm-click="uploadFirmware">{{t('confirmUpload')}}</msgPopupVue>
	<msgPopupVue mode="Confirm" v-model:pop="popConfirmOnline" @confirm-click="installUpdate">{{t('confirmOnline')}}</msgPopupVue>
	<msgPopupVue mode="Alert" v-model:pop="popUploadFail">{{t('uploadFail')}}</msgPopupVue>
	<msgPopupVue mode="Logout" v-model:pop="popUploadSuccess">{{t('uploadSuccess')}}</msgPopupVue>
	<msgPopupVue mode="Alert" v-model:pop="popNoFile">{{t('noFile')}}</msgPopupVue>
	<msgPopupVue mode="Alert" v-model:pop="popInvalidFile">{{t('invalidFile')}}</msgPopupVue>
</template>

<style scoped>
	.upd-msg { margin: .6rem 0 0; color: var(--muted); white-space: pre-line; }
	.btn-group + form { margin-top: 1.5rem; }
</style>

<i18n lang="json">
	{
		"enUS": {
			"header": "Firmware upgrade",
			"online": "Online update",
			"installed": "Installed:",
			"latest": "Latest release:",
			"changelog": "changelog",
			"check": "Check for updates",
			"install": "Download and install",
			"download": "Download:",
			"st_checking": "Checking GitHub…",
			"st_latest": "This device is up to date.",
			"st_downloading": "Downloading from GitHub…",
			"st_uploading": "Download verified. Uploading to the device…",
			"st_net": "Couldn't reach GitHub. The browser you're using needs internet access; the device itself doesn't.",
			"st_verify": "The download doesn't match the release (size or checksum). Nothing was installed; try again.",
			"confirmOnline": "Download this release and install it? The device reboots afterwards. On a link, update the station end first.",
			"uploadFW": "Upload Firmware",
			"selectFile": "Select Firmware File:",
			"progress": "Progress:",
			"btnUpload": "Upload",
			"confirmUpload": "Are you sure to upgrade??",
			"uploadFail": "Upload firmware fail!!, please retry!!",
			"uploadSuccess": "Upload successful!! Device will now reboot, back to login page in 3s",
			"noFile": "No File selected!!",
			"invalidFile": "Invalid file type!! Please select a .bin firmware file!!"
		},
		"zhCN": {
			"header": "设备固件升级",
			"uploadFW": "上传固件",
			"selectFile": "选择固件文件:",
			"progress": "进度:",
			"btnUpload": "上传",
			"confirmUpload": "确认要更新固件??",
			"uploadFail": "上传失败!!,请重试!!",
			"uploadSuccess": "上传成功!!设备将会重启, 3秒后回到登录页面",
			"noFile": "没有选择文件!!",
			"invalidFile": "文件类型无效!! 请选择 .bin 固件文件!!"
		}
	}
</i18n>
