<template>
  <div class="app-layout">
    <header class="app-header">
      <div class="header-left">
        <el-button class="home-btn" text @click="$router.push('/')">
          <el-icon :size="18"><HomeFilled /></el-icon>
          <span class="home-label">首页</span>
        </el-button>
        <el-divider direction="vertical" />
        <el-menu
          mode="horizontal"
          :default-active="activeMenu"
          :ellipsis="false"
          @select="handleSelect"
          class="global-nav"
        >
          <el-menu-item index="/hr">
            <el-icon><User /></el-icon>
            <span>人事管理</span>
          </el-menu-item>
          <el-menu-item index="/payroll">
            <el-icon><Money /></el-icon>
            <span>薪酬管理</span>
          </el-menu-item>
          <el-menu-item index="/accounting">
            <el-icon><Document /></el-icon>
            <span>记账管理</span>
          </el-menu-item>
          <el-menu-item index="/invoices">
            <el-icon><Tickets /></el-icon>
            <span>发票管理</span>
          </el-menu-item>
          <el-menu-item index="/tax-reports">
            <el-icon><TrendCharts /></el-icon>
            <span>税务报表</span>
          </el-menu-item>
          <el-menu-item index="/tax-calendar">
            <el-icon><Calendar /></el-icon>
            <span>税务日历</span>
          </el-menu-item>
        </el-menu>
      </div>
      <div class="header-right">
        <el-button class="settings-btn" text @click="$router.push('/data')">
          <el-icon :size="16"><Coin /></el-icon>
          <span class="settings-label">数据管理</span>
        </el-button>
        <el-divider direction="vertical" />
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
import { computed, onMounted, ref } from "vue";
import { useRoute, useRouter } from "vue-router";
import { HomeFilled, Setting, Coin, User, Money, Document, Tickets, TrendCharts, Calendar } from "@element-plus/icons-vue";
import { check } from "@tauri-apps/plugin-updater";
import { ElMessageBox } from "element-plus";
import { relaunch } from "@tauri-apps/plugin-process";

const route = useRoute();
const router = useRouter();

const updateAvailable = ref(false);

onMounted(async () => {
  try {
    const update = await check();
    if (update) {
      updateAvailable.value = true;
      await ElMessageBox.confirm(
        `发现新版本 ${update.version}，是否立即更新？\n\n更新内容：${update.body || "请查看发布说明"}`,
        "软件更新",
        { confirmButtonText: "立即更新", cancelButtonText: "稍后", type: "info" }
      );
      await update.downloadAndInstall();
      await relaunch();
    }
  } catch (e) {
    // 取消或检查失败，静默忽略
  }
});

const activeMenu = computed(() => {
  const parts = route.path.split("/").filter(Boolean);
  if (parts.length === 0) return "/";
  if (parts[0] === "hr" || parts[0] === "payroll") return "/" + parts[0];
  return route.path;
});

const handleSelect = (index: string) => {
  router.push(index);
};
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
  height: 56px;
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

.global-nav {
  border-bottom: none;
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
