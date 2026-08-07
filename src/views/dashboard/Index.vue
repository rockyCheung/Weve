<template>
  <div class="dashboard">
    <!-- 核心指标卡片 -->
    <el-row :gutter="20" style="margin-bottom: 20px;">
      <el-col :span="6">
        <el-card shadow="hover">
          <div class="stat-card">
            <div class="stat-icon" style="background: rgba(64, 158, 255, 0.1);">
              <el-icon :size="24" color="#409eff"><User /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-value">{{ stats.active_employee_count }}</div>
              <div class="stat-label">在职员工</div>
              <div class="stat-sub">共 {{ stats.employee_count }} 人</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="hover">
          <div class="stat-card">
            <div class="stat-icon" style="background: rgba(103, 194, 58, 0.1);">
              <el-icon :size="24" color="#67c23a"><Money /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-value">{{ formatMoney(stats.monthly_income) }}</div>
              <div class="stat-label">本月收入</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="hover">
          <div class="stat-card">
            <div class="stat-icon" style="background: rgba(245, 108, 108, 0.1);">
              <el-icon :size="24" color="#f56c6c"><Document /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-value">{{ formatMoney(stats.monthly_expense) }}</div>
              <div class="stat-label">本月支出</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card shadow="hover">
          <div class="stat-card">
            <div class="stat-icon" style="background: rgba(230, 162, 60, 0.1);">
              <el-icon :size="24" color="#e6a23c"><TrendCharts /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-value">{{ formatMoney(stats.monthly_profit) }}</div>
              <div class="stat-label">本月利润</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 税务提醒卡片 -->
    <el-row :gutter="20" style="margin-bottom: 20px;">
      <el-col :span="8">
        <el-card shadow="hover" @click="$router.push('/finance/tax-reports')" style="cursor: pointer;">
          <div class="reminder-card">
            <div class="reminder-count" style="color: #e6a23c;">{{ stats.pending_tax_reports }}</div>
            <div class="reminder-label">待申报税务报表</div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="8">
        <el-card shadow="hover" @click="$router.push('/finance/tax-calendar')" style="cursor: pointer;">
          <div class="reminder-card">
            <div class="reminder-count" style="color: #f56c6c;">{{ stats.overdue_tax_items }}</div>
            <div class="reminder-label">已逾期税务事项</div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="8">
        <el-card shadow="hover" @click="$router.push('/finance/tax-calendar')" style="cursor: pointer;">
          <div class="reminder-card">
            <div class="reminder-count" style="color: #409eff;">{{ stats.upcoming_deadlines }}</div>
            <div class="reminder-label">即将到期（7天内）</div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 趋势图与待办事项 -->
    <el-row :gutter="20">
      <el-col :span="16">
        <el-card>
          <template #header>
            <span>近6月收支趋势</span>
          </template>
          <v-chart :option="chartOption" style="height: 350px" autoresize />
        </el-card>
      </el-col>
      <el-col :span="8">
        <el-card class="activity-card">
          <template #header>
            <span>待办事项</span>
            <el-tag size="small" type="danger" v-if="activities.length > 0" style="float: right;">
              {{ activities.length }} 项
            </el-tag>
          </template>
          <div class="activity-list">
            <el-timeline>
              <el-timeline-item
                v-for="item in activities"
                :key="item.id"
                :timestamp="item.date"
                :type="getActivityType(item.urgency)"
              >
                <div class="activity-item">
                  <div class="activity-title">{{ item.title }}</div>
                  <div class="activity-desc">{{ item.description }}</div>
                </div>
              </el-timeline-item>
            </el-timeline>
            <el-empty v-if="activities.length === 0" description="暂无待办事项" />
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 快速入口 -->
    <el-row :gutter="20" style="margin-top: 20px;">
      <el-col :span="24">
        <el-card>
          <template #header>
            <span>快速入口</span>
          </template>
          <el-row :gutter="16">
            <el-col :span="4" v-for="item in quickLinks" :key="item.path">
              <el-card shadow="hover" class="quick-link" @click="$router.push(item.path)" :body-style="{ padding: '16px' }">
                <div class="quick-link-content">
                  <el-icon :size="28" :color="item.color"><component :is="item.icon" /></el-icon>
                  <div class="quick-link-label">{{ item.label }}</div>
                </div>
              </el-card>
            </el-col>
          </el-row>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { User, Money, TrendCharts, Document, Tickets, Calendar } from '@element-plus/icons-vue'
import { use } from 'echarts/core'
import { CanvasRenderer } from 'echarts/renderers'
import { LineChart, BarChart } from 'echarts/charts'
import { GridComponent, TooltipComponent, LegendComponent, TitleComponent } from 'echarts/components'
import VChart from 'vue-echarts'

use([CanvasRenderer, LineChart, BarChart, GridComponent, TooltipComponent, LegendComponent, TitleComponent])

// ==================== 类型定义 ====================

interface DashboardStats {
  employee_count: number
  active_employee_count: number
  monthly_income: number
  monthly_expense: number
  monthly_profit: number
  pending_approvals: number
  pending_tax_reports: number
  overdue_tax_items: number
  upcoming_deadlines: number
}

interface MonthlyTrend {
  month: string
  income: number
  expense: number
}

interface Activity {
  id: number
  activity_type: string
  title: string
  description: string
  date: string
  urgency: string
}

// ==================== 状态管理 ====================

const stats = ref<DashboardStats>({
  employee_count: 0,
  active_employee_count: 0,
  monthly_income: 0,
  monthly_expense: 0,
  monthly_profit: 0,
  pending_approvals: 0,
  pending_tax_reports: 0,
  overdue_tax_items: 0,
  upcoming_deadlines: 0,
})

const activities = ref<Activity[]>([])
const chartOption = ref({})

// ==================== 快速入口 ====================

const quickLinks = [
  { path: '/hr/employees', label: '员工管理', icon: User, color: '#409eff' },
  { path: '/hr/payroll', label: '工资表', icon: Money, color: '#67c23a' },
  { path: '/finance/accounting', label: '记账', icon: Document, color: '#e6a23c' },
  { path: '/finance/invoices', label: '发票', icon: Tickets, color: '#f56c6c' },
  { path: '/finance/tax-reports', label: '税务报表', icon: TrendCharts, color: '#909399' },
  { path: '/finance/tax-calendar', label: '税务日历', icon: Calendar, color: '#409eff' },
]

// ==================== 方法 ====================

const formatMoney = (value: number): string => {
  if (value >= 10000) {
    return (value / 10000).toFixed(2) + '万'
  }
  return value.toFixed(2)
}

const getActivityType = (urgency: string): string => {
  const map: Record<string, string> = {
    overdue: 'danger',
    urgent: 'warning',
    normal: 'info',
  }
  return map[urgency] || 'info'
}

const fetchDashboardData = async () => {
  try {
    const result: DashboardStats = await invoke('get_dashboard_stats')
    stats.value = result
  } catch (e) {
    console.error('获取仪表盘数据失败:', e)
  }
}

const fetchTrendData = async () => {
  try {
    const trends: MonthlyTrend[] = await invoke('get_monthly_trend', { months: 6 })
    
    chartOption.value = {
      title: { text: '', left: 'center' },
      tooltip: {
        trigger: 'axis',
        axisPointer: { type: 'cross' },
      },
      legend: {
        data: ['收入', '支出'],
        bottom: 0,
      },
      grid: {
        left: '3%',
        right: '4%',
        bottom: '15%',
        containLabel: true,
      },
      xAxis: {
        type: 'category',
        boundaryGap: false,
        data: trends.map(t => t.month),
      },
      yAxis: {
        type: 'value',
        axisLabel: {
          formatter: (value: number) => {
            if (value >= 10000) return (value / 10000).toFixed(0) + '万'
            return value.toFixed(0)
          },
        },
      },
      series: [
        {
          name: '收入',
          type: 'line',
          smooth: true,
          lineStyle: { width: 3, color: '#67c23a' },
          areaStyle: { color: 'rgba(103, 194, 58, 0.1)' },
          itemStyle: { color: '#67c23a' },
          data: trends.map(t => t.income),
        },
        {
          name: '支出',
          type: 'line',
          smooth: true,
          lineStyle: { width: 3, color: '#f56c6c' },
          areaStyle: { color: 'rgba(245, 108, 108, 0.1)' },
          itemStyle: { color: '#f56c6c' },
          data: trends.map(t => t.expense),
        },
      ],
    }
  } catch (e) {
    console.error('获取趋势数据失败:', e)
  }
}

const fetchActivities = async () => {
  try {
    activities.value = await invoke('get_recent_activities')
  } catch (e) {
    console.error('获取待办事项失败:', e)
  }
}

onMounted(async () => {
  await Promise.all([
    fetchDashboardData(),
    fetchTrendData(),
    fetchActivities(),
  ])
})
</script>

<style scoped>
.stat-card {
  display: flex;
  align-items: center;
  padding: 8px;
}

.stat-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.stat-info {
  margin-left: 16px;
  flex: 1;
}

.stat-value {
  font-size: 24px;
  font-weight: bold;
  color: #303133;
}

.stat-label {
  font-size: 14px;
  color: #909399;
  margin-top: 4px;
}

.stat-sub {
  font-size: 12px;
  color: #c0c4cc;
}

.reminder-card {
  text-align: center;
  padding: 8px 0;
}

.reminder-count {
  font-size: 36px;
  font-weight: bold;
}

.reminder-label {
  font-size: 14px;
  color: #909399;
  margin-top: 8px;
}

.activity-item {
  padding: 4px 0;
}

.activity-title {
  font-weight: 500;
  font-size: 14px;
}

.activity-desc {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

.activity-list {
  max-height: 350px;
  overflow-y: auto;
  padding-right: 8px;
}

.quick-link {
  cursor: pointer;
  transition: all 0.3s;
}

.quick-link:hover {
  transform: translateY(-4px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.quick-link-content {
  text-align: center;
}

.quick-link-label {
  margin-top: 8px;
  font-size: 13px;
  color: #606266;
}
</style>