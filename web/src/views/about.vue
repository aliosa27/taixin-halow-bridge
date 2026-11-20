<script setup lang="ts">
  import { useI18n } from 'vue-i18n';
  import { Icon } from '@vicons/utils';
  import { CodeOutlined, BugReportOutlined } from '@vicons/material';

  const { t } = useI18n({
    inheritLocale: true,
    useScope: 'local'
  });

  const GH = 'https://github.com/';
  const REPO = 'aliosa27/taixin-halow-bridge';
  const MAIL = 'aliosa27@aliosa27.me';
  const sources = [
    { k: 'thisProject', r: REPO },
    { k: 'vendorFw', r: 'Taixin-Semiconductor/TXW8301_WNB' },
    { k: 'vendorUi', r: 'Taixin-Semiconductor/hgic-wnb-web' }
  ];
</script>

<template>
  <div class="header">
    <h1>{{t('header')}}</h1>
    <h2>{{t('sub')}}</h2>
  </div>
  <div class="content cards">
    <section class="card">
      <h3><Icon><CodeOutlined /></Icon>{{t('source')}}</h3>
      <dl class="kv">
        <template v-for="s in sources" :key="s.r">
          <dt>{{t(s.k)}}</dt>
          <dd><a :href="GH + s.r" target="_blank" rel="noopener">{{s.r}}</a></dd>
        </template>
      </dl>
      <p class="hint">{{t('license')}}</p>
    </section>
    <section class="card">
      <h3><Icon><BugReportOutlined /></Icon>{{t('issues')}}</h3>
      <dl class="kv">
        <dt>GitHub</dt>
        <dd><a :href="GH + REPO + '/issues'" target="_blank" rel="noopener">{{REPO}}/issues</a></dd>
        <dt>{{t('email')}}</dt>
        <dd><a :href="'mailto:' + MAIL">{{MAIL}}</a></dd>
      </dl>
      <p class="hint">{{t('issueHint')}}</p>
    </section>
  </div>
</template>

<style scoped>
  .hint { margin: .8rem 0 0; font-size: .82em; color: var(--muted); }
</style>

<i18n lang="json">
  {
    "enUS": {
      "header": "About",
      "sub": "Where this firmware comes from and how to report problems",
      "source": "Source code",
      "thisProject": "This firmware",
      "vendorFw": "Vendor SDK",
      "vendorUi": "Vendor web UI",
      "license": "Built from the Taixin SDK with patches; the web UI is derived from the vendor UI (Apache-2.0).",
      "issues": "Report an issue",
      "email": "Email",
      "issueHint": "Include the firmware version and build date from Status, and the UART log if the device misbehaves at boot."
    }
  }
</i18n>
