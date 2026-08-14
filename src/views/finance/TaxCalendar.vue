<template>
  <div>
    <el-row :gutter="20">
      <!-- 统计卡片 -->
      <el-col :span="24" style="margin-bottom: 20px;">
        <el-row :gutter="16">
          <el-col :span="6">
            <el-card shadow="hover">
              <div class="stat-card">
                <div class="stat-value" style="color: #409eff;">{{ pendingCount }}</div>
                <div class="stat-title">待申报</div>
              </div>
            </el-card>
          </el-col>
          <el-col :span="6">
            <el-card shadow="hover">
              <div class="stat-card">
                <div class="stat-value" style="color: #e6a23c;">{{ urgentCount }}</div>
                <div class="stat-title">7天内到期</div>
              </div>
            </el-card>
          </el-col>
          <el-col :span="6">
            <el-card shadow="hover">
              <div class="stat-card">
                <div class="stat-value" style="color: #f56c6c;">{{ overdueCount }}</div>
                <div class="stat-title">已逾期</div>
              </div>
            </el-card>
          </el-col>
          <el-col :span="6">
            <el-card shadow="hover">
              <div class="stat-card">
                <div class="stat-value" style="color: #67c23a;">{{ submittedCount }}</div>
                <div class="stat-title">已申报</div>
              </div>
            </el-card>
          </el-col>
        </el-row>
      </el-col>

      <!-- 筛选与操作 -->
      <el-col :span="24">
        <el-card>
          <template #header>
            <span>税务日历</span>
            <div style="float: right;">
              <el-select v-model="selectedYear" style="width: 120px; margin-right: 8px;">
                <el-option v-for="y in years" :key="y" :label="`${y}年`" :value="y" />
              </el-select>
              <el-select v-model="statusFilter" placeholder="状态筛选" clearable style="width: 130px; margin-right: 8px;">
                <el-option label="待申报" value="pending" />
                <el-option label="即将到期" value="urgent" />
                <el-option label="已逾期" value="overdue" />
                <el-option label="已申报" value="submitted" />
              </el-select>
              <el-button type="primary" @click="refreshCalendar">刷新日历</el-button>
              <el-button type="success" @click="generateCalendar">生成全年日历</el-button>
              <el-button type="warning" @click="openCustomItems">自定义事项</el-button>
            </div>
          </template>

          <el-table :data="filteredCalendar" stripe v-loading="loading" style="width: 100%;">
            <el-table-column prop="tax_type_label" label="税种" width="120">
              <template #default="scope">
                <el-tag :type="getTaxTypeTag(scope.row.tax_type)" size="small">
                  {{ scope.row.tax_type_label }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="period_label" label="所属期间" width="150" />
            <el-table-column prop="due_date" label="截止日期" width="120" />
            <el-table-column label="倒计时" width="120">
              <template #default="scope">
                <el-tag :type="getDaysTag(scope.row.days_remaining)" size="small">
                  {{ getDaysText(scope.row.days_remaining) }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="tax_amount" label="税额" width="120">
              <template #default="scope">
                <span v-if="scope.row.is_custom">-</span>
                <span v-else>¥{{ scope.row.tax_amount.toFixed(2) }}</span>
              </template>
            </el-table-column>
            <el-table-column label="状态" width="100">
              <template #default="scope">
                <el-tag :type="getStatusTag(scope.row.status)" size="small">
                  {{ getStatusText(scope.row.status) }}
                </el-tag>
              </template>
            </el-table-column>
            <el-table-column label="操作" width="220">
              <template #default="scope">
                <template v-if="scope.row.is_custom">
                  <el-button
                    v-if="scope.row.status !== 'submitted'"
                    size="small"
                    type="success"
                    @click="markCustomSubmitted(scope.row)"
                  >
                    标记完成
                  </el-button>
                  <el-button v-else size="small" @click="markCustomSubmitted(scope.row)">
                    撤销
                  </el-button>
                </template>
                <template v-else>
                  <el-button
                    v-if="scope.row.status === 'pending' || scope.row.status === 'overdue'"
                    size="small"
                    type="success"
                    @click="markSubmitted(scope.row)"
                  >
                    标记已申报
                  </el-button>
                  <el-button
                    v-if="!scope.row.is_reminded && scope.row.status !== 'submitted'"
                    size="small"
                    type="warning"
                    @click="markReminded(scope.row.id)"
                  >
                    标记已提醒
                  </el-button>
                </template>
              </template>
            </el-table-column>
          </el-table>

          <el-empty v-if="filteredCalendar.length === 0 && !loading" description="暂无税务日历数据" />
        </el-card>
      </el-col>

      <!-- 逾期提醒弹窗 -->
      <el-dialog v-model="reminderDialogVisible" title="税务申报提醒" width="500px" :close-on-click-modal="false">
        <el-alert
          v-if="reminderData.overdue_count > 0"
          title="有已逾期的申报项目！"
          type="error"
          :description="`${reminderData.overdue_count} 项已逾期，请尽快处理。`"
          show-icon
          :closable="false"
          style="margin-bottom: 16px;"
        />
        <el-alert
          v-if="reminderData.urgent_count > 0"
          title="有即将到期的申报项目"
          type="warning"
          :description="`${reminderData.urgent_count} 项将在7天内到期。`"
          show-icon
          :closable="false"
          style="margin-bottom: 16px;"
        />
        <el-table :data="reminderData.items" stripe size="small" max-height="300">
          <el-table-column prop="tax_type_label" label="税种" width="80" />
          <el-table-column prop="period_label" label="期间" width="120" />
          <el-table-column prop="due_date" label="截止日" width="100" />
          <el-table-column label="状态" width="80">
            <template #default="scope">
              <el-tag :type="scope.row.days_remaining < 0 ? 'danger' : 'warning'" size="small">
                {{ scope.row.days_remaining < 0 ? '逾期' : `${scope.row.days_remaining}天` }}
              </el-tag>
            </template>
          </el-table-column>
        </el-table>
        <template #footer>
          <el-button @click="reminderDialogVisible = false">我知道了</el-button>
        </template>
      </el-dialog>

      <!-- 自定义事项管理弹窗 -->
      <el-dialog v-model="customItemsDialogVisible" title="自定义税务事项" width="760px">
        <el-alert
          title="不同企业类型的待办税务事项不同，例如小微企业一般涉及印花税（每年1月1日—20日申报，逾期罚款）。可在此新增、修改或停用重要事项。"
          type="info"
          show-icon
          :closable="false"
          style="margin-bottom: 16px;"
        />
        <div style="margin-bottom: 12px; text-align: right;">
          <el-button type="primary" size="small" @click="openCustomForm()">新增事项</el-button>
        </div>
        <el-table :data="customItems" stripe size="small" max-height="400">
          <el-table-column prop="name" label="事项名称" min-width="150" />
          <el-table-column label="申报频率" width="90">
            <template #default="scope">{{ frequencyLabel(scope.row.frequency) }}</template>
          </el-table-column>
          <el-table-column label="截止日期" width="150">
            <template #default="scope">{{ customDueText(scope.row) }}</template>
          </el-table-column>
          <el-table-column prop="remark" label="备注" min-width="140" show-overflow-tooltip />
          <el-table-column label="启用" width="70">
            <template #default="scope">
              <el-tag :type="scope.row.is_enabled ? 'success' : 'info'" size="small">
                {{ scope.row.is_enabled ? '启用' : '停用' }}
              </el-tag>
            </template>
          </el-table-column>
          <el-table-column label="操作" width="130">
            <template #default="scope">
              <el-button size="small" @click="openCustomForm(scope.row)">编辑</el-button>
              <el-button size="small" type="danger" @click="deleteCustomItem(scope.row.id)">删除</el-button>
            </template>
          </el-table-column>
        </el-table>
        <el-empty v-if="customItems.length === 0" description="暂无自定义事项，点击右上角新增" />
      </el-dialog>

      <!-- 自定义事项编辑弹窗 -->
      <el-dialog v-model="customFormVisible" :title="customEditId ? '编辑事项' : '新增事项'" width="480px">
        <el-form :model="customForm" label-width="110px">
          <el-form-item label="事项名称">
            <el-input v-model="customForm.name" placeholder="如：印花税、工商年报、汇算清缴" />
          </el-form-item>
          <el-form-item label="申报频率">
            <el-radio-group v-model="customForm.frequency">
              <el-radio label="annual">每年</el-radio>
              <el-radio label="quarterly">每季度</el-radio>
              <el-radio label="monthly">每月</el-radio>
              <el-radio label="once">一次性</el-radio>
            </el-radio-group>
          </el-form-item>
          <el-form-item v-if="customForm.frequency === 'once'" label="申报日期">
            <el-date-picker
              v-model="customForm.once_date"
              type="date"
              value-format="YYYY-MM-DD"
              placeholder="选择日期"
            />
          </el-form-item>
          <template v-else>
            <el-form-item v-if="customForm.frequency !== 'monthly'" label="申报月份">
              <el-select v-model="customForm.due_month" style="width: 120px">
                <el-option v-for="m in 12" :key="m" :label="`${m}月`" :value="m" />
              </el-select>
              <span v-if="customForm.frequency === 'quarterly'" class="form-tip">按此月及每后推3个月申报</span>
            </el-form-item>
            <el-form-item label="截止日">
              <el-select v-model="customForm.due_day" style="width: 120px">
                <el-option v-for="d in 31" :key="d" :label="`${d}日`" :value="d" />
              </el-select>
            </el-form-item>
          </template>
          <el-form-item label="备注">
            <el-input v-model="customForm.remark" type="textarea" :rows="2" placeholder="填写说明，如逾期罚款规则" />
          </el-form-item>
          <el-form-item label="是否启用">
            <el-switch v-model="customForm.is_enabled" />
          </el-form-item>
        </el-form>
        <template #footer>
          <el-button @click="customFormVisible = false">取消</el-button>
          <el-button type="primary" @click="submitCustomItem">确定</el-button>
        </template>
      </el-dialog>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import dayjs from 'dayjs'

// ==================== 类型定义 ====================

interface TaxCalendarItem {
  id: number
  tax_type: string
  tax_type_label: string
  period: string
  period_label: string
  due_date: string
  days_remaining: number
  status: string
  is_reminded: boolean
  tax_amount: number
  is_custom: boolean
  custom_item_id: number | null
}

interface CustomTaxItem {
  id: number
  name: string
  frequency: string
  due_month: number | null
  due_day: number
  once_date: string | null
  remark: string
  is_enabled: boolean
}

interface ReminderData {
  items: TaxCalendarItem[]
  urgent_count: number
  overdue_count: number
}

// ==================== 状态管理 ====================

const loading = ref(false)
const selectedYear = ref(dayjs().year())
const statusFilter = ref('')
const reminderDialogVisible = ref(false)
const reminderData = ref<ReminderData>({ items: [], urgent_count: 0, overdue_count: 0 })
const calendarItems = ref<TaxCalendarItem[]>([])

const customItems = ref<CustomTaxItem[]>([])
const customItemsDialogVisible = ref(false)
const customFormVisible = ref(false)
const customEditId = ref<number | null>(null)
const customForm = reactive({
  name: "",
  frequency: "annual",
  due_month: 1,
  due_day: 15,
  once_date: "",
  remark: "",
  is_enabled: true,
})

const years = computed(() => {
  const currentYear = dayjs().year()
  return [currentYear - 1, currentYear, currentYear + 1]
})

// ==================== 计算属性 ====================

const pendingCount = computed(() => 
  calendarItems.value.filter(i => i.status === 'pending').length
)

const urgentCount = computed(() => 
  calendarItems.value.filter(i => i.days_remaining >= 0 && i.days_remaining <= 7 && i.status !== 'submitted').length
)

const overdueCount = computed(() => 
  calendarItems.value.filter(i => i.status === 'overdue').length
)

const submittedCount = computed(() => 
  calendarItems.value.filter(i => i.status === 'submitted').length
)

const filteredCalendar = computed(() => {
  if (!statusFilter.value) return calendarItems.value
  if (statusFilter.value === 'urgent') {
    return calendarItems.value.filter(
      i => i.days_remaining >= 0 && i.days_remaining <= 7 && i.status !== 'submitted'
    )
  }
  return calendarItems.value.filter(i => i.status === statusFilter.value)
})

// ==================== 方法 ====================

const getTaxTypeTag = (type: string) => {
  const map: Record<string, string> = {
    vat: 'danger',
    cit: 'warning',
    surcharge: 'info',
    personal_income: 'success',
    custom: 'primary',
  }
  return map[type] || ''
}

const getDaysTag = (days: number) => {
  if (days < 0) return 'danger'
  if (days <= 7) return 'warning'
  if (days <= 30) return 'info'
  return 'success'
}

const getDaysText = (days: number) => {
  if (days < 0) return `已逾期 ${Math.abs(days)} 天`
  if (days === 0) return '今天到期'
  return `剩余 ${days} 天`
}

const getStatusTag = (status: string) => {
  const map: Record<string, string> = {
    pending: 'info',
    submitted: 'success',
    overdue: 'danger',
  }
  return map[status] || ''
}

const getStatusText = (status: string) => {
  const map: Record<string, string> = {
    pending: '待申报',
    submitted: '已申报',
    overdue: '已逾期',
  }
  return map[status] || status
}

const fetchCalendar = async () => {
  loading.value = true
  try {
    calendarItems.value = await invoke('get_tax_calendar', { year: selectedYear.value })
  } catch (e) {
    console.error('获取税务日历失败:', e)
  } finally {
    loading.value = false
  }
}

const generateCalendar = async () => {
  try {
    await invoke('generate_tax_calendar', { year: selectedYear.value })
    ElMessage.success('全年税务日历已生成')
    await fetchCalendar()
  } catch (e) {
    ElMessage.error('生成失败: ' + e)
  }
}

const refreshCalendar = () => {
  fetchCalendar()
  checkReminders()
}

const markSubmitted = async (item: TaxCalendarItem) => {
  try {
    await invoke('mark_as_submitted', { id: item.id })
    ElMessage.success(`${item.tax_type_label} ${item.period_label} 已标记为已申报`)
    await fetchCalendar()
  } catch (e) {
    ElMessage.error('操作失败: ' + e)
  }
}

const markReminded = async (id: number) => {
  try {
    await invoke('mark_as_reminded', { id })
    ElMessage.success('已标记提醒')
    await fetchCalendar()
  } catch (e) {
    ElMessage.error('操作失败: ' + e)
  }
}

// ==================== 自定义事项管理 ====================

const frequencyLabel = (frequency: string) => {
  const map: Record<string, string> = {
    monthly: '每月',
    quarterly: '每季度',
    annual: '每年',
    once: '一次性',
  }
  return map[frequency] || frequency
}

const customDueText = (item: CustomTaxItem) => {
  if (item.frequency === 'once') {
    return item.once_date || '-'
  }
  if (item.frequency === 'monthly') {
    return `每月 ${item.due_day} 日`
  }
  if (item.frequency === 'quarterly') {
    return `每季度 ${item.due_month ?? 1} 月 ${item.due_day} 日`
  }
  return `每年 ${item.due_month ?? 1} 月 ${item.due_day} 日`
}

const fetchCustomItems = async () => {
  try {
    customItems.value = await invoke('get_custom_tax_items')
  } catch (e) {
    console.error('获取自定义事项失败:', e)
  }
}

const openCustomItems = () => {
  customItemsDialogVisible.value = true
  fetchCustomItems()
}

const openCustomForm = (item?: CustomTaxItem) => {
  if (item) {
    customEditId.value = item.id
    Object.assign(customForm, {
      name: item.name,
      frequency: item.frequency,
      due_month: item.due_month ?? 1,
      due_day: item.due_day,
      once_date: item.once_date || '',
      remark: item.remark,
      is_enabled: item.is_enabled,
    })
  } else {
    customEditId.value = null
    Object.assign(customForm, {
      name: '',
      frequency: 'annual',
      due_month: 1,
      due_day: 15,
      once_date: '',
      remark: '',
      is_enabled: true,
    })
  }
  customFormVisible.value = true
}

const submitCustomItem = async () => {
  if (!customForm.name.trim()) {
    ElMessage.warning('请输入事项名称')
    return
  }
  if (customForm.frequency === 'once' && !customForm.once_date) {
    ElMessage.warning('请选择申报日期')
    return
  }
  const input = {
    name: customForm.name.trim(),
    frequency: customForm.frequency,
    due_month: customForm.frequency === 'monthly' ? null : customForm.due_month,
    due_day: customForm.due_day,
    once_date: customForm.frequency === 'once' ? customForm.once_date : null,
    remark: customForm.remark,
    is_enabled: customForm.is_enabled,
  }
  try {
    if (customEditId.value) {
      await invoke('update_custom_tax_item', { id: customEditId.value, input })
      ElMessage.success('事项已更新')
    } else {
      await invoke('create_custom_tax_item', { input })
      ElMessage.success('事项已添加')
    }
    customFormVisible.value = false
    await fetchCustomItems()
    await fetchCalendar()
  } catch (e) {
    ElMessage.error('保存失败: ' + e)
  }
}

const deleteCustomItem = async (id: number) => {
  try {
    await invoke('delete_custom_tax_item', { id })
    ElMessage.success('已删除')
    await fetchCustomItems()
    await fetchCalendar()
  } catch (e) {
    ElMessage.error('删除失败: ' + e)
  }
}

const markCustomSubmitted = async (item: TaxCalendarItem) => {
  try {
    const submitted = item.status !== 'submitted'
    await invoke('mark_custom_item_submitted', {
      itemId: item.custom_item_id,
      period: item.period,
      submitted,
    })
    ElMessage.success(submitted ? '已标记完成' : '已撤销')
    await fetchCalendar()
  } catch (e) {
    ElMessage.error('操作失败: ' + e)
  }
}

const checkReminders = async () => {
  try {
    const data: ReminderData = await invoke('check_upcoming_deadlines')
    reminderData.value = data
    if (data.overdue_count > 0 || data.urgent_count > 0) {
      reminderDialogVisible.value = true
    }
  } catch (e) {
    console.error('检查提醒失败:', e)
  }
}

onMounted(async () => {
  await fetchCalendar()
  // 如果没有数据，自动生成
  if (calendarItems.value.length === 0) {
    await generateCalendar()
  }
  await checkReminders()
})
</script>

<style scoped>
.stat-card {
  text-align: center;
  padding: 8px 0;
}
.stat-value {
  font-size: 32px;
  font-weight: bold;
}
.stat-title {
  font-size: 14px;
  color: #909399;
  margin-top: 8px;
}
.form-tip {
  font-size: 12px;
  color: #909399;
  margin-left: 8px;
}
</style>