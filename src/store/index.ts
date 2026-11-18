import { defineStore } from 'pinia';
import { ref } from 'vue';
import { strObj, WIFI_MODE } from '@/utils';
import { type deviceConfigClass } from '@/wnbCom';

/**
 * Basic configuration data for dashboard, authToken, what functionality to be enable
 */
export const useDashboardStore = defineStore('dashboard', () => {
  const authToken = ref<string>('');
  const deviceConfig = ref<deviceConfigClass>({} as deviceConfigClass);

  return { authToken, deviceConfig }
});

export type formDataStore = {[index: string]: strObj};

/**
 * Store getted form data, speed up page access
 */
export const useFormDataStore = defineStore('formData', () => {
  const dataStore = ref<formDataStore>({});
  return { dataStore }
});

/**
 * SetupHelper's data
 */
export const useSetupHelperStore = defineStore('setupHelperData', () => {
  const deviceOpMode = ref<WIFI_MODE>(WIFI_MODE.NONE);

  return { deviceOpMode }
});