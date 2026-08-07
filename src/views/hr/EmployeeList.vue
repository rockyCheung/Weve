<template>
  <div>
    <el-card>
      <template #header>
        <span>员工花名册</span>
        <div style="float: right;">
          <el-input
            v-model="searchKeyword"
            placeholder="搜索姓名/手机号"
            clearable
            style="width: 200px; margin-right: 8px;"
          />
          <el-button type="primary" @click="showAddDialog">添加员工</el-button>
          <el-button @click="showImportDialog">批量导入</el-button>
        </div>
      </template>

      <el-table :data="filteredEmployees" stripe v-loading="loading" style="width: 100%;">
        <el-table-column prop="name" label="姓名" width="100" />
        <el-table-column prop="phone" label="手机号" width="130" />
        <el-table-column prop="position" label="岗位" width="120" />
        <el-table-column prop="id_card" label="身份证号" width="180" />
        <el-table-column prop="hire_date" label="入职日期" width="110" />
        <el-table-column prop="salary" label="薪资" width="110">
          <template #default="scope">¥{{ scope.row.salary.toFixed(2) }}</template>
        </el-table-column>
        <el-table-column label="状态" width="80">
          <template #default="scope">
            <el-tag :type="scope.row.is_active ? 'success' : 'info'" size="small">
              {{ scope.row.is_active ? '在职' : '离职' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="220" fixed="right">
          <template #default="scope">
            <el-button size="small" @click="editEmployee(scope.row)">编辑</el-button>
            <el-button
              size="small"
              :type="scope.row.is_active ? 'warning' : 'success'"
              @click="toggleActive(scope.row)"
            >
              {{ scope.row.is_active ? '离职' : '复职' }}
            </el-button>
            <el-button size="small" type="danger" @click="deleteEmployee(scope.row.id)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 添加/编辑员工对话框 -->
    <el-dialog v-model="dialogVisible" :title="isEdit ? '编辑员工' : '添加员工'" width="500px">
      <el-form ref="formRef" :model="form" :rules="rules" label-width="90px">
        <el-form-item label="姓名" prop="name">
          <el-input v-model="form.name" />
        </el-form-item>
        <el-form-item label="手机号" prop="phone">
          <el-input v-model="form.phone" />
        </el-form-item>
        <el-form-item label="岗位">
          <el-input v-model="form.position" />
        </el-form-item>
        <el-form-item label="身份证号">
          <el-input v-model="form.id_card" />
        </el-form-item>
        <el-form-item label="入职日期">
          <el-date-picker v-model="form.hire_date" type="date" value-format="YYYY-MM-DD" />
        </el-form-item>
        <el-form-item label="薪资">
          <el-input-number v-model="form.salary" :min="0" :precision="2" :step="500" />
        </el-form-item>
        <el-form-item label="社保基数">
          <el-input-number v-model="form.insurance_base" :min="0" :precision="2" :step="500" />
          <div class="form-tip">为 0 则默认使用薪资作为基数</div>
        </el-form-item>
        <el-form-item label="公积金基数">
          <el-input-number v-model="form.housing_fund_base" :min="0" :precision="2" :step="500" />
          <div class="form-tip">为 0 则默认使用薪资作为基数</div>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="submitForm" :loading="submitting">确定</el-button>
      </template>
    </el-dialog>

    <!-- 批量导入对话框 -->
    <el-dialog v-model="importDialogVisible" title="批量导入员工" width="500px">
      <el-alert title="请准备 CSV 文件，格式：姓名,手机号,岗位,身份证号,入职日期,薪资" type="info" :closable="false" show-icon style="margin-bottom: 16px;" />
      <el-upload
        drag
        accept=".csv,.xlsx"
        :auto-upload="false"
        :on-change="handleFileChange"
        :limit="1"
      >
        <el-icon class="el-icon--upload"><upload-filled /></el-icon>
        <div class="el-upload__text">拖拽文件到此处或 <em>点击选择</em></div>
      </el-upload>
      <template #footer>
        <el-button @click="importDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="importEmployees" :disabled="!importFile">开始导入</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import { UploadFilled } from '@element-plus/icons-vue'
import type { FormInstance, UploadFile } from 'element-plus'

interface Employee {
  id: number
  name: string
  phone: string
  position: string
  id_card: string
  hire_date: string
  salary: number
  insurance_base: number
  housing_fund_base: number
  is_active: boolean
  created_at: string
  updated_at: string
}

const employees = ref<Employee[]>([])
const loading = ref(false)
const submitting = ref(false)
const searchKeyword = ref('')
const dialogVisible = ref(false)
const importDialogVisible = ref(false)
const isEdit = ref(false)
const editingId = ref<number | null>(null)
const formRef = ref<FormInstance>()
const importFile = ref<File | null>(null)

const filteredEmployees = computed(() => {
  if (!searchKeyword.value) return employees.value
  const keyword = searchKeyword.value.toLowerCase()
  return employees.value.filter(
    e => e.name.toLowerCase().includes(keyword) || e.phone.includes(keyword)
  )
})

const form = ref({
  name: '',
  phone: '',
  position: '',
  id_card: '',
  hire_date: '',
  salary: 0,
  insurance_base: 0,
  housing_fund_base: 0,
})

const rules = {
  name: [{ required: true, message: '请输入姓名', trigger: 'blur' }],
  phone: [
    { required: true, message: '请输入手机号', trigger: 'blur' },
    { pattern: /^1[3-9]\d{9}$/, message: '手机号格式不正确', trigger: 'blur' },
  ],
}

const fetchEmployees = async () => {
  loading.value = true
  try {
    employees.value = await invoke('get_all_employees')
  } catch (e) {
    ElMessage.error('获取员工列表失败: ' + e)
  } finally {
    loading.value = false
  }
}

const showAddDialog = () => {
  isEdit.value = false
  editingId.value = null
  form.value = { name: '', phone: '', position: '', id_card: '', hire_date: '', salary: 0, insurance_base: 0, housing_fund_base: 0 }
  dialogVisible.value = true
}

const editEmployee = (employee: Employee) => {
  isEdit.value = true
  editingId.value = employee.id
  form.value = {
    name: employee.name,
    phone: employee.phone,
    position: employee.position,
    id_card: employee.id_card,
    hire_date: employee.hire_date,
    salary: employee.salary,
    insurance_base: employee.insurance_base,
    housing_fund_base: employee.housing_fund_base,
  }
  dialogVisible.value = true
}

const submitForm = async () => {
  if (!formRef.value) return
  const valid = await formRef.value.validate().catch(() => false)
  if (!valid) return

  submitting.value = true
  try {
    if (isEdit.value && editingId.value) {
      await invoke('update_employee', {
        input: {
          id: editingId.value,
          name: form.value.name,
          phone: form.value.phone,
          position: form.value.position,
          id_card: form.value.id_card,
          hire_date: form.value.hire_date,
          salary: form.value.salary,
          insurance_base: form.value.insurance_base,
          housing_fund_base: form.value.housing_fund_base,
        },
      })
      ElMessage.success('员工信息已更新')
    } else {
      await invoke('create_employee', { input: form.value })
      ElMessage.success('员工添加成功')
    }
    dialogVisible.value = false
    await fetchEmployees()
  } catch (e) {
    ElMessage.error('操作失败: ' + e)
  } finally {
    submitting.value = false
  }
}

const toggleActive = async (employee: Employee) => {
  try {
    await ElMessageBox.confirm(
      `确定要将 ${employee.name} ${employee.is_active ? '离职' : '复职'} 吗？`,
      '提示'
    )
    await invoke('update_employee', {
      input: { id: employee.id, is_active: !employee.is_active },
    })
    ElMessage.success('状态已更新')
    await fetchEmployees()
  } catch {
    // 取消操作
  }
}

const deleteEmployee = async (id: number) => {
  try {
    await ElMessageBox.confirm('确定要删除该员工记录吗？', '提示', { type: 'warning' })
    await invoke('delete_employee', { id })
    ElMessage.success('员工已删除')
    await fetchEmployees()
  } catch {
    // 取消操作
  }
}

const showImportDialog = () => {
  importFile.value = null
  importDialogVisible.value = true
}

const handleFileChange = (uploadFile: UploadFile) => {
  if (uploadFile.raw) {
    importFile.value = uploadFile.raw
  }
}

const importEmployees = async () => {
  if (!importFile.value) {
    ElMessage.warning('请选择文件')
    return
  }
  // 简化：实际应解析 CSV 并批量调用 create_employee
  ElMessage.success('批量导入功能待完善，请手动添加员工')
  importDialogVisible.value = false
}

onMounted(fetchEmployees)
</script>

<style scoped>
.form-tip {
  font-size: 12px;
  color: #909399;
  margin-top: 2px;
}
</style>