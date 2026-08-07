<template>
  <div class="hr-container">
    <el-menu
      mode="horizontal"
      :default-active="activeMenu"
      @select="handleMenuSelect"
      class="sub-nav"
    >
      <el-menu-item index="/hr/employees">
        <el-icon><User /></el-icon>
        <span>员工花名册</span>
      </el-menu-item>
      <el-menu-item index="/hr/attendance">
        <el-icon><Clock /></el-icon>
        <span>考勤打卡</span>
      </el-menu-item>
      <el-menu-item index="/hr/payroll">
        <el-icon><Money /></el-icon>
        <span>工资表</span>
      </el-menu-item>
    </el-menu>
    <div class="hr-content">
      <router-view />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useRouter, useRoute } from "vue-router";
import { User, Clock, Money } from "@element-plus/icons-vue";

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
.hr-content {
  margin-top: 20px;
}
</style>
