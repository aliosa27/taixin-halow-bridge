<script setup lang="ts">
	import { ref } from 'vue';
	  // @ts-ignore
	import { useI18n } from 'vue-i18n';
	import { RouterLink, RouterView, useRouter } from 'vue-router';
	import { Icon } from '@vicons/utils';

	const { t } = useI18n({
		inheritLocale: true,
		useScope: 'local'
	});

</script>

<template>
	<div class="header">
		<h1>{{t('header')}}</h1>
		<div class="tabs">
			<ul class="tab-list">
				<li class="tab-item">
					<span>{{t('opMode')}}</span>
					<!--<RouterLink :to="{ name: 'modeSelect' }" @click="routerLinkToNum = 0">{{t('opMode')}}</RouterLink>-->
				</li>
				<li class="tab-item">
					<RouterLink :to="{ name: 'configDevice' }" @click="routerLinkToNum = 1">{{t('config')}}</RouterLink>
				</li>
				<li class="tab-item">
					<RouterLink :to="{ name: 'confirmSetup' }" @click="routerLinkToNum = 2">{{t('confirm')}}</RouterLink>
				</li>
			</ul>
		</div>
	</div>
	<div class="content">
		<RouterView></RouterView>
	</div>
</template>


<i18n lang="json">
	{
		"enUS": {
			"header": "Setup Helper",
			"opMode": "Operation Mode",
			"config": "Configuration",
			"confirm": "Confirm Settings"
		},
		"zhCN": {
			"header": "设置向导",
			"opMode": "工作模式",
			"config": "设备配置",
			"confirm": "确认配置"
		}
	}
</i18n>