<script setup lang="ts">
  import { computed } from 'vue';
  import { useI18n } from 'vue-i18n';

  const props = defineProps<{
    pop: boolean,
    mode: 'Alert' | 'Confirm' | 'Logout'
  }>();

  const emits = defineEmits<{
    (event: 'update:pop', value: boolean): void
    (event: 'confirmClick'): void
  }>();

  const popMsg = computed({
    get() {
      return props.pop;
    },
    set(newVal) {
      emits('update:pop', newVal);
    }
  });

  const { t } = useI18n({
    inheritLocale: true,
    useScope: 'local'
  });

  function confirmClick() {
    emits('confirmClick');
    popMsg.value = false;
  }

</script>

<template>
  <Transition name="bounce">
    <div v-if="popMsg" id="modal" class="modal">
      <div class="modal-content">
        <div class="modal-header">
          <h2 v-if="mode === 'Alert'">
            {{t('alert')}}
          </h2>
          <h2 v-else>
            {{t('confirm')}}
          </h2>
        </div>
        <div class="modal-body">
          <p><slot></slot></p>
        </div>
        <div class="modal-footer">
          <div class="btn-popup" v-if="mode === 'Alert'">
            <button class="btn-blue" @click="popMsg = false">{{t('closeBtn')}}</button>
          </div>
          <div class="btn-popup" v-if="mode === 'Confirm'">
            <button class="btn-blue" @click="confirmClick">{{t('confirmBtn')}}</button>
            <button @click="popMsg = false">{{t('cancelBtn')}}</button>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<i18n lang="json">
  {
    "enUS": {
      "alert": "Alert",
      "confirm": "Confirm Action",
      "confirmBtn": "Confirm",
      "cancelBtn": "Cancel",
      "closeBtn": "Close"
    },
    "zhCN": {
      "alert": "注意",
      "confirm": "确认动作",
      "confirmBtn": "确认",
      "cancelBtn": "取消",
      "closeBtn": "关闭"
    }
  }
</i18n>