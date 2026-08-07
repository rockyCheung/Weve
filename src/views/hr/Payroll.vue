<template>
  <div>
    <el-card>
      <template #header>
        <span>工资表生成</span>
      </template>
      <el-form :inline="true">
        <el-form-item label="所属月份">
          <el-date-picker
            v-model="selectedPeriod"
            type="month"
            value-format="YYYY-MM"
            placeholder="选择月份"
          />
        </el-form-item>
        <el-form-item>
          <el-button
            type="primary"
            @click="generatePayroll"
            :loading="generating"
          >
            生成工资表
          </el-button>
          <el-button
            @click="exportPayroll"
            :disabled="payrollRecords.length === 0"
          >
            导出Excel
          </el-button>
        </el-form-item>
      </el-form>

      <div v-if="payrollRecords.length > 0" style="margin-top: 16px">
        <el-alert
          title="工资表已生成"
          type="success"
          :closable="false"
          style="margin-bottom: 16px"
        >
          <template #default>
            <span>共 {{ payrollRecords.length }} 名员工&nbsp;&nbsp;|&nbsp;&nbsp;</span>
            <span>工资总额 ¥{{ format(totalSalary) }}&nbsp;&nbsp;|&nbsp;&nbsp;</span>
            <span>实发合计 ¥{{ format(totalNetSalary) }}&nbsp;&nbsp;|&nbsp;&nbsp;</span>
            <span>代扣个税 ¥{{ format(totalIncomeTax) }}</span>
          </template>
        </el-alert>

        <el-table :data="payrollRecords" stripe border size="small" style="width: 100%">
          <el-table-column prop="employee_name" label="姓名" width="80" fixed />

          <el-table-column label="公司缴纳" align="center">
            <el-table-column prop="pension_employer" label="养老" width="90">
              <template #default="s">¥{{ fmt(s.row.pension_employer) }}</template>
            </el-table-column>
            <el-table-column prop="medical_employer" label="医疗" width="90">
              <template #default="s">¥{{ fmt(s.row.medical_employer) }}</template>
            </el-table-column>
            <el-table-column prop="unemployment_employer" label="失业" width="80">
              <template #default="s">¥{{ fmt(s.row.unemployment_employer) }}</template>
            </el-table-column>
            <el-table-column prop="injury_employer" label="工伤" width="80">
              <template #default="s">¥{{ fmt(s.row.injury_employer) }}</template>
            </el-table-column>
            <el-table-column prop="housing_fund_employer" label="公积金" width="90">
              <template #default="s">¥{{ fmt(s.row.housing_fund_employer) }}</template>
            </el-table-column>
          </el-table-column>

          <el-table-column label="个人缴纳" align="center">
            <el-table-column prop="pension_employee" label="养老" width="90">
              <template #default="s">¥{{ fmt(s.row.pension_employee) }}</template>
            </el-table-column>
            <el-table-column prop="medical_employee" label="医疗" width="90">
              <template #default="s">¥{{ fmt(s.row.medical_employee) }}</template>
            </el-table-column>
            <el-table-column prop="unemployment_employee" label="失业" width="80">
              <template #default="s">¥{{ fmt(s.row.unemployment_employee) }}</template>
            </el-table-column>
            <el-table-column prop="injury_employee" label="工伤" width="80">
              <template #default="s">¥{{ fmt(s.row.injury_employee) }}</template>
            </el-table-column>
            <el-table-column prop="housing_fund_employee" label="公积金" width="90">
              <template #default="s">¥{{ fmt(s.row.housing_fund_employee) }}</template>
            </el-table-column>
          </el-table-column>

          <el-table-column prop="income_tax" label="个税" width="90">
            <template #default="s">¥{{ fmt(s.row.income_tax) }}</template>
          </el-table-column>
          <el-table-column prop="net_salary" label="实发" width="100" fixed="right">
            <template #default="s">
              <el-tag type="success">¥{{ fmt(s.row.net_salary) }}</el-tag>
            </template>
          </el-table-column>
        </el-table>
      </div>

      <el-empty v-else description="请选择月份并生成工资表" />
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ElMessage } from "element-plus";
import dayjs from "dayjs";

interface PayrollRecord {
  employee_id: number;
  employee_name: string;
  base_salary: number;
  attendance_days: number;
  overtime_hours: number;
  bonus: number;
  deductions: number;
  insurance_base: number;
  pension_employer: number;
  pension_employee: number;
  medical_employer: number;
  medical_employee: number;
  unemployment_employer: number;
  unemployment_employee: number;
  injury_employer: number;
  injury_employee: number;
  housing_fund_employer: number;
  housing_fund_employee: number;
  income_tax: number;
  net_salary: number;
  period: string;
}

const selectedPeriod = ref(dayjs().format("YYYY-MM"));
const generating = ref(false);
const payrollRecords = ref<PayrollRecord[]>([]);

const fmt = (v: number) => v.toLocaleString("zh-CN", { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const format = (v: number) => v.toLocaleString("zh-CN", { minimumFractionDigits: 2, maximumFractionDigits: 2 });

const totalSalary = computed(() => payrollRecords.value.reduce((sum, r) => sum + r.base_salary, 0));
const totalNetSalary = computed(() => payrollRecords.value.reduce((sum, r) => sum + r.net_salary, 0));
const totalIncomeTax = computed(() => payrollRecords.value.reduce((sum, r) => sum + r.income_tax, 0));

const generatePayroll = async () => {
  if (!selectedPeriod.value) {
    ElMessage.warning("请选择所属月份");
    return;
  }
  generating.value = true;
  try {
    payrollRecords.value = await invoke("generate_payroll", {
      period: selectedPeriod.value,
    });
    ElMessage.success("工资表生成成功");
  } catch (e) {
    ElMessage.error("生成失败: " + e);
  } finally {
    generating.value = false;
  }
};

const exportPayroll = () => {
  ElMessage.success("导出功能待实现，数据已在页面展示");
};
</script>
