/**
 * 临时文件管理工具
 * 负责跟踪和管理所有临时分析结果文件
 */

import { invoke } from '@tauri-apps/api/tauri';
import { readDir, removeFile, exists, FileEntry } from '@tauri-apps/api/fs';
import { join } from '@tauri-apps/api/path';
import { AnalysisHistoryManager } from './analysisHistoryManager';

export interface TempFileInfo {
  path: string;
  name: string;
  size: number;
  created: Date;
  isTracked: boolean; // 是否被历史记录跟踪
  canSafeDelete: boolean; // 是否可以安全删除
}

export interface TempDirectoryStats {
  totalFiles: number;
  totalSize: number;
  trackedFiles: number;
  untrackedFiles: number;
  orphanedFiles: number; // 孤儿文件（历史记录已删除但文件仍存在）
  safeToDeleteFiles: number;
}

export class TempFileManager {
  private static readonly TEMP_DIR_NAME = 'temp_analysis_results';
  
  /**
   * 获取临时目录路径
   */
  static async getTempDirectoryPath(): Promise<string> {
    try {
      const appDir = await invoke('get_app_directory') as string;
      return await join(appDir, this.TEMP_DIR_NAME);
    } catch (error) {
      // 如果获取应用目录失败，使用相对路径
      console.warn('Failed to get app directory, using relative path:', error);
      return this.TEMP_DIR_NAME;
    }
  }

  /**
   * 扫描临时目录，获取所有文件信息
   */
  static async scanTempDirectory(): Promise<TempFileInfo[]> {
    try {
      const tempDirPath = await this.getTempDirectoryPath();
      
      // 检查目录是否存在
      if (!await exists(tempDirPath)) {
        console.log('临时目录不存在:', tempDirPath);
        return [];
      }

      const entries = await readDir(tempDirPath);
      const analysisHistory = AnalysisHistoryManager.getHistory();
      const trackedPaths = new Set<string>();
      
      // 收集所有被历史记录跟踪的文件路径
      analysisHistory.records.forEach(record => {
        trackedPaths.add(record.outputFile.path);
        if (record.offsitePoolFile) {
          trackedPaths.add(record.offsitePoolFile.path);
        }
      });

      const tempFiles: TempFileInfo[] = [];

      for (const entry of entries) {
        if (entry.children) continue; // 跳过目录
        
        try {
          const filePath = await join(tempDirPath, entry.name!);
          const isTracked = trackedPaths.has(filePath);
          const fileExists = await exists(filePath);
          
          if (!fileExists) continue; // 跳过不存在的文件

          // 获取文件统计信息
          const stats = await invoke('get_file_stats', { filePath }) as any;
          
          tempFiles.push({
            path: filePath,
            name: entry.name!,
            size: stats.size || 0,
            created: stats.modified ? new Date(stats.modified) : new Date(),
            isTracked,
            canSafeDelete: !isTracked || await this.canSafelyDelete(filePath, analysisHistory.records)
          });
        } catch (error) {
          console.warn(`Failed to process file ${entry.name}:`, error);
        }
      }

      return tempFiles;
    } catch (error) {
      console.error('Failed to scan temp directory:', error);
      return [];
    }
  }

  /**
   * 检查文件是否可以安全删除
   */
  private static async canSafelyDelete(filePath: string, records: any[]): Promise<boolean> {
    // 如果文件没有被任何历史记录引用，可以安全删除
    const referencingRecords = records.filter(record => 
      record.outputFile.path === filePath || 
      (record.offsitePoolFile && record.offsitePoolFile.path === filePath)
    );
    
    if (referencingRecords.length === 0) {
      return true; // 孤儿文件，可以安全删除
    }

    // 如果引用的记录都标记为已删除，也可以安全删除
    return referencingRecords.every(record => 
      (record.outputFile.path === filePath && record.outputFile.deleted) ||
      (record.offsitePoolFile && record.offsitePoolFile.path === filePath && record.offsitePoolFile.deleted)
    );
  }

  /**
   * 获取临时目录统计信息
   */
  static async getTempDirectoryStats(): Promise<TempDirectoryStats> {
    const files = await this.scanTempDirectory();
    
    return {
      totalFiles: files.length,
      totalSize: files.reduce((sum, file) => sum + file.size, 0),
      trackedFiles: files.filter(f => f.isTracked).length,
      untrackedFiles: files.filter(f => !f.isTracked).length,
      orphanedFiles: files.filter(f => !f.isTracked).length,
      safeToDeleteFiles: files.filter(f => f.canSafeDelete).length
    };
  }

  /**
   * 清理可安全删除的临时文件
   */
  static async cleanupSafeFiles(): Promise<{ deleted: number; failed: number; errors: string[] }> {
    const files = await this.scanTempDirectory();
    const safeFiles = files.filter(f => f.canSafeDelete);
    
    let deleted = 0;
    let failed = 0;
    const errors: string[] = [];

    for (const file of safeFiles) {
      try {
        if (await exists(file.path)) {
          await removeFile(file.path);
          deleted++;
          console.log(`Deleted safe temp file: ${file.name}`);
        } else {
          deleted++; // 文件不存在，认为已删除
        }
      } catch (error) {
        failed++;
        const errorMsg = error instanceof Error ? error.message : '未知错误';
        errors.push(`删除文件 ${file.name} 失败: ${errorMsg}`);
        console.error(`Failed to delete temp file ${file.path}:`, error);
      }
    }

    return { deleted, failed, errors };
  }

  /**
   * 清理所有临时文件（危险操作）
   */
  static async cleanupAllTempFiles(): Promise<{ deleted: number; failed: number; errors: string[] }> {
    try {
      const tempDirPath = await this.getTempDirectoryPath();
      
      if (!await exists(tempDirPath)) {
        return { deleted: 0, failed: 0, errors: [] };
      }

      const entries = await readDir(tempDirPath);
      let deleted = 0;
      let failed = 0;
      const errors: string[] = [];

      for (const entry of entries) {
        if (entry.children) continue; // 跳过目录
        
        try {
          const filePath = await join(tempDirPath, entry.name!);
          if (await exists(filePath)) {
            await removeFile(filePath);
            deleted++;
            console.log(`Deleted temp file: ${entry.name}`);
          }
        } catch (error) {
          failed++;
          const errorMsg = error instanceof Error ? error.message : '未知错误';
          errors.push(`删除文件 ${entry.name} 失败: ${errorMsg}`);
          console.error(`Failed to delete temp file ${entry.name}:`, error);
        }
      }

      return { deleted, failed, errors };
    } catch (error) {
      console.error('Failed to cleanup all temp files:', error);
      return { 
        deleted: 0, 
        failed: 0, 
        errors: [error instanceof Error ? error.message : '清理临时文件时发生未知错误'] 
      };
    }
  }

  /**
   * 清理孤儿文件并同步历史记录
   * 删除文件不存在的历史记录，删除没有历史记录引用的文件
   */
  static async syncHistoryWithFiles(): Promise<{ 
    deletedRecords: number; 
    deletedFiles: number; 
    errors: string[] 
  }> {
    const files = await this.scanTempDirectory();
    const history = AnalysisHistoryManager.getHistory();
    const errors: string[] = [];
    let deletedRecords = 0;
    let deletedFiles = 0;

    // 创建文件路径集合以便快速查找
    const existingFiles = new Set(files.map(f => f.path));

    // 清理引用不存在文件的历史记录
    const recordsToDelete: string[] = [];
    
    for (const record of history.records) {
      const mainFileExists = existingFiles.has(record.outputFile.path);
      const poolFileExists = !record.offsitePoolFile || existingFiles.has(record.offsitePoolFile.path);
      
      // 如果主文件和资金池文件都不存在，删除整个记录
      if (!mainFileExists && !poolFileExists) {
        recordsToDelete.push(record.id);
      }
    }

    // 删除无效的历史记录
    for (const recordId of recordsToDelete) {
      try {
        const result = await AnalysisHistoryManager.deleteRecord(recordId);
        if (result.success) {
          deletedRecords++;
        } else {
          errors.push(`删除历史记录 ${recordId} 失败: ${result.errors.join(', ')}`);
        }
      } catch (error) {
        errors.push(`删除历史记录 ${recordId} 时发生错误: ${error}`);
      }
    }

    // 删除没有历史记录引用的孤儿文件
    const orphanFiles = files.filter(f => !f.isTracked);
    for (const file of orphanFiles) {
      try {
        if (await exists(file.path)) {
          await removeFile(file.path);
          deletedFiles++;
          console.log(`Deleted orphan file: ${file.name}`);
        }
      } catch (error) {
        errors.push(`删除孤儿文件 ${file.name} 失败: ${error}`);
      }
    }

    return { deletedRecords, deletedFiles, errors };
  }

  /**
   * 获取超过指定天数的文件
   */
  static async getOldFiles(daysOld: number): Promise<TempFileInfo[]> {
    const files = await this.scanTempDirectory();
    const cutoffDate = new Date();
    cutoffDate.setDate(cutoffDate.getDate() - daysOld);

    return files.filter(file => file.created < cutoffDate);
  }

  /**
   * 清理超过指定天数的文件
   */
  static async cleanupOldFiles(daysOld: number): Promise<{ deleted: number; failed: number; errors: string[] }> {
    const oldFiles = await this.getOldFiles(daysOld);
    let deleted = 0;
    let failed = 0;
    const errors: string[] = [];

    for (const file of oldFiles) {
      try {
        if (await exists(file.path)) {
          await removeFile(file.path);
          deleted++;
          console.log(`Deleted old temp file: ${file.name} (${file.created.toISOString()})`);
        }
      } catch (error) {
        failed++;
        const errorMsg = error instanceof Error ? error.message : '未知错误';
        errors.push(`删除过期文件 ${file.name} 失败: ${errorMsg}`);
      }
    }

    // 同步更新历史记录
    if (deleted > 0) {
      try {
        await this.syncHistoryWithFiles();
      } catch (error) {
        console.warn('同步历史记录时发生错误:', error);
      }
    }

    return { deleted, failed, errors };
  }
}