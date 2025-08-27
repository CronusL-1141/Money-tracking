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
} from '@mui/icons-material';
import { useTheme } from '@mui/material/styles';
import { useTranslation } from 'react-i18next';
import { AnalysisHistoryRecord } from '../types/analysisHistory';
import { AnalysisHistoryManager } from '../utils/analysisHistoryManager';
import { formatLocalTime } from '../utils/timeUtils';

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
  const { t } = useTranslation();
  const [records, setRecords] = useState<AnalysisHistoryRecord[]>([]);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [recordToDelete, setRecordToDelete] = useState<AnalysisHistoryRecord | null>(null);
  const [operationLoading, setOperationLoading] = useState<string | null>(null);

  // 加载历史记录
  const loadHistory = () => {
    console.log('正在加载历史记录...');
    const history = AnalysisHistoryManager.getHistory();
    console.log('加载到的历史记录:', history);
    console.log('记录数量:', history.records.length);
    setRecords(history.records);
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

  // 处理删除记录
  const handleDeleteRecord = async (record: AnalysisHistoryRecord) => {
    setRecordToDelete(record);
    setDeleteDialogOpen(true);
  };

  const confirmDelete = async () => {
    if (!recordToDelete) return;

    setOperationLoading(`delete_${recordToDelete.id}`);
    try {
      const success = await AnalysisHistoryManager.deleteRecord(recordToDelete.id);
      if (success) {
        loadHistory(); // 重新加载历史记录
        setDeleteDialogOpen(false);
        setRecordToDelete(null);
      } else {
        // TODO: 显示错误通知
        console.error('删除历史记录失败');
      }
    } catch (error) {
      console.error('删除历史记录出错:', error);
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
        // TODO: 显示错误通知
        console.error('打开分析结果失败');
      }
    } catch (error) {
      console.error('打开分析结果出错:', error);
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
        console.error('打开场外资金池记录失败');
      }
    } catch (error) {
      console.error('打开场外资金池记录出错:', error);
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
        console.log('另存为操作取消或失败');
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
            label="成功"
            color="success"
            size="small"
            variant="outlined"
          />
        );
      case 'failed':
        return (
          <Chip
            icon={<Error />}
            label="失败"
            color="error"
            size="small"
            variant="outlined"
          />
        );
      case 'processing':
        return (
          <Chip
            icon={<Speed />}
            label="处理中"
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
      `${stats.totalRecords.toLocaleString()}条记录`,
      `${AnalysisHistoryManager.formatProcessingTime(stats.processingTime)}`,
      stats.validationFixes > 0 ? `修复${stats.validationFixes}处错误` : null,
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
            分析历史记录 ({records.length})
          </Typography>
        </Box>
        <IconButton size="small">
          {expanded ? <ExpandLess /> : <ExpandMore />}
        </IconButton>
      </Box>

      {/* 历史记录内容 */}
      <Collapse in={expanded}>
        <Box sx={{ mt: 1 }}>
          {records.length === 0 ? (
            <Alert severity="info" sx={{ mt: 1 }}>
              还没有分析历史记录。完成第一次分析后，记录将显示在这里。
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
                              {record.algorithmDisplayName}
                            </Typography>
                            {getStatusChip(record)}
                          </Box>
                          <Typography variant="body2" color="text.secondary" gutterBottom>
                            输入文件: {record.inputFile.name} ({AnalysisHistoryManager.formatFileSize(record.inputFile.size)})
                          </Typography>
                          <Typography variant="body2" color="text.secondary" gutterBottom>
                            主分析结果: {record.outputFile.name}
                          </Typography>
                          {record.offsitePoolFile && (
                            <Typography variant="body2" color="text.secondary" gutterBottom>
                              配套场外资金池记录: {record.offsitePoolFile.name}
                            </Typography>
                          )}
                          <Typography variant="body2" color="text.secondary" gutterBottom>
                            {formatStatistics(record)}
                          </Typography>
                          <Typography variant="caption" color="text.secondary">
                            {formatLocalTime(record.timestamp, 'display')}
                          </Typography>
                        </Box>

                        {/* 操作按钮 */}
                        <Box sx={{ display: 'flex', gap: 0.5 }}>
                          <Tooltip title="打开分析结果">
                            <IconButton
                              size="small"
                              onClick={() => handleOpenRecord(record)}
                              disabled={isAnalyzing || operationLoading === `open_${record.id}` || record.status !== 'success'}
                            >
                              <FolderOpen fontSize="small" />
                            </IconButton>
                          </Tooltip>
                          <Tooltip title="另存为">
                            <IconButton
                              size="small"
                              onClick={() => handleSaveAsRecord(record)}
                              disabled={isAnalyzing || operationLoading === `saveas_${record.id}` || record.status !== 'success'}
                            >
                              <SaveAlt fontSize="small" />
                            </IconButton>
                          </Tooltip>
                          {record.offsitePoolFile && (
                            <Tooltip title="打开场外资金池记录">
                              <IconButton
                                size="small"
                                onClick={() => handleOpenOffsitePoolRecord(record)}
                                disabled={isAnalyzing || operationLoading === `open_pool_${record.id}` || record.status !== 'success'}
                                color="secondary"
                              >
                                <TableChart fontSize="small" />
                              </IconButton>
                            </Tooltip>
                          )}
                          <Tooltip title="删除记录">
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
        <DialogTitle>删除分析记录</DialogTitle>
        <DialogContent>
          <DialogContentText>
            确定要删除这个分析记录吗？这将同时删除相关的结果文件。此操作无法撤销。
          </DialogContentText>
          {recordToDelete && (
            <Box sx={{ mt: 2, p: 2, bgcolor: 'background.default', borderRadius: 1 }}>
              <Typography variant="body2" color="text.secondary">
                <strong>算法:</strong> {recordToDelete.algorithmDisplayName}
              </Typography>
              <Typography variant="body2" color="text.secondary">
                <strong>文件:</strong> {recordToDelete.inputFile.name}
              </Typography>
              <Typography variant="body2" color="text.secondary">
                <strong>时间:</strong> {formatLocalTime(recordToDelete.timestamp, 'display')}
              </Typography>
            </Box>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeleteDialogOpen(false)}>取消</Button>
          <Button
            onClick={confirmDelete}
            color="error"
            variant="contained"
            disabled={operationLoading !== null}
          >
            删除
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
};

export default AnalysisHistoryPanel;