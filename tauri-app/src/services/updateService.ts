import { invoke } from '@tauri-apps/api/tauri';
import { relaunch } from '@tauri-apps/api/process';
import { 
  checkUpdate, 
  installUpdate,
  onUpdaterEvent,
  UpdateManifest,
  UpdateResult,
  UpdateStatus
} from '@tauri-apps/api/updater';
import { ask, message } from '@tauri-apps/api/dialog';

export interface UpdateInfo {
  available: boolean;
  version?: string;
  notes?: string;
  date?: string;
  signature?: string;
}

export interface UpdateProgress {
  event: 'Started' | 'Progress' | 'Finished';
  data?: {
    contentLength?: number;
    chunkLength?: number;
  };
}

export class UpdateService {
  private static instance: UpdateService;
  private updateInProgress = false;
  private progressCallback?: (progress: UpdateProgress) => void;

  private constructor() {
    this.setupUpdateListener();
  }

  public static getInstance(): UpdateService {
    if (!UpdateService.instance) {
      UpdateService.instance = new UpdateService();
    }
    return UpdateService.instance;
  }

  /**
   * 设置更新事件监听器
   */
  private async setupUpdateListener(): Promise<void> {
    try {
      await onUpdaterEvent(({ error, status }) => {
        console.log('更新器事件:', { error, status });
        
        if (error) {
          console.error('更新过程中发生错误:', error);
          this.progressCallback?.({ 
            event: 'Finished',
            data: undefined
          });
          return;
        }

        switch (status) {
          case UpdateStatus.PENDING:
            console.log('开始检查更新...');
            break;
          case UpdateStatus.ERROR:
            console.error('更新检查失败');
            this.progressCallback?.({ 
              event: 'Finished',
              data: undefined
            });
            break;
          case UpdateStatus.DONE:
            console.log('更新安装完成');
            this.progressCallback?.({ 
              event: 'Finished',
              data: undefined
            });
            break;
          case UpdateStatus.UPTODATE:
            console.log('已是最新版本');
            break;
        }
      });
    } catch (error) {
      console.error('设置更新监听器失败:', error);
    }
  }

  /**
   * 检查是否有可用更新
   */
  public async checkForUpdates(): Promise<UpdateInfo> {
    try {
      console.log('检查更新中...');
      const update: UpdateResult = await checkUpdate();
      
      if (update.manifest) {
        const manifest: UpdateManifest = update.manifest;
        return {
          available: true,
          version: manifest.version,
          notes: manifest.body,
          date: manifest.date,
          signature: manifest.signature
        };
      } else {
        return {
          available: false
        };
      }
    } catch (error) {
      console.error('检查更新失败:', error);
      return {
        available: false
      };
    }
  }

  /**
   * 安装更新
   */
  public async installUpdate(
    progressCallback?: (progress: UpdateProgress) => void
  ): Promise<boolean> {
    if (this.updateInProgress) {
      console.warn('更新已在进行中');
      return false;
    }

    try {
      this.updateInProgress = true;
      this.progressCallback = progressCallback;

      // 提示用户确认安装更新
      const confirmed = await ask(
        '发现新版本！是否立即下载并安装更新？\n\n注意：安装过程中应用程序将重启。',
        {
          title: '更新确认',
          type: 'info'
        }
      );

      if (!confirmed) {
        this.updateInProgress = false;
        return false;
      }

      progressCallback?.({ 
        event: 'Started',
        data: undefined
      });

      // 开始安装更新
      await installUpdate();
      
      // 安装完成，准备重启应用
      const restartConfirmed = await ask(
        '更新安装完成！是否立即重启应用程序以使用新版本？',
        {
          title: '重启确认',
          type: 'info'
        }
      );

      if (restartConfirmed) {
        await relaunch();
      }

      this.updateInProgress = false;
      return true;

    } catch (error) {
      console.error('安装更新失败:', error);
      this.updateInProgress = false;
      
      await message(
        `更新安装失败：${error}\n\n请稍后重试或联系技术支持。`,
        {
          title: '更新错误',
          type: 'error'
        }
      );
      
      return false;
    }
  }

  /**
   * 自动检查更新（静默）
   */
  public async autoCheckForUpdates(): Promise<boolean> {
    try {
      const updateInfo = await this.checkForUpdates();
      
      if (updateInfo.available) {
        // 显示更新通知
        const userWantsUpdate = await ask(
          `发现新版本 ${updateInfo.version}！\n\n${updateInfo.notes || '无更新说明'}\n\n是否立即更新？`,
          {
            title: '发现新版本',
            type: 'info'
          }
        );

        if (userWantsUpdate) {
          return await this.installUpdate();
        }
      }
      
      return false;
    } catch (error) {
      console.error('自动检查更新失败:', error);
      return false;
    }
  }

  /**
   * 获取当前应用版本
   */
  public async getCurrentVersion(): Promise<string> {
    try {
      return await invoke<string>('get_app_version');
    } catch (error) {
      console.error('获取应用版本失败:', error);
      return 'unknown';
    }
  }

  /**
   * 手动触发更新检查
   */
  public async manualCheckForUpdates(): Promise<void> {
    try {
      const updateInfo = await this.checkForUpdates();
      
      if (updateInfo.available) {
        const userWantsUpdate = await ask(
          `发现新版本 ${updateInfo.version}！\n\n更新内容：\n${updateInfo.notes || '无更新说明'}\n\n是否立即下载并安装？`,
          {
            title: '发现新版本',
            type: 'info'
          }
        );

        if (userWantsUpdate) {
          await this.installUpdate((progress) => {
            console.log('更新进度:', progress);
          });
        }
      } else {
        await message(
          '您当前使用的已经是最新版本！',
          {
            title: '检查更新',
            type: 'info'
          }
        );
      }
    } catch (error) {
      await message(
        `检查更新失败：${error}\n\n请检查网络连接或稍后重试。`,
        {
          title: '更新错误',
          type: 'error'
        }
      );
    }
  }
}

// 导出单例实例
export const updateService = UpdateService.getInstance();