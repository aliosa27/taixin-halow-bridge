<script setup lang="ts">
	import { useI18n } from 'vue-i18n';
	import { Icon } from '@vicons/utils';
	import { ArrowBackFilled, ArrowForwardFilled, SaveAltOutlined } from '@vicons/material';

	const { t } = useI18n({
		useScope: 'local',
		inhertLocale: true
	});

	defineProps({
		backBtnDisable: { type: Boolean, default: false },
		nextBtnDisable: { type: Boolean, default: false },
		nextBtn2Confirm: { type: Boolean, default: false }
	});

	const emits = defineEmits<{
		(e: 'backBtn'): void,
		(e: 'nextBtn'): void,
		(e: 'finBtn'): void
	}>();

</script>

<template>
	<div class="btn-step">
		<button class="btn-blue" style="float: left" :disabled="backBtnDisable" @click="emits('backBtn')">
			<Icon><ArrowBackFilled /></Icon>
			{{t('back')}}
		</button>
		<button v-if="!nextBtn2Confirm" class="btn-blue" style="float: right;" :disabled="nextBtnDisable" @click="emits('nextBtn')">
			<Icon><ArrowForwardFilled /></Icon>
			{{t('next')}}
		</button>
		<button v-else class="btn-blue" style="float: right;" @click="emits('finBtn')">
			<Icon><SaveAltOutlined /></Icon>
			{{t('finish')}}
		</button>
	</div>
</template>

<i18n lang="json">
	{
		"enUS": {
			"back": "Back",
			"next": "Next",
			"finish": "Finish"
		},
		"zhCN": {
			"back": "上一步",
			"next": "下一步",
			"finish": "完成"
		}
	}
</i18n>