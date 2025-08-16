import { invoke } from '@tauri-apps/api/tauri';
import { dialog } from '@tauri-apps/api';

/**
 * 文件操作服务
 */
export class FileService {
  
  /**
   * 选择Excel文件
   */
  static async selectExcelFile(): Promise<string | null> {
    try {
      const selected = await dialog.open({
        title: '选择Excel文件',
        multiple: false,
        filters: [
          {
            name: 'Excel文件',
            extensions: ['xlsx', 'xls'],
          },
          {
            name: '所有文件',
            extensions: ['*'],
          },
        ],
      });
      
      if (selected && typeof selected === 'string') {
        return selected;
      }
      return null;
    } catch (error) {
      console.error('选择文件失败:', error);
      throw new Error(`文件选择失败: ${error}`);
    }
  }
  
  /**
   * 选择保存文件位置
   */
  static async selectSaveLocation(defaultName: string = 'audit_result.xlsx'): Promise<string | null> {
    try {
      const selected = await dialog.save({
        title: '选择保存位置',
        defaultPath: defaultName,
        filters: [
          {
            name: 'Excel文件',
            extensions: ['xlsx'],
          },
          {
            name: '所有文件',
            extensions: ['*'],
          },
        ],
      });
      
      if (selected) {
        return selected;
      }
      return null;
    } catch (error) {
      console.error('选择保存位置失败:', error);
      throw new Error(`选择保存位置失败: ${error}`);
    }
  }
  
  /**
   * 通过Tauri命令选择文件（备用方法）
   */
  static async selectFileViaCommand(
    title: string,
    filters: Array<{ name: string; extensions: string[] }>
  ): Promise<string | null> {
    try {
      const result = await invoke('select_file', {
        title,
        filters: filters.map(f => [f.name, f.extensions]),
      });
      
      return result as string | null;
    } catch (error) {
      console.error('文件选择失败:', error);
      throw new Error(`文件选择失败: ${error}`);
    }
  }
  
  /**
   * 检查文件是否存在
   */
  static async fileExists(path: string): Promise<boolean> {
    try {
      // 通过Tauri API检查文件是否存在
      const { exists } = await import('@tauri-apps/api/fs');
      return await exists(path);
    } catch (error) {
      console.error('检查文件存在性失败:', error);
      return false;
    }
  }
  
  /**
   * 获取文件信息
   */
  static async getFileInfo(path: string): Promise<{
    name: string;
    size: number;
    lastModified: Date;
  } | null> {
    try {
      const { exists } = await import('@tauri-apps/api/fs');
      const fileExists = await exists(path);
      if (!fileExists) {
        return null;
      }
      // 为了简化，返回基本信息
      const stats = { size: 0, isFile: true, isDir: false };
      
      // 从路径提取文件名
      const name = path.split(/[/\\]/).pop() || '';
      
      return {
        name,
        size: stats.size,
        lastModified: new Date(),
      };
    } catch (error) {
      console.error('获取文件信息失败:', error);
      return null;
    }
  }
  
  /**
   * 验证Excel文件
   */
  static validateExcelFile(filePath: string): boolean {
    if (!filePath) return false;
    
    const allowedExtensions = ['.xlsx', '.xls'];
    const extension = filePath.toLowerCase().substring(filePath.lastIndexOf('.'));
    
    return allowedExtensions.includes(extension);
  }
  
  /**
   * 格式化文件大小
   */
  static formatFileSize(bytes: number): string {
    if (bytes === 0) return '0 Bytes';
    
    const k = 1024;
    const sizes = ['Bytes', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }
  
  /**
   * 生成输出文件名
   */
  static generateOutputFileName(inputPath: string, algorithm: string, suffix?: string): string {
    const inputName = inputPath.split(/[/\\]/).pop() || 'audit';
    const nameWithoutExt = inputName.replace(/\.[^/.]+$/, '');
    const timestamp = new Date().toISOString().slice(0, 19).replace(/:/g, '-');
    
    let outputName = `${nameWithoutExt}_${algorithm}`;
    if (suffix) {
      outputName += `_${suffix}`;
    }
    outputName += `_${timestamp}.xlsx`;
    
    return outputName;
  }
}

// 导出常用的辅助函数
export const selectExcelFile = FileService.selectExcelFile;
export const selectSaveLocation = FileService.selectSaveLocation;
export const validateExcelFile = FileService.validateExcelFile;
export const formatFileSize = FileService.formatFileSize;
export const generateOutputFileName = FileService.generateOutputFileName;