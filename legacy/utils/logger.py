"""
日志工具类
"""

import logging
import os
from datetime import datetime
from typing import Optional
from config import Config

class AuditLogger:
    """审计日志器"""
    
    def __init__(self, name: str = "audit", log_dir: str = "logs"):
        """
        初始化日志器
        
        Args:
            name: 日志器名称
            log_dir: 日志目录
        """
        self.name = name
        self.log_dir = log_dir
        
        # 创建日志目录
        os.makedirs(log_dir, exist_ok=True)
        
        # 创建日志器
        self.logger = logging.getLogger(name)
        self.logger.setLevel(logging.DEBUG)
        
        # 清除现有的处理器
        self.logger.handlers.clear()
        
        # 添加文件处理器
        self._setup_file_handlers()
        
        # 添加控制台处理器
        self._setup_console_handler()
    
    def _setup_file_handlers(self):
        """设置文件处理器"""
        # 主日志文件
        main_handler = logging.FileHandler(
            os.path.join(self.log_dir, f"{self.name}.log"),
            encoding='utf-8'
        )
        main_handler.setLevel(logging.INFO)
        
        # 详细日志文件
        detail_handler = logging.FileHandler(
            os.path.join(self.log_dir, f"{self.name}_detail.log"),
            encoding='utf-8'
        )
        detail_handler.setLevel(logging.DEBUG)
        
        # 错误日志文件
        error_handler = logging.FileHandler(
            os.path.join(self.log_dir, f"{self.name}_error.log"),
            encoding='utf-8'
        )
        error_handler.setLevel(logging.ERROR)
        
        # 设置格式化器
        formatter = logging.Formatter(
            '%(asctime)s - %(name)s - %(levelname)s - %(message)s',
            datefmt='%Y-%m-%d %H:%M:%S'
        )
        
        main_handler.setFormatter(formatter)
        detail_handler.setFormatter(formatter)
        error_handler.setFormatter(formatter)
        
        # 添加处理器
        self.logger.addHandler(main_handler)
        self.logger.addHandler(detail_handler)
        self.logger.addHandler(error_handler)
    
    def _setup_console_handler(self):
        """设置控制台处理器"""
        console_handler = logging.StreamHandler()
        console_handler.setLevel(logging.INFO)
        
        # 控制台格式化器（简化版）
        console_formatter = logging.Formatter(
            '%(levelname)s - %(message)s'
        )
        console_handler.setFormatter(console_formatter)
        
        self.logger.addHandler(console_handler)
    
    def info(self, message: str):
        """记录信息日志"""
        self.logger.info(message)
    
    def debug(self, message: str):
        """记录调试日志"""
        self.logger.debug(message)
    
    def warning(self, message: str):
        """记录警告日志"""
        self.logger.warning(message)
    
    def error(self, message: str):
        """记录错误日志"""
        self.logger.error(message)
    
    def critical(self, message: str):
        """记录严重错误日志"""
        self.logger.critical(message)
    
    def log_transaction(self, row_idx: int, transaction_type: str, amount: float, 
                       fund_attribute: str, personal_ratio: float, company_ratio: float,
                       behavior: str):
        """
        记录交易日志
        
        Args:
            row_idx: 行索引
            transaction_type: 交易类型
            amount: 金额
            fund_attribute: 资金属性
            personal_ratio: 个人占比
            company_ratio: 公司占比
            behavior: 行为性质
        """
        message = (
            f"交易处理 - 行{row_idx}: "
            f"类型={transaction_type}, "
            f"金额={amount:,.2f}, "
            f"属性={fund_attribute}, "
            f"个人占比={personal_ratio:.2%}, "
            f"公司占比={company_ratio:.2%}, "
            f"行为={behavior}"
        )
        self.info(message)
    
    def log_balance_check(self, row_idx: int, expected: float, actual: float, 
                         difference: float):
        """
        记录余额检查日志
        
        Args:
            row_idx: 行索引
            expected: 期望余额
            actual: 实际余额
            difference: 差异
        """
        if abs(difference) > Config.BALANCE_TOLERANCE:
            message = (
                f"余额不匹配 - 行{row_idx}: "
                f"期望={expected:,.2f}, "
                f"实际={actual:,.2f}, "
                f"差异={difference:,.2f}"
            )
            self.warning(message)
        else:
            message = f"余额检查通过 - 行{row_idx}: 差异={difference:,.2f}"
            self.debug(message)
    
    def log_investment_product(self, product_id: str, action: str, amount: float,
                              personal_amount: float, company_amount: float):
        """
        记录投资产品操作日志
        
        Args:
            product_id: 产品ID
            action: 操作类型
            amount: 总金额
            personal_amount: 个人金额
            company_amount: 公司金额
        """
        message = (
            f"投资产品操作 - {product_id}: "
            f"操作={action}, "
            f"总金额={amount:,.2f}, "
            f"个人金额={personal_amount:,.2f}, "
            f"公司金额={company_amount:,.2f}"
        )
        self.info(message)
    
    def log_error(self, error: Exception, context: str = ""):
        """
        记录错误日志
        
        Args:
            error: 异常对象
            context: 错误上下文
        """
        message = f"错误 - {context}: {str(error)}"
        self.error(message)
        
        # 记录详细错误信息
        import traceback
        self.logger.debug(f"详细错误信息:\n{traceback.format_exc()}")
    
    def log_performance(self, operation: str, duration: float, rows_processed: int = 0):
        """
        记录性能日志
        
        Args:
            operation: 操作名称
            duration: 耗时（秒）
            rows_processed: 处理行数
        """
        if rows_processed > 0:
            rate = rows_processed / duration
            message = (
                f"性能统计 - {operation}: "
                f"耗时={duration:.2f}秒, "
                f"处理行数={rows_processed}, "
                f"处理速度={rate:.2f}行/秒"
            )
        else:
            message = f"性能统计 - {operation}: 耗时={duration:.2f}秒"
        
        self.info(message)

# 全局日志器实例
audit_logger = AuditLogger() 