/**
 * 分析历史记录管理工具
 */

import { AnalysisHistoryRecord, AnalysisHistoryStorage } from '../types/analysisHistory';
import { invoke } from '@tauri-apps/api/tauri';
import { save } from '@tauri-apps/api/dialog';
import { copyFile, removeFile, exists } from '@tauri-apps/api/fs';
import { getUserMaxHistoryCount } from './storageUtils';

export class AnalysisHistoryManager {
  private static readonly STORAGE_KEY = 'analysis-history';
  private static readonly DEFAULT_MAX_RECORDS = 50;
  private static readonly TEMP_RESULTS_DIR = 'temp_analysis_results';

  /**
   * 获取分析历史记录
   */
  static getHistory(): AnalysisHistoryStorage {
    try {
      const stored = localStorage.getItem(this.STORAGE_KEY);
      if (stored) {
        const parsed = JSON.parse(stored);
        // 转换时间戳
        parsed.lastUpdated = new Date(parsed.lastUpdated);
        parsed.records = parsed.records.map((record: any) => ({
          ...record,
          timestamp: new Date(record.timestamp)
        }));
        return parsed;
      }
    } catch (error) {
      console.warn('Failed to load analysis history:', error);
    }
    
    return {
      records: [],
      maxRecords: this.DEFAULT_MAX_RECORDS,
      lastUpdated: new Date()
    };
  }

  /**
   * 保存分析历史记录
   */
  static saveHistory(history: AnalysisHistoryStorage): void {
    try {
      // 确保不超过最大记录数
      if (history.records.length > history.maxRecords) {
        // 删除最旧的记录对应的文件
        const recordsToDelete = history.records.slice(history.maxRecords);
        for (const record of recordsToDelete) {
          this.deleteRecordFile(record).catch(console.warn);
        }
        history.records = history.records.slice(0, history.maxRecords);
      }
      
      history.lastUpdated = new Date();
      localStorage.setItem(this.STORAGE_KEY, JSON.stringify(history));
    } catch (error) {
      console.error('Failed to save analysis history:', error);
    }
  }

  /**
   * 添加新的分析记录
   */
  static addRecord(record: AnalysisHistoryRecord): { needsCleanup: boolean } {
    const history = this.getHistory();
    const maxHistoryCount = getUserMaxHistoryCount();
    
    // 添加到历史记录开头（最新的在前面）
    history.records.unshift(record);
    
    // 检查是否需要清理
    const needsCleanup = history.records.length > maxHistoryCount;
    
    // 如果不需要清理，直接保存；否则暂时不自动删除，让用户决定
    if (!needsCleanup) {
      this.saveHistory(history);
    } else {
      // 即使超出限制，也保存当前记录，让用户看到完整的历史并决定如何处理
      this.saveHistory(history);
    }
    
    return { needsCleanup };
  }

  /**
   * 删除分析记录
   */
  static async deleteRecord(recordId: string): Promise<boolean> {
    try {
      const history = this.getHistory();
      const recordIndex = history.records.findIndex(r => r.id === recordId);
      
      if (recordIndex === -1) {
        return false;
      }
      
      const record = history.records[recordIndex];
      
      // 删除文件
      await this.deleteRecordFile(record);
      
      // 从历史记录中移除
      history.records.splice(recordIndex, 1);
      this.saveHistory(history);
      
      return true;
    } catch (error) {
      console.error('Failed to delete analysis record:', error);
      return false;
    }
  }

  /**
   * 删除记录对应的文件
   */
  private static async deleteRecordFile(record: AnalysisHistoryRecord): Promise<void> {
    try {
      // 删除主输出文件
      if (await exists(record.outputFile.path)) {
        await removeFile(record.outputFile.path);
      }
      
      // 删除场外资金池记录文件（如果存在）
      if (record.offsitePoolFile && await exists(record.offsitePoolFile.path)) {
        await removeFile(record.offsitePoolFile.path);
      }
    } catch (error) {
      console.warn(`Failed to delete files for record ${record.id}:`, error);
    }
  }

  /**
   * 打开分析结果文件
   */
  static async openRecord(record: AnalysisHistoryRecord): Promise<boolean> {
    try {
      console.log('尝试打开文件:', record.outputFile.path);
      
      const fileExists = await exists(record.outputFile.path);
      console.log('文件是否存在:', fileExists);
      
      if (!fileExists) {
        console.error('文件不存在:', record.outputFile.path);
        throw new Error('文件不存在');
      }
      
      console.log('调用open_file命令...');
      await invoke('open_file', { filePath: record.outputFile.path });
      console.log('open_file命令调用成功');
      return true;
    } catch (error) {
      console.error('Failed to open analysis result:', error);
      console.error('Error details:', error);
      return false;
    }
  }

  /**
   * 打开场外资金池记录文件
   */
  static async openOffsitePoolRecord(record: AnalysisHistoryRecord): Promise<boolean> {
    try {
      if (!record.offsitePoolFile) {
        console.warn('该记录没有场外资金池记录文件');
        return false;
      }

      console.log('尝试打开场外资金池记录文件:', record.offsitePoolFile.path);
      
      const fileExists = await exists(record.offsitePoolFile.path);
      console.log('场外资金池记录文件是否存在:', fileExists);
      
      if (!fileExists) {
        console.error('场外资金池记录文件不存在:', record.offsitePoolFile.path);
        throw new Error('场外资金池记录文件不存在');
      }
      
      console.log('调用open_file命令打开场外资金池记录...');
      await invoke('open_file', { filePath: record.offsitePoolFile.path });
      console.log('场外资金池记录文件打开成功');
      return true;
    } catch (error) {
      console.error('Failed to open offsite pool record:', error);
      console.error('Error details:', error);
      return false;
    }
  }

  /**
   * 另存为分析结果
   */
  static async saveAsRecord(record: AnalysisHistoryRecord): Promise<boolean> {
    try {
      if (!(await exists(record.outputFile.path))) {
        throw new Error('源文件不存在');
      }
      
      // 显示保存对话框
      const savePath = await save({
        defaultPath: record.outputFile.name,
        filters: [{
          name: 'Excel文件',
          extensions: ['xlsx']
        }]
      });
      
      if (!savePath) {
        return false; // 用户取消
      }
      
      // 复制文件
      await copyFile(record.outputFile.path, savePath);
      return true;
    } catch (error) {
      console.error('Failed to save as analysis result:', error);
      return false;
    }
  }

  /**
   * 生成临时结果文件路径
   */
  static generateTempResultPath(algorithm: string, inputFileName: string): string {
    const timestamp = new Date().toISOString().slice(0, 19).replace(/[:\-T]/g, '');
    const algorithmName = algorithm === 'FIFO' ? 'FIFO' : '差额计算法';
    const fileName = `${algorithmName}_${inputFileName.replace('.xlsx', '')}_${timestamp}.xlsx`;
    return `${this.TEMP_RESULTS_DIR}/${fileName}`;
  }

  /**
   * 创建分析记录ID
   */
  static generateRecordId(): string {
    return `analysis_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
  }

  /**
   * 格式化算法显示名称
   */
  static formatAlgorithmName(algorithm: string): string {
    switch (algorithm) {
      case 'FIFO':
        return 'FIFO算法';
      case 'BALANCE_METHOD':
        return '差额计算法';
      default:
        return algorithm;
    }
  }

  /**
   * 格式化文件大小
   */
  static formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  /**
   * 格式化处理时间
   */
  static formatProcessingTime(milliseconds: number): string {
    if (milliseconds < 1000) {
      return `${milliseconds}ms`;
    } else if (milliseconds < 60000) {
      return `${(milliseconds / 1000).toFixed(1)}s`;
    } else {
      const minutes = Math.floor(milliseconds / 60000);
      const seconds = ((milliseconds % 60000) / 1000).toFixed(0);
      return `${minutes}m ${seconds}s`;
    }
  }
}