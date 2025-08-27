// 全局文件拖拽管理器 - 防止重复处理
class FileDropManager {
  private static instance: FileDropManager;
  private lastDrop: {
    filePath: string;
    timestamp: number;
  } = { filePath: '', timestamp: 0 };

  private constructor() {}

  static getInstance(): FileDropManager {
    if (!FileDropManager.instance) {
      FileDropManager.instance = new FileDropManager();
    }
    return FileDropManager.instance;
  }

  // 检查是否应该跳过此次文件拖拽处理
  shouldSkipDrop(filePath: string): boolean {
    const now = Date.now();
    const timeDiff = now - this.lastDrop.timestamp;
    
    // 1. 相同文件在3秒内不重复处理
    // 2. 任何文件在1秒内不重复处理（防止系统重复事件）
    const shouldSkip = (
      (this.lastDrop.filePath === filePath && timeDiff < 3000) ||
      (timeDiff < 1000)
    );

    if (shouldSkip) {
      console.log(`[全局防重复] 跳过文件拖拽: ${filePath.split(/[/\\]/).pop()}, 时间差: ${timeDiff}ms`);
      return true;
    }

    // 记录本次拖拽
    this.lastDrop = { filePath, timestamp: now };
    console.log(`[全局防重复] 允许文件拖拽: ${filePath.split(/[/\\]/).pop()}`);
    return false;
  }
}

export default FileDropManager;