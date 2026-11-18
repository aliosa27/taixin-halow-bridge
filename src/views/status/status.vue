<script setup lang="ts">
  import { useI18n } from 'vue-i18n';
  import { RouterLink, RouterView } from 'vue-router';
  import { Icon } from '@vicons/utils';
  import {
    InfoOutlined,
    GroupsOutlined
  } from '@vicons/material'

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
          <RouterLink :to="{ name: 'statusOverview' }">
            <Icon><InfoOutlined /></Icon>
            {{t('Overview')}}
          </RouterLink>
        </li>
        <li class="tab-item">
          <RouterLink :to="{ name: 'statusClientInfo' }">
            <Icon><GroupsOutlined /></Icon>
            {{t('clientInfo')}}
          </RouterLink>
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
      "header": "Status",
      "Overview": "Overview",
      "clientInfo": "Clients"
    },
    "zhCN": {
      "header": "设备状态",
      "Overview": "概览",
      "clientInfo": "客户端信息"
    }
  }
</i18n>