<template>
  <div>
    <el-tabs v-model="activeTab" @tab-change="handleTabChange">
      <!-- 增值税月报 -->
      <el-tab-pane label="增值税月报" name="vat">
        <el-card>
          <template #header>
            <span>增值税月报</span>
            <el-button size="small" type="primary" style="float: right;" @click="exportReport('vat_monthly')" :disabled="!vatReport">
              导出Excel
            </el-button>
          </template>
          <el-form :inline="true" :model="periodForm">
            <el-form-item label="所属月份">
              <el-date-picker v-model="periodForm.vatPeriod" type="month" value-format="YYYY-MM" placeholder="选择月份" />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="generateReport('vat_monthly')" :loading="loading.vat">生成报表</el-button>
            </el-form-item>
          </el-form>
          <div v-if="vatReport" style="margin-top: 16px;">
            <el-descriptions :column="2" border>
              <el-descriptions-item label="所属期">{{ vatReport.period }}</el-descriptions-item>
              <el-descriptions-item label="销项发票数">{{ vatReport.output_count }} 张</el-descriptions-item>
              <el-descriptions-item label="销项金额">¥{{ vatReport.output_amount.toFixed(2) }}</el-descriptions-item>
              <el-descriptions-item label="进项发票数">{{ vatReport.input_count }} 张</el-descriptions-item>
              <el-descriptions-item label="进项金额">¥{{ vatReport.input_amount.toFixed(2) }}</el-descriptions-item>
              <el-descriptions-item label="销项税额">¥{{ vatReport.output_tax.toFixed(2) }}</el-descriptions-item>
              <el-descriptions-item label="进项税额">¥{{ vatReport.input_tax.toFixed(2) }}</el-descriptions-item>
              <el-descriptions-item label="应缴增值税">¥{{ vatReport.tax_due.toFixed(2) }}</el-descriptions-item>
              <el-descriptions-item label="城建税(7%)">¥{{ vatReport.surcharge_city.toFixed(2) }}</el-descriptions-item>
              <el-descriptions-item label="教育费附加(3%)">¥{{ vatReport.surcharge_education.toFixed(2) }}</el-descriptions-item>
              <el-descriptions-item label="地方教育附加(2%)">¥{{ vatReport.surcharge_local.toFixed(2) }}</el-descriptions-item>
              <el-descriptions-item label="合计应缴">
                <el-tag type="danger" size="large">¥{{ vatReport.total_tax_due.toFixed(2) }}</el-tag>
              </el-descriptions-item>
            </el-descriptions>

            <!-- 按税率明细 -->
            <el-table :data="vatReport.items" stripe style="margin-top: 16px;" border size="small">
              <el-table-column label="税率" width="100">
                <template #default="scope">{{ (scope.row.tax_rate * 100).toFixed(0) }}%</template>
              </el-table-column>
              <el-table-column prop="output_amount" label="销项金额" width="150">
                <template #default="scope">¥{{ scope.row.output_amount.toFixed(2) }}</template>
              </el-table-column>
              <el-table-column prop="output_tax" label="销项税额" width="150">
                <template #default="scope">¥{{ scope.row.output_tax.toFixed(2) }}</template>
              </el-table-column>
              <el-table-column prop="input_amount" label="进项金额" width="150">
                <template #default="scope">¥{{ scope.row.input_amount.toFixed(2) }}</template>
              </el-table-column>
              <el-table-column prop="input_tax" label="进项税额" width="150">
                <template #default="scope">¥{{ scope.row.input_tax.toFixed(2) }}</template>
              </el-table-column>
            </el-table>
          </div>
          <el-empty v-else description="请选择月份并生成增值税月报" />
        </el-card>
      </el-tab-pane>

      <!-- 企业所得税季报 -->
      <el-tab-pane label="企业所得税季报" name="cit">
        <el-card>
          <template #header>
            <span>企业所得税季报</span>
            <el-button size="small" type="primary" style="float: right;" @click="exportReport('cit_quarterly')" :disabled="!citReport">
              导出Excel
            </el-button>
          </template>
          <el-form :inline="true" :model="periodForm">
            <el-form-item label="所属季度">
              <el-date-picker v-model="periodForm.citPeriod" type="month" value-format="YYYY-MM" placeholder="选择季度末月" />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="generateReport('cit_quarterly')" :loading="loading.cit">生成报表</el-button>
            </el-form-item>
          </el-form>
          <div v-if="citReport" style="margin-top: 16px;">
            <el-alert
              v-if="citReport.is_small_profit"
              title="享受小微企业税收优惠"
              type="success"
              :description="`适用优惠税率 ${(citReport.tax_rate * 100).toFixed(1)}%`"
              show-icon
              :closable="false"
              style="margin-bottom: 16px;"
            />
            <el-descriptions :column="2" border>
              <el-descriptions-item label="所属期">{{ citReport.period }}</el-descriptions-item>
              <el-descriptions-item label="营业收入">¥{{ citReport.revenue.toFixed(2) }}</el-descriptions-item>
              <el-descriptions-item label="营业成本">¥{{ citReport.cost.toFixed(2) }}</el-descriptions-item>
              <el-descriptions-item label="毛利">¥{{ citReport.gross_profit.toFixed(2) }}</el-descriptions-item>
              <el-descriptions-item label="期间费用">¥{{ citReport.expenses.toFixed(2) }}</el-descriptions-item>
              <el-descriptions-item label="利润总额">¥{{ citReport.profit.toFixed(2) }}</el-descriptions-item>
              <el-descriptions-item label="适用税率">{{ (citReport.tax_rate * 100).toFixed(1) }}%</el-descriptions-item>
              <el-descriptions-item label="应缴企业所得税">
                <el-tag type="danger" size="large">¥{{ citReport.tax_due.toFixed(2) }}</el-tag>
              </el-descriptions-item>
            </el-descriptions>
          </div>
          <el-empty v-else description="请选择季度并生成企业所得税季报" />
        </el-card>
      </el-tab-pane>

      <!-- 个人所得税 -->
      <el-tab-pane label="个人所得税" name="pit">
        <el-card>
          <template #header>
            <span>个人所得税代扣代缴汇总</span>
            <el-button size="small" type="primary" style="float: right;" @click="exportReport('personal_income')" :disabled="!pitReport">
              导出Excel
            </el-button>
          </template>
          <el-form :inline="true" :model="periodForm">
            <el-form-item label="所属月份">
              <el-date-picker v-model="periodForm.pitPeriod" type="month" value-format="YYYY-MM" placeholder="选择月份" />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="generateReport('personal_income')" :loading="loading.pit">生成报表</el-button>
            </el-form-item>
          </el-form>
          <div v-if="pitReport" style="margin-top: 16px;">
            <el-descriptions :column="3" border style="margin-bottom: 16px;">
              <el-descriptions-item label="所属月份">{{ pitReport.period }}</el-descriptions-item>
              <el-descriptions-item label="员工人数">{{ pitReport.employees.length }} 人</el-descriptions-item>
              <el-descriptions-item label="代扣代缴总额">
                <el-tag type="danger">¥{{ pitReport.total_tax.toFixed(2) }}</el-tag>
              </el-descriptions-item>
            </el-descriptions>
            <el-table :data="pitReport.employees" stripe border size="small">
              <el-table-column prop="employee_name" label="姓名" width="100" />
              <el-table-column prop="id_card" label="身份证号" width="180" />
              <el-table-column prop="income" label="收入额" width="120">
                <template #default="scope">¥{{ scope.row.income.toFixed(2) }}</template>
              </el-table-column>
              <el-table-column prop="deduction" label="减除费用" width="100">
                <template #default="scope">¥{{ scope.row.deduction.toFixed(2) }}</template>
              </el-table-column>
              <el-table-column prop="special_deduction" label="专项扣除" width="120">
                <template #default="scope">¥{{ scope.row.special_deduction.toFixed(2) }}</template>
              </el-table-column>
              <el-table-column prop="taxable_income" label="应纳税所得额" width="130">
                <template #default="scope">¥{{ scope.row.taxable_income.toFixed(2) }}</template>
              </el-table-column>
              <el-table-column prop="tax_rate" label="税率" width="80">
                <template #default="scope">{{ (scope.row.tax_rate * 100).toFixed(0) }}%</template>
              </el-table-column>
              <el-table-column prop="tax_due" label="应缴税额" width="120">
                <template #default="scope">¥{{ scope.row.tax_due.toFixed(2) }}</template>
              </el-table-column>
            </el-table>
          </div>
          <el-empty v-else description="请选择月份并生成个人所得税报表" />
        </el-card>
      </el-tab-pane>

      <!-- 财务报表 -->
      <el-tab-pane label="财务报表" name="financial">
        <el-card>
          <template #header>
            <span>财务报表</span>
            <el-button size="small" type="primary" style="float: right;" @click="exportReport('income_statement')" :disabled="!incomeStatement">
              导出利润表
            </el-button>
            <el-button size="small" type="primary" style="float: right; margin-right: 8px;" @click="exportReport('balance_sheet')" :disabled="!balanceSheet">
              导出资产负债表
            </el-button>
          </template>
          <el-form :inline="true" :model="periodForm">
            <el-form-item label="报表周期">
              <el-select v-model="periodForm.periodType" @change="onPeriodTypeChange" style="width: 110px">
                <el-option label="月报" value="monthly" />
                <el-option label="季报" value="quarterly" />
                <el-option label="半年报" value="semiannual" />
                <el-option label="年报" value="annual" />
              </el-select>
            </el-form-item>
            <el-form-item>
              <el-date-picker
                v-if="periodForm.periodType === 'monthly'"
                v-model="periodForm.financialPeriod"
                type="month"
                value-format="YYYY-MM"
                placeholder="选择月份"
              />
              <el-select
                v-else-if="periodForm.periodType === 'quarterly'"
                v-model="periodForm.financialPeriod"
                style="width: 140px"
              >
                <el-option
                  v-for="q in quarters"
                  :key="q.value"
                  :label="q.label"
                  :value="q.value"
                />
              </el-select>
              <el-select
                v-else-if="periodForm.periodType === 'semiannual'"
                v-model="periodForm.financialPeriod"
                style="width: 140px"
              >
                <el-option
                  v-for="h in semiannuals"
                  :key="h.value"
                  :label="h.label"
                  :value="h.value"
                />
              </el-select>
              <el-date-picker
                v-else
                v-model="periodForm.financialPeriod"
                type="year"
                value-format="YYYY"
                placeholder="选择年份"
              />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="generateReport('income_statement')" :loading="loading.income">生成利润表</el-button>
              <el-button type="primary" @click="generateReport('balance_sheet')" :loading="loading.balance">生成资产负债表</el-button>
              <el-button @click="generateReport('cash_flow')" :loading="loading.cashFlow">生成现金流量表</el-button>
            </el-form-item>
          </el-form>

          <el-tabs v-model="financialSubTab">
            <el-tab-pane label="利润表" name="income">
              <div v-if="incomeStatement">
                <el-descriptions :column="2" border>
                  <el-descriptions-item label="营业收入">¥{{ incomeStatement.revenue.toFixed(2) }}</el-descriptions-item>
                  <el-descriptions-item label="营业成本">¥{{ incomeStatement.cost.toFixed(2) }}</el-descriptions-item>
                  <el-descriptions-item label="毛利">¥{{ incomeStatement.gross_profit.toFixed(2) }}</el-descriptions-item>
                  <el-descriptions-item label="销售费用">¥{{ incomeStatement.selling_expenses.toFixed(2) }}</el-descriptions-item>
                  <el-descriptions-item label="管理费用">¥{{ incomeStatement.admin_expenses.toFixed(2) }}</el-descriptions-item>
                  <el-descriptions-item label="财务费用">¥{{ incomeStatement.finance_expenses.toFixed(2) }}</el-descriptions-item>
                  <el-descriptions-item label="营业利润">¥{{ incomeStatement.operating_profit.toFixed(2) }}</el-descriptions-item>
                  <el-descriptions-item label="营业外收入">¥{{ incomeStatement.non_operating_income.toFixed(2) }}</el-descriptions-item>
                  <el-descriptions-item label="营业外支出">¥{{ incomeStatement.non_operating_expenses.toFixed(2) }}</el-descriptions-item>
                  <el-descriptions-item label="利润总额">¥{{ incomeStatement.profit_before_tax.toFixed(2) }}</el-descriptions-item>
                  <el-descriptions-item label="所得税费用">¥{{ incomeStatement.income_tax.toFixed(2) }}</el-descriptions-item>
                  <el-descriptions-item label="净利润">
                    <el-tag :type="incomeStatement.net_profit >= 0 ? 'success' : 'danger'" size="large">
                      ¥{{ incomeStatement.net_profit.toFixed(2) }}
                    </el-tag>
                  </el-descriptions-item>
                </el-descriptions>
              </div>
              <el-empty v-else description="请生成利润表" />
            </el-tab-pane>

            <el-tab-pane label="资产负债表" name="balance">
              <div v-if="balanceSheet">
                <el-row :gutter="16">
                  <el-col :span="12">
                    <h4>资产</h4>
                    <el-table :data="balanceSheet.assets" stripe size="small" max-height="400">
                      <el-table-column prop="account_name" label="科目" />
                      <el-table-column prop="balance" label="金额" width="150">
                        <template #default="scope">¥{{ scope.row.balance.toFixed(2) }}</template>
                      </el-table-column>
                    </el-table>
                    <div style="margin-top: 8px; font-weight: bold;">
                      资产总计：¥{{ balanceSheet.total_assets.toFixed(2) }}
                    </div>
                  </el-col>
                  <el-col :span="12">
                    <h4>负债及所有者权益</h4>
                    <el-table :data="balanceSheet.liabilities" stripe size="small" max-height="200">
                      <el-table-column prop="account_name" label="负债" />
                      <el-table-column prop="balance" label="金额" width="150">
                        <template #default="scope">¥{{ scope.row.balance.toFixed(2) }}</template>
                      </el-table-column>
                    </el-table>
                    <el-table :data="balanceSheet.equity" stripe size="small" max-height="200" style="margin-top: 8px;">
                      <el-table-column prop="account_name" label="所有者权益" />
                      <el-table-column prop="balance" label="金额" width="150">
                        <template #default="scope">¥{{ scope.row.balance.toFixed(2) }}</template>
                      </el-table-column>
                    </el-table>
                    <div style="margin-top: 8px; font-weight: bold;">
                      负债及权益总计：¥{{ (balanceSheet.total_liabilities + balanceSheet.total_equity).toFixed(2) }}
                    </div>
                  </el-col>
                </el-row>
              </div>
              <el-empty v-else description="请生成资产负债表" />
            </el-tab-pane>

            <el-tab-pane label="现金流量表" name="cashflow">
              <div v-if="cashFlowReport">
                <el-descriptions :column="2" border>
                  <el-descriptions-item label="经营活动现金流入">¥{{ cashFlowReport.operating_cash_in.toFixed(2) }}</el-descriptions-item>
                  <el-descriptions-item label="经营活动现金流出">¥{{ cashFlowReport.operating_cash_out.toFixed(2) }}</el-descriptions-item>
                  <el-descriptions-item label="经营活动净现金流">
                    <el-tag :type="cashFlowReport.operating_net >= 0 ? 'success' : 'danger'">
                      ¥{{ cashFlowReport.operating_net.toFixed(2) }}
                    </el-tag>
                  </el-descriptions-item>
                  <el-descriptions-item label="投资活动净现金流">
                    <el-tag :type="cashFlowReport.investing_net >= 0 ? 'success' : 'danger'">
                      ¥{{ cashFlowReport.investing_net.toFixed(2) }}
                    </el-tag>
                  </el-descriptions-item>
                  <el-descriptions-item label="筹资活动净现金流">
                    <el-tag :type="cashFlowReport.financing_net >= 0 ? 'success' : 'danger'">
                      ¥{{ cashFlowReport.financing_net.toFixed(2) }}
                    </el-tag>
                  </el-descriptions-item>
                  <el-descriptions-item label="现金净增加额">
                    <el-tag type="danger" size="large">¥{{ cashFlowReport.net_increase.toFixed(2) }}</el-tag>
                  </el-descriptions-item>
                </el-descriptions>
              </div>
              <el-empty v-else description="请生成现金流量表" />
            </el-tab-pane>
          </el-tabs>
        </el-card>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import dayjs from 'dayjs'

// ==================== 类型定义 ====================

interface VatReport {
  period: string
  output_count: number
  output_amount: number
  output_tax: number
  input_count: number
  input_amount: number
  input_tax: number
  tax_due: number
  surcharge_city: number
  surcharge_education: number
  surcharge_local: number
  total_tax_due: number
  items: { tax_rate: number; output_amount: number; output_tax: number; input_amount: number; input_tax: number }[]
}

interface CitReport {
  period: string
  revenue: number
  cost: number
  gross_profit: number
  expenses: number
  profit: number
  tax_rate: number
  tax_due: number
  is_small_profit: boolean
}

interface PitReport {
  period: string
  employees: PitEmployee[]
  total_income: number
  total_tax: number
}

interface PitEmployee {
  employee_name: string
  id_card: string
  income: number
  deduction: number
  special_deduction: number
  taxable_income: number
  tax_rate: number
  quick_deduction: number
  tax_due: number
}

interface IncomeStatement {
  period: string
  revenue: number
  cost: number
  gross_profit: number
  selling_expenses: number
  admin_expenses: number
  finance_expenses: number
  operating_profit: number
  non_operating_income: number
  non_operating_expenses: number
  profit_before_tax: number
  income_tax: number
  net_profit: number
}

interface BalanceSheet {
  period: string
  assets: BalanceSheetItem[]
  liabilities: BalanceSheetItem[]
  equity: BalanceSheetItem[]
  total_assets: number
  total_liabilities: number
  total_equity: number
}

interface BalanceSheetItem {
  account_code: string
  account_name: string
  balance: number
}

interface CashFlowReport {
  period: string
  operating_cash_in: number
  operating_cash_out: number
  operating_net: number
  investing_cash_in: number
  investing_cash_out: number
  investing_net: number
  financing_cash_in: number
  financing_cash_out: number
  financing_net: number
  net_increase: number
}

// ==================== 状态管理 ====================

const activeTab = ref('vat')
const financialSubTab = ref('income')

const periodForm = reactive({
  vatPeriod: dayjs().format('YYYY-MM'),
  citPeriod: dayjs().format('YYYY-MM'),
  pitPeriod: dayjs().format('YYYY-MM'),
  periodType: 'monthly',
  financialPeriod: dayjs().format('YYYY-MM'),
})

const year = dayjs().format('YYYY')
const quarters = [
  { label: year + ' Q1 (1-3月)', value: year + '-Q1' },
  { label: year + ' Q2 (4-6月)', value: year + '-Q2' },
  { label: year + ' Q3 (7-9月)', value: year + '-Q3' },
  { label: year + ' Q4 (10-12月)', value: year + '-Q4' },
]
const semiannuals = [
  { label: year + ' 上半年', value: year + '-H1' },
  { label: year + ' 下半年', value: year + '-H2' },
]

const onPeriodTypeChange = (val: string) => {
  switch (val) {
    case 'monthly':
      periodForm.financialPeriod = dayjs().format('YYYY-MM')
      break
    case 'quarterly':
      periodForm.financialPeriod = year + '-Q1'
      break
    case 'semiannual':
      periodForm.financialPeriod = year + '-H1'
      break
    case 'annual':
      periodForm.financialPeriod = year
      break
  }
}

const loading = reactive({
  vat: false,
  cit: false,
  pit: false,
  income: false,
  balance: false,
  cashFlow: false,
})

const vatReport = ref<VatReport | null>(null)
const citReport = ref<CitReport | null>(null)
const pitReport = ref<PitReport | null>(null)
const incomeStatement = ref<IncomeStatement | null>(null)
const balanceSheet = ref<BalanceSheet | null>(null)
const cashFlowReport = ref<CashFlowReport | null>(null)

// ==================== 方法 ====================

const handleTabChange = (_tab: string) => {
  // 切换标签时不做特殊处理
}

const generateReport = async (reportType: string) => {
  let period = ''
  let loadingKey = ''

  switch (reportType) {
    case 'vat_monthly':
      period = periodForm.vatPeriod
      loadingKey = 'vat'
      break
    case 'cit_quarterly':
      period = periodForm.citPeriod
      loadingKey = 'cit'
      break
    case 'personal_income':
      period = periodForm.pitPeriod
      loadingKey = 'pit'
      break
    case 'income_statement':
      period = periodForm.financialPeriod
      loadingKey = 'income'
      break
    case 'balance_sheet':
      period = periodForm.financialPeriod
      loadingKey = 'balance'
      break
    case 'cash_flow':
      period = periodForm.financialPeriod
      loadingKey = 'cashFlow'
      break
  }

  if (!period) {
    ElMessage.warning('请选择所属期间')
    return
  }

  loading[loadingKey as keyof typeof loading] = true
  try {
    const input: any = { report_type: reportType, period }
    if (['balance_sheet', 'income_statement', 'cash_flow'].includes(reportType)) {
      input.period_type = periodForm.periodType
    }
    const result: any = await invoke('generate_tax_report', { input })

    switch (reportType) {
      case 'vat_monthly':
        vatReport.value = result as VatReport
        break
      case 'cit_quarterly':
        citReport.value = result as CitReport
        break
      case 'personal_income':
        pitReport.value = result as PitReport
        break
      case 'income_statement':
        incomeStatement.value = result as IncomeStatement
        break
      case 'balance_sheet':
        balanceSheet.value = result as BalanceSheet
        break
      case 'cash_flow':
        cashFlowReport.value = result as CashFlowReport
        break
    }

    ElMessage.success('报表生成成功')
  } catch (e) {
    ElMessage.error('生成失败: ' + e)
  } finally {
    loading[loadingKey as keyof typeof loading] = false
  }
}

const exportReport = async (reportType: string) => {
  let period = ''
  switch (reportType) {
    case 'vat_monthly':
      period = periodForm.vatPeriod
      break
    case 'cit_quarterly':
      period = periodForm.citPeriod
      break
    case 'personal_income':
      period = periodForm.pitPeriod
      break
    case 'income_statement':
    case 'balance_sheet':
      period = periodForm.financialPeriod
      break
  }

  if (!period) {
    ElMessage.warning('请先生成报表')
    return
  }

  try {
    const path = await invoke('export_report', {
      reportType,
      period,
      format: 'xlsx',
    })
    ElMessage.success(`报表已导出到: ${path}`)
  } catch (e) {
    ElMessage.error('导出失败: ' + e)
  }
}

onMounted(() => {
  // 自动生成当前月份的增值税月报
  generateReport('vat_monthly')
})
</script>