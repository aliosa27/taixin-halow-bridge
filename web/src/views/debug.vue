<script setup lang="ts">
  import { ref, nextTick, onUnmounted } from 'vue';
  import { useI18n } from 'vue-i18n';
  import { atCommand, logRead, RET } from '@/wnbCom';
  import { useDashboardStore } from '@/store';
  import { logoutIn3s } from '@/router';
  import msgPopupVue from '@/components/msgPopup.vue';
  import { Icon } from '@vicons/utils';
  import { TerminalOutlined, ArticleOutlined } from '@vicons/material';

  const { t } = useI18n({
    inheritLocale: true,
    useScope: 'local'
  });
  const { t: tg } = useI18n({
    useScope: 'global'
  });

  const dashboardStore = useDashboardStore();
  const popAuthFail = ref<boolean>(false);

  const quickCmds = ['AT+VERSION', 'AT+WIFIMODE?', 'AT+SSID?', 'AT+CHAN_LIST?', 'AT+CHANNEL?', 'AT+BSS_BW?', 'AT+RSSI'];
  const cmd = ref<string>('');
  const busy = ref<boolean>(false);
  const out = ref<string>('');
  const outEl = ref<HTMLElement | null>(null);
  const history: string[] = [];
  let histIdx = 0;

  async function send(c?: string) {
    const line = (c ?? cmd.value).trim();
    if (!line || busy.value) return;
    if (!/^at/i.test(line)) {
      out.value += '> ' + line + '\n' + t('notAT') + '\n\n';
      return;
    }
    if (history[history.length - 1] !== line) history.push(line);
    histIdx = history.length;
    cmd.value = '';
    busy.value = true;
    out.value += '> ' + line + '\n';
    const resp = await atCommand(line, dashboardStore.authToken);
    busy.value = false;
    if (resp.state === RET.AUTH_FAIL) {
      popAuthFail.value = true;
      logoutIn3s();
      return;
    }
    if (resp.state === RET.OK) {
      out.value += (resp.text.replace(/\r/g, '').trim() || t('noReply')) + '\n\n';
    } else {
      out.value += (resp.state === RET.TIME_OUT ? t('timeout') : t('failed')) + '\n\n';
    }
    await nextTick();
    if (outEl.value) outEl.value.scrollTop = outEl.value.scrollHeight;
  }

  /* ---- Device log: polls logread=<seq> once a second while running ---- */
  const LOG_MAX = 60000;   // characters kept in the browser
  const logOn = ref<boolean>(false);
  const logText = ref<string>('');
  const logErr = ref<string>('');
  const logEl = ref<HTMLElement | null>(null);
  let seq = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;

  async function poll() {
    if (!logOn.value) return;
    const r = await logRead(seq, dashboardStore.authToken);
    if (!logOn.value) return;
    if (r.state === RET.AUTH_FAIL) {
      stopLog();
      popAuthFail.value = true;
      logoutIn3s();
      return;
    }
    if (r.state === RET.OK) {
      logErr.value = '';
      seq = r.seq;
      if (r.text) {
        const el = logEl.value;
        const atEnd = !el || el.scrollHeight - el.scrollTop - el.clientHeight < 30;
        let s = logText.value + r.text.replace(/\x1b\[[0-9;]*m/g, '').replace(/\r/g, '');
        if (s.length > LOG_MAX) s = s.slice(s.length - LOG_MAX);
        logText.value = s;
        if (atEnd) {
          await nextTick();
          if (logEl.value) logEl.value.scrollTop = logEl.value.scrollHeight;
        }
      }
    } else {
      logErr.value = t('logErr');
    }
    timer = setTimeout(poll, 1000);
  }

  function startLog() {
    logOn.value = true;
    poll();
  }

  function stopLog() {
    logOn.value = false;
    if (timer) clearTimeout(timer);
  }

  function saveLog() {
    const a = document.createElement('a');
    a.href = URL.createObjectURL(new Blob([logText.value], { type: 'text/plain' }));
    a.download = 'halow-bridge-log.txt';
    a.click();
    URL.revokeObjectURL(a.href);
  }

  onUnmounted(stopLog);

  function onKey(e: KeyboardEvent) {
    if (e.key === 'ArrowUp' && histIdx > 0) {
      cmd.value = history[--histIdx];
      e.preventDefault();
    } else if (e.key === 'ArrowDown') {
      histIdx = Math.min(histIdx + 1, history.length);
      cmd.value = history[histIdx] ?? '';
      e.preventDefault();
    }
  }
</script>

<template>
  <div class="header">
    <h1>{{t('header')}}</h1>
    <h2>{{t('sub')}}</h2>
  </div>
  <div class="content">
    <section class="card">
      <h3><Icon><TerminalOutlined /></Icon>{{t('console')}}</h3>
      <pre class="term" ref="outEl">{{out || t('consoleHint')}}</pre>
      <form class="at-line" @submit.prevent="send()">
        <input type="text" id="atcmd" v-model="cmd" :placeholder="t('placeholder')" autocomplete="off"
          spellcheck="false" @keydown="onKey" />
        <button class="btn-blue" type="submit" :disabled="busy">{{busy ? t('running') : t('send')}}</button>
        <button type="button" @click="out = ''">{{t('clear')}}</button>
      </form>
      <div class="chips">
        <button type="button" v-for="q in quickCmds" :key="q" :disabled="busy" @click="send(q)">{{q}}</button>
      </div>
      <p class="hint">{{t('atHelp')}}</p>
    </section>
    <section class="card log-card">
      <h3><Icon><ArticleOutlined /></Icon>{{t('log')}}
        <span class="chip" :class="{ ok: logOn && !logErr }">{{logErr || (logOn ? t('live') : t('stopped'))}}</span>
      </h3>
      <pre class="term" ref="logEl">{{logText || t('logHint')}}</pre>
      <div class="at-line">
        <button v-if="!logOn" class="btn-blue" type="button" @click="startLog">{{t('start')}}</button>
        <button v-else class="btn-blue" type="button" @click="stopLog">{{t('stop')}}</button>
        <button type="button" @click="logText = ''">{{t('clear')}}</button>
        <button type="button" :disabled="!logText" @click="saveLog">{{t('save')}}</button>
      </div>
      <p class="hint">{{t('logHelp')}}</p>
    </section>
  </div>
  <msgPopupVue v-model:pop="popAuthFail" mode="Logout">{{tg('authTMout')}}</msgPopupVue>
</template>

<style scoped>
  .term {
    margin: 0 0 .75rem;
    padding: .75rem .9rem;
    height: 20rem;
    overflow: auto;
    background: #0b0f14;
    color: #d6e2ee;
    border-radius: 8px;
    font: .82rem/1.45 ui-monospace, Consolas, monospace;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .at-line { display: flex; gap: .5rem; }
  .at-line input { flex: 1; width: auto; min-width: 0; font-family: ui-monospace, Consolas, monospace; }
  .chips { display: flex; flex-wrap: wrap; gap: .4rem; margin-top: .6rem; }
  .chips button { font: .8rem ui-monospace, Consolas, monospace; padding: .3em .7em; }
  .hint { margin: .7rem 0 0; color: var(--muted); font-size: .85em; }
  .log-card { margin-top: 1rem; }
  .log-card h3 .chip { margin-left: auto; font-weight: 500; }
</style>

<i18n lang="json">
  {
    "enUS": {
      "header": "Debug",
      "sub": "Send AT commands and follow the device log.",
      "console": "AT console",
      "consoleHint": "Replies appear here. Try AT+VERSION.",
      "placeholder": "AT+…   (↑/↓ for history)",
      "send": "Send",
      "running": "Running…",
      "clear": "Clear",
      "notAT": "Commands must start with AT.",
      "noReply": "(no reply — this command prints to the device log instead; see below)",
      "timeout": "(timed out)",
      "failed": "(request failed)",
      "atHelp": "Commands run exactly as on the UART, including ones that change or erase settings. Settings changed here bypass the web UI's checks and may need a reboot to take effect. Some commands print to the device log rather than replying here.",
      "log": "Device log",
      "live": "live",
      "stopped": "stopped",
      "logErr": "unavailable",
      "logHint": "Press Start to follow the device log (the same output as the UART).",
      "start": "Start",
      "stop": "Stop",
      "save": "Save",
      "logHelp": "Shows log lines from the moment it is started; the device keeps the last 1 KB, so keep this page open to follow it. Capture stops on its own 30 s after this page stops polling."
    }
  }
</i18n>
