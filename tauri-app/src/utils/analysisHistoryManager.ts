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
  static async deleteRecord(recordId: string): Promise<{
    success: boolean;
    allDeleted: boolean;
    errors: string[];
    partiallyDeleted: boolean;
  }> {
    try {
      const history = this.getHistory();
      const recordIndex = history.records.findIndex(r => r.id === recordId);
      
      if (recordIndex === -1) {
        return { success: false, allDeleted: false, errors: ['记录不存在'], partiallyDeleted: false };
      }
      
      const record = history.records[recordIndex];
      const result = await this.deleteRecordFiles(record);
      
      if (result.allDeleted) {
        // 所有文件都删除成功，移除记录
        history.records.splice(recordIndex, 1);
        this.saveHistory(history);
        return { success: true, allDeleted: true, errors: [], partiallyDeleted: false };
      } else if (result.partiallyDeleted) {
        // 部分文件删除成功，更新记录状态但保留记录
        history.records[recordIndex] = result.updatedRecord;
        this.saveHistory(history);
        return { success: true, allDeleted: false, errors: result.errors, partiallyDeleted: true };
      } else {
        // 所有文件删除失败，保留记录不变
        return { success: false, allDeleted: false, errors: result.errors, partiallyDeleted: false };
      }
    } catch (error) {
      console.error('Failed to delete analysis record:', error);
      return { success: false, allDeleted: false, errors: [error instanceof Error ? error.message : '未知错误'], partiallyDeleted: false };
    }
  }

  /**
   * 删除记录对应的文件，返回详细结果
   */
  private static async deleteRecordFiles(record: AnalysisHistoryRecord): Promise<{
    allDeleted: boolean;
    partiallyDeleted: boolean;
    errors: string[];
    updatedRecord: AnalysisHistoryRecord;
  }> {
    const errors: string[] = [];
    let mainFileDeleted = false;
    let poolFileDeleted = false;
    
    // 创建更新后的记录副本
    const updatedRecord: AnalysisHistoryRecord = JSON.parse(JSON.stringify(record));
    
    // 尝试删除主输出文件
    try {
      if (await exists(record.outputFile.path)) {
        await removeFile(record.outputFile.path);
        mainFileDeleted = true;
        updatedRecord.outputFile.deleted = true;
        delete updatedRecord.outputFile.deleteError;
      } else {
        // 文件不存在，认为已删除
        mainFileDeleted = true;
        updatedRecord.outputFile.deleted = true;
      }
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : '未知错误';
      errors.push(`主分析结果删除失败: ${errorMsg}`);
      updatedRecord.outputFile.deleteError = errorMsg;
    }
    
    // 尝试删除场外资金池记录文件（如果存在）
    if (record.offsitePoolFile) {
      try {
        if (await exists(record.offsitePoolFile.path)) {
          await removeFile(record.offsitePoolFile.path);
          poolFileDeleted = true;
          updatedRecord.offsitePoolFile!.deleted = true;
          delete updatedRecord.offsitePoolFile!.deleteError;
        } else {
          // 文件不存在，认为已删除
          poolFileDeleted = true;
          updatedRecord.offsitePoolFile!.deleted = true;
        }
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : '未知错误';
        errors.push(`场外资金池记录删除失败: ${errorMsg}`);
        updatedRecord.offsitePoolFile!.deleteError = errorMsg;
      }
    } else {
      // 没有场外文件，认为该部分已完成
      poolFileDeleted = true;
    }
    
    const allDeleted = mainFileDeleted && poolFileDeleted;
    const partiallyDeleted = mainFileDeleted || poolFileDeleted;
    
    return {
      allDeleted,
      partiallyDeleted,
      errors,
      updatedRecord
    };
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

  /**
   * 检查单个记录的文件状态并更新
   */
  static async updateRecordFileStatus(record: AnalysisHistoryRecord): Promise<{
    updated: boolean;
    record: AnalysisHistoryRecord;
  }> {
    let updated = false;
    const updatedRecord = JSON.parse(JSON.stringify(record));

    // 检查主输出文件状态
    try {
      const mainFileExists = await exists(record.outputFile.path);
      const currentMainDeleted = record.outputFile.deleted || false;
      
      if (!mainFileExists && !currentMainDeleted) {
        updatedRecord.outputFile.deleted = true;
        updatedRecord.outputFile.deleteError = '文件已被外部删除';
        updated = true;
      } else if (mainFileExists && currentMainDeleted) {
        // 文件重新出现了，清除删除状态
        delete updatedRecord.outputFile.deleted;
        delete updatedRecord.outputFile.deleteError;
        updated = true;
      }
    } catch (error) {
      console.warn('检查主输出文件状态时出错:', error);
    }

    // 检查场外资金池文件状态
    if (record.offsitePoolFile) {
      try {
        const poolFileExists = await exists(record.offsitePoolFile.path);
        const currentPoolDeleted = record.offsitePoolFile.deleted || false;
        
        if (!poolFileExists && !currentPoolDeleted) {
          updatedRecord.offsitePoolFile!.deleted = true;
          updatedRecord.offsitePoolFile!.deleteError = '文件已被外部删除';
          updated = true;
        } else if (poolFileExists && currentPoolDeleted) {
          // 文件重新出现了，清除删除状态
          delete updatedRecord.offsitePoolFile!.deleted;
          delete updatedRecord.offsitePoolFile!.deleteError;
          updated = true;
        }
      } catch (error) {
        console.warn('检查场外资金池文件状态时出错:', error);
      }
    }

    return { updated, record: updatedRecord };
  }

  /**
   * 批量更新所有记录的文件状态
   */
  static async syncAllRecordsFileStatus(): Promise<{
    totalChecked: number;
    totalUpdated: number;
    errors: string[];
  }> {
    const history = this.getHistory();
    const errors: string[] = [];
    let totalUpdated = 0;
    let updated = false;

    console.log(`开始同步 ${history.records.length} 条历史记录的文件状态...`);

    for (let i = 0; i < history.records.length; i++) {
      try {
        const result = await this.updateRecordFileStatus(history.records[i]);
        if (result.updated) {
          history.records[i] = result.record;
          totalUpdated++;
          updated = true;
        }
      } catch (error) {
        const errorMsg = error instanceof Error ? error.message : '未知错误';
        errors.push(`记录 ${history.records[i].id} 状态更新失败: ${errorMsg}`);
      }
    }

    // 如果有更新，保存历史记录
    if (updated) {
      this.saveHistory(history);
      console.log(`文件状态同步完成，更新了 ${totalUpdated} 条记录`);
    }

    return {
      totalChecked: history.records.length,
      totalUpdated,
      errors
    };
  }

  /**
   * 获取包含实时文件状态的历史记录
   */
  static async getHistoryWithRealTimeStatus(): Promise<AnalysisHistoryStorage> {
    const history = this.getHistory();
    let hasUpdates = false;

    // 并发检查所有记录的文件状态
    const updatePromises = history.records.map(async (record, index) => {
      const result = await this.updateRecordFileStatus(record);
      if (result.updated) {
        hasUpdates = true;
        return { index, record: result.record };
      }
      return null;
    });

    const updates = await Promise.all(updatePromises);

    // 应用更新
    updates.forEach(update => {
      if (update) {
        history.records[update.index] = update.record;
      }
    });

    // 如果有更新，保存到本地存储
    if (hasUpdates) {
      this.saveHistory(history);
    }

    return history;
  }
}