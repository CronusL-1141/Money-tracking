import React, { createContext, useContext, useState, ReactNode, useCallback, useEffect } from 'react';
import { QueryHistoryStorage, DataMigration, DataCleanup } from '../utils/storageUtils';

// 审计页面状态
interface AuditPageState {
  algorithm: 'FIFO' | 'BALANCE_METHOD';
  inputFile: string;
  isAnalyzing: boolean;
  progress: number;
  analysisLog: string[];
  currentStep: string;
  isDragOver: boolean;
}

// 时点查询页面状态
interface TimePointQueryPageState {
  filePath: string;
  rowNumber: string;
  algorithm: 'FIFO' | 'BALANCE_METHOD';
  queryResult: any;
  isQuerying: boolean;
  history: any[];
  isDragOver: boolean;
  queryLog: string[];  // 添加查询专用日志
}

interface AppStateContextType {
  // 应用初始化状态
  isInitialized: boolean;
  
  // 审计页面状态管理
  auditState: AuditPageState;
  updateAuditState: (updates: Partial<AuditPageState>) => void;
  resetAuditState: () => void;
  appendAuditLog: (message: string) => void;
  clearAuditLog: () => void;
  
  // 时点查询页面状态管理
  queryState: TimePointQueryPageState;
  updateQueryState: (updates: Partial<TimePointQueryPageState>) => void;
  resetQueryState: () => void;
  addQueryHistory: (item: any) => void;
  clearQueryHistory: () => void;
  appendQueryLog: (message: string) => void;
  clearQueryLog: () => void;
}

const AppStateContext = createContext<AppStateContextType | undefined>(undefined);

export const useAppState = (): AppStateContextType => {
  const context = useContext(AppStateContext);
  if (!context) {
    throw new Error('useAppState must be used within an AppStateProvider');
  }
  return context;
};

interface AppStateProviderProps {
  children: ReactNode;
}

// 默认审计状态
const defaultAuditState: AuditPageState = {
  algorithm: 'FIFO',
  inputFile: '',
  isAnalyzing: false,
  progress: 0,
  analysisLog: [],
  currentStep: '',
  isDragOver: false,
};

// 默认查询状态
const defaultQueryState: TimePointQueryPageState = {
  filePath: '',
  rowNumber: '',
  algorithm: 'FIFO',
  queryResult: null,
  isQuerying: false,
  history: [],
  isDragOver: false,
  queryLog: [],  // 添加查询日志
};

export const AppStateProvider: React.FC<AppStateProviderProps> = ({ children }) => {
  const [auditState, setAuditState] = useState<AuditPageState>(defaultAuditState);
  const [queryState, setQueryState] = useState<TimePointQueryPageState>(defaultQueryState);
  const [isInitialized, setIsInitialized] = useState(false);

  // 审计状态管理方法
  const updateAuditState = useCallback((updates: Partial<AuditPageState>) => {
    setAuditState(prev => ({ ...prev, ...updates }));
  }, []);

  const resetAuditState = useCallback(() => {
    setAuditState(defaultAuditState);
  }, []);

  const appendAuditLog = useCallback((message: string) => {
    setAuditState(prev => ({
      ...prev,
      analysisLog: [...prev.analysisLog, message]
    }));
  }, []);

  const clearAuditLog = useCallback(() => {
    setAuditState(prev => ({
      ...prev,
      analysisLog: []
    }));
  }, []);

  // 查询状态管理方法
  const updateQueryState = useCallback((updates: Partial<TimePointQueryPageState>) => {
    setQueryState(prev => ({ ...prev, ...updates }));
  }, []);

  const resetQueryState = useCallback(() => {
    setQueryState(defaultQueryState);
  }, []);

  const addQueryHistory = useCallback((item: any) => {
    // 使用存储工具自动处理去重和保存
    const updatedHistory = QueryHistoryStorage.addRecord(item);
    
    setQueryState(prev => ({
      ...prev,
      history: updatedHistory
    }));
  }, []);

  const clearQueryHistory = useCallback(() => {
    // 清空本地存储
    QueryHistoryStorage.clear();
    
    setQueryState(prev => ({
      ...prev,
      history: []
    }));
  }, []);

  // 查询日志管理方法
  const appendQueryLog = useCallback((message: string) => {
    setQueryState(prev => ({
      ...prev,
      queryLog: [...prev.queryLog, message]
    }));
  }, []);

  const clearQueryLog = useCallback(() => {
    setQueryState(prev => ({
      ...prev,
      queryLog: []
    }));
  }, []);

  // 初始化应用数据
  useEffect(() => {
    const initializeAppData = async () => {
      try {
        console.log('Initializing application data...');

        // 检查是否需要数据迁移
        if (DataMigration.needsMigration()) {
          DataMigration.migrate();
        }

        // 清理过期数据
        DataCleanup.cleanupExpiredData();

        // 从本地存储加载查询历史
        const savedHistory = QueryHistoryStorage.load();
        console.log(`Loaded ${savedHistory.length} query history records`);

        // 更新查询状态
        setQueryState(prev => ({
          ...prev,
          history: savedHistory
        }));

        setIsInitialized(true);
        console.log('Application data initialization completed');
      } catch (error) {
        console.error('Failed to initialize application data:', error);
        setIsInitialized(true); // 即使失败也要标记为已初始化，避免阻塞应用
      }
    };

    initializeAppData();
  }, []);

  const value: AppStateContextType = {
    isInitialized,
    auditState,
    updateAuditState,
    resetAuditState,
    appendAuditLog,
    clearAuditLog,
    queryState,
    updateQueryState,
    resetQueryState,
    addQueryHistory,
    clearQueryHistory,
    appendQueryLog,
    clearQueryLog,
  };

  return (
    <AppStateContext.Provider value={value}>
      {children}
    </AppStateContext.Provider>
  );
};

export default AppStateProvider;
