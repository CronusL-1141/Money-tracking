/**
 * 统一的时间格式化工具
 * 确保所有时间显示都使用本地电脑时间且格式一致
 */

/**
 * 获取当前本地时间的格式化字符串
 * @param format - 时间格式类型
 * @returns 格式化的时间字符串
 */
export function getCurrentLocalTime(format: 'log' | 'display' | 'filename' | 'iso' = 'log'): string {
  const now = new Date();
  
  switch (format) {
    case 'log':
      // 日志格式：2024-08-20 17:30:45
      return now.toLocaleString('zh-CN', {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        hour12: false
      }).replace(/\//g, '-');
    
    case 'display':
      // 显示格式：2024年8月20日 17:30:45
      return now.toLocaleString('zh-CN', {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        hour12: false
      });
    
    case 'filename':
      // 文件名格式：2024-08-20_17-30-45
      return now.toISOString().slice(0, 19).replace(/:/g, '-').replace('T', '_');
    
    case 'iso':
      // ISO格式：2024-08-20T17:30:45.123Z (本地时区)
      return now.toISOString();
    
    default:
      return now.toLocaleString();
  }
}

/**
 * 格式化给定的时间对象
 * @param date - 要格式化的时间对象
 * @param format - 时间格式类型
 * @param locale - 本地化设置，如果不提供则使用浏览器默认
 * @returns 格式化的时间字符串
 */
export function formatLocalTime(date: Date | string | number, format: 'log' | 'display' | 'filename' | 'iso' = 'log', locale?: string): string {
  const targetDate = new Date(date);
  
  // 检查是否为有效日期
  if (isNaN(targetDate.getTime())) {
    return getCurrentLocalTime(format);
  }
  
  // 确定使用的本地化设置
  const localeToUse = locale || navigator.language || 'zh-CN';
  
  switch (format) {
    case 'log':
      // 日志格式：2024-08-20 17:30:45 (中文) 或 8/20/2024, 5:30:45 PM (英文)
      return targetDate.toLocaleString(localeToUse, {
        year: 'numeric',
        month: '2-digit',
        day: '2-digit',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        hour12: localeToUse.startsWith('zh') ? false : true
      }).replace(/\//g, '-');
    
    case 'display':
      // 显示格式：2024年8月20日 17:30:45 (中文) 或 August 20, 2024, 5:30:45 PM (英文)
      return targetDate.toLocaleString(localeToUse, {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
        hour12: localeToUse.startsWith('zh') ? false : true
      });
    
    case 'filename':
      // 文件名格式：2024-08-20_17-30-45
      return targetDate.toISOString().slice(0, 19).replace(/:/g, '-').replace('T', '_');
    
    case 'iso':
      // ISO格式：2024-08-20T17:30:45.123Z
      return targetDate.toISOString();
    
    default:
      return targetDate.toLocaleString();
  }
}

/**
 * 创建带时间戳的日志消息
 * @param message - 日志消息
 * @param level - 日志级别 ('info' | 'success' | 'warning' | 'error')
 * @returns 带时间戳的完整日志消息
 */
export function createLogMessage(message: string, level: 'info' | 'success' | 'warning' | 'error' = 'info'): string {
  const timestamp = getCurrentLocalTime('log');
  const prefix = {
    info: 'ℹ️',
    success: '✅',
    warning: '⚠️',
    error: '❌'
  };
  
  return `[${timestamp}] ${prefix[level]} ${message}`;
}

/**
 * 获取相对时间描述（如：刚刚，5分钟前，1小时前）
 * @param date - 目标时间
 * @param locale - 本地化设置，如果不提供则使用浏览器默认
 * @returns 相对时间描述
 */
export function getRelativeTime(date: Date | string | number, locale?: string): string {
  const targetDate = new Date(date);
  const now = new Date();
  const diffMs = now.getTime() - targetDate.getTime();
  const diffSeconds = Math.floor(diffMs / 1000);
  const diffMinutes = Math.floor(diffSeconds / 60);
  const diffHours = Math.floor(diffMinutes / 60);
  const diffDays = Math.floor(diffHours / 24);
  
  // 确定使用的本地化设置
  const localeToUse = locale || navigator.language || 'zh-CN';
  const isZh = localeToUse.startsWith('zh');
  
  if (diffSeconds < 30) return isZh ? '刚刚' : 'just now';
  if (diffSeconds < 60) return isZh ? `${diffSeconds}秒前` : `${diffSeconds}s ago`;
  if (diffMinutes < 60) return isZh ? `${diffMinutes}分钟前` : `${diffMinutes}m ago`;
  if (diffHours < 24) return isZh ? `${diffHours}小时前` : `${diffHours}h ago`;
  if (diffDays < 30) return isZh ? `${diffDays}天前` : `${diffDays}d ago`;
  
  // 超过30天显示具体日期
  return formatLocalTime(targetDate, 'display', localeToUse);
}
