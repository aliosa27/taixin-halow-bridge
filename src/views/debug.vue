<script setup lang="ts">
  import { ref, reactive, nextTick, onUnmounted } from 'vue';
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

  function authFailed() {
    popAuthFail.value = true;
    logoutIn3s();
  }

  /* ---- AT console ---- */
  const quickCmds = ['AT+VERSION', 'AT+WIFIMODE?', 'AT+SSID?', 'AT+CHAN_LIST?', 'AT+CHANNEL?', 'AT+BSS_BW?'];
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
    if (resp.state === RET.AUTH_FAIL) return authFailed();
    if (resp.state === RET.OK) {
      out.value += (resp.text.replace(/\r/g, '').trim() || t('noReply')) + '\n\n';
    } else {
      out.value += (resp.state === RET.TIME_OUT ? t('timeout') : t('failed')) + '\n\n';
    }
    await nextTick();
    if (outEl.value) outEl.value.scrollTop = outEl.value.scrollHeight;
  }

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

  /* ---- Log window ---- */
  const LOG_MAX = 60000;   // characters kept in the browser
  const logOpen = ref<boolean>(false);
  const logText = ref<string>('');
  const paused = ref<boolean>(false);
  const follow = ref<boolean>(true);
  const logErr = ref<string>('');
  const logEl = ref<HTMLElement | null>(null);
  const dbg = reactive<{ [k: string]: boolean }>({ heap: false, top: false, net: false });
  let seq = 0;
  let timer: ReturnType<typeof setTimeout> | undefined;

  async function poll() {
    if (!logOpen.value) return;
    if (!paused.value) {
      const r = await logRead(seq, dashboardStore.authToken);
      if (r.state === RET.AUTH_FAIL) {
        closeLog();
        return authFailed();
      }
      if (r.state === RET.OK) {
        logErr.value = '';
        seq = r.seq;
        if (r.text) {
          // strip ANSI colour codes, normalise newlines
          let s = logText.value + r.text.replace(/\x1b\[[0-9;]*m/g, '').replace(/\r/g, '');
          if (s.length > LOG_MAX) s = s.slice(s.length - LOG_MAX);
          logText.value = s;
          if (follow.value) {
            await nextTick();
            if (logEl.value) logEl.value.scrollTop = logEl.value.scrollHeight;
          }
        }
      } else {
        logErr.value = t('logErr');
      }
    }
    timer = setTimeout(poll, 1000);
  }

  function openLog() {
    logOpen.value = true;
    poll();
  }

  function closeLog() {
    logOpen.value = false;
    if (timer) clearTimeout(timer);
  }

  async function toggleDbg(k: string) {
    dbg[k] = !dbg[k];
    const resp = await atCommand('AT+SYSDBG=' + k + ',' + (dbg[k] ? '1' : '0'), dashboardStore.authToken);
    if (resp.state === RET.AUTH_FAIL) authFailed();
  }

  function saveLog() {
    const a = document.createElement('a');
    a.href = URL.createObjectURL(new Blob([logText.value], { type: 'text/plain' }));
    a.download = 'wnb-log.txt';
    a.click();
    URL.revokeObjectURL(a.href);
  }

  onUnmounted(closeLog);

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

    <section class="card">
      <h3><Icon><ArticleOutlined /></Icon>{{t('logs')}}</h3>
      <p class="hint">{{t('logHelp')}}</p>
      <button class="btn-blue" type="button" @click="openLog">{{t('openLog')}}</button>
    </section>
  </div>

  <Transition name="bounce">
    <div v-if="logOpen" class="modal">
      <div class="modal-content log-win">
        <div class="log-head">
          <h2>{{t('logs')}}</h2>
          <span class="chip" :class="{ ok: !paused && !logErr }">{{logErr || (paused ? t('paused') : t('live'))}}</span>
          <span class="spacer"></span>
          <button type="button" @click="paused = !paused">{{paused ? t('resume') : t('pause')}}</button>
          <button type="button" @click="logText = ''">{{t('clear')}}</button>
          <button type="button" @click="saveLog">{{t('save')}}</button>
          <button class="btn-blue" type="button" @click="closeLog">{{t('close')}}</button>
        </div>
        <pre class="term log" ref="logEl">{{logText || t('waiting')}}</pre>
        <div class="log-foot">
          <label><input type="checkbox" v-model="follow" /> {{t('follow')}}</label>
          <span class="spacer"></span>
          <span class="muted">{{t('periodic')}}</span>
          <button type="button" v-for="k in ['heap', 'top', 'net']" :key="k" :class="{ 'btn-blue': dbg[k] }" @click="toggleDbg(k)">{{t('dbg_' + k)}}</button>
        </div>
      </div>
    </div>
  </Transition>
  <msgPopupVue v-model:pop="popAuthFail" mode="Logout">{{tg('authTMout')}}</msgPopupVue>
</template>

<style scoped>
  .card { margin-bottom: 1rem; }
  .term {
    margin: 0 0 .75rem;
    padding: .75rem .9rem;
    height: 16rem;
    overflow: auto;
    background: #0b0f14;
    color: #d6e2ee;
    border-radius: 8px;
    font: .82rem/1.45 ui-monospace, Consolas, monospace;
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }
  .at-line { display: flex; gap: .5rem; }
  .at-line input { flex: 1; width: auto; font-family: ui-monospace, Consolas, monospace; }
  .chips { display: flex; flex-wrap: wrap; gap: .4rem; margin-top: .6rem; }
  .chips button { font: .8rem ui-monospace, Consolas, monospace; padding: .3em .7em; }
  .hint { margin: .7rem 0 0; color: var(--muted); font-size: .85em; }
  .card .hint:first-of-type + button { margin-top: .8rem; }
  .log-win { max-width: 1000px; display: flex; flex-direction: column; height: min(85vh, 760px); }
  .log-head, .log-foot { display: flex; flex-wrap: wrap; align-items: center; gap: .5rem; }
  .log-head h2 { margin: 0; font-size: 1.1rem; }
  .log-head { margin-bottom: .75rem; }
  .term.log { flex: 1; height: auto; min-height: 10rem; }
  .log-foot button { padding: .3em .8em; font-size: .85em; }
  .spacer { flex: 1; }
  .muted { color: var(--muted); font-size: .85em; }
</style>

<i18n lang="json">
  {
    "enUS": {
      "header": "Debug",
      "sub": "Send AT commands and watch the device log.",
      "console": "AT console",
      "consoleHint": "Replies appear here. Try AT+VERSION.",
      "placeholder": "AT+…   (↑/↓ for history)",
      "send": "Send",
      "running": "Running…",
      "clear": "Clear",
      "notAT": "Commands must start with AT.",
      "noReply": "(no reply — this command may print to the log instead)",
      "timeout": "(timed out)",
      "failed": "(request failed)",
      "atHelp": "Commands run exactly as on the UART, including ones that change or erase settings. Many commands print their output to the log rather than replying here.",
      "logs": "Device log",
      "logHelp": "Shows everything the firmware prints to the UART console, live. The device keeps the last 4 KB while the window is open; log capture stops 30 s after the window is closed. While it runs, it replaces the netlog UDP log output.",
      "openLog": "Open log window",
      "live": "Live",
      "paused": "Paused",
      "pause": "Pause",
      "resume": "Resume",
      "save": "Save",
      "close": "Close",
      "follow": "Auto-scroll",
      "waiting": "Waiting for output…",
      "logErr": "Connection lost",
      "periodic": "Periodic output:",
      "dbg_heap": "Heap",
      "dbg_top": "CPU",
      "dbg_net": "Network"
    }
  }
</i18n>
