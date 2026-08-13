import { createRouter, createWebHistory } from "vue-router";
import type { RouteRecordRaw } from "vue-router";

const routes: RouteRecordRaw[] = [
  {
    path: "/",
    component: () => import("@/views/Layout.vue"),
    children: [
      {
        path: "",
        name: "Dashboard",
        component: () => import("@/views/dashboard/Index.vue"),
      },
      {
        path: "hr",
        name: "HR",
        component: () => import("@/views/hr/Index.vue"),
        children: [
          {
            path: "employees",
            name: "EmployeeList",
            component: () => import("@/views/hr/EmployeeList.vue"),
          },
          {
            path: "attendance",
            name: "Attendance",
            component: () => import("@/views/hr/Attendance.vue"),
          },
        ],
      },
      {
        path: "payroll",
        name: "Payroll",
        component: () => import("@/views/payroll/Index.vue"),
        redirect: "/payroll/salary",
        children: [
          {
            path: "salary",
            name: "Salary",
            component: () => import("@/views/payroll/Payroll.vue"),
          },
          {
            path: "performance",
            name: "Performance",
            component: () => import("@/views/payroll/Performance.vue"),
          },
          {
            path: "bonus",
            name: "Bonus",
            component: () => import("@/views/payroll/Bonus.vue"),
          },
          {
            path: "target",
            name: "Target",
            component: () => import("@/views/payroll/Target.vue"),
          },
        ],
      },
      {
        path: "finance",
        name: "Finance",
        component: () => import("@/views/finance/Index.vue"),
        children: [
          {
            path: "accounting",
            name: "Accounting",
            component: () => import("@/views/finance/Accounting.vue"),
          },
          {
            path: "invoices",
            name: "Invoices",
            component: () => import("@/views/finance/Invoices.vue"),
          },
          {
            path: "tax-reports",
            name: "TaxReports",
            component: () => import("@/views/finance/TaxReports.vue"),
          },
          {
            path: "tax-calendar",
            name: "TaxCalendar",
            component: () => import("@/views/finance/TaxCalendar.vue"),
          },
        ],
      },
      {
        path: "settings",
        name: "Settings",
        component: () => import("@/views/settings/Index.vue"),
      },
      {
        path: "data",
        name: "DataManagement",
        component: () => import("@/views/data/Index.vue"),
      },
    ],
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
