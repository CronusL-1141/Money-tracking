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
    console.log(`🧪 Testing: ${testName}`);
    try {
      const startTime = Date.now();
      const result = await testFn();
      const duration = Date.now() - startTime;
      
      console.log(`✅ ${testName} - Passed (${duration}ms)`);
      this.testResults.push({
        name: testName,
        status: 'PASS',
        duration,
        result
      });
      this.passCount++;
      return result;
    } catch (error) {
      console.error(`❌ ${testName} - Failed:`, error);
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
    console.log('🚀 Starting Rust backend functionality tests...\n');
    
    try {
      // 1. 基础环境测试
      await this.runTest('检查Python环境', async () => {
        const result = await window.__TAURI__.invoke('check_python_env');
        console.log('Python环境信息:', result);
        if (!result.python_available) {
          throw new Error('Python environment not available');
        }
        return result;
      });

      // 2. 算法列表测试
      await this.runTest('获取算法列表', async () => {
        const algorithms = await window.__TAURI__.invoke('get_algorithms');
        console.log('可用算法:', algorithms);
        if (!Array.isArray(algorithms) || algorithms.length === 0) {
          throw new Error('Algorithm list is empty');
        }
        if (!algorithms.includes('FIFO') || !algorithms.includes('BALANCE_METHOD')) {
          throw new Error('Missing required algorithms');
        }
        return algorithms;
      });

      // 3. 应用配置测试
      await this.runTest('获取应用配置', async () => {
        const config = await window.__TAURI__.invoke('get_app_config');
        console.log('应用配置:', config);
        if (!config.default_algorithm || !config.language) {
          throw new Error('Configuration data incomplete');
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
          throw new Error('Configuration update failed');
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
          throw new Error('Process status format error');
        }
        return status;
      });

      // 6. 查询历史测试
      await this.runTest('获取查询历史', async () => {
        const history = await window.__TAURI__.invoke('get_query_history');
        console.log('查询历史记录数:', history.length);
        if (!Array.isArray(history)) {
          throw new Error('History record format error');
        }
        return history;
      });

      // 7. 文件路径验证测试
      await this.runTest('验证无效文件路径', async () => {
        const isValid = await window.__TAURI__.invoke('validate_file_path', { 
          path: '/nonexistent/file.txt' 
        });
        if (isValid) {
          throw new Error('Invalid path incorrectly validated as valid');
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
          throw new Error('Non-existent file marked as existing');
        }
        return fileInfo;
      });

    } catch (error) {
      console.error('Error occurred during testing:', error);
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