<template>
  <div>
    <el-tabs v-model="activeTab">
      <!-- 企业信息 -->
      <el-tab-pane label="企业信息" name="company">
        <el-card>
          <template #header>
            <span>企业基本信息</span>
            <el-button
              size="small"
              type="primary"
              style="float: right"
              @click="saveCompanyInfo"
              :loading="saving"
            >
              保存设置
            </el-button>
          </template>
          <el-form :model="companyForm" label-width="150px">
            <el-form-item label="企业名称">
              <el-input
                v-model="companyForm.company_name"
                placeholder="请输入企业全称"
              />
            </el-form-item>
            <el-form-item label="统一社会信用代码">
              <el-input
                v-model="companyForm.company_tax_id"
                placeholder="18位统一社会信用代码"
                maxlength="18"
              />
            </el-form-item>
            <el-form-item label="注册地址">
              <el-input
                v-model="companyForm.company_address"
                type="textarea"
                :rows="2"
                placeholder="企业注册地址"
              />
            </el-form-item>
            <el-form-item label="联系电话">
              <el-input
                v-model="companyForm.company_phone"
                placeholder="企业联系电话"
              />
            </el-form-item>
            <el-form-item label="开户银行">
              <el-input
                v-model="companyForm.company_bank"
                placeholder="开户银行名称"
              />
            </el-form-item>
            <el-form-item label="银行账号">
              <el-input
                v-model="companyForm.company_bank_account"
                placeholder="银行账号"
              />
            </el-form-item>
          </el-form>
        </el-card>
      </el-tab-pane>

      <!-- 税务参数 -->
      <el-tab-pane label="税务参数" name="tax">
        <el-card>
          <template #header>
            <span>税务参数配置</span>
            <el-button
              size="small"
              type="primary"
              style="float: right"
              @click="saveTaxParams"
              :loading="saving"
            >
              保存参数
            </el-button>
          </template>
          <el-form :model="taxForm" label-width="180px">
            <el-form-item label="纳税人类型">
              <el-radio-group v-model="taxForm.taxpayer_type">
                <el-radio label="一般纳税人">一般纳税人</el-radio>
                <el-radio label="小规模纳税人">小规模纳税人</el-radio>
              </el-radio-group>
              <div class="form-tip">影响增值税申报周期和税率选择</div>
            </el-form-item>
            <el-form-item label="增值税税率">
              <el-select v-model="taxForm.vat_rate" style="width: 200px">
                <el-option label="13%" value="0.13" />
                <el-option label="9%" value="0.09" />
                <el-option label="6%" value="0.06" />
                <el-option label="3%" value="0.03" />
                <el-option label="1%" value="0.01" />
              </el-select>
              <div class="form-tip">一般纳税人常用13%，小规模纳税人常用3%</div>
            </el-form-item>
            <el-form-item label="企业所得税税率">
              <el-input-number
                v-model="taxForm.cit_rate"
                :min="0"
                :max="1"
                :step="0.01"
                :precision="2"
                style="width: 200px"
              />
              <div class="form-tip">标准税率25%，小微企业享受优惠税率</div>
            </el-form-item>
            <el-form-item label="城建税税率">
              <el-select v-model="taxForm.city_tax_rate" style="width: 200px">
                <el-option label="7%（市区）" value="0.07" />
                <el-option label="5%（县城/镇）" value="0.05" />
                <el-option label="1%（其他地区）" value="0.01" />
              </el-select>
            </el-form-item>
            <el-form-item label="教育费附加率">
              <el-input-number
                v-model="taxForm.education_surcharge"
                :min="0"
                :max="1"
                :step="0.01"
                :precision="2"
                style="width: 200px"
                disabled
              />
              <div class="form-tip">法定3%</div>
            </el-form-item>
            <el-form-item label="地方教育附加率">
              <el-input-number
                v-model="taxForm.local_surcharge"
                :min="0"
                :max="1"
                :step="0.01"
                :precision="2"
                style="width: 200px"
                disabled
              />
              <div class="form-tip">法定2%</div>
            </el-form-item>
            <el-divider />
            <el-form-item label="小微企业年利润阈值">
              <el-input-number
                v-model="taxForm.small_profit_threshold"
                :min="0"
                :step="100000"
                :precision="0"
                style="width: 200px"
              />
              <span style="margin-left: 8px">元</span>
              <div class="form-tip">年应纳税所得额低于此值享受优惠税率</div>
            </el-form-item>
          </el-form>
        </el-card>
      </el-tab-pane>

      <!-- 社保公积金 -->
      <el-tab-pane label="社保公积金" name="insurance">
        <el-card>
          <template #header>
            <span>社保公积金费率配置</span>
            <el-button
              size="small"
              type="primary"
              style="float: right"
              @click="saveInsuranceParams"
              :loading="savingInsurance"
            >
              保存参数
            </el-button>
          </template>
          <el-form :model="insuranceForm" label-width="180px">
            <h4 style="margin-bottom: 12px; color: #409eff">养老保险</h4>
            <el-form-item label="单位比例">
              <el-input-number
                v-model="insuranceForm.pension_employer_rate"
                :min="0" :max="1" :step="0.01" :precision="3"
                style="width: 200px"
              />
              <span style="margin-left: 8px">（默认16%）</span>
            </el-form-item>
            <el-form-item label="个人比例">
              <el-input-number
                v-model="insuranceForm.pension_employee_rate"
                :min="0" :max="1" :step="0.01" :precision="3"
                style="width: 200px"
              />
              <span style="margin-left: 8px">（默认8%）</span>
            </el-form-item>

            <el-divider />
            <h4 style="margin-bottom: 12px; color: #409eff">医疗保险（含生育）</h4>
            <el-form-item label="单位比例">
              <el-input-number
                v-model="insuranceForm.medical_employer_rate"
                :min="0" :max="1" :step="0.01" :precision="3"
                style="width: 200px"
              />
              <span style="margin-left: 8px">（默认9.8%）</span>
            </el-form-item>
            <el-form-item label="个人比例">
              <el-input-number
                v-model="insuranceForm.medical_employee_rate"
                :min="0" :max="1" :step="0.01" :precision="3"
                style="width: 200px"
              />
              <span style="margin-left: 8px">（默认2%）</span>
            </el-form-item>
            <el-form-item label="个人固定额">
              <el-input-number
                v-model="insuranceForm.medical_employee_fixed"
                :min="0" :precision="0"
                style="width: 200px"
              />
              <span style="margin-left: 8px">元/人/月（大病统筹，北京默认3元）</span>
            </el-form-item>

            <el-divider />
            <h4 style="margin-bottom: 12px; color: #409eff">失业保险</h4>
            <el-form-item label="单位比例">
              <el-input-number
                v-model="insuranceForm.unemployment_employer_rate"
                :min="0" :max="1" :step="0.01" :precision="3"
                style="width: 200px"
              />
              <span style="margin-left: 8px">（默认0.5%）</span>
            </el-form-item>
            <el-form-item label="个人比例">
              <el-input-number
                v-model="insuranceForm.unemployment_employee_rate"
                :min="0" :max="1" :step="0.01" :precision="3"
                style="width: 200px"
              />
              <span style="margin-left: 8px">（默认0.5%）</span>
            </el-form-item>

            <el-divider />
            <h4 style="margin-bottom: 12px; color: #409eff">工伤保险</h4>
            <el-form-item label="单位比例">
              <el-input-number
                v-model="insuranceForm.injury_employer_rate"
                :min="0" :max="1" :step="0.01" :precision="3"
                style="width: 200px"
              />
              <span style="margin-left: 8px">（默认0.2%）</span>
            </el-form-item>
            <el-form-item label="个人比例">
              <el-input-number
                v-model="insuranceForm.injury_employee_rate"
                :min="0" :max="1" :step="0.01" :precision="3"
                style="width: 200px"
              />
              <span style="margin-left: 8px">（默认0%，个人不缴）</span>
            </el-form-item>

            <el-divider />
            <h4 style="margin-bottom: 12px; color: #409eff">缴费基数</h4>
            <el-form-item label="社保基数下限">
              <el-input-number
                v-model="insuranceForm.insurance_base_min"
                :min="0" :step="100" :precision="0"
                style="width: 200px"
              />
              <span style="margin-left: 8px">元</span>
            </el-form-item>
            <el-form-item label="社保基数上限">
              <el-input-number
                v-model="insuranceForm.insurance_base_max"
                :min="0" :step="100" :precision="0"
                style="width: 200px"
              />
              <span style="margin-left: 8px">元</span>
            </el-form-item>

            <el-divider />
            <h4 style="margin-bottom: 12px; color: #409eff">住房公积金</h4>
            <el-form-item label="单位缴存比例">
              <el-input-number
                v-model="insuranceForm.housing_fund_employer_rate"
                :min="0"
                :max="1"
                :step="0.01"
                :precision="2"
                style="width: 200px"
              />
              <span style="margin-left: 8px">（默认5%）</span>
            </el-form-item>
            <el-form-item label="个人缴存比例">
              <el-input-number
                v-model="insuranceForm.housing_fund_employee_rate"
                :min="0"
                :max="1"
                :step="0.01"
                :precision="2"
                style="width: 200px"
              />
              <span style="margin-left: 8px">（默认5%）</span>
            </el-form-item>
            <el-form-item label="缴存基数下限">
              <el-input-number
                v-model="insuranceForm.housing_fund_base_min"
                :min="0"
                :step="100"
                :precision="0"
                style="width: 200px"
              />
              <span style="margin-left: 8px">元</span>
            </el-form-item>
            <el-form-item label="缴存基数上限">
              <el-input-number
                v-model="insuranceForm.housing_fund_base_max"
                :min="0"
                :step="100"
                :precision="0"
                style="width: 200px"
              />
              <span style="margin-left: 8px">元</span>
            </el-form-item>
          </el-form>
        </el-card>
      </el-tab-pane>

      <!-- 数据管理 -->
      <el-tab-pane label="数据管理" name="data">
        <el-row :gutter="20">
          <el-col :span="12">
            <el-card>
              <template #header>
                <span>数据备份与恢复</span>
              </template>
              <el-alert
                title="定期备份数据可防止意外丢失"
                type="info"
                :closable="false"
                show-icon
                style="margin-bottom: 16px"
              />
              <el-row :gutter="16" style="margin-bottom: 16px">
                <el-col :span="12">
                  <el-button
                    type="primary"
                    @click="backupData"
                    :loading="backingUp"
                    style="width: 100%"
                    >创建备份</el-button
                  >
                </el-col>
                <el-col :span="12">
                  <el-button
                    type="warning"
                    @click="restoreData"
                    style="width: 100%"
                    >恢复备份</el-button
                  >
                </el-col>
              </el-row>
              <div v-if="backups.length > 0">
                <h4 style="margin-bottom: 12px">历史备份列表</h4>
                <el-table :data="backups" stripe size="small" max-height="300">
                  <el-table-column
                    prop="created_at"
                    label="备份时间"
                    width="180"
                  />
                  <el-table-column
                    prop="file_size"
                    label="文件大小"
                    width="120"
                  >
                    <template #default="scope">{{
                      formatFileSize(scope.row.file_size)
                    }}</template>
                  </el-table-column>
                  <el-table-column label="操作" width="120">
                    <template #default="scope">
                      <el-button
                        size="small"
                        type="danger"
                        @click="deleteBackup(scope.row.file_name)"
                        >删除</el-button
                      >
                    </template>
                  </el-table-column>
                </el-table>
              </div>
              <el-empty v-else description="暂无备份记录" />
            </el-card>
          </el-col>
          <el-col :span="12">
            <el-card>
              <template #header>
                <span>数据导出</span>
              </template>
              <el-alert
                title="导出全部数据为 JSON 格式，可用于审计或迁移"
                type="info"
                :closable="false"
                show-icon
                style="margin-bottom: 16px"
              />
              <el-button
                type="success"
                @click="exportAllData"
                :loading="exporting"
                style="width: 100%"
                >导出全部数据</el-button
              >
              <div style="margin-top: 16px">
                <el-tag type="info"
                  >导出内容包含：员工信息、考勤记录、会计科目、凭证、发票、税务报表、系统设置</el-tag
                >
              </div>
            </el-card>
            <el-card style="margin-top: 16px">
              <template #header>
                <span>数据库信息</span>
              </template>
              <el-descriptions :column="1" border>
                <el-descriptions-item label="数据库类型"
                  >SQLite</el-descriptions-item
                >
                <el-descriptions-item label="数据位置">{{
                  dbPath
                }}</el-descriptions-item>
                <el-descriptions-item label="备份目录">{{
                  backupDir
                }}</el-descriptions-item>
              </el-descriptions>
            </el-card>
          </el-col>
        </el-row>
      </el-tab-pane>

      <!-- 关于 -->
      <el-tab-pane label="关于" name="about">
        <el-card>
          <template #header>
            <span>关于小微企业管理助手</span>
          </template>
          <el-descriptions :column="1" border>
            <el-descriptions-item label="软件名称"
              >小微企业管理助手</el-descriptions-item
            >
            <el-descriptions-item label="版本号">v1.0.0</el-descriptions-item>
            <el-descriptions-item label="技术架构"
              >Tauri 2.x + Vue 3 + Rust</el-descriptions-item
            >
            <el-descriptions-item label="数据库"
              >SQLite（本地存储）</el-descriptions-item
            >
            <el-descriptions-item label="功能说明"
              >专为小微企业设计的极简人事管理与专业财务管理工具，支持自动生成税务申报报表。</el-descriptions-item
            >
          </el-descriptions>
        </el-card>
      </el-tab-pane>
    </el-tabs>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { ElMessage, ElMessageBox } from "element-plus";

interface BackupInfo {
  file_name: string;
  file_size: number;
  created_at: string;
}

const activeTab = ref("company");
const saving = ref(false);
const savingInsurance = ref(false);
const backingUp = ref(false);
const exporting = ref(false);
const dbPath = ref("");
const backupDir = ref("");
const backups = ref<BackupInfo[]>([]);

const companyForm = reactive({
  company_name: "",
  company_tax_id: "",
  company_address: "",
  company_phone: "",
  company_bank: "",
  company_bank_account: "",
});

const taxForm = reactive({
  taxpayer_type: "一般纳税人",
  vat_rate: "0.13",
  cit_rate: 0.25,
  city_tax_rate: "0.07",
  education_surcharge: 0.03,
  local_surcharge: 0.02,
  small_profit_threshold: 3000000,
});

const insuranceForm = reactive({
  pension_employer_rate: 0.16,
  pension_employee_rate: 0.08,
  medical_employer_rate: 0.098,
  medical_employee_rate: 0.02,
  medical_employee_fixed: 3,
  unemployment_employer_rate: 0.005,
  unemployment_employee_rate: 0.005,
  injury_employer_rate: 0.002,
  injury_employee_rate: 0,
  insurance_base_min: 7162,
  insurance_base_max: 35283,
  housing_fund_employer_rate: 0.05,
  housing_fund_employee_rate: 0.05,
  housing_fund_base_min: 2420,
  housing_fund_base_max: 35283,
});

const formatFileSize = (bytes: number): string => {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + " KB";
  return (bytes / (1024 * 1024)).toFixed(1) + " MB";
};

const loadSettings = async () => {
  try {
    const settings: Record<string, string> = await invoke("get_all_settings");
    if (settings.company_name) companyForm.company_name = settings.company_name;
    if (settings.company_tax_id)
      companyForm.company_tax_id = settings.company_tax_id;
    if (settings.company_address)
      companyForm.company_address = settings.company_address;
    if (settings.company_phone)
      companyForm.company_phone = settings.company_phone;
    if (settings.company_bank) companyForm.company_bank = settings.company_bank;
    if (settings.company_bank_account)
      companyForm.company_bank_account = settings.company_bank_account;
    if (settings.taxpayer_type) taxForm.taxpayer_type = settings.taxpayer_type;
    if (settings.vat_rate) taxForm.vat_rate = settings.vat_rate;
    if (settings.cit_rate) taxForm.cit_rate = parseFloat(settings.cit_rate);
    if (settings.city_tax_rate) taxForm.city_tax_rate = settings.city_tax_rate;
    if (settings.education_surcharge)
      taxForm.education_surcharge = parseFloat(settings.education_surcharge);
    if (settings.local_surcharge)
      taxForm.local_surcharge = parseFloat(settings.local_surcharge);
    if (settings.small_profit_threshold)
      taxForm.small_profit_threshold = parseInt(
        settings.small_profit_threshold,
      );
    if (settings.insurance_employer_rate)
      insuranceForm.pension_employer_rate = parseFloat(settings.insurance_employer_rate);
    if (settings.insurance_employee_rate)
      insuranceForm.pension_employee_rate = parseFloat(settings.insurance_employee_rate);
    if (settings.pension_employer_rate)
      insuranceForm.pension_employer_rate = parseFloat(settings.pension_employer_rate);
    if (settings.pension_employee_rate)
      insuranceForm.pension_employee_rate = parseFloat(settings.pension_employee_rate);
    if (settings.medical_employer_rate)
      insuranceForm.medical_employer_rate = parseFloat(settings.medical_employer_rate);
    if (settings.medical_employee_rate)
      insuranceForm.medical_employee_rate = parseFloat(settings.medical_employee_rate);
    if (settings.unemployment_employer_rate)
      insuranceForm.unemployment_employer_rate = parseFloat(settings.unemployment_employer_rate);
    if (settings.unemployment_employee_rate)
      insuranceForm.unemployment_employee_rate = parseFloat(settings.unemployment_employee_rate);
    if (settings.injury_employer_rate)
      insuranceForm.injury_employer_rate = parseFloat(settings.injury_employer_rate);
    if (settings.injury_employee_rate)
      insuranceForm.injury_employee_rate = parseFloat(settings.injury_employee_rate);
    if (settings.insurance_base_min)
      insuranceForm.insurance_base_min = parseInt(settings.insurance_base_min);
    if (settings.insurance_base_max)
      insuranceForm.insurance_base_max = parseInt(settings.insurance_base_max);
    if (settings.housing_fund_employer_rate)
      insuranceForm.housing_fund_employer_rate = parseFloat(settings.housing_fund_employer_rate);
    if (settings.housing_fund_employee_rate)
      insuranceForm.housing_fund_employee_rate = parseFloat(settings.housing_fund_employee_rate);
    if (settings.housing_fund_base_min)
      insuranceForm.housing_fund_base_min = parseInt(settings.housing_fund_base_min);
    if (settings.housing_fund_base_max)
      insuranceForm.housing_fund_base_max = parseInt(settings.housing_fund_base_max);
  } catch (e) {
    console.error("加载设置失败:", e);
  }
};

const saveCompanyInfo = async () => {
  saving.value = true;
  try {
    const settings = Object.entries(companyForm).map(([key, value]) => ({
      key,
      value: String(value),
    }));
    await invoke("save_settings_batch", { settings });
    ElMessage.success("企业信息保存成功");
  } catch (e) {
    ElMessage.error("保存失败: " + e);
  } finally {
    saving.value = false;
  }
};

const saveTaxParams = async () => {
  saving.value = true;
  try {
    const settings = Object.entries(taxForm).map(([key, value]) => ({
      key,
      value: String(value),
    }));
    await invoke("save_settings_batch", { settings });
    ElMessage.success("税务参数保存成功");
  } catch (e) {
    ElMessage.error("保存失败: " + e);
  } finally {
    saving.value = false;
  }
};

const saveInsuranceParams = async () => {
  savingInsurance.value = true;
  try {
    const settings = Object.entries(insuranceForm).map(([key, value]) => ({
      key,
      value: String(value),
    }));
    await invoke("save_settings_batch", { settings });
    ElMessage.success("社保公积金参数保存成功");
  } catch (e) {
    ElMessage.error("保存失败: " + e);
  } finally {
    savingInsurance.value = false;
  }
};

const backupData = async () => {
  backingUp.value = true;
  try {
    const path = await invoke("backup_database");
    ElMessage.success(`备份成功: ${path}`);
    await loadBackups();
  } catch (e) {
    ElMessage.error("备份失败: " + e);
  } finally {
    backingUp.value = false;
  }
};

const restoreData = async () => {
  try {
    await ElMessageBox.confirm(
      "恢复备份将覆盖当前所有数据，此操作不可撤销，确定继续吗？",
      "警告",
      {
        type: "warning",
        confirmButtonText: "确定恢复",
        cancelButtonText: "取消",
      },
    );
    if (backups.value.length === 0) {
      ElMessage.warning("没有可恢复的备份");
      return;
    }
    const latestBackup = backups.value[0];
    await invoke("restore_database", { backupPath: latestBackup.file_name });
    ElMessage.success("数据恢复成功，请重启应用");
  } catch (e: any) {
    if (e !== "cancel") {
      ElMessage.error("恢复失败: " + e);
    }
  }
};

const loadBackups = async () => {
  try {
    backups.value = await invoke("list_backups");
  } catch (e) {
    console.error("加载备份列表失败:", e);
  }
};

const deleteBackup = async (fileName: string) => {
  try {
    await ElMessageBox.confirm("确定要删除此备份吗？", "提示");
    await invoke("delete_backup", { fileName });
    ElMessage.success("备份已删除");
    await loadBackups();
  } catch {
    // 取消操作
  }
};

const exportAllData = async () => {
  exporting.value = true;
  try {
    const path = await invoke("export_all_data");
    ElMessage.success(`数据已导出到: ${path}`);
  } catch (e) {
    ElMessage.error("导出失败: " + e);
  } finally {
    exporting.value = false;
  }
};

onMounted(async () => {
  await loadSettings();
  await loadBackups();
  dbPath.value = "用户数据目录/small-business-manager/data.db";
  backupDir.value = "用户数据目录/small-business-manager/backups/";
});
</script>

<style scoped>
.form-tip {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}
</style>
