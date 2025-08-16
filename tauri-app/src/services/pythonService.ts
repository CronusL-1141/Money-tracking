import { invoke } from '@tauri-apps/api/tauri';
import { 
  PythonEnvStatus, 
  AuditConfig as OldAuditConfig, 
  AuditResult as OldAuditResult, 
  TimePointQuery as OldTimePointQuery, 
  QueryResult as OldQueryResult 
} from '../types/python';
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

/**
 * Python环境和审计服务 - 使用新的Rust后端接口
 */
export class PythonService {
  
  /**
   * 检查Python环境状态
   */
  static async checkEnvironment(): Promise<any> {
    try {
      return await RustCommands.checkPythonEnv();
    } catch (error) {
      console.error('检查Python环境失败:', error);
      throw new Error(`无法检查Python环境: ${error}`);
    }
  }
  
  /**
   * 获取可用算法列表
   */
  static async getAvailableAlgorithms(): Promise<string[]> {
    try {
      return await RustCommands.getAlgorithms();
    } catch (error) {
      console.error('获取算法列表失败:', error);
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
      console.error('审计分析失败:', error);
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
      console.error('时点查询失败:', error);
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
      console.error('获取查询历史失败:', error);
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
      console.error('清空查询历史失败:', error);
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
      console.error('删除历史记录失败:', error);
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
      console.error('获取进程状态失败:', error);
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
      console.error('获取应用配置失败:', error);
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
      console.error('更新应用配置失败:', error);
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
      console.error('获取文件信息失败:', error);
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
      console.error('导出查询结果失败:', error);
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
      console.error('验证文件路径失败:', error);
      throw new Error(`无法验证文件路径: ${error}`);
    }
  }
}

/**
 * 辅助函数：检查Python环境
 */
export async function checkPythonEnvironment(): Promise<any> {
  return PythonService.checkEnvironment();
}

/**
 * 辅助函数：获取算法列表
 */
export async function getAlgorithms(): Promise<string[]> {
  return PythonService.getAvailableAlgorithms();
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
  return PythonService.runAudit(config);
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
  return PythonService.runAudit(config);
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
  return PythonService.executeTimePointQuery(query);
}

/**
 * 辅助函数：获取查询历史
 */
export async function getQueryHistory(): Promise<QueryHistory[]> {
  return PythonService.getQueryHistory();
}

/**
 * 辅助函数：获取进程状态
 */
export async function getProcessStatus(): Promise<ProcessStatus> {
  return PythonService.getProcessStatus();
}

/**
 * 辅助函数：获取应用配置
 */
export async function getAppConfig(): Promise<AppConfig> {
  return PythonService.getAppConfig();
}

/**
 * 辅助函数：更新应用配置
 */
export async function updateAppConfig(config: AppConfig): Promise<void> {
  return PythonService.updateAppConfig(config);
}

/**
 * 辅助函数：获取文件信息
 */
export async function getFileInfo(path: string): Promise<FileInfo> {
  return PythonService.getFileInfo(path);
}

/**
 * 辅助函数：验证文件路径
 */
export async function validateFilePath(path: string): Promise<boolean> {
  return PythonService.validateFilePath(path);
}