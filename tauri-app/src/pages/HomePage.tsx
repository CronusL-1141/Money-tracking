import React from 'react';
import {
  Box,
  Typography,
  Card,
  CardContent,
  Grid,
  Button,
  Avatar,
  List,
  ListItem,
  ListItemAvatar,
  ListItemText,
} from '@mui/material';
import {
  Assessment as AssessmentIcon,
  Search as SearchIcon,
  TrendingUp as TrendingUpIcon,
  Speed as SpeedIcon,
} from '@mui/icons-material';
import { useTranslation } from 'react-i18next';
import { useNavigate } from 'react-router-dom';

const HomePage: React.FC = () => {
  const { t } = useTranslation();
  const navigate = useNavigate();

  const features = [
    {
      icon: <AssessmentIcon color="primary" />,
      titleKey: 'analysis.title',
      descriptionKey: 'analysis.algorithm_description.fifo',
      action: () => navigate('/audit'),
    },
    {
      icon: <SearchIcon color="primary" />,
      titleKey: 'query.title',
      descriptionKey: 'query.query_result',
      action: () => navigate('/query'),
    },
  ];

  const algorithms = [
    {
      name: 'FIFO',
      description: t('analysis.algorithm_description.fifo'),
      icon: <TrendingUpIcon />,
    },
    {
      name: t('analysis.balance_method'),
      description: t('analysis.algorithm_description.balance_method'),
      icon: <SpeedIcon />,
    },
  ];

  return (
    <Box sx={{ maxWidth: 1200, mx: 'auto' }}>
      {/* 欢迎标题 */}
      <Box sx={{ mb: 4, textAlign: 'center' }}>
        <Typography variant="h3" component="h1" gutterBottom>
          {t('app.title')}
        </Typography>
        <Typography variant="h6" color="text.secondary">
          {t('app.subtitle')}
        </Typography>
      </Box>

      {/* 快速操作卡片 */}
      <Grid container spacing={3} sx={{ mb: 4 }}>
        {features.map((feature, index) => (
          <Grid item xs={12} md={6} key={index}>
            <Card 
              sx={{ 
                height: '100%',
                transition: 'transform 0.2s, box-shadow 0.2s',
                '&:hover': {
                  transform: 'translateY(-4px)',
                  boxShadow: (theme) => theme.shadows[8],
                  cursor: 'pointer',
                },
              }}
              onClick={feature.action}
            >
              <CardContent sx={{ p: 3 }}>
                <Box sx={{ display: 'flex', alignItems: 'center', mb: 2 }}>
                  <Avatar sx={{ bgcolor: 'primary.light', mr: 2 }}>
                    {feature.icon}
                  </Avatar>
                  <Typography variant="h5" component="h2">
                    {t(feature.titleKey)}
                  </Typography>
                </Box>
                <Typography variant="body1" color="text.secondary">
                  {t(feature.descriptionKey)}
                </Typography>
                <Box sx={{ mt: 2, display: 'flex', justifyContent: 'flex-end' }}>
                  <Button variant="contained" size="small">
                    {t('common.next')}
                  </Button>
                </Box>
              </CardContent>
            </Card>
          </Grid>
        ))}
      </Grid>

      {/* 算法介绍 */}
      <Card sx={{ mb: 4 }}>
        <CardContent sx={{ p: 3 }}>
          <Typography variant="h5" component="h2" gutterBottom>
            {t('audit.algorithm')}
          </Typography>
          <Typography variant="body1" color="text.secondary" sx={{ mb: 3 }}>
            系统支持两种核心算法，您可以根据审计需求选择合适的分析方法
          </Typography>
          
          <List>
            {algorithms.map((algorithm, index) => (
              <ListItem key={index} sx={{ px: 0 }}>
                <ListItemAvatar>
                  <Avatar sx={{ bgcolor: 'primary.light' }}>
                    {algorithm.icon}
                  </Avatar>
                </ListItemAvatar>
                <ListItemText
                  primary={
                    <Typography variant="h6" component="div">
                      {algorithm.name}
                    </Typography>
                  }
                  secondary={
                    <Typography variant="body2" color="text.secondary">
                      {algorithm.description}
                    </Typography>
                  }
                />
              </ListItem>
            ))}
          </List>
        </CardContent>
      </Card>

      {/* 系统特性 */}
      <Card>
        <CardContent sx={{ p: 3 }}>
          <Typography variant="h5" component="h2" gutterBottom>
            系统特性
          </Typography>
          <Grid container spacing={2}>
            <Grid item xs={12} sm={6} md={3}>
              <Box sx={{ textAlign: 'center' }}>
                <Typography variant="h4" color="primary" gutterBottom>
                  50万+
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  最大数据处理量
                </Typography>
              </Box>
            </Grid>
            <Grid item xs={12} sm={6} md={3}>
              <Box sx={{ textAlign: 'center' }}>
                <Typography variant="h4" color="primary" gutterBottom>
                  双算法
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  FIFO + 差额计算
                </Typography>
              </Box>
            </Grid>
            <Grid item xs={12} sm={6} md={3}>
              <Box sx={{ textAlign: 'center' }}>
                <Typography variant="h4" color="primary" gutterBottom>
                  离线
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  完全本地化处理
                </Typography>
              </Box>
            </Grid>
            <Grid item xs={12} sm={6} md={3}>
              <Box sx={{ textAlign: 'center' }}>
                <Typography variant="h4" color="primary" gutterBottom>
                  专业
                </Typography>
                <Typography variant="body2" color="text.secondary">
                  审计级别报告
                </Typography>
              </Box>
            </Grid>
          </Grid>
        </CardContent>
      </Card>
    </Box>
  );
};

export default HomePage;