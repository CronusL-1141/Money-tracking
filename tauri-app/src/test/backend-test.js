/**
 * Rust后端功能测试脚本
 * 用于验证所有后端命令是否正常工作
 */

// 注意：这个脚本需要在Tauri环境中运行
// 可以通过浏览器控制台执行

class BackendTester {
  constructor() {
    this.testResults = [];
    this.failCount = 0;
    this.passCount = 0;
  }

  async runTest(testName, testFn) {
    console.log(`🧪 正在测试: ${testName}`);
    try {
      const startTime = Date.now();
      const result = await testFn();
      const duration = Date.now() - startTime;
      
      console.log(`✅ ${testName} - 通过 (${duration}ms)`);
      this.testResults.push({
        name: testName,
        status: 'PASS',
        duration,
        result
      });
      this.passCount++;
      return result;
    } catch (error) {
      console.error(`❌ ${testName} - 失败:`, error);
      this.testResults.push({
        name: testName,
        status: 'FAIL',
        error: error.message
      });
      this.failCount++;
      throw error;
    }
  }

  async runAllTests() {
    console.log('🚀 开始Rust后端功能测试...\n');
    
    try {
      // 1. 基础环境测试
      await this.runTest('检查Python环境', async () => {
        const result = await window.__TAURI__.invoke('check_python_env');
        console.log('Python环境信息:', result);
        if (!result.python_available) {
          throw new Error('Python环境不可用');
        }
        return result;
      });

      // 2. 算法列表测试
      await this.runTest('获取算法列表', async () => {
        const algorithms = await window.__TAURI__.invoke('get_algorithms');
        console.log('可用算法:', algorithms);
        if (!Array.isArray(algorithms) || algorithms.length === 0) {
          throw new Error('算法列表为空');
        }
        if (!algorithms.includes('FIFO') || !algorithms.includes('BALANCE_METHOD')) {
          throw new Error('缺少必要的算法');
        }
        return algorithms;
      });

      // 3. 应用配置测试
      await this.runTest('获取应用配置', async () => {
        const config = await window.__TAURI__.invoke('get_app_config');
        console.log('应用配置:', config);
        if (!config.default_algorithm || !config.language) {
          throw new Error('配置数据不完整');
        }
        return config;
      });

      // 4. 更新应用配置测试
      await this.runTest('更新应用配置', async () => {
        const currentConfig = await window.__TAURI__.invoke('get_app_config');
        const newConfig = {
          ...currentConfig,
          theme: currentConfig.theme === 'light' ? 'dark' : 'light'
        };
        
        await window.__TAURI__.invoke('update_app_config', { newConfig });
        const updatedConfig = await window.__TAURI__.invoke('get_app_config');
        
        if (updatedConfig.theme !== newConfig.theme) {
          throw new Error('配置更新失败');
        }
        
        // 恢复原配置
        await window.__TAURI__.invoke('update_app_config', { newConfig: currentConfig });
        return updatedConfig;
      });

      // 5. 进程状态测试
      await this.runTest('获取进程状态', async () => {
        const status = await window.__TAURI__.invoke('get_process_status');
        console.log('进程状态:', status);
        if (typeof status.running !== 'boolean') {
          throw new Error('进程状态格式错误');
        }
        return status;
      });

      // 6. 查询历史测试
      await this.runTest('获取查询历史', async () => {
        const history = await window.__TAURI__.invoke('get_query_history');
        console.log('查询历史记录数:', history.length);
        if (!Array.isArray(history)) {
          throw new Error('历史记录格式错误');
        }
        return history;
      });

      // 7. 文件路径验证测试
      await this.runTest('验证无效文件路径', async () => {
        const isValid = await window.__TAURI__.invoke('validate_file_path', { 
          path: '/nonexistent/file.txt' 
        });
        if (isValid) {
          throw new Error('无效路径被错误验证为有效');
        }
        return !isValid;
      });

      // 8. 文件信息测试（测试不存在的文件）
      await this.runTest('获取不存在文件的信息', async () => {
        const fileInfo = await window.__TAURI__.invoke('get_file_info', { 
          path: '/nonexistent/file.txt' 
        });
        console.log('文件信息:', fileInfo);
        if (fileInfo.exists) {
          throw new Error('不存在的文件被标记为存在');
        }
        return fileInfo;
      });

    } catch (error) {
      console.error('测试过程中发生错误:', error);
    }

    // 输出测试总结
    this.printSummary();
  }

  printSummary() {
    console.log('\n📊 测试结果总结:');
    console.log('='.repeat(50));
    console.log(`总测试数: ${this.testResults.length}`);
    console.log(`✅ 通过: ${this.passCount}`);
    console.log(`❌ 失败: ${this.failCount}`);
    console.log(`成功率: ${((this.passCount / this.testResults.length) * 100).toFixed(1)}%`);
    
    if (this.failCount > 0) {
      console.log('\n❌ 失败的测试:');
      this.testResults
        .filter(r => r.status === 'FAIL')
        .forEach(r => console.log(`  - ${r.name}: ${r.error}`));
    }
    
    console.log('\n详细测试结果:', this.testResults);
  }
}

// 创建全局测试实例
if (typeof window !== 'undefined') {
  window.backendTester = new BackendTester();
  
  // 自动运行测试的函数
  window.runBackendTests = () => {
    return window.backendTester.runAllTests();
  };
  
  console.log('后端测试器已就绪！');
  console.log('使用 runBackendTests() 开始测试');
} else {
  console.log('此脚本需要在Tauri环境中运行');
}