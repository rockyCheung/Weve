<template>
  <div>
    <el-card>
      <template #header>
        <span>业绩提成</span>
      </template>
      <el-form :inline="true">
        <el-form-item label="所属月份">
          <el-date-picker v-model="period" type="month" value-format="YYYY-MM" />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="fetchTargets" :loading="loading">查询</el-button>
        </el-form-item>
      </el-form>

      <el-table :data="rows" stripe border size="small" style="margin-top: 16px">
        <el-table-column prop="employee_name" label="姓名" width="120" />
        <el-table-column label="目标额" width="140">
          <template #default="scope">
            <el-input-number v-model="scope.row.target_amount" :min="0" :precision="2" size="small" />
          </template>
        </el-table-column>
        <el-table-column label="达成额" width="140">
          <template #default="scope">
            <el-input-number v-model="scope.row.achieved_amount" :min="0" :precision="2" size="small" />
          </template>
        </el-table-column>
        <el-table-column label="提成比例" width="120">
          <template #default="scope">
            <el-input-number v-model="scope.row.commission_rate" :min="0" :max="1" :step="0.01" :precision="3" size="small" />
          </template>
        </el-table-column>
        <el-table-column label="提成额" width="120">
          <template #default="scope">¥{{ (scope.row.achieved_amount * scope.row.commission_rate).toFixed(2) }}</template>
        </el-table-column>
        <el-table-column label="操作" width="80">
          <template #default="scope">
            <el-button size="small" type="primary" @click="save(scope.row)">保存</el-button>
          </template>
        </el-table-column>
      </el-table>
      <el-empty v-if="rows.length === 0 && !loading" description="暂无在职员工" />
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ElMessage } from "element-plus";
import dayjs from "dayjs";

interface Row {
  employee_id: number;
  employee_name: string;
  target_amount: number;
  achieved_amount: number;
  commission_rate: number;
}

const period = ref(dayjs().format("YYYY-MM"));
const loading = ref(false);
const rows = ref<Row[]>([]);

const fetchTargets = async () => {
  if (!period.value) return;
  loading.value = true;
  try {
    const [employees, targets] = await Promise.all([
      invoke("get_all_employees") as Promise<any[]>,
      invoke("get_performance_targets", { period: period.value }) as Promise<any[]>,
    ]);
    const targetMap: Record<number, any> = {};
    for (const t of targets) targetMap[t.employee_id] = t;

    rows.value = employees
      .filter((e) => e.is_active)
      .map((e) => {
        const t = targetMap[e.id];
        return {
          employee_id: e.id,
          employee_name: e.name,
          target_amount: t?.target_amount ?? 0,
          achieved_amount: t?.achieved_amount ?? 0,
          commission_rate: t?.commission_rate ?? 0,
        };
      });
  } catch (e) {
    ElMessage.error("查询失败: " + e);
  } finally {
    loading.value = false;
  }
};

const save = async (row: Row) => {
  try {
    await invoke("save_performance_target", {
      input: {
        employee_id: row.employee_id,
        period: period.value,
        target_amount: row.target_amount,
        achieved_amount: row.achieved_amount,
        commission_rate: row.commission_rate,
      },
    });
    ElMessage.success(`${row.employee_name} 业绩已保存`);
  } catch (e) {
    ElMessage.error("保存失败: " + e);
  }
};

onMounted(fetchTargets);
</script>
