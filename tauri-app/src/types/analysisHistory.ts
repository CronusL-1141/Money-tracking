/**
 * 分析历史记录类型定义
 */

export interface AnalysisHistoryRecord {
  /** 唯一标识符 */
  id: string;
  
  /** 分析时间 */
  timestamp: Date;
  
  /** 算法类型 */
  algorithm: 'FIFO' | 'BALANCE_METHOD';
  
  /** 算法显示名称 */
  algorithmDisplayName: string;
  
  /** 输入文件信息 */
  inputFile: {
    name: string;
    path: string;
    size: number;
  };
  
  /** 输出文件信息 */
  outputFile: {
    name: string;
    path: string;
    size?: number;
  };
  
  /** 处理统计信息 */
  statistics: {
    totalRecords: number;
    processingTime: number; // 毫秒
    validationErrors: number;
    validationFixes: number;
  };
  
  /** 分析状态 */
  status: 'success' | 'failed' | 'processing';
  
  /** 错误信息（如果失败） */
  error?: string;
  
  /** 分析摘要 */
  summary?: {
    totalMisappropriation: number;
    totalAdvancePayment: number;
    finalBalance: number;
    investmentPools: number;
  };
}

export interface AnalysisHistoryStorage {
  /** 历史记录列表 */
  records: AnalysisHistoryRecord[];
  
  /** 最大记录数 */
  maxRecords: number;
  
  /** 最后更新时间 */
  lastUpdated: Date;
}