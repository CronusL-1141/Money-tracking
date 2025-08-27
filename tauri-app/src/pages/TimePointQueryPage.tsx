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
  Save as SaveIcon,
  Clear as ClearIcon,
  CloudUpload as UploadIcon,
  FolderOpen as FolderIcon,
  Description as FileIcon,
} from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import i18n from 'i18next';
import { open } from '@tauri-apps/api/dialog';
import { listen } from '@tauri-apps/api/event';
import { useNotification } from '../contexts/NotificationContext';
import { useAppState } from '../contexts/AppStateContext';
import { RustCommands, TimePointQuery, QueryResult, FundPool, FundPoolQueryResult, FundPoolRecord } from '../types/rust-commands';
import { invoke } from '@tauri-apps/api/tauri';
import { getCurrentLocalTime, formatLocalTime, createLogMessage } from '../utils/timeUtils';

const TimePointQueryPage: React.FC = () => {
  const { t } = useTranslation();
  const { showNotification } = useNotification();
  const theme = useTheme();
  const { 
    queryState, 
    updateQueryState, 
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
  const dropZoneRef = useRef<HTMLDivElement>(null);

  // 设置Tauri文件拖拽监听
  useEffect(() => {
    let unlisten: (() => void) | undefined;

    const setupFileDrop = async () => {
      try {
        unlisten = await listen('tauri://file-drop', (event) => {
          const files = event.payload as string[];
          if (files.length > 0) {
            const filePath = files[0];
            const fileName = filePath.split(/[/\\]/).pop() || '';
            
            // 检查文件扩展名
            if (fileName.toLowerCase().endsWith('.xlsx') || fileName.toLowerCase().endsWith('.xls')) {
              // 只在不是重复选择时添加日志
              if (queryState.filePath !== filePath) {
                appendQueryLog(createLogMessage(`已选择文件：${fileName}`, 'success'));
              }
              updateQueryState({ filePath });
              showNotification({
                type: 'success',
                title: t('notifications.success.file_drag_success'),
                message: t('notifications.success.file_selected', { filename: fileName }),
              });
            } else {
              showNotification({
                type: 'warning',
                title: t('notifications.errors.file_format_unsupported'),
                message: t('notifications.errors.please_select_excel'),
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
  }, [showNotification]);

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
        // 只在不是重复选择时添加日志
        if (queryState.filePath !== selected) {
          appendQueryLog(createLogMessage(`已选择文件：${fileName}`, 'success'));
        }
        updateQueryState({ filePath: selected });
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
      
      if (queryResult.success && queryResult.data) {
        // 正确提取嵌套的数据结构
        const data = queryResult.data;
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
          recent_steps: data.recent_steps
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
        updateQueryState({ queryResult: newQueryResult });
        
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

  const handleSaveResult = async () => {
    if (!queryResult) {
      return;
    }

    try {
      // 构造保存数据
      const saveData = {
        query_info: {
          file_path: filePath,
          row_number: queryResult.target_row,
          algorithm: queryResult.algorithm,
          query_time: queryResult.query_time
        },
        result_data: queryResult
      };

      // 使用浏览器下载功能保存为JSON文件
      const dataStr = JSON.stringify(saveData, null, 2);
      const blob = new Blob([dataStr], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      
      const link = document.createElement('a');
      link.href = url;
      link.download = `time_point_query_row_${queryResult.target_row}_${getCurrentLocalTime('filename')}.json`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);

      showNotification({
        type: 'success',
        title: t('notifications.success.save_success'),
        message: t('notifications.success.save_completed'),
      });
    } catch (error) {
      console.error('Save failed:', error);
      showNotification({
        type: 'error',
        title: t('notifications.errors.save_failed'),
        message: t('notifications.errors.save_error', { error }),
      });
    }
  };

  const handleClearHistory = () => {
    clearQueryHistory();
    clearQueryLog();
    appendQueryLog(createLogMessage('查询历史和日志已清空', 'info'));
  };

  // 资金池查询处理函数
  const handleFundPoolQuery = async () => {
    if (!selectedPool || !filePath || !rowNumber || isQueryingPool) return;
    
    setIsQueryingPool(true);
    try {
      const result = await RustCommands.queryFundPool(
        selectedPool,
        filePath,
        parseInt(rowNumber),
        algorithm
      );
      
      setFundPoolResult(result);
      
      if (result.success) {
        appendQueryLog(createLogMessage(`资金池查询成功：${selectedPool}`, 'success'));
        appendQueryLog(createLogMessage(`找到 ${result.summary?.record_count || 0} 条交易记录`, 'info'));
      } else {
        appendQueryLog(createLogMessage(`资金池查询失败：${result.message || '未知错误'}`, 'error'));
      }
    } catch (error) {
      appendQueryLog(createLogMessage(`资金池查询异常：${error}`, 'error'));
      console.error('Fund pool query failed:', error);
    } finally {
      setIsQueryingPool(false);
    }
  };

  return (
    <Box sx={{ maxWidth: 1200, mx: 'auto', p: 2 }}>
      <Typography variant="h4" component="h1" gutterBottom>
        {t('query.title')}
      </Typography>
      
      <Grid container spacing={3}>
        {/* 查询配置面板 */}
        <Grid item xs={12} md={6}>
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
                // onClick={(e) => {
                //   // 检查点击目标是否为按钮或按钮内部元素
                //   const isButtonClick = (e.target as Element).closest('button') !== null;
                //   if (!isButtonClick) {
                //     handleSelectFile();
                //   }
                // }}
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

        {/* 查询结果面板 */}
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
                <Typography variant="h6">
                  {t('query.query_result')}
                </Typography>
                {queryResult && (
                  <Button
                    variant="outlined"
                    size="small"
                    startIcon={<SaveIcon />}
                    onClick={handleSaveResult}
                  >
                    {t('query.save_result')}
                  </Button>
                )}
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
                      return i18nString.includes('{') ? directString : i18nString;
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
                      
                      return i18nString.includes('{') ? directString : i18nString;
                    })()}
                  </Typography>
                  <TableContainer component={Paper} sx={{ mb: 2 }}>
                    <Table size="small">
                      <TableBody>
                        <TableRow>
                          <TableCell>{t('table.headers.row_number')}</TableCell>
                          <TableCell>{queryResult.target_row}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>{t('table.headers.timestamp')}</TableCell>
                          <TableCell>{queryResult.target_row_data?.timestamp || '--'}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>{t('table.headers.income_amount')}</TableCell>
                          <TableCell>¥{parseFloat(queryResult.target_row_data?.income_amount || 0).toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>{t('table.headers.expense_amount')}</TableCell>
                          <TableCell>¥{parseFloat(queryResult.target_row_data?.expense_amount || 0).toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>{t('table.headers.balance')}</TableCell>
                          <TableCell>¥{parseFloat(queryResult.target_row_data?.balance || 0).toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>{t('table.headers.fund_attr')}</TableCell>
                          <TableCell>{queryResult.target_row_data?.fund_attr || '--'}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>{t('table.headers.flow_type')}</TableCell>
                          <TableCell>{queryResult.target_row_data?.flow_type || '--'}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>{t('table.headers.behavior')}</TableCell>
                          <TableCell>{queryResult.target_row_data?.behavior || '--'}</TableCell>
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
                            <TableCell>{t('table.headers.personal_balance')}</TableCell>
                            <TableCell>¥{parseFloat(queryResult.tracker_state.personal_balance).toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}</TableCell>
                          </TableRow>
                        )}
                        {queryResult.tracker_state?.company_balance !== undefined && (
                          <TableRow>
                            <TableCell>{t('table.headers.company_balance')}</TableCell>
                            <TableCell>¥{parseFloat(queryResult.tracker_state.company_balance).toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}</TableCell>
                          </TableRow>
                        )}
                        {queryResult.tracker_state?.total_misappropriation !== undefined && (
                          <TableRow>
                            <TableCell>{t('table.headers.cumulative_misappropriation')}</TableCell>
                            <TableCell>¥{parseFloat(queryResult.tracker_state.total_misappropriation).toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}</TableCell>
                          </TableRow>
                        )}
                        {queryResult.tracker_state?.funding_gap !== undefined && (
                          <TableRow>
                            <TableCell>{t('table.headers.funding_gap')}</TableCell>
                            <TableCell style={{color: queryResult.tracker_state.funding_gap >= 0 ? theme.palette.error.main : theme.palette.success.main}}>
                              ¥{parseFloat(queryResult.tracker_state.funding_gap).toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}
                            </TableCell>
                          </TableRow>
                        )}
                      </TableBody>
                    </Table>
                  </TableContainer>

                  <Typography variant="subtitle2" gutterBottom>
                    {t('ui.panels.processing_stats')}
                  </Typography>
                  <TableContainer component={Paper}>
                    <Table size="small">
                      <TableBody>
                        <TableRow>
                          <TableCell>{t('table.headers.total_rows')}</TableCell>
                          <TableCell>{queryResult.total_rows}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>{t('table.headers.processed_rows')}</TableCell>
                          <TableCell>{queryResult.processing_stats?.last_processed_row}</TableCell>
                        </TableRow>
                        <TableRow>
                          <TableCell>{t('table.headers.error_count')}</TableCell>
                          <TableCell style={{color: queryResult.processing_stats?.error_count > 0 ? theme.palette.error.main : theme.palette.success.main}}>
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

        {/* 资金池查询区域 */}
        {queryResult?.available_fund_pools && queryResult.available_fund_pools.length > 0 && (
          <Grid item xs={12}>
            <Card>
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  资金池查询
                </Typography>
                
                <Box sx={{ display: 'flex', gap: 2, alignItems: 'flex-end', mb: 3 }}>
                  <FormControl sx={{ minWidth: 200 }} size="small">
                    <InputLabel>选择资金池</InputLabel>
                    <Select
                      value={selectedPool}
                      onChange={(e) => setSelectedPool(e.target.value)}
                      label="选择资金池"
                    >
                      {queryResult.available_fund_pools.map((pool: FundPool) => (
                        <MenuItem key={pool.name} value={pool.name}>
                          {pool.name} (¥{parseFloat(pool.total_amount).toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})})
                        </MenuItem>
                      ))}
                    </Select>
                  </FormControl>
                  
                  <Button
                    variant="contained"
                    onClick={handleFundPoolQuery}
                    disabled={!selectedPool || isQueryingPool}
                    startIcon={<SearchIcon />}
                    sx={{ minWidth: 120 }}
                  >
                    {isQueryingPool ? '查询中...' : '查询详情'}
                  </Button>
                </Box>
                
                {/* 资金池详情表格 */}
                {fundPoolResult?.success && fundPoolResult.records && (
                  <Box>
                    <Typography variant="subtitle2" gutterBottom>
                      {fundPoolResult.pool_name} - 详细交易记录
                    </Typography>
                    <TableContainer component={Paper} sx={{ maxHeight: 400 }}>
                      <Table size="small" stickyHeader>
                        <TableHead>
                          <TableRow>
                            <TableCell>交易时间</TableCell>
                            <TableCell>入金</TableCell>
                            <TableCell>出金</TableCell>
                            <TableCell>总余额</TableCell>
                            <TableCell>单笔资金占比</TableCell>
                            <TableCell>总资金占比</TableCell>
                          </TableRow>
                        </TableHead>
                        <TableBody>
                          {fundPoolResult.records.map((record, index) => (
                            <TableRow key={index} sx={{
                              '&:last-child': {
                                backgroundColor: theme.palette.action.hover,
                                fontWeight: 'bold'
                              }
                            }}>
                              <TableCell>{record.交易时间}</TableCell>
                              <TableCell>
                                {typeof record.入金 === 'number' 
                                  ? `¥${parseFloat(record.入金).toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}` 
                                  : record.入金}
                              </TableCell>
                              <TableCell>
                                {typeof record.出金 === 'number' 
                                  ? `¥${parseFloat(record.出金).toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}` 
                                  : record.出金}
                              </TableCell>
                              <TableCell>
                                {typeof record.总余额 === 'number' 
                                  ? `¥${parseFloat(record.总余额).toLocaleString('zh-CN', {minimumFractionDigits: 2, maximumFractionDigits: 2})}` 
                                  : record.总余额}
                              </TableCell>
                              <TableCell>{record.单笔资金占比}</TableCell>
                              <TableCell>{record.总资金占比}</TableCell>
                            </TableRow>
                          ))}
                        </TableBody>
                      </Table>
                    </TableContainer>
                  </Box>
                )}
                
                {fundPoolResult && !fundPoolResult.success && (
                  <Alert severity="error" sx={{ mt: 2 }}>
                    {fundPoolResult.message || '资金池查询失败'}
                  </Alert>
                )}
              </CardContent>
            </Card>
          </Grid>
        )}

        {/* 查询日志 */}
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