/**
 * 本地存储管理工具
 * 用于持久化应用数据到 localStorage
 */

import type { QueryHistory } from '../types/app';

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
 * 查询历史数据的序列化和反序列化
 */
export class QueryHistoryStorage {
  private static readonly STORAGE_KEY = STORAGE_KEYS.QUERY_HISTORY;
  private static readonly MAX_HISTORY_COUNT = 100;

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
      // 限制历史记录数量
      const limitedHistory = history.slice(0, this.MAX_HISTORY_COUNT);
      const serializedData = this.serialize(limitedHistory);
      
      return storage.set(this.STORAGE_KEY, {
        data: serializedData,
        version: '1.0',
        savedAt: new Date().toISOString(),
        count: limitedHistory.length
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
   * 自动处理重复检测和数量限制
   */
  static addRecord(newRecord: QueryHistory): QueryHistory[] {
    const currentHistory = this.load();
    
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

    // 限制数量
    updatedHistory = updatedHistory.slice(0, this.MAX_HISTORY_COUNT);

    // 保存到存储
    this.save(updatedHistory);

    return updatedHistory;
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
 * 清理工具
 */
export class DataCleanup {
  /**
   * 清理过期数据
   */
  static cleanupExpiredData(): void {
    try {
      const stats = QueryHistoryStorage.getStats();
      
      // 如果历史记录过多，保留最新的50条
      if (stats.count > 50) {
        const history = QueryHistoryStorage.load();
        const recentHistory = history.slice(0, 50);
        QueryHistoryStorage.save(recentHistory);
        console.log(`Cleaned up query history: kept ${recentHistory.length} recent records`);
      }
    } catch (error) {
      console.error('Failed to cleanup expired data:', error);
    }
  }

  /**
   * 完全重置应用数据
   */
  static resetAllData(): boolean {
    try {
      QueryHistoryStorage.clear();
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
