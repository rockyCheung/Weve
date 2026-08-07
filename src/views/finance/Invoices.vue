<template>
  <div>
    <el-tabs v-model="activeTab" @tab-change="handleTabChange">
      <!-- 销项发票 -->
      <el-tab-pane label="销项发票" name="output">
        <el-card>
          <template #header>
            <span>销项发票管理</span>
            <el-button size="small" type="primary" style="float: right;" @click="showAddDialog('output')">
              添加销项发票
            </el-button>
          </template>
          
          <el-form :inline="true" :model="queryForm" style="margin-bottom: 16px;">
            <el-form-item label="开始日期">
              <el-date-picker v-model="queryForm.start_date" type="date" value-format="YYYY-MM-DD" />
            </el-form-item>
            <el-form-item label="结束日期">
              <el-date-picker v-model="queryForm.end_date" type="date" value-format="YYYY-MM-DD" />
            </el-form-item>
            <el-form-item label="搜索">
              <el-input v-model="queryForm.keyword" placeholder="发票号/购买方" clearable style="width: 200px;" />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="fetchInvoices">查询</el-button>
              <el-button @click="showSummary">统计</el-button>
            </el-form-item>
          </el-form>

          <el-table :data="invoiceList" stripe v-loading="loading" style="width: 100%;">
            <el-table-column prop="invoice_no" label="发票号码" width="160" />
            <el-table-column prop="date" label="开票日期" width="100" />
            <el-table-column prop="buyer_name" label="购买方名称" min-width="150" />
            <el-table-column prop="buyer_tax_id" label="购买方税号" width="160" />
            <el-table-column prop="amount" label="不含税金额" width="120">
              <template #default="scope">¥{{ scope.row.amount.toFixed(2) }}</template>
            </el-table-column>
            <el-table-column prop="tax_rate" label="税率" width="80">
              <template #default="scope">{{ (scope.row.tax_rate * 100).toFixed(0) }}%</template>
            </el-table-column>
            <el-table-column prop="tax_amount" label="税额" width="120">
              <template #default="scope">¥{{ scope.row.tax_amount.toFixed(2) }}</template>
            </el-table-column>
            <el-table-column prop="total_amount" label="价税合计" width="120">
              <template #default="scope">¥{{ scope.row.total_amount.toFixed(2) }}</template>
            </el-table-column>
            <el-table-column label="操作" width="100" fixed="right">
              <template #default="scope">
                <el-button size="small" type="danger" @click="deleteInvoice(scope.row.id)">删除</el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-tab-pane>

      <!-- 进项发票 -->
      <el-tab-pane label="进项发票" name="input">
        <el-card>
          <template #header>
            <span>进项发票管理</span>
            <el-button size="small" type="primary" style="float: right;" @click="showAddDialog('input')">
              添加进项发票
            </el-button>
          </template>
          
          <el-form :inline="true" :model="queryForm" style="margin-bottom: 16px;">
            <el-form-item label="开始日期">
              <el-date-picker v-model="queryForm.start_date" type="date" value-format="YYYY-MM-DD" />
            </el-form-item>
            <el-form-item label="结束日期">
              <el-date-picker v-model="queryForm.end_date" type="date" value-format="YYYY-MM-DD" />
            </el-form-item>
            <el-form-item label="认证状态">
              <el-select v-model="verifyFilter" placeholder="全部" clearable style="width: 120px;">
                <el-option label="已认证" :value="true" />
                <el-option label="未认证" :value="false" />
              </el-select>
            </el-form-item>
            <el-form-item label="搜索">
              <el-input v-model="queryForm.keyword" placeholder="发票号/销售方" clearable style="width: 200px;" />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="fetchInvoices">查询</el-button>
              <el-button @click="showSummary">统计</el-button>
            </el-form-item>
          </el-form>

          <el-table :data="filteredInputInvoices" stripe v-loading="loading" style="width: 100%;">
            <el-table-column prop="invoice_no" label="发票号码" width="160" />
            <el-table-column prop="date" label="开票日期" width="100" />
            <el-table-column prop="buyer_name" label="销售方名称" min-width="150" />
            <el-table-column prop="buyer_tax_id" label="销售方税号" width="160" />
            <el-table-column prop="amount" label="不含税金额" width="120">
              <template #default="scope">¥{{ scope.row.amount.toFixed(2) }}</template>
            </el-table-column>
            <el-table-column prop="tax_rate" label="税率" width="80">
              <template #default="scope">{{ (scope.row.tax_rate * 100).toFixed(0) }}%</template>
            </el-table-column>
            <el-table-column prop="tax_amount" label="税额" width="120">
              <template #default="scope">¥{{ scope.row.tax_amount.toFixed(2) }}</template>
            </el-table-column>
            <el-table-column label="认证状态" width="100">
              <template #default="scope">
                <el-switch
                  v-model="scope.row.is_verified"
                  active-text="已认证"
                  inactive-text="未认证"
                  @change="(val: boolean) => toggleVerify(scope.row.id, val)"
                />
              </template>
            </el-table-column>
            <el-table-column label="操作" width="100" fixed="right">
              <template #default="scope">
                <el-button size="small" type="danger" @click="deleteInvoice(scope.row.id)">删除</el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-tab-pane>

      <!-- 发票统计 -->
      <el-tab-pane label="发票统计" name="summary">
        <el-card>
          <template #header>
            <span>发票统计</span>
          </template>
          <el-form :inline="true" :model="summaryForm">
            <el-form-item label="发票类型">
              <el-select v-model="summaryForm.invoice_type" style="width: 150px;">
                <el-option label="销项发票" value="output" />
                <el-option label="进项发票" value="input" />
              </el-select>
            </el-form-item>
            <el-form-item label="开始日期">
              <el-date-picker v-model="summaryForm.start_date" type="date" value-format="YYYY-MM-DD" />
            </el-form-item>
            <el-form-item label="结束日期">
              <el-date-picker v-model="summaryForm.end_date" type="date" value-format="YYYY-MM-DD" />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="fetchSummary">统计</el-button>
            </el-form-item>
          </el-form>

          <div v-if="summaryData" style="margin-top: 16px;">
            <el-descriptions :column="3" border>
              <el-descriptions-item label="发票数量">{{ summaryData.total_count }} 张</el-descriptions-item>
              <el-descriptions-item label="不含税金额">¥{{ summaryData.total_amount.toFixed(2) }}</el-descriptions-item>
              <el-descriptions-item label="税额合计">¥{{ summaryData.total_tax.toFixed(2) }}</el-descriptions-item>
              <el-descriptions-item label="价税合计" :span="3">
                <el-tag type="danger" size="large">¥{{ summaryData.total_with_tax.toFixed(2) }}</el-tag>
              </el-descriptions-item>
            </el-descriptions>

            <el-table :data="summaryData.by_rate" stripe style="margin-top: 16px;" border>
              <el-table-column label="税率" width="120">
                <template #default="scope">{{ (scope.row.tax_rate * 100).toFixed(0) }}%</template>
              </el-table-column>
              <el-table-column prop="count" label="数量" width="100" />
              <el-table-column prop="amount" label="不含税金额" width="150">
                <template #default="scope">¥{{ scope.row.amount.toFixed(2) }}</template>
              </el-table-column>
              <el-table-column prop="tax" label="税额" width="150">
                <template #default="scope">¥{{ scope.row.tax.toFixed(2) }}</template>
              </el-table-column>
            </el-table>
          </div>
        </el-card>
      </el-tab-pane>

      <!-- 到期提醒 -->
      <el-tab-pane label="到期提醒" name="reminders">
        <el-card>
          <template #header>
            <span>进项发票认证到期提醒</span>
          </template>
          <el-alert
            title="进项发票需在开具之日起 360 天内完成认证，逾期将无法抵扣进项税额。"
            type="warning"
            :closable="false"
            show-icon
            style="margin-bottom: 16px;"
          />
          <el-table :data="dueReminders" stripe v-loading="loadingReminders">
            <el-table-column prop="invoice_no" label="发票号码" width="160" />
            <el-table-column prop="date" label="开票日期" width="120" />
            <el-table-column prop="buyer_name" label="销售方" min-width="150" />
            <el-table-column prop="amount" label="金额" width="120">
              <template #default="scope">¥{{ scope.row.amount.toFixed(2) }}</template>
            </el-table-column>
            <el-table-column prop="due_days" label="剩余天数" width="120">
              <template #default="scope">
                <el-tag :type="scope.row.due_days <= 30 ? 'danger' : scope.row.due_days <= 90 ? 'warning' : 'info'">
                  {{ scope.row.due_days <= 0 ? '已逾期' : `${scope.row.due_days} 天` }}
                </el-tag>
              </template>
            </el-table-column>
          </el-table>
          <el-empty v-if="dueReminders.length === 0 && !loadingReminders" description="暂无到期提醒" />
        </el-card>
      </el-tab-pane>
    </el-tabs>

    <!-- 添加发票对话框 -->
    <el-dialog v-model="dialogVisible" :title="dialogTitle" width="500px">
      <el-form :model="invoiceForm" label-width="110px">
        <el-form-item label="发票号码" required>
          <el-input v-model="invoiceForm.invoice_no" />
        </el-form-item>
        <el-form-item label="开票日期" required>
          <el-date-picker v-model="invoiceForm.date" type="date" value-format="YYYY-MM-DD" style="width: 100%;" />
        </el-form-item>
        <el-form-item :label="activeTab === 'output' ? '购买方名称' : '销售方名称'" required>
          <el-input v-model="invoiceForm.buyer_name" />
        </el-form-item>
        <el-form-item :label="activeTab === 'output' ? '购买方税号' : '销售方税号'" required>
          <el-input v-model="invoiceForm.buyer_tax_id" />
        </el-form-item>
        <el-form-item label="不含税金额" required>
          <el-input-number v-model="invoiceForm.amount" :min="0.01" :precision="2" style="width: 100%;" />
        </el-form-item>
        <el-form-item label="税率" required>
          <el-select v-model="invoiceForm.tax_rate" style="width: 100%;">
            <el-option label="13%" :value="0.13" />
            <el-option label="9%" :value="0.09" />
            <el-option label="6%" :value="0.06" />
            <el-option label="3%" :value="0.03" />
            <el-option label="0%" :value="0.0" />
          </el-select>
        </el-form-item>
        <el-form-item label="税额（自动计算）">
          <el-input :value="(invoiceForm.amount * invoiceForm.tax_rate).toFixed(2)" disabled />
        </el-form-item>
        <el-form-item label="价税合计（自动计算）">
          <el-input :value="(invoiceForm.amount * (1 + invoiceForm.tax_rate)).toFixed(2)" disabled />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="dialogVisible = false">取消</el-button>
        <el-button type="primary" @click="submitInvoice" :loading="submitting">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import dayjs from 'dayjs'

// ==================== 类型定义 ====================

interface InvoiceRecord {
  id: number
  invoice_type: string
  invoice_no: string
  date: string
  buyer_name: string
  buyer_tax_id: string
  amount: number
  tax_rate: number
  tax_amount: number
  total_amount: number
  is_verified: boolean
  created_at: string
}

interface InvoiceSummary {
  total_count: number
  total_amount: number
  total_tax: number
  total_with_tax: number
  by_rate: { tax_rate: number; count: number; amount: number; tax: number }[]
}

interface DueReminder {
  id: number
  invoice_no: string
  date: string
  buyer_name: string
  amount: number
  due_days: number
}

// ==================== 状态管理 ====================

const activeTab = ref('output')
const loading = ref(false)
const loadingReminders = ref(false)
const submitting = ref(false)
const dialogVisible = ref(false)
const dialogTitle = ref('添加发票')

const queryForm = reactive({
  start_date: dayjs().startOf('month').format('YYYY-MM-DD'),
  end_date: dayjs().format('YYYY-MM-DD'),
  keyword: '',
})

const summaryForm = reactive({
  invoice_type: 'output',
  start_date: dayjs().startOf('year').format('YYYY-MM-DD'),
  end_date: dayjs().format('YYYY-MM-DD'),
})

const verifyFilter = ref<boolean | ''>('')
const invoiceList = ref<InvoiceRecord[]>([])
const summaryData = ref<InvoiceSummary | null>(null)
const dueReminders = ref<DueReminder[]>([])

const invoiceForm = reactive({
  invoice_no: '',
  date: '',
  buyer_name: '',
  buyer_tax_id: '',
  amount: 0,
  tax_rate: 0.13,
})

// ==================== 计算属性 ====================

const filteredInputInvoices = computed(() => {
  if (verifyFilter.value === '') return invoiceList.value
  return invoiceList.value.filter(inv => inv.is_verified === verifyFilter.value)
})

// ==================== 方法 ====================

const handleTabChange = (tab: string) => {
  if (tab === 'reminders') {
    fetchDueReminders()
  }
}

const fetchInvoices = async () => {
  loading.value = true
  try {
    invoiceList.value = await invoke('query_invoices', {
      query: {
        invoice_type: activeTab.value,
        start_date: queryForm.start_date,
        end_date: queryForm.end_date,
        keyword: queryForm.keyword || null,
      },
    })
  } catch (e) {
    ElMessage.error('查询发票失败: ' + e)
  } finally {
    loading.value = false
  }
}

const showAddDialog = (type: string) => {
  dialogTitle.value = type === 'output' ? '添加销项发票' : '添加进项发票'
  invoiceForm.invoice_no = ''
  invoiceForm.date = dayjs().format('YYYY-MM-DD')
  invoiceForm.buyer_name = ''
  invoiceForm.buyer_tax_id = ''
  invoiceForm.amount = 0
  invoiceForm.tax_rate = 0.13
  dialogVisible.value = true
}

const submitInvoice = async () => {
  if (!invoiceForm.invoice_no || !invoiceForm.date || !invoiceForm.buyer_name || !invoiceForm.buyer_tax_id || invoiceForm.amount <= 0) {
    ElMessage.warning('请完整填写发票信息')
    return
  }
  submitting.value = true
  try {
    await invoke('add_invoice', {
      input: {
        invoice_type: activeTab.value,
        invoice_no: invoiceForm.invoice_no,
        date: invoiceForm.date,
        buyer_name: invoiceForm.buyer_name,
        buyer_tax_id: invoiceForm.buyer_tax_id,
        amount: invoiceForm.amount,
        tax_rate: invoiceForm.tax_rate,
      },
    })
    ElMessage.success('发票添加成功')
    dialogVisible.value = false
    await fetchInvoices()
  } catch (e) {
    ElMessage.error('添加失败: ' + e)
  } finally {
    submitting.value = false
  }
}

const deleteInvoice = async (id: number) => {
  try {
    await invoke('delete_invoice', { id })
    ElMessage.success('发票已删除')
    await fetchInvoices()
  } catch (e) {
    ElMessage.error('删除失败: ' + e)
  }
}

const toggleVerify = async (id: number, verified: boolean) => {
  try {
    await invoke('verify_invoice', { id, verified })
    ElMessage.success(verified ? '已认证' : '已取消认证')
  } catch (e) {
    ElMessage.error('操作失败: ' + e)
  }
}

const showSummary = async () => {
  activeTab.value = 'summary'
  await fetchSummary()
}

const fetchSummary = async () => {
  try {
    summaryData.value = await invoke('get_invoice_summary', {
      invoiceType: summaryForm.invoice_type,
      startDate: summaryForm.start_date,
      endDate: summaryForm.end_date,
    })
  } catch (e) {
    ElMessage.error('统计失败: ' + e)
  }
}

const fetchDueReminders = async () => {
  loadingReminders.value = true
  try {
    dueReminders.value = await invoke('get_invoice_due_reminders', { daysAhead: 90 })
  } catch (e) {
    ElMessage.error('获取提醒失败: ' + e)
  } finally {
    loadingReminders.value = false
  }
}

onMounted(() => {
  fetchInvoices()
  fetchDueReminders()
})
</script>
