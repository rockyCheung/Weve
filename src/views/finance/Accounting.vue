<template>
  <div>
    <el-tabs v-model="activeTab">
      <!-- 凭证录入 -->
      <el-tab-pane name="vouchers">
        <template #label>
          <el-icon><Edit /></el-icon>
          <span>凭证录入</span>
        </template>
        <el-row :gutter="16">
          <el-col :span="12">
            <el-card>
              <template #header>
                <span>录入凭证</span>
              </template>
              <el-form :model="voucherForm" label-width="80px">
                <el-form-item label="日期" required>
                  <el-date-picker
                    v-model="voucherForm.date"
                    type="date"
                    value-format="YYYY-MM-DD"
                  />
                </el-form-item>
                <el-form-item label="摘要">
                  <el-input v-model="voucherForm.summary" />
                </el-form-item>
                <el-form-item label="附件">
                  <div style="width: 100%">
                    <el-button size="small" @click="selectAttachments">
                      <el-icon style="margin-right: 4px"><Paperclip /></el-icon>
                      添加附件
                    </el-button>
                    <el-tag
                      v-for="(att, idx) in voucherAttachments"
                      :key="idx"
                      closable
                      size="small"
                      style="margin-left: 8px"
                      @close="removeAttachment(idx)"
                    >
                      {{ att.originalName }}
                    </el-tag>
                  </div>
                </el-form-item>
                <el-form-item label="分录">
                  <div
                    v-for="(entry, index) in voucherForm.entries"
                    :key="index"
                    class="entry-row"
                  >
                    <el-row :gutter="8">
                      <el-col :span="8">
                        <el-select
                          v-model="entry.account_id"
                          placeholder="科目"
                          filterable
                          size="small"
                        >
                          <el-option
                            v-for="acc in flatAccounts"
                            :key="acc.id"
                            :label="`${acc.code} ${acc.name}`"
                            :value="acc.id"
                          />
                        </el-select>
                      </el-col>
                      <el-col :span="6">
                        <el-input-number
                          v-model="entry.debit_amount"
                          :precision="2"
                          size="small"
                          placeholder="借方"
                        />
                      </el-col>
                      <el-col :span="6">
                        <el-input-number
                          v-model="entry.credit_amount"
                          :precision="2"
                          size="small"
                          placeholder="贷方"
                        />
                      </el-col>
                      <el-col :span="4">
                        <el-button
                          type="danger"
                          size="small"
                          @click="removeEntry(index)"
                          v-if="voucherForm.entries.length > 2"
                          >×</el-button
                        >
                      </el-col>
                    </el-row>
                  </div>
                </el-form-item>
                <el-form-item>
                  <el-button size="small" @click="addEntry">添加分录</el-button>
                </el-form-item>
                <el-form-item>
                  <el-tag>借方合计: ¥{{ debitTotal.toFixed(2) }}</el-tag>
                  <el-tag style="margin-left: 8px"
                    >贷方合计: ¥{{ creditTotal.toFixed(2) }}</el-tag
                  >
                  <el-tag
                    v-if="isBalanced"
                    type="success"
                    style="margin-left: 8px"
                    >✓ 平衡</el-tag
                  >
                  <el-tag v-else type="danger" style="margin-left: 8px"
                    >不平衡</el-tag
                  >
                </el-form-item>
                <el-form-item>
                  <el-button
                    type="primary"
                    @click="submitVoucher"
                    :disabled="!isBalanced"
                    >提交凭证</el-button
                  >
                </el-form-item>
              </el-form>
            </el-card>
          </el-col>
          <el-col :span="12">
            <!-- 薪资凭证自动生成 -->
            <el-card>
              <template #header>
                <span>薪资凭证自动生成</span>
              </template>
              <el-form :inline="true">
                <el-form-item label="所属月份">
                  <el-date-picker
                    v-model="payrollVoucherPeriod"
                    type="month"
                    value-format="YYYY-MM"
                    placeholder="选择月份"
                  />
                </el-form-item>
              </el-form>
              <el-table :data="payrollVoucherTypes" stripe size="small">
                <el-table-column prop="label" label="凭证类型" width="160" />
                <el-table-column label="录制日期" width="180">
                  <template #default="scope">
                    <el-date-picker
                      v-model="scope.row.date"
                      type="date"
                      value-format="YYYY-MM-DD"
                      size="small"
                    />
                  </template>
                </el-table-column>
                <el-table-column label="操作">
                  <template #default="scope">
                    <el-button
                      size="small"
                      type="success"
                      @click="generatePayrollVoucher(scope.row.type, scope.row.date)"
                      :loading="scope.row.loading"
                    >
                      生成凭证
                    </el-button>
                  </template>
                </el-table-column>
              </el-table>
            </el-card>
          </el-col>
        </el-row>

        <!-- 凭证查询 -->
        <el-row style="margin-top: 16px">
          <el-col :span="24">
            <el-card>
              <template #header>
                <span>凭证查询</span>
              </template>
              <el-form :inline="true" :model="voucherQuery">
                <el-form-item label="开始">
                  <el-date-picker
                    v-model="voucherQuery.start_date"
                    type="date"
                    value-format="YYYY-MM-DD"
                  />
                </el-form-item>
                <el-form-item label="结束">
                  <el-date-picker
                    v-model="voucherQuery.end_date"
                    type="date"
                    value-format="YYYY-MM-DD"
                  />
                </el-form-item>
                <el-form-item>
                  <el-button type="primary" @click="fetchVouchers"
                    >查询</el-button
                  >
                </el-form-item>
              </el-form>
              <el-table
                :data="voucherList"
                stripe
                size="small"
                max-height="400"
              >
                <el-table-column prop="voucher_no" label="凭证号" width="130" />
                <el-table-column prop="date" label="日期" width="110" />
                <el-table-column prop="summary" label="摘要" />
                <el-table-column label="操作" width="140">
                  <template #default="scope">
                    <el-button size="small" @click="viewVoucher(scope.row)"
                      >查看</el-button
                    >
                    <el-button
                      size="small"
                      type="danger"
                      style="margin-left: 6px"
                      @click="deleteVoucher(scope.row)"
                      >删除</el-button
                    >
                  </template>
                </el-table-column>
              </el-table>
            </el-card>
          </el-col>
        </el-row>
      </el-tab-pane>

      <!-- 试算平衡 -->
      <el-tab-pane name="trial">
        <template #label>
          <el-icon><ScaleToOriginal /></el-icon>
          <span>试算平衡</span>
        </template>
        <el-card>
          <template #header>
            <span>试算平衡表</span>
          </template>
          <el-form :inline="true">
            <el-form-item label="截止日期">
              <el-date-picker
                v-model="trialDate"
                type="date"
                value-format="YYYY-MM-DD"
              />
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="fetchTrialBalance" :loading="trialLoading"
                >试算</el-button
              >
            </el-form-item>
          </el-form>
          <el-table :data="trialBalanceData" stripe border size="small">
            <el-table-column prop="account_code" label="科目编码" width="100" />
            <el-table-column prop="account_name" label="科目名称" width="150" />
            <el-table-column prop="debit_total" label="借方发生额" width="140">
              <template #default="scope"
                >¥{{ formatAmount(scope.row.debit_total) }}</template
              >
            </el-table-column>
            <el-table-column prop="credit_total" label="贷方发生额" width="140">
              <template #default="scope"
                >¥{{ formatAmount(scope.row.credit_total) }}</template
              >
            </el-table-column>
            <el-table-column prop="balance" label="余额" width="140">
              <template #default="scope"
                >¥{{ formatAmount(scope.row.balance) }}</template
              >
            </el-table-column>
          </el-table>
          <div v-if="trialBalanceData.length > 0" style="margin-top: 16px">
            <el-tag :type="isTrialBalanced ? 'success' : 'danger'" size="large">
              {{ isTrialBalanced ? "✓ 试算平衡" : "✗ 试算不平衡，请检查凭证" }}
            </el-tag>
          </div>
        </el-card>
      </el-tab-pane>

      <!-- 银行流水导入 -->
      <el-tab-pane name="bank">
        <template #label>
          <el-icon><CreditCard /></el-icon>
          <span>银行流水</span>
        </template>
        <el-card>
          <template #header>
            <span>银行流水导入</span>
          </template>
          <el-alert
            title="支持 CSV 格式，列顺序：日期,摘要,金额,余额"
            type="info"
            :closable="false"
            show-icon
            style="margin-bottom: 16px"
          />
          <el-upload
            drag
            accept=".csv"
            :auto-upload="false"
            :on-change="handleBankFileChange"
            :limit="1"
          >
            <el-icon class="el-icon--upload"><upload-filled /></el-icon>
            <div class="el-upload__text">
              拖拽 CSV 文件到此处或 <em>点击选择</em>
            </div>
          </el-upload>
          <div v-if="bankTransactions.length > 0" style="margin-top: 16px">
            <p>已解析 {{ bankTransactions.length }} 条交易记录</p>
            <el-button type="primary" @click="importBankData"
              >导入并生成凭证</el-button
            >
          </div>
        </el-card>
      </el-tab-pane>

      <!-- 科目管理 -->
      <el-tab-pane name="accounts">
        <template #label>
          <el-icon><Folder /></el-icon>
          <span>科目管理</span>
        </template>
        <el-row :gutter="16">
          <el-col :span="12">
            <el-card>
              <template #header>
                <span>科目列表</span>
                <el-button
                  size="small"
                  type="primary"
                  style="float: right"
                  @click="showAddAccountDialog"
                  >添加科目</el-button
                >
              </template>
              <el-tree
                :data="accountTree"
                :props="{ label: 'name', children: 'children' }"
                node-key="id"
                default-expand-all
                :expand-on-click-node="false"
                @node-click="handleAccountClick"
              >
                <template #default="{ data }">
                  <span class="account-node">
                    <span class="account-code">{{ data.code }}</span>
                    <span class="account-name">{{ data.name }}</span>
                    <el-tag
                      size="small"
                      :type="categoryTag(data.category)"
                      class="account-category"
                    >
                      {{ categoryLabel(data.category) }}
                    </el-tag>
                  </span>
                </template>
              </el-tree>
            </el-card>
          </el-col>
          <el-col :span="12">
            <el-card v-if="selectedAccount">
              <template #header>
                <span>科目详情</span>
              </template>
              <el-descriptions :column="1" border>
                <el-descriptions-item label="科目编码">{{
                  selectedAccount.code
                }}</el-descriptions-item>
                <el-descriptions-item label="科目名称">{{
                  selectedAccount.name
                }}</el-descriptions-item>
                <el-descriptions-item label="类别">{{
                  categoryLabel(selectedAccount.category)
                }}</el-descriptions-item>
                <el-descriptions-item label="状态">
                  <el-tag
                    :type="selectedAccount.is_active ? 'success' : 'danger'"
                    size="small"
                  >
                    {{ selectedAccount.is_active ? "启用" : "停用" }}
                  </el-tag>
                </el-descriptions-item>
              </el-descriptions>
              <div style="margin-top: 16px">
                <el-button
                  size="small"
                  @click="showEditAccountDialog(selectedAccount)"
                  >编辑</el-button
                >
                <el-button
                  size="small"
                  type="danger"
                  @click="deleteAccount(selectedAccount.id)"
                  >删除</el-button
                >
              </div>
            </el-card>
            <el-empty v-else description="请选择一个科目" />
          </el-col>
        </el-row>
      </el-tab-pane>
    </el-tabs>

    <!-- 科目对话框 -->
    <el-dialog
      v-model="accountDialogVisible"
      :title="isEditAccount ? '编辑科目' : '添加科目'"
      width="400px"
    >
      <el-form :model="accountForm" label-width="80px">
        <el-form-item label="编码" required>
          <el-input v-model="accountForm.code" :disabled="isEditAccount" />
        </el-form-item>
        <el-form-item label="名称" required>
          <el-input v-model="accountForm.name" />
        </el-form-item>
        <el-form-item label="类别" required>
          <el-select v-model="accountForm.category">
            <el-option label="资产" value="asset" />
            <el-option label="负债" value="liability" />
            <el-option label="权益" value="equity" />
            <el-option label="收入" value="income" />
            <el-option label="费用" value="expense" />
          </el-select>
        </el-form-item>
        <el-form-item label="上级科目">
          <el-tree-select
            v-model="accountForm.parent_id"
            :data="accountTree"
            :props="{ label: 'name', value: 'id' }"
            placeholder="选择上级科目（可选）"
            clearable
            filterable
          />
        </el-form-item>

      </el-form>
      <template #footer>
        <el-button @click="accountDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="submitAccount">确定</el-button>
      </template>
    </el-dialog>

    <!-- 凭证详情对话框 -->
    <el-dialog v-model="voucherDetailVisible" title="凭证详情" width="600px">
      <el-descriptions :column="2" border v-if="currentVoucher">
        <el-descriptions-item label="凭证号">{{
          currentVoucher.voucher_no
        }}</el-descriptions-item>
        <el-descriptions-item label="日期">{{
          currentVoucher.date
        }}</el-descriptions-item>
        <el-descriptions-item label="摘要" :span="2">{{
          currentVoucher.summary
        }}</el-descriptions-item>
      </el-descriptions>
      <el-table
        :data="currentVoucher?.entries || []"
        stripe
        size="small"
        style="margin-top: 16px"
      >
        <el-table-column prop="account_code" label="科目编码" width="100" />
        <el-table-column prop="account_name" label="科目名称" width="150" />
        <el-table-column prop="debit_amount" label="借方金额" width="120">
          <template #default="scope"
            >¥{{ scope.row.debit_amount.toFixed(2) }}</template
          >
        </el-table-column>
        <el-table-column prop="credit_amount" label="贷方金额" width="120">
          <template #default="scope"
            >¥{{ scope.row.credit_amount.toFixed(2) }}</template
          >
        </el-table-column>
        <el-table-column prop="summary" label="摘要" />
      </el-table>

      <div v-if="currentVoucherAttachments.length > 0" style="margin-top: 16px">
        <h4 style="margin-bottom: 8px">附件</h4>
        <el-tag
          v-for="(att, idx) in currentVoucherAttachments"
          :key="idx"
          type="info"
          style="margin-right: 8px; cursor: pointer"
          @click="openAttachment(att)"
        >
          <el-icon style="margin-right: 2px"><Document /></el-icon>
          {{ att }}
        </el-tag>
      </div>
    </el-dialog>

    <!-- 薪资凭证生成结果 -->
    <el-dialog v-model="payrollVoucherResultVisible" title="薪资凭证生成结果" width="480px">
      <el-descriptions v-if="payrollVoucherResult" :column="1" border>
        <el-descriptions-item label="凭证号">{{
          payrollVoucherResult.voucher_no
        }}</el-descriptions-item>
        <el-descriptions-item label="工资总额"
          >¥{{ payrollVoucherResult.total_salary.toFixed(2) }}</el-descriptions-item
        >
        <el-descriptions-item label="社保-公司部分"
          >¥{{ payrollVoucherResult.employer_insurance.toFixed(2) }}</el-descriptions-item
        >
        <el-descriptions-item label="公积金-公司部分"
          >¥{{ payrollVoucherResult.employer_housing_fund.toFixed(2) }}</el-descriptions-item
        >
        <el-descriptions-item label="社保-个人部分"
          >¥{{ payrollVoucherResult.employee_insurance.toFixed(2) }}</el-descriptions-item
        >
        <el-descriptions-item label="公积金-个人部分"
          >¥{{ payrollVoucherResult.employee_housing_fund.toFixed(2) }}</el-descriptions-item
        >
        <el-descriptions-item label="代扣个税"
          >¥{{ payrollVoucherResult.income_tax_total.toFixed(2) }}</el-descriptions-item
        >
        <el-descriptions-item label="实发合计">
          <el-tag type="success"
            >¥{{ payrollVoucherResult.net_pay_total.toFixed(2) }}</el-tag
          >
        </el-descriptions-item>
        <el-descriptions-item label="社保缴纳总额"
          >¥{{ payrollVoucherResult.total_insurance_pay.toFixed(2) }}</el-descriptions-item
        >
        <el-descriptions-item label="公积金缴纳总额"
          >¥{{ payrollVoucherResult.total_fund_pay.toFixed(2) }}</el-descriptions-item
        >
      </el-descriptions>
      <template #footer>
        <el-button @click="payrollVoucherResultVisible = false">关闭</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ElMessage, ElMessageBox } from "element-plus";
import { UploadFilled, Paperclip, Document, Edit, ScaleToOriginal, CreditCard, Folder } from "@element-plus/icons-vue";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import { open as openPath } from "@tauri-apps/plugin-shell";
import type { UploadFile } from "element-plus";
import dayjs from "dayjs";

// ==================== 类型定义 ====================

interface Account {
  id: number;
  code: string;
  name: string;
  category: string;
  parent_id: number | null;
  is_active: boolean;
  children: Account[];
}

interface VoucherDetail {
  id: number;
  voucher_no: string;
  date: string;
  summary: string;
  entries: VoucherEntryDetail[];
  attachment: string | null;
  created_at: string;
}

interface VoucherEntryDetail {
  account_id: number;
  account_code: string;
  account_name: string;
  debit_amount: number;
  credit_amount: number;
  summary: string;
}

interface PayrollVoucherResult {
  voucher_no: string;
  total_salary: number;
  employer_insurance: number;
  employer_housing_fund: number;
  employee_insurance: number;
  employee_housing_fund: number;
  income_tax_total: number;
  net_pay_total: number;
  total_insurance_pay: number;
  total_fund_pay: number;
}

interface TrialBalanceItem {
  account_id: number;
  account_code: string;
  account_name: string;
  debit_total: number;
  credit_total: number;
  balance: number;
}

// ==================== 状态管理 ====================

const activeTab = ref("vouchers");

// 科目管理
const accountTree = ref<Account[]>([]);
const selectedAccount = ref<Account | null>(null);
const accountDialogVisible = ref(false);
const isEditAccount = ref(false);
const accountForm = ref({
  code: "",
  name: "",
  category: "asset",
  parent_id: null as number | null,
});

// 凭证管理
const voucherForm = reactive({
  date: dayjs().format("YYYY-MM-DD"),
  summary: "",
  entries: [
    { account_id: null, debit_amount: 0, credit_amount: 0, summary: "" },
    { account_id: null, debit_amount: 0, credit_amount: 0, summary: "" },
  ],
});
const voucherQuery = ref({
  start_date: dayjs().startOf("month").format("YYYY-MM-DD"),
  end_date: dayjs().format("YYYY-MM-DD"),
});
const voucherList = ref<VoucherDetail[]>([]);
const voucherDetailVisible = ref(false);
const currentVoucher = ref<VoucherDetail | null>(null);

// 附件管理
const voucherAttachments = ref<{ fileName: string; originalName: string }[]>([]);
const currentVoucherAttachments = computed(() => {
  const att = currentVoucher.value?.attachment;
  if (!att) return [];
  try {
    const arr = JSON.parse(att);
    return Array.isArray(arr) ? arr.map((f) => f.split("_").slice(1).join("_")) : [];
  } catch {
    return [];
  }
});

// 薪资凭证生成
const payrollVoucherPeriod = ref(dayjs().format("YYYY-MM"));
const payrollVoucherResultVisible = ref(false);
const payrollVoucherResult = ref<PayrollVoucherResult | null>(null);

// 计算缺省日期
const lastDayOfPeriod = () => {
  const d = dayjs(payrollVoucherPeriod.value);
  return d.endOf("month").format("YYYY-MM-DD");
};

const payrollVoucherTypes = reactive([
  { type: "accrual", label: "计提工资及社保公积金", date: lastDayOfPeriod(), loading: false },
  { type: "salary_payment", label: "发放工资", date: "", loading: false },
  { type: "insurance_payment", label: "缴纳社保", date: "", loading: false },
  { type: "fund_payment", label: "缴纳公积金", date: "", loading: false },
]);

const generatePayrollVoucher = async (type: string, date: string) => {
  if (!date) {
    ElMessage.warning("请选择录制日期");
    return;
  }
  const row = payrollVoucherTypes.find((r) => r.type === type)!;
  row.loading = true;
  try {
    payrollVoucherResult.value = await invoke("generate_payroll_voucher", {
      period: payrollVoucherPeriod.value,
      voucherType: type,
      voucherDate: date,
    });
    payrollVoucherResultVisible.value = true;
    ElMessage.success("凭证已生成");
    await fetchVouchers();
  } catch (e) {
    ElMessage.error("生成失败: " + e);
  } finally {
    row.loading = false;
  }
};

// 试算平衡
const trialDate = ref(dayjs().format("YYYY-MM-DD"));
const trialBalanceData = ref<TrialBalanceItem[]>([]);
const trialLoading = ref(false);

const formatAmount = (val: number) => {
  return val.toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2 });
};

// 银行流水
const bankTransactions = ref<any[]>([]);

// ==================== 计算属性 ====================

const flatAccounts = computed(() => {
  const flatten = (accounts: Account[]): Account[] => {
    let result: Account[] = [];
    for (const acc of accounts) {
      result.push(acc);
      if (acc.children) {
        result = result.concat(flatten(acc.children));
      }
    }
    return result;
  };
  return flatten(accountTree.value);
});

const debitTotal = computed(() => {
  return voucherForm.entries.reduce((sum, e) => sum + (e.debit_amount || 0), 0);
});

const creditTotal = computed(() => {
  return voucherForm.entries.reduce(
    (sum, e) => sum + (e.credit_amount || 0),
    0,
  );
});

const isBalanced = computed(() => {
  return (
    Math.abs(debitTotal.value - creditTotal.value) < 0.001 &&
    debitTotal.value > 0
  );
});

const isTrialBalanced = computed(() => {
  if (trialBalanceData.value.length === 0) return false;
  const totalDebit = trialBalanceData.value.reduce(
    (s, i) => s + i.debit_total,
    0,
  );
  const totalCredit = trialBalanceData.value.reduce(
    (s, i) => s + i.credit_total,
    0,
  );
  return Math.abs(totalDebit - totalCredit) < 0.001;
});

// ==================== 方法 ====================

const categoryTag = (category: string) => {
  const map: Record<string, string> = {
    asset: "",
    liability: "warning",
    equity: "success",
    income: "info",
    expense: "danger",
  };
  return map[category] || "";
};

const categoryLabel = (category: string) => {
  const map: Record<string, string> = {
    asset: "资产",
    liability: "负债",
    equity: "权益",
    income: "收入",
    expense: "费用",
  };
  return map[category] || category;
};

// 科目管理
const fetchAccounts = async () => {
  try {
    accountTree.value = await invoke("get_account_tree");
  } catch (e) {
    ElMessage.error("获取科目列表失败: " + e);
  }
};

const handleAccountClick = (data: Account) => {
  selectedAccount.value = data;
};

const showAddAccountDialog = () => {
  isEditAccount.value = false;
  accountForm.value = {
    code: "",
    name: "",
    category: "asset",
    parent_id: null,
  };
  accountDialogVisible.value = true;
};

const showEditAccountDialog = (account: Account) => {
  isEditAccount.value = true;
  accountForm.value = {
    code: account.code,
    name: account.name,
    category: account.category,
    parent_id: account.parent_id,
  };
  accountDialogVisible.value = true;
};

const submitAccount = async () => {
  if (!accountForm.value.code || !accountForm.value.name) {
    ElMessage.warning("请填写科目编码和名称");
    return;
  }
  try {
    if (isEditAccount.value) {
      await invoke("update_account", {
        id: selectedAccount.value?.id,
        name: accountForm.value.name,
      });
      ElMessage.success("科目已更新");
    } else {
      await invoke("create_account", { input: accountForm.value });
      ElMessage.success("科目添加成功");
    }
    accountDialogVisible.value = false;
    await fetchAccounts();
  } catch (e) {
    ElMessage.error("操作失败: " + e);
  }
};

const deleteAccount = async (id: number) => {
  try {
    await invoke("delete_account", { id });
    ElMessage.success("科目已删除");
    selectedAccount.value = null;
    await fetchAccounts();
  } catch (e) {
    ElMessage.error("删除失败: " + e);
  }
};

// 凭证管理
const addEntry = () => {
  voucherForm.entries.push({
    account_id: null,
    debit_amount: 0,
    credit_amount: 0,
    summary: "",
  });
};

const removeEntry = (index: number) => {
  voucherForm.entries.splice(index, 1);
};

const selectAttachments = async () => {
  try {
    const selected = await openDialog({
      multiple: true,
      filters: [{ name: "文件", extensions: ["pdf", "png", "jpg", "jpeg", "xlsx", "xls", "doc", "docx", "csv"] }],
    });
    if (!selected) return;
    const paths = Array.isArray(selected) ? selected : [selected];
    for (const p of paths) {
      const fileName = await invoke("save_voucher_attachment", { sourcePath: p }) as string;
      const originalName = (p as string).split(/[\\/]/).pop() || fileName;
      voucherAttachments.value.push({ fileName, originalName });
    }
    ElMessage.success(`已添加 ${paths.length} 个附件`);
  } catch (e) {
    ElMessage.error("添加附件失败: " + e);
  }
};

const removeAttachment = (index: number) => {
  voucherAttachments.value.splice(index, 1);
};

const openAttachment = async (displayName: string) => {
  try {
    // 从当前凭证附件中找回完整文件名
    const att = currentVoucher.value?.attachment;
    if (!att) return;
    const arr = JSON.parse(att) as string[];
    const fullName = arr.find((f) => f.endsWith(displayName)) || arr[0];
    const path = await invoke("get_attachment_full_path", { fileName: fullName }) as string;
    await openPath(path);
  } catch (e) {
    ElMessage.error("打开附件失败: " + e);
  }
};

const submitVoucher = async () => {
  if (!voucherForm.date) {
    ElMessage.warning("请选择日期");
    return;
  }
  for (let i = 0; i < voucherForm.entries.length; i++) {
    const entry = voucherForm.entries[i];
    if (entry.account_id === null || entry.account_id === undefined) {
      ElMessage.warning(`第 ${i + 1} 行分录未选择科目`);
      return;
    }
    if (!entry.debit_amount && !entry.credit_amount) {
      ElMessage.warning(`第 ${i + 1} 行分录借方和贷方金额不能同时为空`);
      return;
    }
  }
  try {
    const attachment = voucherAttachments.value.length > 0
      ? JSON.stringify(voucherAttachments.value.map((a) => a.fileName))
      : null;
    const voucherNo = await invoke("post_voucher", {
      input: { ...voucherForm, attachment },
    });
    ElMessage.success(`凭证已提交，编号: ${voucherNo}`);
    voucherForm.entries = [
      { account_id: null, debit_amount: 0, credit_amount: 0, summary: "" },
      { account_id: null, debit_amount: 0, credit_amount: 0, summary: "" },
    ];
    voucherForm.summary = "";
    voucherAttachments.value = [];
    await fetchVouchers();
  } catch (e) {
    ElMessage.error("提交失败: " + e);
  }
};

const fetchVouchers = async () => {
  try {
    voucherList.value = await invoke("get_vouchers", {
      startDate: voucherQuery.value.start_date,
      endDate: voucherQuery.value.end_date,
    });
  } catch (e) {
    ElMessage.error("查询失败: " + e);
  }
};

const viewVoucher = (voucher: VoucherDetail) => {
  currentVoucher.value = voucher;
  voucherDetailVisible.value = true;
};

const deleteVoucher = async (voucher: { id: number; voucher_no: string }) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除凭证 ${voucher.voucher_no} 吗？`,
      "确认删除",
      { type: "warning" }
    );
    await invoke("delete_voucher", { id: voucher.id });
    ElMessage.success("凭证已删除");
    await fetchVouchers();
  } catch {
    // 取消操作
  }
};

// 试算平衡
const fetchTrialBalance = async () => {
  trialLoading.value = true;
  try {
    trialBalanceData.value = await invoke("trial_balance", {
      endDate: trialDate.value,
    });
  } catch (e) {
    ElMessage.error("生成试算平衡表失败: " + e);
  } finally {
    trialLoading.value = false;
  }
};

// 银行流水
const handleBankFileChange = (uploadFile: UploadFile) => {
  if (uploadFile.raw) {
    // 简化：实际应解析 CSV 文件
    ElMessage.success("银行流水导入功能待完善，请手动录入凭证");
  }
};

const importBankData = async () => {
  ElMessage.success("导入功能待实现");
};

onMounted(() => {
  fetchAccounts();
  fetchVouchers();
});
</script>

<style scoped>
.account-node {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
}
.account-code {
  color: #909399;
  font-family: monospace;
  font-size: 12px;
}
.account-name {
  font-weight: 500;
}
.account-category {
  margin-left: auto;
}
.entry-row {
  margin-bottom: 8px;
  padding: 8px;
  background: #f5f7fa;
  border-radius: 4px;
}
</style>
