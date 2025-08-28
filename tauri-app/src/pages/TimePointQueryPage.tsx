import React, { useState, useRef, useCallback, useEffect } from 'react';
import {
  Box,
  Typography,
  Card,
  CardContent,
  Button,
  TextField,
  Grid,
  Alert,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Table,
  useTheme,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
} from '@mui/material';
import {
  Search as SearchIcon,
  Clear as ClearIcon,
  CloudUpload as UploadIcon,
  FolderOpen as FolderIcon,
  Description as FileIcon,
  FileDownload as ExportIcon,
} from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import i18n from 'i18next';
import { open, save } from '@tauri-apps/api/dialog';
import { listen } from '@tauri-apps/api/event';
import { useNotification } from '../contexts/NotificationContext';
import { useAppState } from '../contexts/AppStateContext';
import { RustCommands, TimePointQuery, QueryResult, FundPool } from '../types/rust-commands';
import { invoke } from '@tauri-apps/api/tauri';
import { getCurrentLocalTime, formatLocalTime, createLogMessage } from '../utils/timeUtils';
import FileDropManager from '../utils/fileDropManager';

// 资金池查询结果接口
interface FundPoolQueryResult {
  success: boolean;
  pool_name: string;
  message?: string;
  records?: any[];
  summary?: {
    current_balance?: number;
    record_count?: number;
  };
}

const TimePointQueryPage: React.FC = () => {
  const { t } = useTranslation();
  const { showNotification } = useNotification();
  const theme = useTheme();
  const { 
    queryState, 
    updateQueryState,
    updateGlobalSelectedFile,
    addQueryHistory, 
    clearQueryHistory,
    appendQueryLog,
    clearQueryLog
  } = useAppState();
  
  // 从全局状态解构所需的值
  const {
    filePath,
    rowNumber,
    algorithm,
    queryResult,
    isQuerying,
    history,
    isDragOver,
    queryLog
  } = queryState;
  const [statusInterval, setStatusInterval] = useState<NodeJS.Timeout | null>(null);
  const [fundPoolResult, setFundPoolResult] = useState<FundPoolQueryResult | null>(null);
  const [selectedPool, setSelectedPool] = useState<string>('');
  const [isQueryingPool, setIsQueryingPool] = useState(false);
  const [selectedCategory, setSelectedCategory] = useState<string>('');
  const [selectedSubCategory, setSelectedSubCategory] = useState<string>('');
  const dropZoneRef = useRef<HTMLDivElement>(null);
  const lastFileSelection = useRef<{filePath: string, fileName: string, timestamp: number}>({filePath: '', fileName: '', timestamp: 0});
  
  // 使用ref存储最新的函数和状态，避免闭包陷阱
  const stateRef = useRef({ queryState, updateGlobalSelectedFile, appendQueryLog, showNotification, t });
  stateRef.current = { queryState, updateGlobalSelectedFile, appendQueryLog, showNotification, t };

  // 设置Tauri文件拖拽监听 - 使用稳定的处理逻辑避免重复监听器
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupFileDrop = async () => {
      try {
        unlisten = await listen('tauri://file-drop', (event) => {
          const files = event.payload as string[];
          if (files.length > 0) {
            const filePath = files[0];
            const fileName = filePath.split(/[/\\]/).pop() || '';
            
            // 使用全局防重复管理器
            if (FileDropManager.getInstance().shouldSkipDrop(filePath)) {
              return;
            }
            
            // 检查文件扩展名
            if (fileName.toLowerCase().endsWith('.xlsx') || fileName.toLowerCase().endsWith('.xls')) {
              // 记录文件选择信息
              const now = Date.now();
              lastFileSelection.current = {filePath: filePath, fileName: fileName, timestamp: now};
              
              // 使用ref获取当前最新状态，避免闭包问题
              const { queryState: currentQueryState, updateGlobalSelectedFile: currentUpdate, appendQueryLog: currentAppend, showNotification: currentNotification, t: currentT } = stateRef.current;
              
              // 只在确实不同文件时添加日志并更新全局文件状态
              if (currentQueryState.filePath !== filePath) {
                console.log(`[时点查询] 文件变更: ${currentQueryState.filePath} -> ${filePath}`);
                // 先添加本页面日志，再更新全局状态
                currentAppend(createLogMessage(`已选择文件：${fileName}`, 'success'));
                currentUpdate(filePath); // 使用全局更新方法，一次性同步所有页面
              } else {
                console.log(`[时点查询] 文件相同，跳过日志添加: ${fileName}`);
              }
              
              currentNotification({
                type: 'success',
                title: currentT('notifications.success.file_drag_success'),
                message: currentT('notifications.success.file_selected', { filename: fileName }),
              });
            } else {
              const { showNotification: currentNotification, t: currentT } = stateRef.current;
              currentNotification({
                type: 'warning',
                title: currentT('notifications.errors.file_format_unsupported'),
                message: currentT('notifications.errors.please_select_excel'),
              });
            }
          }
        });
      } catch (error) {
        console.error('Failed to setup file drag listener:', error);
      }
    };

    setupFileDrop();

    return () => {
      if (unlisten) {
        unlisten();
      }
    };
  }, []); // 完全不依赖任何状态，避免重复设置

  // 获取处理状态的函数
  const fetchProcessStatus = async () => {
    try {
      const status = await invoke<any>('get_process_status');
      // 时点查询不需要获取后端分析日志，使用独立的查询日志系统
      console.log('Query process status:', status);
    } catch (error) {
      console.error('Failed to get processing status:', error);
    }
  };

  // 文件选择处理
  const handleSelectFile = async () => {
    try {
      const selected = await open({
        title: t('notifications.success.file_selection'),
        multiple: false,
        filters: [{
          name: t('file_types.excel_files'),
          extensions: ['xlsx', 'xls']
        }]
      });

      if (selected && typeof selected === 'string') {
        const fileName = selected.split(/[/\\]/).pop() || '';
        
        // 防止1000ms内重复处理相同文件或任何文件选择操作  
        const now = Date.now();
        if ((lastFileSelection.current.filePath === selected && 
             now - lastFileSelection.current.timestamp < 1000) ||
            (now - lastFileSelection.current.timestamp < 300)) {
          return; // 跳过重复处理
        }
        
        // 记录文件选择信息
        lastFileSelection.current = {filePath: selected, fileName: fileName, timestamp: now};
        
        // 只在不是重复选择时添加日志并更新全局文件状态
        if (queryState.filePath !== selected) {
          appendQueryLog(createLogMessage(`已选择文件：${fileName}`, 'success'));
          updateGlobalSelectedFile(selected); // 使用全局更新方法，一次性同步所有页面
        }
        showNotification({
          type: 'success',
          title: t('notifications.success.file_selection'),
          message: t('notifications.success.file_selected', { filename: fileName }),
        });
      }
    } catch (error) {
      console.error('File selection failed:', error);
      showNotification({
        type: 'error',
        title: t('notifications.errors.file_selection_failed'),
        message: t('notifications.errors.file_operation_failed'),
      });
    }
  };

  // 拖拽处理
  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    updateQueryState({ isDragOver: true });
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    updateQueryState({ isDragOver: false });
  }, []);

  const handleDrop = useCallback(async (e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    updateQueryState({ isDragOver: false });

    // 在Tauri应用中，文件拖拽主要通过Tauri的API处理
    // HTML5 File API在桌面应用中无法提供完整文件路径
    showNotification({
      type: 'info',
      title: t('notifications.info.drag_drop_hint'),
      message: t('notifications.info.drag_drop_message'),
    });
  }, [showNotification]);

  const handleQuery = async () => {
    if (!filePath || !rowNumber) {
      showNotification({
        type: 'warning',
        title: t('notifications.errors.missing_parameters'),
        message: t('notifications.errors.select_file_and_row'),
      });
      return;
    }

    const rowNum = parseInt(rowNumber);
    if (isNaN(rowNum) || rowNum <= 0) {
      showNotification({
        type: 'warning',
        title: t('notifications.errors.invalid_row_number'),
        message: t('notifications.errors.enter_valid_row'),
      });
      return;
    }

    updateQueryState({ isQuerying: true });
    
    // 添加查询开始日志
    appendQueryLog(createLogMessage(`开始查询第${rowNum}行数据，算法：${algorithm === 'FIFO' ? 'FIFO计算法' : '差额计算法'}`, 'info'));
    
    // 启动状态轮询
    const interval = setInterval(fetchProcessStatus, 200);
    setStatusInterval(interval);
    
    try {
      console.log('Executing time point query', { filePath, rowNumber: rowNum, algorithm });
      
      // 构建查询参数
      const queryParams: TimePointQuery = {
        file_path: filePath,
        row_number: rowNum,
        algorithm: algorithm,
      };

      // 调用后端真实查询功能
      const queryResult: QueryResult = await RustCommands.timePointQuery(queryParams);
      
      console.log('后端查询结果:', queryResult); // 调试信息
      
      if (queryResult.success && queryResult.data) {
        // 正确提取嵌套的数据结构
        const data = queryResult.data;
        console.log('提取数据对象:', data); // 调试信息
        console.log('data.available_fund_pools:', data.available_fund_pools); // 调试信息
        console.log('data.fund_pool_records:', data.fund_pool_records); // 调试信息
        const newQueryResult = {
          rowNumber: rowNum,
          timestamp: getCurrentLocalTime('iso'),
          rawData: queryResult.data,
          message: queryResult.message,
          
          // 从 data 中提取具体字段
          algorithm: data.algorithm,
          target_row: data.target_row,
          total_rows: data.total_rows,
          processing_time: data.processing_time,
          
          // 嵌套对象
          target_row_data: data.target_row_data,
          tracker_state: data.tracker_state,
          processing_stats: data.processing_stats,
          recent_steps: data.recent_steps,
          available_fund_pools: data.available_fund_pools,
          fund_pool_records: data.fund_pool_records
        };
        
        const completedMessage = (() => {
          const i18nString = t('notifications.success.query_completed', { row: rowNum });
          // 根据当前语言提供回退
          const currentLang = i18n.language || 'zh';
          const directString = currentLang === 'en' 
            ? `Row ${rowNum} data query completed`
            : `第${rowNum}行数据查询完成`;
          console.log('Query completed interpolation:', { rowNum, currentLang, i18nString, directString });
          return i18nString.includes('{') ? directString : i18nString;
        })();
        
        showNotification({
          type: 'success',
          title: t('notifications.success.query_success'),
          message: completedMessage,
        });
        
        // 添加到历史记录
        const historyItem = {
          id: Date.now().toString(),
          timestamp: new Date(),
          fileName: filePath.split(/[/\\]/).pop(),
          rowNumber: rowNum,
          algorithm,
          result: queryResult.data  // 保持原始数据结构
        };
        addQueryHistory(historyItem);
        console.log('准备更新状态，newQueryResult:', newQueryResult); // 调试信息
        console.log('newQueryResult.available_fund_pools:', newQueryResult.available_fund_pools); // 调试信息
        updateQueryState({ queryResult: newQueryResult });
        console.log('状态更新后，queryResult应该是:', newQueryResult); // 调试信息
        
        // 添加查询成功日志
        const processingTime = data.processing_time ? data.processing_time.toFixed(3) : '0.000';
        appendQueryLog(createLogMessage(`查询成功 - 处理时间：${processingTime}秒`, 'success'));
        appendQueryLog(createLogMessage(`获取到第${rowNum}行数据，总行数：${data.total_rows}`, 'info'));
      } else {
        // 查询失败
        updateQueryState({ queryResult: null });
        appendQueryLog(createLogMessage(`查询失败：${queryResult.message || '未知错误'}`, 'error'));
        showNotification({
          type: 'error',
          title: t('notifications.errors.query_failed'),
          message: queryResult.message || t('notifications.errors.query_error'),
        });
      }
      
    } catch (error) {
      console.error('Query failed:', error);
      updateQueryState({ queryResult: null });
      appendQueryLog(createLogMessage(`查询异常：${error}`, 'error'));
      showNotification({
        type: 'error',
        title: t('notifications.errors.query_exception'),
        message: t('notifications.errors.query_execution_error', { error }),
      });
    } finally {
      updateQueryState({ isQuerying: false });
      
      // 停止状态轮询
      if (statusInterval) {
        clearInterval(statusInterval);
        setStatusInterval(null);
      }
      
      // 最后获取一次状态确保日志完整
      setTimeout(fetchProcessStatus, 500);
    }
  };


  const handleClearHistory = () => {
    clearQueryHistory();
    clearQueryLog();
    appendQueryLog(createLogMessage('查询历史和日志已清空', 'info'));
  };

  // 资金池查询处理函数
  const handleFundPoolQuery = async (poolName?: string) => {
    const targetPool = poolName || selectedPool;
    if (!targetPool || !queryResult) return;
    
    setIsQueryingPool(true);
    try {
      appendQueryLog(createLogMessage(`获取资金池详情：${targetPool}`, 'info'));
      
      // 从时点查询结果中获取真实的资金池记录
      const fundPoolRecords = queryResult.fund_pool_records?.[targetPool];
      const selectedPoolData = queryResult.available_fund_pools?.find((pool: any) => pool.name === targetPool);
      
      if (fundPoolRecords && fundPoolRecords.length > 0) {
        // 计算汇总信息
        let totalInflow = 0;
        let totalOutflow = 0;
        let currentBalance = 0;
        
        for (const record of fundPoolRecords) {
          totalInflow += parseFloat(record['入金'] || 0);
          totalOutflow += parseFloat(record['出金'] || 0);
          currentBalance = parseFloat(record['总余额'] || 0); // 使用最后一条记录的余额
        }
        
        setFundPoolResult({
          success: true,
          pool_name: targetPool,
          records: fundPoolRecords,
          summary: {
            total_inflow: totalInflow,
            total_outflow: totalOutflow,
            current_balance: currentBalance,
            record_count: fundPoolRecords.length,
          }
        });
        
        appendQueryLog(createLogMessage(`资金池${targetPool}查询成功：${fundPoolRecords.length}条交易记录`, 'success'));
        appendQueryLog(createLogMessage(`当前余额：¥${currentBalance.toLocaleString()}，累计入金：¥${totalInflow.toLocaleString()}`, 'info'));
      } else if (selectedPoolData) {
        // 资金池存在但没有交易记录
        setFundPoolResult({
          success: true,
          pool_name: targetPool,
          records: [],
          summary: {
            total_inflow: 0,
            total_outflow: 0,
            current_balance: selectedPoolData.total_amount,
            record_count: 0,
          }
        });
        
        appendQueryLog(createLogMessage(`资金池${targetPool}存在但无交易记录`, 'info'));
        appendQueryLog(createLogMessage(`当前余额：¥${selectedPoolData.total_amount.toLocaleString()}`, 'info'));
      } else {
        setFundPoolResult({
          success: false,
          pool_name: targetPool,
          message: '未找到该资金池信息'
        });
        appendQueryLog(createLogMessage(`未找到资金池：${targetPool}`, 'error'));
      }
    } catch (error) {
      appendQueryLog(createLogMessage(`资金池查询异常：${error}`, 'error'));
      console.error('Fund pool query failed:', error);
    } finally {
      setIsQueryingPool(false);
    }
  };

  // 导出当前时点全部资金池信息到Excel
  const handleExportFundPool = async () => {
    try {
      if (!queryResult || !queryResult.available_fund_pools || !queryResult.fund_pool_records) {
        showNotification({
          type: 'warning',
          title: '导出失败',
          message: '没有可导出的资金池数据',
        });
        return;
      }
      
      // 显示文件保存对话框
      const timestamp = new Date().toISOString().slice(0, 19).replace(/[:-]/g, '').replace('T', '_');
      const defaultFileName = `当前时点资金池信息_${timestamp}.xlsx`;
      
      // 确定默认保存目录（输入文件的目录）
      const inputDir = filePath ? filePath.substring(0, filePath.lastIndexOf('\\')) : '';
      const defaultPath = inputDir ? `${inputDir}\\${defaultFileName}` : defaultFileName;
      
      const outputPath = await save({
        title: '另存为Excel文件',
        defaultPath: defaultPath,
        filters: [{
          name: 'Excel文件',
          extensions: ['xlsx']
        }]
      });
      
      if (!outputPath) {
        // 用户取消了保存
        return;
      }
      
      appendQueryLog(createLogMessage('开始导出当前时点全部资金池信息到Excel', 'info'));
      
      // 构造Excel导出数据
      const exportData = {
        query_info: {
          file_path: filePath,
          target_row: rowNumber,
          algorithm: algorithm,
          query_time: queryResult.query_time,
        },
        fund_pools: queryResult.available_fund_pools,
        fund_pool_records: queryResult.fund_pool_records,
        export_type: "current_timepoint_fund_pools",
        output_path: outputPath // 添加用户选择的输出路径
      };
      
      // 调用后端Excel导出功能
      const result = await invoke('export_fund_pools_excel', { request: exportData });
      
      if (result.success) {
        appendQueryLog(createLogMessage(`当前时点资金池信息已导出到：${result.output_path}`, 'success'));
        showNotification({
          type: 'success',
          title: '导出成功',
          message: `当前时点资金池信息已导出到：${result.output_path}`,
        });
      } else {
        throw new Error(result.message || '导出失败');
      }
      
    } catch (error) {
      const errorMsg = `资金池信息导出异常：${error}`;
      appendQueryLog(createLogMessage(errorMsg, 'error'));
      console.error('Fund pool export failed:', error);
      showNotification({
        type: 'error',
        title: '导出异常',
        message: errorMsg,
      });
    }
  };

  return (
    <Box sx={{ maxWidth: 1200, mx: 'auto', p: 2 }}>
      <Typography variant="h4" component="h1" gutterBottom>
        {t('query.title')}
      </Typography>
      
      {/* 第一行和第二行：使用嵌套Grid实现2x2布局 */}
      <Grid container spacing={3}>
        {/* 左侧列容器 */}
        <Grid item xs={12} md={6}>
          <Grid container spacing={6}>
            {/* 左上：查询配置面板 */}
            <Grid item xs={12}>
              <Card>
                <CardContent>
                  <Typography variant="h6" gutterBottom>
                    {t('ui.panels.query_config')}
                  </Typography>
                  
                  {/* 文件拖拽区域 */}
                  <Paper
                    ref={dropZoneRef}
                    onDragOver={handleDragOver}
                    onDragLeave={handleDragLeave}
                    onDrop={handleDrop}
                    sx={{
                      p: 2,
                      mb: 2,
                      border: isDragOver ? `2px dashed ${theme.palette.primary.main}` : `2px dashed ${theme.palette.divider}`,
                      backgroundColor: isDragOver 
                        ? theme.palette.mode === 'dark' ? 'rgba(144, 202, 249, 0.08)' : 'rgba(25, 118, 210, 0.08)'
                        : theme.palette.mode === 'dark' ? 'rgba(255, 255, 255, 0.05)' : 'rgba(0, 0, 0, 0.02)',
                      borderRadius: 2,
                      textAlign: 'center',
                      cursor: 'pointer',
                      transition: 'all 0.3s ease',
                      '&:hover': {
                        borderColor: theme.palette.primary.main,
                        backgroundColor: theme.palette.mode === 'dark' ? 'rgba(255, 255, 255, 0.08)' : 'rgba(0, 0, 0, 0.04)',
                      }
                    }}
                    elevation={isDragOver ? 3 : 1}
                  >
                    <Box>
                      <UploadIcon 
                        sx={{ 
                          fontSize: 32, 
                          color: isDragOver ? theme.palette.primary.main : theme.palette.text.secondary,
                          mb: 0.5 
                        }} 
                      />
                      <Typography variant="body1" gutterBottom>
                        {filePath ? (
                          <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center', gap: 1 }}>
                            <FileIcon />
                            {filePath.split(/[/\\]/).pop()}
                          </Box>
                        ) : (
                          isDragOver ? t('ui.dragdrop.release_to_select') : t('ui.dragdrop.drag_excel_here')
                        )}
                      </Typography>
                      <Typography variant="caption" color="textSecondary" display="block" sx={{ mb: 1 }}>
                        {filePath ? t('ui.dragdrop.click_to_change') : t('ui.dragdrop.supported_formats')}
                      </Typography>
                      <Button
                        variant={filePath ? "outlined" : "contained"}
                        size="small"
                        startIcon={<FolderIcon />}
                        disabled={isQuerying}
                        onClick={(e) => {
                          e.stopPropagation();
                          handleSelectFile();
                        }}
                      >
                        {filePath ? t('ui.buttons.change_file') : t('ui.buttons.browse_file')}
                      </Button>
                    </Box>
                  </Paper>

                  <TextField
                    fullWidth
                    label={t('query.target_row')}
                    value={rowNumber}
                    onChange={(e) => updateQueryState({ rowNumber: e.target.value })}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter' && !isQuerying) {
                        handleQuery();
                      }
                    }}
                    type="number"
                    placeholder={t('placeholders.enter_row_number')}
                    disabled={isQuerying}
                    sx={{ mb: 2 }}
                  />

                  <FormControl fullWidth sx={{ mb: 2 }}>
                    <InputLabel id="algorithm-select-label">
                      {t('audit.algorithm')}
                    </InputLabel>
                    <Select
                      labelId="algorithm-select-label"
                      value={algorithm}
                      label={t('audit.algorithm')}
                      onChange={(e) => updateQueryState({ algorithm: e.target.value as 'FIFO' | 'BALANCE_METHOD' })}
                      disabled={isQuerying}
                    >
                      <MenuItem value="FIFO">{t('audit.fifo')}</MenuItem>
                      <MenuItem value="BALANCE_METHOD">{t('audit.balance_method')}</MenuItem>
                    </Select>
                  </FormControl>

                  <Button
                    variant="contained"
                    startIcon={<SearchIcon />}
                    onClick={handleQuery}
                    disabled={!filePath || !rowNumber || isQuerying}
                    fullWidth
                  >
                    {isQuerying ? t('common.processing') : t('query.query_button')}
                  </Button>
                </CardContent>
              </Card>
            </Grid>

            {/* 左下：资金池信息区域 */}
            <Grid item xs={12}>
              <Card sx={{ height: 'fit-content', maxHeight: '600px' }}>
                <CardContent sx={{ maxHeight: '550px', overflow: 'auto' }}>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
                    <Typography variant="h6">
                      资金池信息
                    </Typography>
                    {queryResult && queryResult.available_fund_pools && queryResult.available_fund_pools.length > 0 && (
                      <Button
                        variant="outlined"
                        size="small"
                        startIcon={<ExportIcon />}
                        onClick={handleExportFundPool}
                      >
                        另存为
                      </Button>
                    )}
                  </Box>
                  
                  {/* 资金池选择 */}
                  {queryResult && queryResult.available_fund_pools && queryResult.available_fund_pools.length > 0 ? (
                    <Box>
                      <Typography variant="subtitle2" sx={{ mb: 2 }}>
                        发现活跃资金池: {queryResult.available_fund_pools.length} 个
                      </Typography>
                      
                      {/* 分类筛选 */}
                      <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 2 }}>
                        <FormControl size="small" sx={{ minWidth: 120 }}>
                          <InputLabel>资金池类型</InputLabel>
                          <Select
                            value={selectedCategory}
                            onChange={(e) => {
                              setSelectedCategory(e.target.value);
                              setSelectedSubCategory('');
                              setSelectedPool('');
                            }}
                            label="资金池类型"
                          >
                            <MenuItem value="">全部类型</MenuItem>
                            {(() => {
                              // 提取所有资金池的类型（"-"之前的部分）
                              const categories = Array.from(new Set(
                                queryResult.available_fund_pools.map((pool: FundPool) => {
                                  const parts = pool.name.split('-');
                                  return parts[0] || pool.name;
                                })
                              )).sort();
                              return categories.map(category => (
                                <MenuItem key={category} value={category}>
                                  {category}
                                </MenuItem>
                              ));
                            })()}
                          </Select>
                        </FormControl>

                        <FormControl size="small" sx={{ minWidth: 150 }}>
                          <InputLabel>子类型</InputLabel>
                          <Select
                            value={selectedSubCategory}
                            onChange={(e) => {
                              setSelectedSubCategory(e.target.value);
                              setSelectedPool('');
                            }}
                            label="子类型"
                            disabled={!selectedCategory}
                          >
                            <MenuItem value="">全部子类型</MenuItem>
                            {(() => {
                              if (!selectedCategory) return [];
                              // 获取选定类型下的所有子类型
                              const subCategories = Array.from(new Set(
                                queryResult.available_fund_pools
                                  .filter((pool: FundPool) => pool.name.startsWith(selectedCategory + '-'))
                                  .map((pool: FundPool) => {
                                    const parts = pool.name.split('-');
                                    return parts.slice(1).join('-') || '其他';
                                  })
                              )).sort();
                              return subCategories.map(subCategory => (
                                <MenuItem key={subCategory} value={subCategory}>
                                  {subCategory}
                                </MenuItem>
                              ));
                            })()}
                          </Select>
                        </FormControl>
                      </Box>
                      
                      <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 3 }}>
                        <Button
                          variant="contained"
                          onClick={() => {
                            // 通过类型和子类型自动确定资金池
                            if (selectedCategory && selectedSubCategory) {
                              const targetPoolName = `${selectedCategory}-${selectedSubCategory}`;
                              const matchingPool = queryResult.available_fund_pools.find(
                                (pool: FundPool) => pool.name === targetPoolName
                              );
                              if (matchingPool) {
                                setSelectedPool(targetPoolName);
                                handleFundPoolQuery(targetPoolName);
                              }
                            }
                          }}
                          disabled={!selectedCategory || !selectedSubCategory || isQueryingPool}
                          startIcon={<SearchIcon />}
                          sx={{ minWidth: 120 }}
                        >
                          {isQueryingPool ? '查询中...' : '查询详情'}
                        </Button>
                      </Box>
                    </Box>
                  ) : queryResult ? (
                    <Box sx={{ mb: 3 }}>
                      <Alert severity="info">
                        当前时点尚未发现投资产品资金池
                        <br />
                        <small>资金池通常出现在申购/赎回等投资产品交易中</small>
                      </Alert>
                    </Box>
                  ) : (
                    <Box sx={{ mb: 3 }}>
                      <Alert severity="info">
                        请先执行时点查询以获取资金池信息
                        <br />
                        <small>选择Excel文件并输入行号后点击查询按钮</small>
                      </Alert>
                    </Box>
                  )}
                </CardContent>
              </Card>
            </Grid>
          </Grid>
        </Grid>

        {/* 右侧：查询结果面板 - 占据右侧整列 */}
        <Grid item xs={12} md={6}>
          <Card sx={{ height: 'fit-content' }}>
            <CardContent>
              <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
                <Typography variant="h6">
                  {t('query.query_result')}
                </Typography>
              </Box>
              
              {queryResult ? (
                <Box>
                  <Alert severity="success" sx={{ mb: 2 }}>
                    {(() => {
                      const algorithm = queryResult.algorithm || 'Unknown';
                      const time = queryResult.processing_time?.toFixed(3) || '0.000';
                      console.log('QueryResult for translation:', { algorithm, time });
                      
                      // 尝试直接字符串插值作为备选
                      const directString = `${t('table.data.query_success_completed')} - ${t('analysis.algorithm')}: ${algorithm} | ${t('ui.labels.time')}: ${time}s`;
                      const i18nString = t('table.data.query_success_with_time', { algorithm, time });
                      
                      console.log('Direct string:', directString);
                      console.log('i18n string:', i18nString);
                      
                      // 如果i18n插值失败，回退到直接字符串
                      const hasPlaceholder = i18nString.indexOf('{') !== -1;
                      return hasPlaceholder ? directString : i18nString;
                    })()}
                  </Alert>
                  
                  <Typography variant="subtitle2" gutterBottom>
                    {(() => {
                      const row = queryResult.target_row;
                      console.log('Row interpolation debug:', { row });
                      
                      const i18nString = t('table.data.transaction_data_row', { row });
                      const directString = `${t('table.data.transaction_data_prefix')} (${t('table.data.row')} ${row})`;
                      
                      console.log('Row i18n string:', i18nString);
                      console.log('Row direct string:', directString);
                      
                      const hasPlaceholder = i18nString.indexOf('{') !== -1;
                      return hasPlaceholder ? directString : i18nString;
                    })()}
                  </Typography>
                  <TableContainer component={Paper} sx={{ mb: 2 }}>
                    <Table size="small">
                      <TableBody>
                        <TableRow>
                          <TableCell sx={{ width: '30%', fontWeight: 'bold' }}>{t('table.headers.row_number')}</TableCell>
                          <TableCell sx={{ width: '70%', textAlign: 'center' }}>{queryResult.target_row}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell sx={{ width: '30%', fontWeight: 'bold' }}>{t('table.headers.timestamp')}</TableCell>
                          <TableCell sx={{ width: '70%', whiteSpace: 'nowrap', textAlign: 'center' }}>{queryResult.target_row_data?.timestamp || '--'}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell sx={{ width: '30%', fontWeight: 'bold' }}>{t('table.headers.income_amount')}</TableCell>
                          <TableCell sx={{ width: '70%', whiteSpace: 'nowrap', textAlign: 'center' }}>¥{parseFloat(queryResult.target_row_data?.income_amount || 0).toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell sx={{ width: '30%', fontWeight: 'bold' }}>{t('table.headers.expense_amount')}</TableCell>
                          <TableCell sx={{ width: '70%', whiteSpace: 'nowrap', textAlign: 'center' }}>¥{parseFloat(queryResult.target_row_data?.expense_amount || 0).toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell sx={{ width: '30%', fontWeight: 'bold' }}>{t('table.headers.balance')}</TableCell>
                          <TableCell sx={{ width: '70%', whiteSpace: 'nowrap', textAlign: 'center' }}>¥{parseFloat(queryResult.target_row_data?.balance || 0).toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell sx={{ width: '30%', fontWeight: 'bold' }}>{t('table.headers.fund_attr')}</TableCell>
                          <TableCell sx={{ width: '70%', textAlign: 'center' }}>{queryResult.target_row_data?.fund_attr || '--'}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell sx={{ width: '30%', fontWeight: 'bold' }}>{t('table.headers.flow_type')}</TableCell>
                          <TableCell sx={{ width: '70%', textAlign: 'center' }}>{queryResult.target_row_data?.flow_type || '--'}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell sx={{ width: '30%', fontWeight: 'bold' }}>{t('table.headers.behavior')}</TableCell>
                          <TableCell sx={{ width: '70%', wordBreak: 'break-word' }}>{queryResult.target_row_data?.behavior || '--'}</TableCell>
                        </TableRow>
                      </TableBody>
                    </Table>
                  </TableContainer>

                  <Typography variant="subtitle2" gutterBottom>
                    {t('ui.panels.tracker_status')}
                  </Typography>
                  <TableContainer component={Paper} sx={{ mb: 2 }}>
                    <Table size="small">
                      <TableBody>
                        {queryResult.tracker_state?.personal_balance !== undefined && (
                          <TableRow>
                            <TableCell sx={{ width: '30%', fontWeight: 'bold' }}>{t('table.headers.personal_balance')}</TableCell>
                            <TableCell sx={{ width: '70%', whiteSpace: 'nowrap', textAlign: 'center' }}>¥{parseFloat(queryResult.tracker_state.personal_balance).toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}</TableCell>
                          </TableRow>
                        )}
                        {queryResult.tracker_state?.company_balance !== undefined && (
                          <TableRow>
                            <TableCell sx={{ width: '30%', fontWeight: 'bold' }}>{t('table.headers.company_balance')}</TableCell>
                            <TableCell sx={{ width: '70%', whiteSpace: 'nowrap', textAlign: 'center' }}>¥{parseFloat(queryResult.tracker_state.company_balance).toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}</TableCell>
                          </TableRow>
                        )}
                        {queryResult.tracker_state?.net_flow !== undefined && (
                          <TableRow>
                            <TableCell sx={{ width: '30%', fontWeight: 'bold' }}>{t('table.headers.net_flow')}</TableCell>
                            <TableCell sx={{ width: '70%', whiteSpace: 'nowrap', textAlign: 'center' }}>¥{parseFloat(queryResult.tracker_state.net_flow).toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}</TableCell>
                          </TableRow>
                        )}
                      </TableBody>
                    </Table>
                  </TableContainer>

                  <Typography variant="subtitle2" gutterBottom>
                    {t('ui.panels.processing_stats')}
                  </Typography>
                  <TableContainer component={Paper} sx={{ mb: 2 }}>
                    <Table size="small">
                      <TableBody>
                        <TableRow>
                          <TableCell sx={{ width: '30%', fontWeight: 'bold' }}>{t('table.headers.total_rows')}</TableCell>
                          <TableCell sx={{ width: '70%', textAlign: 'center' }}>{queryResult.total_rows}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell sx={{ width: '30%', fontWeight: 'bold' }}>{t('table.headers.processed_rows')}</TableCell>
                          <TableCell sx={{ width: '70%', textAlign: 'center' }}>{queryResult.processing_stats?.last_processed_row}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell sx={{ width: '30%', fontWeight: 'bold' }}>{t('table.headers.error_count')}</TableCell>
                          <TableCell sx={{ 
                            width: '70%', 
                            textAlign: 'center',
                            color: queryResult.processing_stats?.error_count > 0 ? theme.palette.error.main : theme.palette.success.main
                          }}>
                            {queryResult.processing_stats?.error_count || 0}
                          </TableCell>
                        </TableRow>
                      </TableBody>
                    </Table>
                  </TableContainer>

                </Box>
              ) : (
                <Alert severity="info">
                  {t('placeholders.no_results')}
                </Alert>
              )}
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* 第三行：资金池详情显示 - 全宽度展示详细交易记录 */}
      {fundPoolResult && (
        <Grid container spacing={3} sx={{ mt: 2 }}>
          <Grid item xs={12}>
            <Card>
              <CardContent>
                {fundPoolResult.success && fundPoolResult.records && fundPoolResult.records.length > 0 && (
                  <Box>
                    <Typography variant="subtitle2" gutterBottom>
                      {fundPoolResult.pool_name} - 详细交易记录 ({fundPoolResult.records.length}条)
                    </Typography>
                    <TableContainer component={Paper} sx={{ maxHeight: 400 }}>
                      <Table size="small" stickyHeader>
                        <TableHead>
                          <TableRow>
                            <TableCell sx={{ textAlign: 'center' }}>交易时间</TableCell>
                            <TableCell sx={{ textAlign: 'center' }}>资金流向</TableCell>
                            <TableCell sx={{ textAlign: 'center' }}>总余额</TableCell>
                            <TableCell sx={{ textAlign: 'center' }}>个人余额</TableCell>
                            <TableCell sx={{ textAlign: 'center' }}>公司余额</TableCell>
                            <TableCell sx={{ textAlign: 'center' }}>累计申购</TableCell>
                            <TableCell sx={{ textAlign: 'center' }}>累计赎回</TableCell>
                            <TableCell sx={{ textAlign: 'center' }}>净盈亏</TableCell>
                          </TableRow>
                        </TableHead>
                        <TableBody>
                          {fundPoolResult.records.map((record, index) => {
                            const inflow = parseFloat(record['入金'] || 0);
                            const outflow = parseFloat(record['出金'] || 0);
                            
                            let flowDisplay = '';
                            if (inflow > 0 && outflow > 0) {
                              flowDisplay = `入金 ¥${inflow.toLocaleString('zh-CN', {minimumFractionDigits: 2})} / 出金 ¥${outflow.toLocaleString('zh-CN', {minimumFractionDigits: 2})}`;
                            } else if (inflow > 0) {
                              flowDisplay = `入金 ¥${inflow.toLocaleString('zh-CN', {minimumFractionDigits: 2})}`;
                            } else if (outflow > 0) {
                              flowDisplay = `出金 ¥${outflow.toLocaleString('zh-CN', {minimumFractionDigits: 2})}`;
                            } else {
                              flowDisplay = '--';
                            }
                            
                            // 解析个人余额和公司余额
                            const personalBalance = record['个人余额'] || '0.00 (0%)';
                            const companyBalance = record['公司余额'] || '0.00 (0%)';
                            
                            // 提取金额和百分比
                            const parseBalanceData = (balanceStr) => {
                              const match = balanceStr.match(/^([\d.,]+)\s*\(([^)]+)\)$/);
                              if (match) {
                                return {
                                  amount: parseFloat(match[1].replace(/,/g, '')),
                                  percentage: match[2]
                                };
                              }
                              return { amount: 0, percentage: '0%' };
                            };
                            
                            const personalData = parseBalanceData(personalBalance);
                            const companyData = parseBalanceData(companyBalance);
                            
                            return (
                              <TableRow key={index}>
                                <TableCell sx={{ textAlign: 'center' }}>{record['交易时间'] || '--'}</TableCell>
                                <TableCell sx={{ textAlign: 'center' }}>{flowDisplay}</TableCell>
                                <TableCell sx={{ textAlign: 'center' }}>¥{parseFloat(record['总余额'] || 0).toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}</TableCell>
                                <TableCell sx={{ textAlign: 'center' }}>
                                  <Box>
                                    <div>¥{personalData.amount.toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}</div>
                                    <div style={{ fontSize: '0.8em', color: '#666' }}>{personalData.percentage}</div>
                                  </Box>
                                </TableCell>
                                <TableCell sx={{ textAlign: 'center' }}>
                                  <Box>
                                    <div>¥{companyData.amount.toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}</div>
                                    <div style={{ fontSize: '0.8em', color: '#666' }}>{companyData.percentage}</div>
                                  </Box>
                                </TableCell>
                                <TableCell sx={{ textAlign: 'center' }}>¥{parseFloat(record['累计申购'] || 0).toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}</TableCell>
                                <TableCell sx={{ textAlign: 'center' }}>¥{parseFloat(record['累计赎回'] || 0).toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}</TableCell>
                                <TableCell sx={{ textAlign: 'center' }}>¥{parseFloat(record['净盈亏'] || 0).toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}</TableCell>
                              </TableRow>
                            );
                          })}
                        </TableBody>
                      </Table>
                    </TableContainer>
                  </Box>
                )}
                
                {fundPoolResult.success && (!fundPoolResult.records || fundPoolResult.records.length === 0) && (
                  <Box>
                    <Typography variant="subtitle2" gutterBottom>
                      {fundPoolResult.pool_name} - 资金池信息
                    </Typography>
                    <Alert severity="info">
                      该资金池在当前时点没有交易记录。
                    </Alert>
                  </Box>
                )}
                
                {!fundPoolResult.success && (
                  <Alert severity="error">
                    {fundPoolResult.message || '资金池查询失败'}
                  </Alert>
                )}
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      )}

      {/* 第四行：查询日志 */}
      <Grid container spacing={3} sx={{ mt: 2 }}>
        <Grid item xs={12}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
                <Typography variant="h6">
                  {t('ui.panels.query_log')}
                </Typography>
                {queryLog.length > 0 && (
                  <Button
                    size="small"
                    variant="outlined"
                    onClick={() => {
                      const logText = queryLog.join('\n');
                      navigator.clipboard.writeText(logText).then(() => {
                        showNotification({
                          type: 'success',
                          title: t('notifications.success.copy_success'),
                          message: t('notifications.success.copy_completed', { count: queryLog.length }),
                        });
                      }).catch(err => {
                        console.error('Copy failed:', err);
                        showNotification({
                          type: 'error',
                          title: t('notifications.errors.copy_failed'),
                          message: t('notifications.errors.clipboard_access_denied'),
                        });
                      });
                    }}
                    sx={{ fontSize: '0.75rem', minWidth: 'auto', px: 1.5 }}
                  >
                    {t('ui.buttons.copy_log')}
                  </Button>
                )}
              </Box>
              
              <Paper
                sx={{
                  p: 2,
                  maxHeight: 300,
                  overflow: 'auto',
                  backgroundColor: theme.palette.mode === 'dark' ? 'rgba(255, 255, 255, 0.05)' : '#f8f9fa',
                  fontFamily: 'Consolas, "Courier New", monospace',
                  fontSize: '0.8rem',
                  lineHeight: 1.4,
                  userSelect: 'text',
                  cursor: 'text',
                  border: `1px solid ${theme.palette.divider}`,
                  borderRadius: 1,
                  '& *': {
                    userSelect: 'text',
                  },
                  '&::-webkit-scrollbar': {
                    width: '8px',
                  },
                  '&::-webkit-scrollbar-track': {
                    backgroundColor: theme.palette.mode === 'dark' ? 'rgba(255, 255, 255, 0.1)' : '#f1f1f1',
                  },
                  '&::-webkit-scrollbar-thumb': {
                    backgroundColor: theme.palette.mode === 'dark' ? 'rgba(255, 255, 255, 0.3)' : '#c1c1c1',
                    borderRadius: '4px',
                  },
                  '&::-webkit-scrollbar-thumb:hover': {
                    backgroundColor: theme.palette.mode === 'dark' ? 'rgba(255, 255, 255, 0.5)' : '#a8a8a8',
                  },
                }}
                variant="outlined"
              >
                {queryLog.length > 0 ? (
                  <Box>
                    {queryLog.map((log, index) => (
                      <Box
                        key={index}
                        sx={{ 
                          mb: 0.3,
                          padding: '2px 4px',
                          borderRadius: '2px',
                          backgroundColor: log.includes('ERROR') || log.includes('错误') || log.includes('失败') || log.includes('Failed') || log.includes(t('process_status.failed')) ? `${theme.palette.error.main}20` : 
                                          log.includes('WARNING') || log.includes('警告') || log.includes('Warning') ? `${theme.palette.warning.main}20` :
                                          log.includes('SUCCESS') || log.includes('完成') || log.includes('成功') || log.includes('Success') || log.includes(t('process_status.completed')) ? `${theme.palette.success.main}20` : 'transparent',
                          color: log.includes('ERROR') || log.includes('错误') || log.includes('失败') || log.includes('Failed') || log.includes(t('process_status.failed')) ? theme.palette.error.main : 
                                 log.includes('WARNING') || log.includes('警告') || log.includes('Warning') ? theme.palette.warning.main :
                                 log.includes('SUCCESS') || log.includes('完成') || log.includes('成功') || log.includes('Success') || log.includes(t('process_status.completed')) ? theme.palette.success.main : theme.palette.text.primary,
                          whiteSpace: 'pre-wrap',
                          wordBreak: 'break-all',
                        }}
                      >
                        {log}
                      </Box>
                    ))}
                    {/* 自动滚动到底部的占位符 - 只在查询进行中时滚动 */}
                    <div ref={(el) => {
                      if (el && isQuerying) {
                        el.scrollIntoView({ 
                          behavior: 'smooth', 
                          block: 'nearest',
                          inline: 'nearest' 
                        });
                      }
                    }} />
                  </Box>
                ) : (
                  <Typography variant="body2" color="text.secondary" sx={{ fontStyle: 'italic' }}>
                    {t('ui.status.query_log_placeholder')}
                    <br />
                    <small>{t('ui.status.text_selection_hint')}</small>
                  </Typography>
                )}
              </Paper>
            </CardContent>
          </Card>
        </Grid>

        {/* 查询历史 */}
        <Grid item xs={12}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
                <Typography variant="h6">
                  {t('query.query_history')} ({history.length})
                </Typography>
                {history.length > 0 && (
                  <Button
                    variant="outlined"
                    size="small"
                    startIcon={<ClearIcon />}
                    onClick={handleClearHistory}
                  >
                    {t('query.clear_history')}
                  </Button>
                )}
              </Box>
              
              {history.length > 0 ? (
                <TableContainer component={Paper}>
                  <Table>
                    <TableHead>
                      <TableRow>
                        <TableCell>{t('table.headers.timestamp')}</TableCell>
                        <TableCell>{t('table.headers.file')}</TableCell>
                        <TableCell>{t('table.headers.row_number')}</TableCell>
                        <TableCell>{t('table.headers.algorithm')}</TableCell>
                        <TableCell>{t('common.actions')}</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {history.slice(0, 10).map((item) => (
                        <TableRow key={item.id}>
                          <TableCell>
                            {formatLocalTime(item.timestamp, 'display')}
                          </TableCell>
                          <TableCell>{item.fileName}</TableCell>
                          <TableCell>{item.rowNumber}</TableCell>
                          <TableCell>{item.algorithm}</TableCell>
                          <TableCell>
                            <Button 
                              size="small" 
                              onClick={() => updateQueryState({ queryResult: item.result })}
                            >
                              {t('ui.buttons.view_details')}
                            </Button>
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </TableContainer>
              ) : (
                <Alert severity="info">
                  {t('placeholders.no_data')}
                </Alert>
              )}
            </CardContent>
          </Card>
        </Grid>
      </Grid>
    </Box>
  );
};

export default TimePointQueryPage;