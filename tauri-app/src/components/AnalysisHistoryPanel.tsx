/**
 * 分析历史记录面板组件
 */
import React, { useState, useEffect } from 'react';
import {
  Box,
  Typography,
  Card,
  CardContent,
  List,
  ListItem,
  ListItemText,
  ListItemSecondaryAction,
  IconButton,
  Chip,
  Divider,
  Button,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  DialogContentText,
  Tooltip,
  Alert,
  Collapse,
} from '@mui/material';
import {
  FolderOpen,
  SaveAlt,
  Delete,
  ExpandMore,
  ExpandLess,
  History,
  CheckCircle,
  Error,
  Speed,
  Assessment,
  TableChart,
  Refresh,
} from '@mui/icons-material';
import { useTheme } from '@mui/material/styles';
import { useTranslation } from 'react-i18next';
import { AnalysisHistoryRecord } from '../types/analysisHistory';
import { AnalysisHistoryManager } from '../utils/analysisHistoryManager';

interface AnalysisHistoryPanelProps {
  /** 是否显示历史记录面板 */
  expanded?: boolean;
  /** 展开状态改变回调 */
  onExpandedChange?: (expanded: boolean) => void;
  /** 当前分析状态，用于禁用操作 */
  isAnalyzing?: boolean;
  /** 刷新触发器，变化时重新加载历史记录 */
  refreshTrigger?: number;
}

export const AnalysisHistoryPanel: React.FC<AnalysisHistoryPanelProps> = ({
  expanded = false,
  onExpandedChange,
  isAnalyzing = false,
  refreshTrigger = 0
}) => {
  const theme = useTheme();
  const { t, i18n } = useTranslation();
  
  // 格式化时间显示，支持多语言
  const formatDisplayTime = (date: Date) => {
    const locale = i18n.language === 'zh' ? 'zh-CN' : 'en-US';
    return date.toLocaleString(locale, {
      year: 'numeric',
      month: i18n.language === 'zh' ? 'long' : 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
      hour12: i18n.language === 'zh' ? false : true
    });
  };
  const [records, setRecords] = useState<AnalysisHistoryRecord[]>([]);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [recordToDelete, setRecordToDelete] = useState<AnalysisHistoryRecord | null>(null);
  const [operationLoading, setOperationLoading] = useState<string | null>(null);
  const [isRefreshing, setIsRefreshing] = useState(false);

  // 加载历史记录
  const loadHistory = async () => {
    console.log('正在加载历史记录并检测文件状态...');
    try {
      const history = await AnalysisHistoryManager.getHistoryWithRealTimeStatus();
      console.log('加载到的历史记录:', history);
      console.log('记录数量:', history.records.length);
      setRecords(history.records);
    } catch (error) {
      console.error('加载历史记录时出错:', error);
      // 降级到普通加载方式
      const history = AnalysisHistoryManager.getHistory();
      setRecords(history.records);
    }
  };

  useEffect(() => {
    loadHistory();
  }, []);

  // 当刷新触发器变化时重新加载历史记录
  useEffect(() => {
    if (refreshTrigger > 0) {
      console.log('刷新触发器变化，重新加载历史记录:', refreshTrigger);
      loadHistory();
    }
  }, [refreshTrigger]);

  // 手动刷新文件状态
  const handleManualRefresh = async (event: React.MouseEvent) => {
    event.stopPropagation(); // 阻止触发展开/收起
    
    if (isRefreshing) return;
    
    setIsRefreshing(true);
    try {
      console.log('手动刷新文件状态...');
      const syncResult = await AnalysisHistoryManager.syncAllRecordsFileStatus();
      
      // 重新加载历史记录以显示最新状态
      await loadHistory();
      
      // 显示刷新结果通知
      if (syncResult.totalUpdated > 0) {
        showNotification?.({
          type: 'success',
          title: t('analysis_history.status_updated'),
          message: t('analysis_history.status_check_completed', { 
            totalChecked: syncResult.totalChecked, 
            totalUpdated: syncResult.totalUpdated 
          })
        });
      } else {
        showNotification?.({
          type: 'info',
          title: t('analysis_history.status_normal'),
          message: t('analysis_history.all_files_current', { 
            totalChecked: syncResult.totalChecked 
          })
        });
      }
      
      if (syncResult.errors.length > 0) {
        console.warn('File status sync errors occurred:', syncResult.errors);
      }
    } catch (error) {
      console.error('Manual file status refresh failed:', error);
      showNotification?.({
        type: 'error',
        title: t('analysis_history.refresh_failed'),
        message: t('analysis_history.refresh_error')
      });
    } finally {
      setIsRefreshing(false);
    }
  };

  // 处理删除记录
  const handleDeleteRecord = async (record: AnalysisHistoryRecord) => {
    setRecordToDelete(record);
    setDeleteDialogOpen(true);
  };

  const confirmDelete = async () => {
    if (!recordToDelete) return;

    setOperationLoading(`delete_${recordToDelete.id}`);
    try {
      const result = await AnalysisHistoryManager.deleteRecord(recordToDelete.id);
      
      if (result.success) {
        loadHistory(); // 重新加载历史记录
        setDeleteDialogOpen(false);
        setRecordToDelete(null);
        
        if (result.allDeleted) {
          showNotification?.({ 
            type: 'success', 
            title: t('analysis_history.delete_success'),
            message: t('analysis_history.delete_success_message')
          });
        } else if (result.partiallyDeleted) {
          showNotification?.({ 
            type: 'warning', 
            title: t('analysis_history.partial_delete_success'),
            message: t('analysis_history.partial_delete_message', { errors: result.errors.join(', ') })
          });
        }
      } else {
        showNotification?.({ 
          type: 'error', 
          title: t('analysis_history.delete_failed'),
          message: t('analysis_history.delete_failed_message', { errors: result.errors.join(', ') })
        });
      }
    } catch (error) {
      console.error('删除历史记录出错:', error);
      showNotification?.({ 
        type: 'error', 
        title: t('analysis_history.operation_failed'),
        message: t('analysis_history.delete_error')
      });
    } finally {
      setOperationLoading(null);
    }
  };

  // 处理打开记录
  const handleOpenRecord = async (record: AnalysisHistoryRecord) => {
    setOperationLoading(`open_${record.id}`);
    try {
      const success = await AnalysisHistoryManager.openRecord(record);
      if (!success) {
        showNotification?.({ 
          type: 'error', 
          title: t('analysis_history.open_failed'),
          message: t('analysis_history.open_main_failed')
        });
      }
    } catch (error) {
      console.error('打开分析结果出错:', error);
      showNotification?.({ 
        type: 'error', 
        title: t('analysis_history.operation_failed'),
        message: t('analysis_history.file_operation_error')
      });
    } finally {
      setOperationLoading(null);
    }
  };

  // 处理打开场外资金池记录
  const handleOpenOffsitePoolRecord = async (record: AnalysisHistoryRecord) => {
    setOperationLoading(`open_pool_${record.id}`);
    try {
      const success = await AnalysisHistoryManager.openOffsitePoolRecord(record);
      if (!success) {
        showNotification?.({ 
          type: 'error', 
          title: t('analysis_history.open_failed'),
          message: t('analysis_history.open_pool_failed')
        });
      }
    } catch (error) {
      console.error('打开场外资金池记录出错:', error);
      showNotification?.({ 
        type: 'error', 
        title: t('analysis_history.operation_failed'),
        message: t('analysis_history.file_operation_error')
      });
    } finally {
      setOperationLoading(null);
    }
  };

  // 处理另存为
  const handleSaveAsRecord = async (record: AnalysisHistoryRecord) => {
    setOperationLoading(`saveas_${record.id}`);
    try {
      const success = await AnalysisHistoryManager.saveAsRecord(record);
      if (!success) {
        // TODO: 显示错误通知 (可能是用户取消了)
        console.log(t('analysis_history.save_as_failed'));
      }
    } catch (error) {
      console.error('另存为分析结果出错:', error);
    } finally {
      setOperationLoading(null);
    }
  };

  // 获取状态芯片
  const getStatusChip = (record: AnalysisHistoryRecord) => {
    switch (record.status) {
      case 'success':
        return (
          <Chip
            icon={<CheckCircle />}
            label={t('analysis_history.status.success')}
            color="success"
            size="small"
            variant="outlined"
          />
        );
      case 'failed':
        return (
          <Chip
            icon={<Error />}
            label={t('analysis_history.status.failed')}
            color="error"
            size="small"
            variant="outlined"
          />
        );
      case 'processing':
        return (
          <Chip
            icon={<Speed />}
            label={t('analysis_history.status.processing')}
            color="primary"
            size="small"
            variant="outlined"
          />
        );
      default:
        return null;
    }
  };

  // 格式化统计信息
  const formatStatistics = (record: AnalysisHistoryRecord) => {
    const stats = record.statistics;
    return [
      t('analysis_history.stats.records_count', { count: stats.totalRecords.toLocaleString() }),
      `${AnalysisHistoryManager.formatProcessingTime(stats.processingTime)}`,
      stats.validationFixes > 0 ? t('analysis_history.stats.validation_fixes', { count: stats.validationFixes }) : null,
    ].filter(Boolean).join(' · ');
  };

  return (
    <Box>
      {/* 历史记录标题栏 */}
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          py: 2,
          cursor: 'pointer',
          '&:hover': {
            bgcolor: theme.palette.action.hover,
          },
          borderRadius: 1,
          px: 2,
        }}
        onClick={() => onExpandedChange?.(!expanded)}
      >
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <History />
          <Typography variant="h6">
            {t('analysis_history.title')} ({records.length})
          </Typography>
        </Box>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
          <Tooltip title={t('analysis_history.refresh_status')}>
            <IconButton 
              size="small" 
              onClick={handleManualRefresh}
              disabled={isRefreshing}
              sx={{
                opacity: isRefreshing ? 0.6 : 1,
                transform: isRefreshing ? 'rotate(360deg)' : 'none',
                transition: 'transform 0.5s ease-in-out',
              }}
            >
              <Refresh fontSize="small" />
            </IconButton>
          </Tooltip>
          <IconButton size="small">
            {expanded ? <ExpandLess /> : <ExpandMore />}
          </IconButton>
        </Box>
      </Box>

      {/* 历史记录内容 */}
      <Collapse in={expanded}>
        <Box sx={{ mt: 1 }}>
          {records.length === 0 ? (
            <Alert severity="info" sx={{ mt: 1 }}>
              {t('analysis_history.no_records')}
            </Alert>
          ) : (
            <Card variant="outlined">
              <List>
                {records.map((record, index) => (
                  <React.Fragment key={record.id}>
                    <ListItem
                      sx={{
                        flexDirection: 'column',
                        alignItems: 'stretch',
                        py: 2,
                      }}
                    >
                      {/* 记录主要信息 */}
                      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', width: '100%', mb: 1 }}>
                        <Box sx={{ flex: 1 }}>
                          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 0.5 }}>
                            <Assessment color="primary" fontSize="small" />
                            <Typography variant="subtitle1" sx={{ fontWeight: 500 }}>
                              {t(`algorithms.${record.algorithm}`)}
                            </Typography>
                            {getStatusChip(record)}
                          </Box>
                          <Typography variant="body2" color="text.secondary" gutterBottom>
                            {t('analysis_history.file_info.input_file')}: {record.inputFile.name} ({AnalysisHistoryManager.formatFileSize(record.inputFile.size)})
                          </Typography>
                          <Typography 
                            variant="body2" 
                            color="text.secondary" 
                            gutterBottom
                            sx={{
                              textDecoration: record.outputFile.deleted ? 'line-through' : 'none',
                              opacity: record.outputFile.deleted ? 0.6 : 1
                            }}
                          >
                            {t('analysis_history.file_info.main_result')}: {record.outputFile.name}
                            {record.outputFile.deleted && t('analysis_history.file_info.file_deleted')}
                            {record.outputFile.deleteError && (
                              <span style={{ color: theme.palette.error.main, marginLeft: 8 }}>
                                {t('analysis_history.file_info.delete_failed', { error: record.outputFile.deleteError })}
                              </span>
                            )}
                          </Typography>
                          {record.offsitePoolFile && (
                            <Typography 
                              variant="body2" 
                              color="text.secondary" 
                              gutterBottom
                              sx={{
                                textDecoration: record.offsitePoolFile.deleted ? 'line-through' : 'none',
                                opacity: record.offsitePoolFile.deleted ? 0.6 : 1
                              }}
                            >
                              {t('analysis_history.file_info.pool_record')}: {record.offsitePoolFile.name}
                              {record.offsitePoolFile.deleted && t('analysis_history.file_info.file_deleted')}
                              {record.offsitePoolFile.deleteError && (
                                <span style={{ color: theme.palette.error.main, marginLeft: 8 }}>
                                  {t('analysis_history.file_info.delete_failed', { error: record.offsitePoolFile.deleteError })}
                                </span>
                              )}
                            </Typography>
                          )}
                          <Typography variant="body2" color="text.secondary" gutterBottom>
                            {formatStatistics(record)}
                          </Typography>
                          <Typography variant="caption" color="text.secondary">
                            {formatDisplayTime(record.timestamp)}
                          </Typography>
                        </Box>

                        {/* 操作按钮 */}
                        <Box sx={{ display: 'flex', gap: 0.5 }}>
                          <Tooltip title={
                            record.outputFile.deleted 
                              ? t('analysis_history.tooltips.open_main_deleted')
                              : record.outputFile.deleteError 
                                ? t('analysis_history.tooltips.open_main_error', { error: record.outputFile.deleteError })
                                : t('analysis_history.tooltips.open_main')
                          }>
                            <span>
                              <IconButton
                                size="small"
                                onClick={() => handleOpenRecord(record)}
                                disabled={isAnalyzing || operationLoading === `open_${record.id}` || record.status !== 'success' || record.outputFile.deleted}
                                sx={{
                                  opacity: record.outputFile.deleted ? 0.4 : 1,
                                  '&.Mui-disabled': {
                                    opacity: record.outputFile.deleted ? 0.4 : 0.6
                                  }
                                }}
                              >
                                <FolderOpen fontSize="small" />
                              </IconButton>
                            </span>
                          </Tooltip>
                          <Tooltip title={t('analysis_history.tooltips.save_as')}>
                            <IconButton
                              size="small"
                              onClick={() => handleSaveAsRecord(record)}
                              disabled={isAnalyzing || operationLoading === `saveas_${record.id}` || record.status !== 'success'}
                            >
                              <SaveAlt fontSize="small" />
                            </IconButton>
                          </Tooltip>
                          {record.offsitePoolFile && (
                            <Tooltip title={
                              !record.offsitePoolFile 
                                ? t('analysis_history.file_info.no_pool_file')
                                : record.offsitePoolFile.deleted 
                                  ? t('analysis_history.tooltips.open_pool_deleted')
                                  : record.offsitePoolFile.deleteError 
                                    ? t('analysis_history.tooltips.open_pool_error', { error: record.offsitePoolFile.deleteError })
                                    : t('analysis_history.tooltips.open_pool')
                            }>
                              <span>
                                <IconButton
                                  size="small"
                                  onClick={() => handleOpenOffsitePoolRecord(record)}
                                  disabled={isAnalyzing || operationLoading === `open_pool_${record.id}` || record.status !== 'success' || !record.offsitePoolFile || record.offsitePoolFile.deleted}
                                  color="secondary"
                                  sx={{
                                    opacity: (!record.offsitePoolFile || record.offsitePoolFile.deleted) ? 0.4 : 1,
                                    '&.Mui-disabled': {
                                      opacity: (!record.offsitePoolFile || record.offsitePoolFile.deleted) ? 0.4 : 0.6
                                    }
                                  }}
                                >
                                  <TableChart fontSize="small" />
                                </IconButton>
                              </span>
                            </Tooltip>
                          )}
                          <Tooltip title={t('analysis_history.tooltips.delete_record')}>
                            <IconButton
                              size="small"
                              onClick={() => handleDeleteRecord(record)}
                              disabled={isAnalyzing || operationLoading === `delete_${record.id}`}
                              color="error"
                            >
                              <Delete fontSize="small" />
                            </IconButton>
                          </Tooltip>
                        </Box>
                      </Box>

                      {/* 错误信息（如果有） */}
                      {record.status === 'failed' && record.error && (
                        <Alert severity="error" size="small" sx={{ mt: 1 }}>
                          {record.error}
                        </Alert>
                      )}
                    </ListItem>
                    {index < records.length - 1 && <Divider />}
                  </React.Fragment>
                ))}
              </List>
            </Card>
          )}
        </Box>
      </Collapse>

      {/* 删除确认对话框 */}
      <Dialog
        open={deleteDialogOpen}
        onClose={() => setDeleteDialogOpen(false)}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>{t('analysis_history.delete_dialog.title')}</DialogTitle>
        <DialogContent>
          <DialogContentText>
            {t('analysis_history.delete_dialog.confirm_message')}
          </DialogContentText>
          {recordToDelete && (
            <Box sx={{ mt: 1, mb: 2 }}>
              <Typography variant="body2" sx={{ fontWeight: 500, mb: 1 }}>
                {t('analysis_history.delete_dialog.files_to_delete')}
              </Typography>
              <Typography variant="body2" color="text.secondary" sx={{ ml: 2, mb: 0.5 }}>
                • {t('analysis_history.delete_dialog.main_file')}: {recordToDelete.outputFile.name}
              </Typography>
              {recordToDelete.offsitePoolFile && (
                <Typography variant="body2" color="text.secondary" sx={{ ml: 2, mb: 0.5 }}>
                  • {t('analysis_history.delete_dialog.pool_file')}: {recordToDelete.offsitePoolFile.name}
                </Typography>
              )}
            </Box>
          )}
          <DialogContentText sx={{ color: 'warning.main', fontStyle: 'italic' }}>
            {t('analysis_history.delete_dialog.warning')}
          </DialogContentText>
          {recordToDelete && (
            <Box sx={{ mt: 2, p: 2, bgcolor: 'background.default', borderRadius: 1 }}>
              <Typography variant="body2" color="text.secondary">
                <strong>{t('analysis_history.delete_dialog.record_info.algorithm')}:</strong> {t(`algorithms.${recordToDelete.algorithm}`)}
              </Typography>
              <Typography variant="body2" color="text.secondary">
                <strong>{t('analysis_history.delete_dialog.record_info.file')}:</strong> {recordToDelete.inputFile.name}
              </Typography>
              <Typography variant="body2" color="text.secondary">
                <strong>{t('analysis_history.delete_dialog.record_info.time')}:</strong> {formatDisplayTime(recordToDelete.timestamp)}
              </Typography>
            </Box>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeleteDialogOpen(false)}>{t('analysis_history.delete_dialog.cancel')}</Button>
          <Button
            onClick={confirmDelete}
            color="error"
            variant="contained"
            disabled={operationLoading !== null}
          >
            {t('analysis_history.delete_dialog.confirm')}
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default AnalysisHistoryPanel;