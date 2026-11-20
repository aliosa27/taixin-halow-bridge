<script setup lang="ts">
  import { ref, onBeforeMount } from 'vue';
  import { useI18n } from 'vue-i18n';
  import { useRouter } from 'vue-router';
  import { isAuthenticated } from '@/router';
  import { authorization, wnbPing, deviceConfig, RET } from '@/wnbCom';
  import { useDashboardStore } from '@/store';

  const dashboardStore = useDashboardStore();

  async function getDeviceConfig() {
    const resp = await deviceConfig(dashboardStore.authToken);
    if (resp.state === RET.OK) {
      Object.assign(dashboardStore.deviceConfig, resp.deviceConfig);
      return true;
    } else {
      msgWarn.value = t('ConnErr');
      return false;
    }
  }

  onBeforeMount(async () => {
    let localStorageAuthToken = localStorage.getItem('authToken');
    // give AuthToken random value as firmware could have disable web auth
    const storedAuthToken = localStorageAuthToken ? localStorageAuthToken : 'someRandomStuff';
    const resp = await wnbPing(storedAuthToken);
    if (resp === RET.OK) {
      // Stored token still valid, jump to dashboard
      isAuthenticated.value = true;
      dashboardStore.authToken = storedAuthToken;
      if (await getDeviceConfig())
      router.push({ name: 'statusOverview' });
    } else if (resp === RET.AUTH_FAIL) {
      // Stored token no longer valid, delete it
      localStorage.clear();
    }});

  const router = useRouter();

  const { t } = useI18n({
    inheritLocale: true,
    useScope: 'local'
  });

  const loginPasswd = ref<string>('');
  const msgWarn = ref<string>('');

  async function loginAction() {
    const ret = await authorization(loginPasswd.value);
    if (ret.state === RET.OK) {
      localStorage.setItem('authToken', ret.authToken);
      isAuthenticated.value = true;
      dashboardStore.authToken = ret.authToken;
      if (await getDeviceConfig())
        router.push({name: 'statusOverview'});
    } else if (ret.state === RET.AUTH_FAIL) {
      msgWarn.value = t('invalidPasswd');
    } else {
      msgWarn.value = t('ConnErr');
    }
  }

  function clearInput() {
    loginPasswd.value = '';
    msgWarn.value = '';
  }

  // Prevent button in form reflesh page
  function onSubmit() {}

</script>

<template>
  <Transition name="bounce">
    <div id="modal" class="modal login">
      <div class="modal-content">
        <div class="modal-header">
          <div class="brand">HaLow Bridge</div>
          <h2>{{t('Login')}}</h2>
        </div>
        <div class="modal-body">
          <form v-on:submit.prevent="onSubmit">
            <div class="ctrl-group">
              <label for="webpasswd">{{t('webpasswd')}}</label>
              <input type="password" id="webpasswd" maxlength="32" v-model.trim="loginPasswd" @keyup.enter="loginAction" />
              <span class="msg-warn">{{msgWarn}}</span>
            </div>
          </form>
        </div>
        <div class="modal-footer">
          <div class="btn-popup">
            <button class="btn-blue" @click="loginAction">{{t('LoginBtn')}}</button>
            <button @click="clearInput">{{t('ClearBtn')}}</button>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<i18n lang="json">
{
  "enUS": {
    "Login": "Device Login",
    "webpasswd": "Login Password",
    "invalidPasswd": "Invalid Password!!",
    "ConnErr": "Connection Error, please retry!!",
    "LoginBtn": "Login",
    "ClearBtn": "Clear"
  },
  "zhCN": {
    "Login": "设备登入",
    "webpasswd": "登入密码",
    "invalidPasswd": "密码错误!!",
    "ConnErr": "连接失败，请重试!!",
    "LoginBtn": "登入",
    "ClearBtn": "清除"
  }
}
</i18n>