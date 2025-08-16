import React, { Component, ReactNode } from 'react';
import { Box, Typography, Alert, Button } from '@mui/material';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error?: Error;
  errorInfo?: any;
}

class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): State {
    // 更新 state 使下一次渲染能够显示降级后的 UI
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: any) {
    // 你同样可以将错误日志上报给服务器
    console.error('ErrorBoundary caught an error:', error, errorInfo);
    this.setState({
      error,
      errorInfo
    });
  }

  handleReload = () => {
    this.setState({ hasError: false, error: undefined, errorInfo: undefined });
  };

  render() {
    if (this.state.hasError) {
      // 你可以自定义降级后的 UI 并渲染
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <Box 
          sx={{ 
            maxWidth: 800, 
            mx: 'auto', 
            p: 4,
            display: 'flex',
            flexDirection: 'column',
            alignItems: 'center',
            justifyContent: 'center',
            minHeight: '50vh'
          }}
        >
          <Alert severity="error" sx={{ width: '100%', mb: 3 }}>
            <Typography variant="h6" gutterBottom>
              页面加载出现问题
            </Typography>
            <Typography variant="body2" color="text.secondary" paragraph>
              页面渲染时发生了意外错误。这可能是临时问题，请尝试刷新页面。
            </Typography>
            {this.state.error && (
              <Typography variant="body2" sx={{ fontFamily: 'monospace', mt: 2 }}>
                {this.state.error.message}
              </Typography>
            )}
          </Alert>
          
          <Box sx={{ display: 'flex', gap: 2 }}>
            <Button variant="contained" onClick={this.handleReload}>
              重新加载
            </Button>
            <Button variant="outlined" onClick={() => window.location.reload()}>
              刷新页面
            </Button>
          </Box>
        </Box>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;