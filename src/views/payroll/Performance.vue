<template>
  <div>
    <el-card>
      <template #header>
        <span>绩效考核</span>
      </template>
      <el-alert
        title="仅显示绩效比例大于 0 的员工；绩效比例为 0 的岗位无需绩效考核。总分 = 工作业绩与产出×30% + 专业能力与成长力×20% + 沟通协作与团队影响×20% + 工作态度与职业素养×15% + 创新与战略前瞻×15%"
        type="info"
        :closable="false"
        show-icon
        style="margin-bottom: 16px"
      />
      <el-form :inline="true">
        <el-form-item label="考核月份">
          <el-date-picker v-model="period" type="month" value-format="YYYY-MM" />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="fetchAll" :loading="loading">查询</el-button>
        </el-form-item>
      </el-form>

      <el-table :data="rows" stripe border size="small" style="margin-top: 16px">
        <el-table-column prop="employee_name" label="姓名" width="90" fixed />
        <el-table-column label="业绩产出(30%)" width="160">
          <template #default="scope">
            <el-slider v-model="scope.row.business_score" :min="0" :max="100" :step="1" class="score-slider" />
          </template>
        </el-table-column>
        <el-table-column label="能力成长(20%)" width="160">
          <template #default="scope">
            <el-slider v-model="scope.row.growth_score" :min="0" :max="100" :step="1" class="score-slider" />
          </template>
        </el-table-column>
        <el-table-column label="协作影响(20%)" width="160">
          <template #default="scope">
            <el-slider v-model="scope.row.collaboration_score" :min="0" :max="100" :step="1" class="score-slider" />
          </template>
        </el-table-column>
        <el-table-column label="态度素养(15%)" width="160">
          <template #default="scope">
            <el-slider v-model="scope.row.attitude_score" :min="0" :max="100" :step="1" class="score-slider" />
          </template>
        </el-table-column>
        <el-table-column label="创新前瞻(15%)" width="160">
          <template #default="scope">
            <el-slider v-model="scope.row.innovation_score" :min="0" :max="100" :step="1" class="score-slider" />
          </template>
        </el-table-column>
        <el-table-column label="总分" width="70">
          <template #default="scope">{{ totalOf(scope.row).toFixed(0) }}</template>
        </el-table-column>
        <el-table-column label="等级" width="70">
          <template #default="scope">
            <el-tag :type="gradeTag(gradeOfScore(totalOf(scope.row)))" size="small">{{ gradeOfScore(totalOf(scope.row)) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="系数" width="70">
          <template #default="scope">{{ coeffOfScore(totalOf(scope.row)) }}</template>
        </el-table-column>
        <el-table-column label="操作" width="80" fixed="right">
          <template #default="scope">
            <el-button size="small" type="primary" @click="save(scope.row)">保存</el-button>
          </template>
        </el-table-column>
      </el-table>
      <el-empty v-if="rows.length === 0 && !loading" description="暂无需考核的员工" />
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
  business_score: number;
  growth_score: number;
  collaboration_score: number;
  attitude_score: number;
  innovation_score: number;
}

const period = ref(dayjs().format("YYYY-MM"));
const loading = ref(false);
const rows = ref<Row[]>([]);

const WEIGHTS = { business: 0.3, growth: 0.2, collaboration: 0.2, attitude: 0.15, innovation: 0.15 };

const totalOf = (r: Row) =>
  r.business_score * WEIGHTS.business +
  r.growth_score * WEIGHTS.growth +
  r.collaboration_score * WEIGHTS.collaboration +
  r.attitude_score * WEIGHTS.attitude +
  r.innovation_score * WEIGHTS.innovation;

const gradeOf = (score: number): { grade: string; coefficient: number } => {
  if (score >= 95) return { grade: "S", coefficient: 1.2 };
  if (score >= 85) return { grade: "A", coefficient: 1.0 };
  if (score >= 70) return { grade: "B", coefficient: 0.9 };
  if (score >= 60) return { grade: "C", coefficient: 0.7 };
  return { grade: "D", coefficient: 0.5 };
};

const gradeOfScore = (score: number): string => gradeOf(score).grade;
const coeffOfScore = (score: number): number => gradeOf(score).coefficient;

const gradeTag = (g: string) => {
  const map: Record<string, string> = { S: "success", A: "", B: "info", C: "warning", D: "danger" };
  return map[g] || "info";
};

const fetchAll = async () => {
  if (!period.value) return;
  loading.value = true;
  try {
    const [employees, reviews] = await Promise.all([
      invoke("get_all_employees") as Promise<any[]>,
      invoke("get_performance_reviews", { period: period.value }) as Promise<any[]>,
    ]);
    const reviewMap: Record<number, any> = {};
    for (const r of reviews) reviewMap[r.employee_id] = r;

    rows.value = employees
      .filter((e) => e.is_active && e.grade_performance_ratio > 0)
      .map((e) => {
        const r = reviewMap[e.id];
        return {
          employee_id: e.id,
          employee_name: e.name,
          business_score: r?.business_score ?? 0,
          growth_score: r?.growth_score ?? 0,
          collaboration_score: r?.collaboration_score ?? 0,
          attitude_score: r?.attitude_score ?? 0,
          innovation_score: r?.innovation_score ?? 0,
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
    await invoke("save_performance_review", {
      input: {
        employee_id: row.employee_id,
        period: period.value,
        business_score: row.business_score,
        growth_score: row.growth_score,
        collaboration_score: row.collaboration_score,
        attitude_score: row.attitude_score,
        innovation_score: row.innovation_score,
        reviewer: "",
        remark: "",
      },
    });
    ElMessage.success(`${row.employee_name} 考核已保存`);
    await fetchAll();
  } catch (e) {
    ElMessage.error("保存失败: " + e);
  }
};

onMounted(fetchAll);
</script>

<style scoped>
.score-slider {
  margin: 0 10px;
}
</style>
