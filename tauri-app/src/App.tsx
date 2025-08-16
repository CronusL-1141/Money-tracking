import React, { useEffect, useState } from "react";
import { Routes, Route } from "react-router-dom";
import { Box, CircularProgress, Alert } from "@mui/material";
import { useTranslation } from "react-i18next";

// 组件导入
import Layout from "./components/layout/Layout";
import ErrorBoundary from "./components/ErrorBoundary";
import HomePage from "./pages/HomePage";
import AuditPage from "./pages/AuditPage";
import TimePointQueryPage from "./pages/TimePointQueryPage";
import SettingsPage from "./pages/SettingsPage";
// import TestPage from "./pages/TestPage";

// 服务和类型导入
import { checkPythonEnvironment } from "./services/pythonService";
// import { updateService } from "./services/updateService"; // 独立版本不需要更新服务
import { PythonEnvStatus } from "./types/python";

const App: React.FC = () => {
  const { t } = useTranslation();
  const [loading, setLoading] = useState(true);
  const [envStatus, setEnvStatus] = useState<PythonEnvStatus | null>(null);
  const [error, setError] = useState<string | null>(null);

  // 初始化检查
  useEffect(() => {
    const initializeApp = async () => {
      try {
        setLoading(true);
        setError(null);
        
        // 检查Python环境
        const status = await checkPythonEnvironment();
        setEnvStatus(status);
        
        if (!status.python_available) {
          setError(t('errors.python_not_available'));
        }
        
        // 检查应用更新（后台执行，不阻塞应用启动）
        // 注释掉自动更新检查，避免联网问题
        /*
        setTimeout(async () => {
          try {
            await updateService.autoCheckForUpdates();
          } catch (error) {
            console.warn('自动更新检查失败:', error);
            // 不显示错误，避免干扰用户体验
          }
        }, 3000); // 延迟3秒执行，确保应用完全加载
        */
        
      } catch (err) {
        console.error('初始化失败:', err);
        setError(t('errors.initialization_failed'));
      } finally {
        setLoading(false);
      }
    };

    initializeApp();
  }, [t]);

  // 加载中状态
  if (loading) {
    return (
      <Box
        display="flex"
        justifyContent="center"
        alignItems="center"
        minHeight="100vh"
        flexDirection="column"
        gap={2}
      >
        <CircularProgress size={60} />
        <Box sx={{ fontSize: '1.1rem', color: 'text.secondary' }}>
          {t('common.initializing')}...
        </Box>
      </Box>
    );
  }

  // 错误状态
  if (error) {
    return (
      <Box
        display="flex"
        justifyContent="center"
        alignItems="center"
        minHeight="100vh"
        p={3}
      >
        <Alert severity="error" sx={{ maxWidth: 600 }}>
          <Box sx={{ fontSize: '1.1rem', mb: 1 }}>
            {t('errors.system_error')}
          </Box>
          <Box sx={{ fontSize: '0.9rem' }}>
            {error}
          </Box>
          {envStatus && (
            <Box sx={{ mt: 2, fontSize: '0.8rem', color: 'text.secondary' }}>
              Python路径: {envStatus.python_path?.toString() || 'N/A'}<br/>
              项目根目录: {envStatus.project_root?.toString() || 'N/A'}
            </Box>
          )}
        </Alert>
      </Box>
    );
  }

  // 正常应用界面
  return (
    <Layout>
      <Routes>
        {/* 主页 */}
        <Route path="/" element={
          <ErrorBoundary>
            <HomePage />
          </ErrorBoundary>
        } />
        
        {/* 资金分析页面 */}
        <Route path="/audit" element={
          <ErrorBoundary>
            <AuditPage />
          </ErrorBoundary>
        } />
        
        {/* 时点查询页面 */}
        <Route path="/query" element={
          <ErrorBoundary>
            <TimePointQueryPage />
          </ErrorBoundary>
        } />
        
        {/* 设置页面 */}
        <Route path="/settings" element={
          <ErrorBoundary>
            <SettingsPage />
          </ErrorBoundary>
        } />
        
        {/* 测试页面 */}
        {/* <Route path="/test" element={<TestPage />} /> */}
        
        {/* 默认重定向到主页 */}
        <Route path="*" element={
          <ErrorBoundary>
            <HomePage />
          </ErrorBoundary>
        } />
      </Routes>
    </Layout>
  );
};

export default App;