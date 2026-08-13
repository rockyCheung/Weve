<template>
  <div>
    <el-card>
      <template #header>
        <span>奖金管理</span>
        <el-button size="small" type="primary" style="float: right" @click="showAdd">添加奖金</el-button>
      </template>
      <el-form :inline="true">
        <el-form-item label="所属月份">
          <el-date-picker v-model="period" type="month" value-format="YYYY-MM" />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="fetchBonuses" :loading="loading">查询</el-button>
        </el-form-item>
      </el-form>

      <el-table :data="bonuses" stripe border size="small" style="margin-top: 16px">
        <el-table-column prop="employee_name" label="姓名" width="120" />
        <el-table-column label="奖金类型" width="120">
          <template #default="scope">{{ bonusTypeLabel(scope.row.bonus_type) }}</template>
        </el-table-column>
        <el-table-column prop="amount" label="金额" width="120">
          <template #default="scope">¥{{ scope.row.amount.toFixed(2) }}</template>
        </el-table-column>
        <el-table-column prop="remark" label="备注" />
        <el-table-column label="操作" width="80">
          <template #default="scope">
            <el-button size="small" type="danger" @click="remove(scope.row.id)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
      <el-empty v-if="bonuses.length === 0 && !loading" description="暂无奖金记录" />
    </el-card>

    <el-dialog v-model="dialogVisible" title="添加奖金" width="420px">
      <el-form :model="form" label-width="90px">
        <el-form-item label="员工">
          <el-select v-model="form.employee_id" filterable style="width: 100%">
            <el-option v-for="e in employees" :key="e.id" :label="e.name" :value="e.id" />
          </el-select>
        </el-form-item>
        <el-form-item label="奖金类型">
          <el-select v-model="form.bonus_type" style="width: 100%">
            <el-option label="全勤奖" value="attendance" />
            <el-option label="绩效奖" value="performance" />
            <el-option label="专项奖" value="special" />
            <el-option label="年终奖" value="annual" />
          </el-select>
        </el-form-item>
        <el-form-item label="金额">
          <el-input-number v-model="form.amount" :min="0" :precision="2" style="width: 100%" />
        </el-form-item>
        <el-form-item label="备注">
          <el-input v-model="form.remark" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="submit">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ElMessage } from "element-plus";
import dayjs from "dayjs";

interface Bonus {
  id: number;
  employee_id: number;
  employee_name: string;
  period: string;
  bonus_type: string;
  amount: number;
  remark: string;
}

const period = ref(dayjs().format("YYYY-MM"));
const loading = ref(false);
const bonuses = ref<Bonus[]>([]);
const employees = ref<any[]>([]);
const dialogVisible = ref(false);
const form = ref({ employee_id: null as number | null, bonus_type: "attendance", amount: 0, remark: "" });

const bonusTypeLabel = (t: string) => {
  const map: Record<string, string> = { attendance: "全勤奖", performance: "绩效奖", special: "专项奖", annual: "年终奖" };
  return map[t] || t;
};

const fetchBonuses = async () => {
  if (!period.value) return;
  loading.value = true;
  try {
    bonuses.value = await invoke("get_employee_bonuses", { period: period.value });
  } catch (e) {
    ElMessage.error("查询失败: " + e);
  } finally {
    loading.value = false;
  }
};

const showAdd = () => {
  form.value = { employee_id: null, bonus_type: "attendance", amount: 0, remark: "" };
  dialogVisible.value = true;
};

const submit = async () => {
  if (!form.value.employee_id) {
    ElMessage.warning("请选择员工");
    return;
  }
  try {
    await invoke("save_employee_bonus", {
      input: { ...form.value, period: period.value },
    });
    ElMessage.success("奖金已添加");
    dialogVisible.value = false;
    await fetchBonuses();
  } catch (e) {
    ElMessage.error("保存失败: " + e);
  }
};

const remove = async (id: number) => {
  try {
    await invoke("delete_employee_bonus", { id });
    ElMessage.success("已删除");
    await fetchBonuses();
  } catch (e) {
    ElMessage.error("删除失败: " + e);
  }
};

onMounted(async () => {
  employees.value = await invoke("get_all_employees");
  await fetchBonuses();
});
</script>
