import { invoke } from '@tauri-apps/api/tauri';
import { 
  RustCommands, 
  AuditConfig, 
  AuditResult, 
  TimePointQuery, 
  QueryResult,
  QueryHistory,
  ProcessStatus,
  AppConfig,
  FileInfo
} from '../types/rust-commands';
import { AnalysisHistoryManager } from '../utils/analysisHistoryManager';

/**
 * 系统环境状态接口
 */
export interface SystemEnvStatus {
  system_available: boolean;
  file_system_access: boolean;
  temp_directory_access: boolean;
  work_directory_writable: boolean;
  memory_available: boolean;
  system_info: string;
  work_directory: string;
  backend_engine: string;
  backend_version: string;
  is_dev_mode?: boolean;
}

/**
 * 系统环境和审计服务 - 使用Rust后端
 */
export class SystemService {
  
  /**
   * 检查系统环境状态
   */
  static async checkEnvironment(): Promise<SystemEnvStatus> {
    try {
      return await RustCommands.checkSystemEnv();
    } catch (error) {
      console.error('Failed to check system environment:', error);
      throw new Error(`无法检查系统环境: ${error}`);
    }
  }
  
  /**
   * 获取可用算法列表
   */
  static async getAvailableAlgorithms(): Promise<string[]> {
    try {
      return await RustCommands.getAlgorithms();
    } catch (error) {
      console.error('Failed to get algorithm list:', error);
      throw new Error(`无法获取算法列表: ${error}`);
    }
  }
  
  /**
   * 运行审计分析
   */
  static async runAudit(config: AuditConfig): Promise<AuditResult> {
    try {
      return await RustCommands.runAudit(config);
    } catch (error) {
      console.error('Audit analysis failed:', error);
      throw new Error(`审计分析执行失败: ${error}`);
    }
  }
  
  /**
   * 执行时点查询
   */
  static async executeTimePointQuery(query: TimePointQuery): Promise<QueryResult> {
    try {
      return await RustCommands.timePointQuery(query);
    } catch (error) {
      console.error('Time point query failed:', error);
      throw new Error(`时点查询执行失败: ${error}`);
    }
  }

  /**
   * 获取查询历史
   */
  static async getQueryHistory(): Promise<QueryHistory[]> {
    try {
      return await RustCommands.getQueryHistory();
    } catch (error) {
      console.error('Failed to get query history:', error);
      throw new Error(`无法获取查询历史: ${error}`);
    }
  }

  /**
   * 清空查询历史
   */
  static async clearQueryHistory(): Promise<void> {
    try {
      await RustCommands.clearQueryHistory();
    } catch (error) {
      console.error('Failed to clear query history:', error);
      throw new Error(`无法清空查询历史: ${error}`);
    }
  }

  /**
   * 删除历史记录项
   */
  static async deleteQueryHistoryItem(id: string): Promise<boolean> {
    try {
      return await RustCommands.deleteQueryHistoryItem(id);
    } catch (error) {
      console.error('Failed to delete history record:', error);
      throw new Error(`无法删除历史记录: ${error}`);
    }
  }

  /**
   * 获取进程状态
   */
  static async getProcessStatus(): Promise<ProcessStatus> {
    try {
      return await RustCommands.getProcessStatus();
    } catch (error) {
      console.error('Failed to get process status:', error);
      throw new Error(`无法获取进程状态: ${error}`);
    }
  }

  /**
   * 获取应用配置
   */
  static async getAppConfig(): Promise<AppConfig> {
    try {
      return await RustCommands.getAppConfig();
    } catch (error) {
      console.error('Failed to get app config:', error);
      throw new Error(`无法获取应用配置: ${error}`);
    }
  }

  /**
   * 更新应用配置
   */
  static async updateAppConfig(config: AppConfig): Promise<void> {
    try {
      await RustCommands.updateAppConfig(config);
    } catch (error) {
      console.error('Failed to update app config:', error);
      throw new Error(`无法更新应用配置: ${error}`);
    }
  }

  /**
   * 获取文件信息
   */
  static async getFileInfo(path: string): Promise<FileInfo> {
    try {
      return await RustCommands.getFileInfo(path);
    } catch (error) {
      console.error('Failed to get file info:', error);
      throw new Error(`无法获取文件信息: ${error}`);
    }
  }

  /**
   * 导出查询结果
   */
  static async exportQueryResult(queryId: string, outputPath: string): Promise<boolean> {
    try {
      return await RustCommands.exportQueryResult(queryId, outputPath);
    } catch (error) {
      console.error('Failed to export query result:', error);
      throw new Error(`无法导出查询结果: ${error}`);
    }
  }

  /**
   * 验证文件路径
   */
  static async validateFilePath(path: string): Promise<boolean> {
    try {
      return await RustCommands.validateFilePath(path);
    } catch (error) {
      console.error('Failed to validate file path:', error);
      throw new Error(`无法验证文件路径: ${error}`);
    }
  }

  /**
   * 初始化系统服务 - 应用启动时调用
   */
  static async initialize(): Promise<{
    environmentStatus: SystemEnvStatus;
    fileSyncResult?: {
      totalChecked: number;
      totalUpdated: number;
      errors: string[];
    };
  }> {
    console.log('初始化系统服务...');
    
    try {
      // 检查系统环境
      const environmentStatus = await this.checkEnvironment();
      console.log('系统环境检查完成:', environmentStatus);
      
      let fileSyncResult;
      
      // 如果文件系统可访问，同步分析历史记录的文件状态
      if (environmentStatus.file_system_access) {
        console.log('开始同步分析历史记录文件状态...');
        fileSyncResult = await AnalysisHistoryManager.syncAllRecordsFileStatus();
        console.log('文件状态同步完成:', fileSyncResult);
      } else {
        console.warn('文件系统不可访问，跳过文件状态同步');
      }
      
      return {
        environmentStatus,
        fileSyncResult
      };
    } catch (error) {
      console.error('系统服务初始化失败:', error);
      throw new Error(`系统初始化失败: ${error}`);
    }
  }
}

/**
 * 辅助函数：检查系统环境
 */
export async function checkSystemEnvironment(): Promise<SystemEnvStatus> {
  return SystemService.checkEnvironment();
}

/**
 * 辅助函数：获取算法列表
 */
export async function getAlgorithms(): Promise<string[]> {
  return SystemService.getAvailableAlgorithms();
}

/**
 * 辅助函数：运行FIFO分析
 */
export async function runFIFOAnalysis(inputFile: string, outputFile?: string): Promise<AuditResult> {
  const config: AuditConfig = {
    algorithm: 'FIFO',
    input_file: inputFile,
    output_file: outputFile,
  };
  return SystemService.runAudit(config);
}

/**
 * 辅助函数：运行差额计算法分析
 */
export async function runBalanceMethodAnalysis(inputFile: string, outputFile?: string): Promise<AuditResult> {
  const config: AuditConfig = {
    algorithm: 'BALANCE_METHOD',
    input_file: inputFile,
    output_file: outputFile,
  };
  return SystemService.runAudit(config);
}

/**
 * 辅助函数：执行时点查询
 */
export async function queryTimePoint(
  filePath: string, 
  rowNumber: number, 
  algorithm: 'FIFO' | 'BALANCE_METHOD' = 'FIFO'
): Promise<QueryResult> {
  const query: TimePointQuery = {
    file_path: filePath,
    row_number: rowNumber,
    algorithm: algorithm,
  };
  return SystemService.executeTimePointQuery(query);
}

/**
 * 辅助函数：获取查询历史
 */
export async function getQueryHistory(): Promise<QueryHistory[]> {
  return SystemService.getQueryHistory();
}

/**
 * 辅助函数：获取进程状态
 */
export async function getProcessStatus(): Promise<ProcessStatus> {
  return SystemService.getProcessStatus();
}

/**
 * 辅助函数：获取应用配置
 */
export async function getAppConfig(): Promise<AppConfig> {
  return SystemService.getAppConfig();
}

/**
 * 辅助函数：更新应用配置
 */
export async function updateAppConfig(config: AppConfig): Promise<void> {
  return SystemService.updateAppConfig(config);
}

/**
 * 辅助函数：获取文件信息
 */
export async function getFileInfo(path: string): Promise<FileInfo> {
  return SystemService.getFileInfo(path);
}

/**
 * 辅助函数：验证文件路径
 */
export async function validateFilePath(path: string): Promise<boolean> {
  return SystemService.validateFilePath(path);
}