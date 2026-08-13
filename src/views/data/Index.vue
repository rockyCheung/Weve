<template>
  <div>
    <el-row :gutter="20">
      <el-col :span="12">
        <el-card>
          <template #header>
            <span>数据备份与恢复</span>
          </template>
          <el-alert
            title="定期备份数据可防止意外丢失"
            type="info"
            :closable="false"
            show-icon
            style="margin-bottom: 16px"
          />
          <el-row :gutter="16" style="margin-bottom: 16px">
            <el-col :span="12">
              <el-button
                type="primary"
                @click="backupData"
                :loading="backingUp"
                style="width: 100%"
                >创建备份</el-button
              >
            </el-col>
            <el-col :span="12">
              <el-button
                type="warning"
                @click="restoreData"
                style="width: 100%"
                >恢复备份</el-button
              >
            </el-col>
          </el-row>
          <div v-if="backups.length > 0">
            <h4 style="margin-bottom: 12px">历史备份列表</h4>
            <el-table :data="backups" stripe size="small" max-height="300">
              <el-table-column prop="created_at" label="备份时间" width="180" />
              <el-table-column prop="file_size" label="文件大小" width="120">
                <template #default="scope">{{
                  formatFileSize(scope.row.file_size)
                }}</template>
              </el-table-column>
              <el-table-column label="操作" width="120">
                <template #default="scope">
                  <el-button
                    size="small"
                    type="danger"
                    @click="deleteBackup(scope.row.file_name)"
                    >删除</el-button
                  >
                </template>
              </el-table-column>
            </el-table>
          </div>
          <el-empty v-else description="暂无备份记录" />
        </el-card>
      </el-col>
      <el-col :span="12">
        <el-card>
          <template #header>
            <span>数据导出</span>
          </template>
          <el-alert
            title="导出全部数据为 JSON 格式，可用于审计或迁移"
            type="info"
            :closable="false"
            show-icon
            style="margin-bottom: 16px"
          />
          <el-button
            type="success"
            @click="exportAllData"
            :loading="exporting"
            style="width: 100%"
            >导出全部数据</el-button
          >
          <div style="margin-top: 16px">
            <el-tag type="info"
              >导出内容包含：员工信息、考勤记录、会计科目、凭证、发票、税务报表、系统设置</el-tag
            >
          </div>
        </el-card>
        <el-card style="margin-top: 16px">
          <template #header>
            <span>数据库信息</span>
          </template>
          <el-descriptions :column="1" border>
            <el-descriptions-item label="数据库类型"
              >SQLite</el-descriptions-item
            >
            <el-descriptions-item label="数据位置">{{
              dbPath
            }}</el-descriptions-item>
            <el-descriptions-item label="备份目录">{{
              backupDir
            }}</el-descriptions-item>
          </el-descriptions>
        </el-card>
      </el-col>
    </el-row>

    <!-- 员工数据物理清理 -->
    <el-row :gutter="20" style="margin-top: 20px">
      <el-col :span="24">
        <el-card>
          <template #header>
            <span>员工数据物理清理</span>
          </template>
          <el-alert
            title="技术视角：彻底清除误录员工及其全部关联数据（考勤/绩效/奖金/业绩）。此操作不可恢复，请谨慎使用。"
            type="danger"
            :closable="false"
            show-icon
            style="margin-bottom: 16px"
          />
          <el-form :inline="true">
            <el-form-item label="选择员工">
              <el-select
                v-model="cleanupEmployeeId"
                placeholder="选择要清理的员工"
                filterable
                clearable
                style="width: 220px"
                @change="onCleanupSelect"
              >
                <el-option
                  v-for="e in employees"
                  :key="e.id"
                  :label="`${e.name}（${e.is_active ? '在职' : '离职'}）`"
                  :value="e.id"
                />
              </el-select>
            </el-form-item>
            <el-form-item>
              <el-button
                type="danger"
                :disabled="!cleanupEmployeeId"
                @click="confirmCleanup"
                >物理清理</el-button
              >
            </el-form-item>
          </el-form>

          <el-descriptions v-if="cleanupSummary" :column="4" border size="small">
            <el-descriptions-item label="考勤记录">{{ cleanupSummary.attendance_count }} 条</el-descriptions-item>
            <el-descriptions-item label="绩效考核">{{ cleanupSummary.review_count }} 条</el-descriptions-item>
            <el-descriptions-item label="奖金记录">{{ cleanupSummary.bonus_count }} 条</el-descriptions-item>
            <el-descriptions-item label="业绩记录">{{ cleanupSummary.target_count }} 条</el-descriptions-item>
          </el-descriptions>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ElMessage, ElMessageBox } from "element-plus";

interface BackupInfo {
  file_name: string;
  file_size: number;
  created_at: string;
}

const backingUp = ref(false);
const exporting = ref(false);
const dbPath = ref("");
const backupDir = ref("");
const backups = ref<BackupInfo[]>([]);

// 员工物理清理
interface ForceDeleteResult {
  employee_name: string;
  attendance_count: number;
  review_count: number;
  bonus_count: number;
  target_count: number;
}
const employees = ref<any[]>([]);
const cleanupEmployeeId = ref<number | null>(null);
const cleanupSummary = ref<ForceDeleteResult | null>(null);

const loadEmployees = async () => {
  try {
    employees.value = await invoke("get_all_employees");
  } catch (e) {
    console.error("获取员工列表失败:", e);
  }
};

const onCleanupSelect = async (id: number | null) => {
  cleanupSummary.value = null;
  if (!id) return;
  try {
    cleanupSummary.value = await invoke("get_employee_data_summary", { id }) as ForceDeleteResult;
  } catch (e) {
    ElMessage.error("查询关联数据失败: " + e);
  }
};

const confirmCleanup = async () => {
  if (!cleanupEmployeeId.value) return;
  const name = cleanupSummary.value?.employee_name || "";
  try {
    const input = await ElMessageBox.prompt(
      `即将彻底删除「${name}」及其全部关联数据，此操作不可恢复！请输入员工姓名「${name}」确认：`,
      "危险操作确认",
      {
        type: "error",
        confirmButtonText: "确认清理",
        cancelButtonText: "取消",
        inputValidator: (v: string) => v === name || "姓名不匹配，请重新输入",
      }
    );
    if (input.value === name) {
      const result = await invoke("force_delete_employee", { id: cleanupEmployeeId.value }) as ForceDeleteResult;
      ElMessage.success(`已清理「${name}」：考勤${result.attendance_count}条、绩效${result.review_count}条、奖金${result.bonus_count}条、业绩${result.target_count}条`);
      cleanupEmployeeId.value = null;
      cleanupSummary.value = null;
      await loadEmployees();
    }
  } catch {
    // 取消操作
  }
};

const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
  return (bytes / (1024 * 1024)).toFixed(1) + " MB";
};

const backupData = async () => {
  backingUp.value = true;
  try {
    const path = await invoke("backup_database");
    ElMessage.success(`备份成功: ${path}`);
    await loadBackups();
  } catch (e) {
    ElMessage.error("备份失败: " + e);
  } finally {
    backingUp.value = false;
  }
};

const restoreData = async () => {
  try {
    await ElMessageBox.confirm(
      "恢复备份将覆盖当前所有数据，此操作不可撤销，确定继续吗？",
      "警告",
      {
        type: "warning",
        confirmButtonText: "确定恢复",
        cancelButtonText: "取消",
      },
    );
    if (backups.value.length === 0) {
      ElMessage.warning("没有可恢复的备份");
      return;
    }
    const latestBackup = backups.value[0];
    await invoke("restore_database", { backupPath: latestBackup.file_name });
    ElMessage.success("数据恢复成功，请重启应用");
  } catch (e: any) {
    if (e !== "cancel") {
      ElMessage.error("恢复失败: " + e);
    }
  }
};

const loadBackups = async () => {
  try {
    backups.value = await invoke("list_backups");
  } catch (e) {
    console.error("加载备份列表失败:", e);
  }
};

const deleteBackup = async (fileName: string) => {
  try {
    await ElMessageBox.confirm("确定要删除此备份吗？", "提示");
    await invoke("delete_backup", { fileName });
    ElMessage.success("备份已删除");
    await loadBackups();
  } catch {
    // 取消操作
  }
};

const exportAllData = async () => {
  exporting.value = true;
  try {
    const path = await invoke("export_all_data");
    ElMessage.success(`数据已导出到: ${path}`);
  } catch (e) {
    ElMessage.error("导出失败: " + e);
  } finally {
    exporting.value = false;
  }
};

onMounted(async () => {
  await loadBackups();
  await loadEmployees();
  dbPath.value = "用户数据目录/small-business-manager/data.db";
  backupDir.value = "用户数据目录/small-business-manager/backups/";
});
</script>
