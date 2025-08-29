/*
 * FLUX资金追踪分析系统 v3.3.4
 * Copyright (c) 2025 刘光浚
 * 开发完成日期: 2025年8月28日
 */

import React, { useEffect, useState } from "react";
import { Routes, Route } from "react-router-dom";
import { Box, CircularProgress, Alert, useTheme } from "@mui/material";
import { useTranslation } from "react-i18next";

// 组件导入
import Layout from "./components/layout/Layout";
import ErrorBoundary from "./components/ErrorBoundary";
import HomePage from "./pages/HomePage";
import AuditPage from "./pages/AuditPage";
import TimePointQueryPage from "./pages/TimePointQueryPage";
import SettingsPage from "./pages/SettingsPage";
import AppStateProvider from "./contexts/AppStateContext";
// import TestPage from "./pages/TestPage";

// 服务和类型导入
import { SystemService, SystemEnvStatus } from "./services/systemService";

const App: React.FC = () => {
  const { t } = useTranslation();
  const theme = useTheme();
  const [loading, setLoading] = useState(true);
  const [envStatus, setEnvStatus] = useState<SystemEnvStatus | null>(null);
  const [error, setError] = useState<string | null>(null);

  // 初始化检查
  useEffect(() => {
    const initializeApp = async () => {
      try {
        setLoading(true);
        setError(null);
        
        // 初始化系统服务（包含环境检查和文件状态同步）
        const initResult = await SystemService.initialize();
        setEnvStatus(initResult.environmentStatus);
        
        // 输出文件同步结果（如果有）
        if (initResult.fileSyncResult) {
          const { totalChecked, totalUpdated, errors } = initResult.fileSyncResult;
          if (totalUpdated > 0) {
            console.log(`分析历史文件状态已同步: 检查了 ${totalChecked} 条记录，更新了 ${totalUpdated} 条`);
          }
          if (errors.length > 0) {
            console.warn('文件状态同步过程中出现错误:', errors);
          }
        }
        
        if (!initResult.environmentStatus.system_available) {
          setError('系统环境检查失败，某些功能可能无法正常使用。');
        }
        
      } catch (err) {
        console.error('Initialization failed:', err);
        setError(t('errors.initialization_failed'));
      } finally {
        setLoading(false);
      }
    };

    initializeApp();
  }, [t]);

  // 根据主题模式设置HTML根元素的类名
  useEffect(() => {
    const htmlElement = document.documentElement;
    const bodyElement = document.body;
    
    if (theme.palette.mode === 'dark') {
      htmlElement.classList.add('dark');
      bodyElement.classList.add('dark');
      // 设置动态背景色
      bodyElement.style.backgroundColor = theme.palette.background.default;
    } else {
      htmlElement.classList.remove('dark');
      bodyElement.classList.remove('dark');
      // 设置动态背景色
      bodyElement.style.backgroundColor = theme.palette.background.default;
    }
  }, [theme.palette.mode, theme.palette.background.default]);

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
              {t('settings_labels.work_directory')}: {envStatus.work_directory || 'N/A'}<br/>
              {t('settings_labels.backend_engine')}: {envStatus.backend_engine || 'N/A'}
            </Box>
          )}
        </Alert>
      </Box>
    );
  }

  // 正常应用界面
  return (
    <AppStateProvider>
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
    </AppStateProvider>
  );
};

export default App;