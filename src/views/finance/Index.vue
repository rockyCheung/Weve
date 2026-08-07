<template>
  <div class="finance-container">
    <el-menu
      mode="horizontal"
      :default-active="activeMenu"
      @select="handleMenuSelect"
      class="sub-nav"
    >
      <el-menu-item index="/finance/accounting">
        <el-icon><Document /></el-icon>
        <span>记账管理</span>
      </el-menu-item>
      <el-menu-item index="/finance/invoices">
        <el-icon><Tickets /></el-icon>
        <span>发票管理</span>
      </el-menu-item>
      <el-menu-item index="/finance/tax-reports">
        <el-icon><TrendCharts /></el-icon>
        <span>税务报表</span>
      </el-menu-item>
      <el-menu-item index="/finance/tax-calendar">
        <el-icon><Calendar /></el-icon>
        <span>税务日历</span>
      </el-menu-item>
    </el-menu>
    <div class="finance-content">
      <router-view />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useRouter, useRoute } from "vue-router";
import { Document, Tickets, TrendCharts, Calendar } from "@element-plus/icons-vue";

const router = useRouter();
const route = useRoute();
const activeMenu = ref(route.path);

const handleMenuSelect = (index: string) => {
  router.push(index);
};
</script>

<style scoped>
.sub-nav {
  border-radius: 6px;
  padding: 0 8px;
}
.finance-content {
  margin-top: 20px;
}
</style>
