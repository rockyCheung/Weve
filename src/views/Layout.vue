<template>
  <div class="app-layout">
    <header class="app-header">
      <div class="header-left">
        <el-button class="home-btn" text @click="$router.push('/')">
          <el-icon :size="18"><HomeFilled /></el-icon>
          <span class="home-label">首页</span>
        </el-button>
        <el-divider direction="vertical" />
        <el-breadcrumb separator=">">
          <el-breadcrumb-item
            v-for="item in breadcrumbs"
            :key="item.path"
            :to="item.path"
          >
            {{ item.label }}
          </el-breadcrumb-item>
        </el-breadcrumb>
      </div>
      <div class="header-right">
        <el-button class="settings-btn" text @click="$router.push('/settings')">
          <el-icon :size="16"><Setting /></el-icon>
          <span class="settings-label">系统设置</span>
        </el-button>
      </div>
    </header>
    <main class="app-main">
      <router-view />
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import { HomeFilled, Setting } from "@element-plus/icons-vue";

const route = useRoute();

const breadcrumbMap: Record<string, string> = {
  "/": "首页",
  "/hr": "人事管理",
  "/hr/employees": "员工花名册",
  "/hr/attendance": "考勤打卡",
  "/hr/payroll": "工资表",
  "/finance": "财务管理",
  "/finance/accounting": "记账管理",
  "/finance/invoices": "发票管理",
  "/finance/tax-reports": "税务报表",
  "/finance/tax-calendar": "税务日历",
  "/settings": "系统设置",
};

const breadcrumbs = computed(() => {
  const parts = route.path.split("/").filter(Boolean);
  const crumbs: { path: string; label: string }[] = [];

  let accumulated = "";
  for (const part of parts) {
    accumulated += "/" + part;
    const label = breadcrumbMap[accumulated];
    if (label) {
      crumbs.push({ path: accumulated, label });
    }
  }

  return crumbs;
});
</script>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: #f5f7fa;
}

.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 48px;
  padding: 0 20px;
  background: #fff;
  border-bottom: 1px solid #e4e7ed;
  flex-shrink: 0;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 8px;
}

.home-btn {
  font-size: 14px;
  color: #409eff;
  padding: 4px 8px;
}

.home-btn:hover {
  background: rgba(64, 158, 255, 0.1);
}

.home-label {
  margin-left: 4px;
}

.header-right {
  display: flex;
  align-items: center;
}

.settings-btn {
  font-size: 13px;
  color: #606266;
  padding: 4px 8px;
}

.settings-btn:hover {
  background: rgba(64, 158, 255, 0.1);
  color: #409eff;
}

.app-main {
  flex: 1;
  overflow: auto;
  padding: 20px;
}
</style>
