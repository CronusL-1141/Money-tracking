/**
 * Rust后端命令接口定义
 * 这些类型对应Rust中定义的结构体和命令
 */

import { invoke } from '@tauri-apps/api/tauri';

// 数据类型定义
export interface AuditConfig {
  algorithm: string;
  input_file: string;
  output_file?: string | null;
}

export interface AuditResult {
  success: boolean;
  message: string;
  data?: any;
  output_files: string[];
}

export interface TimePointQuery {
  file_path: string;
  row_number: number;
  algorithm: string;
}

export interface QueryResult {
  success: boolean;
  data?: any;
  message: string;
  processing_time?: number;
  target_row?: number;
  algorithm?: string;
  total_rows?: number;
  query_time?: string;
  target_row_data?: any;
  tracker_state?: any;
  processing_stats?: any;
  recent_steps?: any[];
  errors?: any[];
  available_fund_pools?: FundPool[];
}

export interface FundPool {
  name: string;
  total_amount: number;
  personal_ratio: number;
  company_ratio: number;
}

export interface FundPoolRecord {
  交易时间: string;
  资金池名称: string;
  入金: number | string;
  出金: number | string;
  总余额: number | string;
  单笔资金占比: string;
  总资金占比: string;
}

export interface FundPoolQueryResult {
  success: boolean;
  message?: string;
  pool_name?: string;
  records?: FundPoolRecord[];
  summary?: {
    total_inflow: number;
    total_outflow: number;
    current_balance: number;
    record_count: number;
  };
}

export interface QueryHistory {
  id: string;
  timestamp: string;
  file_path: string;
  row_number: number;
  algorithm: string;
  result?: string;
}

export interface AppConfig {
  default_algorithm: string;
  auto_export: boolean;
  max_history: number;
  language: string;
  theme: string;
}

export interface FileInfo {
  path: string;
  name: string;
  size: number;
  modified: string;
  exists: boolean;
}

export interface ProcessStatus {
  running: boolean;
  command?: string;
  progress?: number;
  message?: string;
  output_log: string[];
}

// Rust命令调用接口
export class RustCommands {
  /**
   * 获取可用算法列表
   */
  static async getAlgorithms(): Promise<string[]> {
    return invoke<string[]>('get_algorithms');
  }

  /**
   * 运行审计分析
   */
  static async runAudit(config: AuditConfig): Promise<AuditResult> {
    return invoke<AuditResult>('run_audit', { config });
  }

  /**
   * 时点查询
   */
  static async timePointQuery(query: TimePointQuery): Promise<QueryResult> {
    return invoke<QueryResult>('time_point_query', { query });
  }

  /**
   * 检查Python环境
   */
  static async checkPythonEnv(): Promise<any> {
    return invoke<any>('check_python_env');
  }

  /**
   * 获取查询历史
   */
  static async getQueryHistory(): Promise<QueryHistory[]> {
    return invoke<QueryHistory[]>('get_query_history');
  }

  /**
   * 清空查询历史
   */
  static async clearQueryHistory(): Promise<void> {
    return invoke<void>('clear_query_history');
  }

  /**
   * 删除历史记录项
   */
  static async deleteQueryHistoryItem(id: string): Promise<boolean> {
    return invoke<boolean>('delete_query_history_item', { id });
  }

  /**
   * 获取进程状态
   */
  static async getProcessStatus(): Promise<ProcessStatus> {
    return invoke<ProcessStatus>('get_process_status');
  }

  /**
   * 获取应用配置
   */
  static async getAppConfig(): Promise<AppConfig> {
    return invoke<AppConfig>('get_app_config');
  }

  /**
   * 更新应用配置
   */
  static async updateAppConfig(newConfig: AppConfig): Promise<void> {
    return invoke<void>('update_app_config', { newConfig });
  }

  /**
   * 获取文件信息
   */
  static async getFileInfo(path: string): Promise<FileInfo> {
    return invoke<FileInfo>('get_file_info', { path });
  }

  /**
   * 导出查询结果
   */
  static async exportQueryResult(queryId: string, outputPath: string): Promise<boolean> {
    return invoke<boolean>('export_query_result', { 
      queryId, 
      outputPath 
    });
  }

  /**
   * 验证文件路径
   */
  static async validateFilePath(path: string): Promise<boolean> {
    return invoke<boolean>('validate_file_path', { path });
  }

  /**
   * 资金池查询
   */
  static async queryFundPool(poolName: string, filePath: string, rowNumber: number, algorithm: string): Promise<FundPoolQueryResult> {
    return invoke<FundPoolQueryResult>('query_fund_pool', { 
      poolName, 
      filePath, 
      rowNumber, 
      algorithm 
    });
  }
}

// 便捷的导出函数
export const {
  getAlgorithms,
  runAudit,
  timePointQuery,
  checkPythonEnv,
  getQueryHistory,
  clearQueryHistory,
  deleteQueryHistoryItem,
  getProcessStatus,
  getAppConfig,
  updateAppConfig,
  getFileInfo,
  exportQueryResult,
  validateFilePath,
  queryFundPool
} = RustCommands;