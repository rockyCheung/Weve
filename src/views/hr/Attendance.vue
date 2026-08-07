<template>
  <div>
    <el-card>
      <template #header>
        <span>考勤打卡</span>
      </template>
      <el-form :inline="true" :model="attendanceForm">
        <el-form-item label="员工" required>
          <el-select
            v-model="attendanceForm.employee_id"
            placeholder="选择员工"
            filterable
            style="width: 200px"
          >
            <el-option
              v-for="emp in employees"
              :key="emp.id"
              :label="`${emp.name} - ${emp.position}`"
              :value="emp.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="日期">
          <el-date-picker
            v-model="attendanceForm.date"
            type="date"
            value-format="YYYY-MM-DD"
            :default-value="new Date()"
          />
        </el-form-item>
        <el-form-item label="签到时间">
          <el-time-picker
            v-model="attendanceForm.check_in"
            value-format="HH:mm:ss"
            placeholder="选择时间"
          />
        </el-form-item>
        <el-form-item label="签退时间">
          <el-time-picker
            v-model="attendanceForm.check_out"
            value-format="HH:mm:ss"
            placeholder="选择时间"
          />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="recordAttendance"
            >记录打卡</el-button
          >
          <el-button @click="quickCheckIn">快速签到（当前时间）</el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <el-card style="margin-top: 20px">
      <template #header>
        <span>考勤查询</span>
      </template>
      <el-form :inline="true" :model="queryForm">
        <el-form-item label="员工">
          <el-select
            v-model="queryForm.employee_id"
            placeholder="全部员工"
            clearable
            filterable
            style="width: 200px"
          >
            <el-option
              v-for="emp in employees"
              :key="emp.id"
              :label="emp.name"
              :value="emp.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="开始日期">
          <el-date-picker
            v-model="queryForm.start_date"
            type="date"
            value-format="YYYY-MM-DD"
          />
        </el-form-item>
        <el-form-item label="结束日期">
          <el-date-picker
            v-model="queryForm.end_date"
            type="date"
            value-format="YYYY-MM-DD"
          />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="fetchAttendance">查询</el-button>
          <el-button @click="resetQuery">重置</el-button>
        </el-form-item>
      </el-form>

      <el-table
        :data="attendanceList"
        stripe
        style="width: 100%"
        v-loading="loading"
      >
        <el-table-column prop="employee_name" label="员工姓名" width="120" />
        <el-table-column prop="date" label="日期" width="120" />
        <el-table-column prop="check_in" label="签到时间" width="120" />
        <el-table-column prop="check_out" label="签退时间" width="120" />
        <el-table-column prop="status" label="状态" width="100">
          <template #default="scope">
            <el-tag :type="statusTagType(scope.row.status)" size="small">
              {{ statusText(scope.row.status) }}
            </el-tag>
          </template>
        </el-table-column>
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ElMessage } from "element-plus";
import dayjs from "dayjs";

interface Employee {
  id: number;
  name: string;
  position: string;
}

interface AttendanceRecord {
  id: number;
  employee_id: number;
  employee_name: string;
  date: string;
  check_in: string | null;
  check_out: string | null;
  status: string;
}

const employees = ref<Employee[]>([]);
const attendanceList = ref<AttendanceRecord[]>([]);
const loading = ref(false);

const attendanceForm = ref({
  employee_id: null as number | null,
  date: dayjs().format("YYYY-MM-DD"),
  check_in: null as string | null,
  check_out: null as string | null,
});

const queryForm = ref({
  employee_id: null as number | null,
  start_date: dayjs().startOf("month").format("YYYY-MM-DD"),
  end_date: dayjs().format("YYYY-MM-DD"),
});

const statusTagType = (status: string) => {
  const map: Record<string, string> = {
    present: "success",
    late: "warning",
    absent: "danger",
  };
  return map[status] || "info";
};

const statusText = (status: string) => {
  const map: Record<string, string> = {
    present: "正常",
    late: "迟到",
    absent: "缺勤",
  };
  return map[status] || status;
};

const fetchEmployees = async () => {
  try {
    const result: any[] = await invoke("get_all_employees");
    employees.value = result
      .filter((e) => e.is_active)
      .map((e: any) => ({
        id: e.id,
        name: e.name,
        position: e.position,
      }));
  } catch (e) {
    console.error("获取员工列表失败:", e);
  }
};

const fetchAttendance = async () => {
  loading.value = true;
  try {
    attendanceList.value = await invoke("query_attendance", {
      query: queryForm.value,
    });
  } catch (e) {
    ElMessage.error("查询失败: " + e);
  } finally {
    loading.value = false;
  }
};

const recordAttendance = async () => {
  if (!attendanceForm.value.employee_id) {
    ElMessage.warning("请选择员工");
    return;
  }
  try {
    await invoke("record_attendance", { input: attendanceForm.value });
    ElMessage.success("打卡记录成功");
    await fetchAttendance();
  } catch (e) {
    ElMessage.error("打卡失败: " + e);
  }
};

const quickCheckIn = () => {
  const now = dayjs();
  attendanceForm.value.date = now.format("YYYY-MM-DD");
  attendanceForm.value.check_in = now.format("HH:mm:ss");
  recordAttendance();
};

const resetQuery = () => {
  queryForm.value = {
    employee_id: null,
    start_date: dayjs().startOf("month").format("YYYY-MM-DD"),
    end_date: dayjs().format("YYYY-MM-DD"),
  };
  fetchAttendance();
};

onMounted(() => {
  fetchEmployees();
  fetchAttendance();
});
</script>
