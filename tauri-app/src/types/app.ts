// 应用相关类型定义

export type ThemeMode = 'light' | 'dark' | 'auto';

export type Language = 'zh' | 'en';

export interface AppSettings {
  theme: ThemeMode;
  language: Language;
  autoSave: boolean;
  notifications: boolean;
  maxHistoryRecords: number;
}

export interface NotificationOptions {
  type: 'success' | 'error' | 'warning' | 'info';
  title: string;
  message?: string;
  duration?: number;
}

export interface FileInfo {
  path: string;
  name: string;
  size: number;
  lastModified: Date;
}

export interface AuditTask {
  id: string;
  name: string;
  algorithm: 'FIFO' | 'BALANCE_METHOD';
  inputFile: string;
  outputFile?: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  progress: number;
  startTime?: Date;
  endTime?: Date;
  errorMessage?: string;
}

export interface QueryHistory {
  id: string;
  timestamp: Date;
  fileName: string;
  rowNumber: number;
  algorithm: 'FIFO' | 'BALANCE_METHOD';
  result?: any;
}

// 审计结果数据结构
export interface AuditSummary {
  algorithm: string;
  totalRows: number;
  processingTime: number;
  finalBalance: {
    personal: number;
    company: number;
    total: number;
  };
  misappropriation: {
    cumulative: number;
    net: number;
    returned: number;
  };
  advance: {
    cumulative: number;
  };
  profits: {
    personal: number;
    company: number;
  };
}

// 时点查询结果数据结构
export interface TimePointResult {
  rowNumber: number;
  timestamp: string;
  transaction: {
    income?: number;
    expense?: number;
    balance: number;
    fundAttribute: string;
  };
  balanceStatus: {
    personal: number;
    company: number;
    total: number;
  };
  cumulativeStats: {
    misappropriation: number;
    advance: number;
    returnedPrincipal: number;
  };
}