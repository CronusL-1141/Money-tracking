/**
 * 本地存储管理工具
 * 用于持久化应用数据到 localStorage
 */

import type { QueryHistory } from '../types/app';
import { AnalysisHistoryManager } from './analysisHistoryManager';

/**
 * 存储键名常量
 */
export const STORAGE_KEYS = {
  QUERY_HISTORY: 'audit_app_query_history',
  USER_SETTINGS: 'audit_app_user_settings',
  APP_VERSION: 'audit_app_version',
} as const;

/**
 * 通用存储接口
 */
interface StorageManager {
  get<T>(key: string, defaultValue?: T): T | null;
  set<T>(key: string, value: T): boolean;
  remove(key: string): boolean;
  clear(): boolean;
}

/**
 * LocalStorage 实现
 */
class LocalStorageManager implements StorageManager {
  /**
   * 从localStorage获取数据
   */
  get<T>(key: string, defaultValue?: T): T | null {
    try {
      const stored = localStorage.getItem(key);
      if (stored === null) {
        return defaultValue ?? null;
      }
      return JSON.parse(stored) as T;
    } catch (error) {
      console.error(`Failed to get data from localStorage for key: ${key}`, error);
      return defaultValue ?? null;
    }
  }

  /**
   * 向localStorage存储数据
   */
  set<T>(key: string, value: T): boolean {
    try {
      localStorage.setItem(key, JSON.stringify(value));
      return true;
    } catch (error) {
      console.error(`Failed to save data to localStorage for key: ${key}`, error);
      return false;
    }
  }

  /**
   * 从localStorage删除数据
   */
  remove(key: string): boolean {
    try {
      localStorage.removeItem(key);
      return true;
    } catch (error) {
      console.error(`Failed to remove data from localStorage for key: ${key}`, error);
      return false;
    }
  }

  /**
   * 清空localStorage
   */
  clear(): boolean {
    try {
      localStorage.clear();
      return true;
    } catch (error) {
      console.error('Failed to clear localStorage', error);
      return false;
    }
  }
}

// 创建存储管理器实例
export const storage = new LocalStorageManager();

/**
 * 获取用户设置的最大历史记录数量
 */
export function getUserMaxHistoryCount(): number {
  try {
    const settings = localStorage.getItem('app-settings');
    if (settings) {
      const parsed = JSON.parse(settings);
      return parsed.maxHistoryRecords || 100;
    }
  } catch (error) {
    console.warn('Failed to get user max history count:', error);
  }
  return 100; // 默认值
}

/**
 * 查询历史数据的序列化和反序列化
 */
export class QueryHistoryStorage {
  private static readonly STORAGE_KEY = STORAGE_KEYS.QUERY_HISTORY;

  /**
   * 序列化查询历史数据
   * 处理Date对象和复杂数据结构
   */
  private static serialize(history: QueryHistory[]): any[] {
    return history.map(item => ({
      ...item,
      timestamp: item.timestamp.toISOString(), // 将Date转换为ISO字符串
    }));
  }

  /**
   * 反序列化查询历史数据
   * 恢复Date对象
   */
  private static deserialize(data: any[]): QueryHistory[] {
    return data.map(item => ({
      ...item,
      timestamp: new Date(item.timestamp), // 将ISO字符串转换回Date对象
    }));
  }

  /**
   * 保存查询历史到本地存储
   */
  static save(history: QueryHistory[]): boolean {
    try {
      // 使用用户设置的限制历史记录数量
      const maxHistoryCount = getUserMaxHistoryCount();
      const limitedHistory = history.slice(0, maxHistoryCount);
      const serializedData = this.serialize(limitedHistory);
      
      return storage.set(this.STORAGE_KEY, {
        data: serializedData,
        version: '1.0',
        savedAt: new Date().toISOString(),
        count: limitedHistory.length,
        maxAllowed: maxHistoryCount
      });
    } catch (error) {
      console.error('Failed to save query history:', error);
      return false;
    }
  }

  /**
   * 从本地存储加载查询历史
   */
  static load(): QueryHistory[] {
    try {
      const stored = storage.get(this.STORAGE_KEY);
      
      if (!stored || !stored.data) {
        console.log('No query history found in storage');
        return [];
      }

      // 检查数据版本兼容性
      if (stored.version !== '1.0') {
        console.warn('Query history version mismatch, clearing data');
        this.clear();
        return [];
      }

      const deserializedData = this.deserialize(stored.data);
      console.log(`Loaded ${deserializedData.length} query history records from storage`);
      
      return deserializedData;
    } catch (error) {
      console.error('Failed to load query history:', error);
      return [];
    }
  }

  /**
   * 清空查询历史
   */
  static clear(): boolean {
    return storage.remove(this.STORAGE_KEY);
  }

  /**
   * 获取指定时间范围内的查询历史
   */
  static getRecordsInTimeRange(startDate: Date, endDate: Date): QueryHistory[] {
    const allHistory = this.load();
    return allHistory.filter(record => {
      const recordDate = new Date(record.timestamp);
      return recordDate >= startDate && recordDate <= endDate;
    });
  }

  /**
   * 删除指定时间范围内的查询历史
   */
  static deleteRecordsInTimeRange(startDate: Date, endDate: Date): { deleted: number; remaining: QueryHistory[] } {
    const allHistory = this.load();
    const toDelete = allHistory.filter(record => {
      const recordDate = new Date(record.timestamp);
      return recordDate >= startDate && recordDate <= endDate;
    });
    
    const remaining = allHistory.filter(record => {
      const recordDate = new Date(record.timestamp);
      return recordDate < startDate || recordDate > endDate;
    });

    this.save(remaining);
    
    return {
      deleted: toDelete.length,
      remaining
    };
  }

  /**
   * 批量删除指定的记录
   */
  static deleteRecordsByIds(recordIds: string[]): { deleted: number; remaining: QueryHistory[] } {
    const allHistory = this.load();
    const remaining = allHistory.filter(record => 
      !recordIds.includes(record.id || `${record.fileName}_${record.rowNumber}_${record.algorithm}`)
    );

    this.save(remaining);

    return {
      deleted: allHistory.length - remaining.length,
      remaining
    };
  }

  /**
   * 获取存储统计信息
   */
  static getStats(): { count: number; lastSaved?: string; version?: string } {
    const stored = storage.get(this.STORAGE_KEY);
    
    if (!stored) {
      return { count: 0 };
    }

    return {
      count: stored.count || 0,
      lastSaved: stored.savedAt,
      version: stored.version
    };
  }

  /**
   * 添加单个历史记录
   * 自动处理重复检测，允许超出限制但会提示用户
   */
  static addRecord(newRecord: QueryHistory): { history: QueryHistory[]; needsCleanup: boolean } {
    const currentHistory = this.load();
    const maxHistoryCount = getUserMaxHistoryCount();
    
    // 检查是否已存在相同的记录（基于文件名、行号、算法）
    const exists = currentHistory.some(item => 
      item.fileName === newRecord.fileName &&
      item.rowNumber === newRecord.rowNumber &&
      item.algorithm === newRecord.algorithm
    );

    let updatedHistory: QueryHistory[];
    
    if (exists) {
      // 如果存在，更新时间戳并移到最前面
      updatedHistory = [
        newRecord,
        ...currentHistory.filter(item => 
          !(item.fileName === newRecord.fileName &&
            item.rowNumber === newRecord.rowNumber &&
            item.algorithm === newRecord.algorithm)
        )
      ];
    } else {
      // 如果不存在，直接添加到最前面
      updatedHistory = [newRecord, ...currentHistory];
    }

    // 总是保存新记录，但检查是否需要提示清理
    const needsCleanup = updatedHistory.length > maxHistoryCount;
    this.save(updatedHistory);

    return { 
      history: updatedHistory,
      needsCleanup 
    };
  }
}

/**
 * 数据迁移工具
 */
export class DataMigration {
  /**
   * 检查是否需要数据迁移
   */
  static needsMigration(): boolean {
    const currentVersion = storage.get(STORAGE_KEYS.APP_VERSION);
    return currentVersion === null || currentVersion !== '1.0';
  }

  /**
   * 执行数据迁移
   */
  static migrate(): void {
    try {
      console.log('Starting data migration...');
      
      // 设置当前版本
      storage.set(STORAGE_KEYS.APP_VERSION, '1.0');
      
      console.log('Data migration completed');
    } catch (error) {
      console.error('Data migration failed:', error);
    }
  }
}

/**
 * 分析历史记录存储统计工具
 */
export class AnalysisHistoryStorage {
  /**
   * 检查是否需要清理分析历史记录
   */
  static checkNeedsCleanup(): boolean {
    const maxHistoryCount = getUserMaxHistoryCount();
    const history = AnalysisHistoryManager.getHistory();
    return history.records.length > maxHistoryCount;
  }
  /**
   * 获取分析历史统计信息
   */
  static getStats(): { count: number; lastAnalysis?: string; totalSize?: number } {
    try {
      const history = AnalysisHistoryManager.getHistory();
      
      if (!history || history.records.length === 0) {
        return { count: 0 };
      }

      // 计算总文件大小（输出文件）
      const totalSize = history.records.reduce((sum, record) => {
        return sum + (record.outputFile?.size || 0);
      }, 0);

      // 获取最近分析时间
      const lastAnalysis = history.records.length > 0 
        ? history.records[0].timestamp.toISOString()
        : undefined;

      return {
        count: history.records.length,
        lastAnalysis,
        totalSize
      };
    } catch (error) {
      console.error('Failed to get analysis history stats:', error);
      return { count: 0 };
    }
  }

  /**
   * 清理分析历史记录
   */
  static clear(): boolean {
    try {
      const history = AnalysisHistoryManager.getHistory();
      
      // 删除所有历史记录（这会触发文件删除）
      const deletePromises = history.records.map(record => 
        AnalysisHistoryManager.deleteRecord(record.id)
      );
      
      // 等待所有删除操作完成（异步）
      Promise.all(deletePromises).then(() => {
        console.log('All analysis history records deleted');
      }).catch(error => {
        console.warn('Some analysis files could not be deleted:', error);
      });
      
      // 立即清空本地存储
      localStorage.removeItem('analysis-history');
      
      return true;
    } catch (error) {
      console.error('Failed to clear analysis history:', error);
      return false;
    }
  }

  /**
   * 清理过期的分析记录（保留最新N条）
   */
  static cleanupOldRecords(keepCount: number = 20): number {
    try {
      const history = AnalysisHistoryManager.getHistory();
      
      if (history.records.length <= keepCount) {
        return 0; // 不需要清理
      }

      const recordsToDelete = history.records.slice(keepCount);
      let deletedCount = 0;

      // 删除多余的记录
      recordsToDelete.forEach(record => {
        AnalysisHistoryManager.deleteRecord(record.id).then(success => {
          if (success) deletedCount++;
        }).catch(console.warn);
      });

      console.log(`Cleaned up ${recordsToDelete.length} old analysis records`);
      return recordsToDelete.length;
    } catch (error) {
      console.error('Failed to cleanup old analysis records:', error);
      return 0;
    }
  }

  /**
   * 获取指定时间范围内的分析历史
   */
  static getRecordsInTimeRange(startDate: Date, endDate: Date): any[] {
    try {
      const history = AnalysisHistoryManager.getHistory();
      return history.records.filter(record => {
        const recordDate = new Date(record.timestamp);
        return recordDate >= startDate && recordDate <= endDate;
      });
    } catch (error) {
      console.error('Failed to get analysis records in time range:', error);
      return [];
    }
  }

  /**
   * 删除指定时间范围内的分析历史
   */
  static async deleteRecordsInTimeRange(startDate: Date, endDate: Date): Promise<{ deleted: number; remaining: number }> {
    try {
      const history = AnalysisHistoryManager.getHistory();
      const toDelete = history.records.filter(record => {
        const recordDate = new Date(record.timestamp);
        return recordDate >= startDate && recordDate <= endDate;
      });

      let deletedCount = 0;
      
      // 逐个删除记录（包括对应的文件）
      for (const record of toDelete) {
        const success = await AnalysisHistoryManager.deleteRecord(record.id);
        if (success) deletedCount++;
      }

      const remainingHistory = AnalysisHistoryManager.getHistory();
      
      return {
        deleted: deletedCount,
        remaining: remainingHistory.records.length
      };
    } catch (error) {
      console.error('Failed to delete analysis records in time range:', error);
      return { deleted: 0, remaining: 0 };
    }
  }

  /**
   * 批量删除指定的分析记录
   */
  static async deleteRecordsByIds(recordIds: string[]): Promise<{ deleted: number; remaining: number }> {
    try {
      let deletedCount = 0;
      
      for (const recordId of recordIds) {
        const success = await AnalysisHistoryManager.deleteRecord(recordId);
        if (success) deletedCount++;
      }

      const remainingHistory = AnalysisHistoryManager.getHistory();
      
      return {
        deleted: deletedCount,
        remaining: remainingHistory.records.length
      };
    } catch (error) {
      console.error('Failed to delete analysis records by ids:', error);
      return { deleted: 0, remaining: 0 };
    }
  }
}

/**
 * 数据清理工具
 */
export class DataCleanup {
  /**
   * 完全重置应用数据
   * 清空所有历史记录和应用设置
   */
  static resetAllData(): boolean {
    try {
      QueryHistoryStorage.clear();
      AnalysisHistoryStorage.clear();
      storage.remove(STORAGE_KEYS.USER_SETTINGS);
      storage.remove(STORAGE_KEYS.APP_VERSION);
      console.log('All application data has been reset');
      return true;
    } catch (error) {
      console.error('Failed to reset application data:', error);
      return false;
    }
  }
}
