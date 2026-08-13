<template>
  <div class="payroll-container">
    <el-menu
      mode="horizontal"
      :default-active="activeMenu"
      @select="handleMenuSelect"
      class="sub-nav"
    >
      <el-menu-item index="/payroll/salary">
        <el-icon><Money /></el-icon>
        <span>工资表</span>
      </el-menu-item>
      <el-menu-item index="/payroll/performance">
        <el-icon><DataAnalysis /></el-icon>
        <span>绩效考核</span>
      </el-menu-item>
      <el-menu-item index="/payroll/bonus">
        <el-icon><Present /></el-icon>
        <span>奖金管理</span>
      </el-menu-item>
      <el-menu-item index="/payroll/target">
        <el-icon><TrendCharts /></el-icon>
        <span>业绩提成</span>
      </el-menu-item>
    </el-menu>
    <div class="payroll-content">
      <router-view />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useRouter, useRoute } from "vue-router";
import { Money, DataAnalysis, Present, TrendCharts } from "@element-plus/icons-vue";

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
.payroll-content {
  margin-top: 20px;
}
</style>
