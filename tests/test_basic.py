"""
基本单元测试
"""

import unittest
import sys
import os

# 添加项目根目录到路径
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from config import Config
from utils.validators import DataValidator, InvestmentProductValidator
from utils.logger import AuditLogger

class TestConfig(unittest.TestCase):
    """测试配置类"""
    
    def test_format_number(self):
        """测试数值格式化"""
        # 测试正常数值
        self.assertEqual(Config.format_number(123.456), 123.46)
        self.assertEqual(Config.format_number(0), 0.0)
        
        # 测试极小值
        self.assertEqual(Config.format_number(1e-10), 0.0)
        self.assertEqual(Config.format_number(-1e-10), 0.0)
        
        # 测试负数
        self.assertEqual(Config.format_number(-123.456), -123.46)
    
    def test_is_investment_product(self):
        """测试投资产品判断"""
        # 测试投资产品
        self.assertTrue(Config.is_investment_product("理财-001"))
        self.assertTrue(Config.is_investment_product("投资-002"))
        self.assertTrue(Config.is_investment_product("保险-003"))
        self.assertTrue(Config.is_investment_product("关联银行卡-004"))
        
        # 测试非投资产品
        self.assertFalse(Config.is_investment_product("个人应收"))
        self.assertFalse(Config.is_investment_product("公司应付"))
        self.assertFalse(Config.is_investment_product(""))
        self.assertFalse(Config.is_investment_product(None))
    
    def test_is_personal_fund(self):
        """测试个人资金判断"""
        # 测试个人资金
        self.assertTrue(Config.is_personal_fund("个人应收"))
        self.assertTrue(Config.is_personal_fund("个人应付"))
        self.assertTrue(Config.is_personal_fund("个人资金"))
        
        # 测试非个人资金
        self.assertFalse(Config.is_personal_fund("公司应收"))
        self.assertFalse(Config.is_personal_fund("公司应付"))
        self.assertFalse(Config.is_personal_fund(""))
    
    def test_is_company_fund(self):
        """测试公司资金判断"""
        # 测试公司资金
        self.assertTrue(Config.is_company_fund("公司应收"))
        self.assertTrue(Config.is_company_fund("公司应付"))
        self.assertTrue(Config.is_company_fund("公司资金"))
        
        # 测试非公司资金
        self.assertFalse(Config.is_company_fund("个人应收"))
        self.assertFalse(Config.is_company_fund("个人应付"))
        self.assertFalse(Config.is_company_fund(""))

class TestDataValidator(unittest.TestCase):
    """测试数据验证器"""
    
    def setUp(self):
        """测试前准备"""
        import pandas as pd
        
        # 创建测试数据
        self.valid_row = pd.Series({
            '交易日期': '2023-01-01',
            '交易收入金额': 1000.0,
            '交易支出金额': 0.0,
            '资金属性': '个人应收'
        })
        
        self.invalid_row = pd.Series({
            '交易日期': 'invalid_date',
            '交易收入金额': -100.0,  # 负数
            '交易支出金额': 'invalid_amount',  # 非数字
            '资金属性': None  # 空值
        })
    
    def test_validate_transaction_data_valid(self):
        """测试有效交易数据验证"""
        errors = DataValidator.validate_transaction_data(self.valid_row)
        self.assertEqual(len(errors), 0)
    
    def test_validate_transaction_data_invalid(self):
        """测试无效交易数据验证"""
        errors = DataValidator.validate_transaction_data(self.invalid_row)
        self.assertGreater(len(errors), 0)
        
        # 检查具体错误
        error_messages = [error.lower() for error in errors]
        self.assertTrue(any('负数' in msg for msg in error_messages))
        self.assertTrue(any('格式错误' in msg for msg in error_messages))
        self.assertTrue(any('不能为空' in msg for msg in error_messages))

class TestInvestmentProductValidator(unittest.TestCase):
    """测试投资产品验证器"""
    
    def test_validate_product_info_valid(self):
        """测试有效产品信息验证"""
        valid_info = {
            '个人金额': 1000.0,
            '公司金额': 2000.0,
            '总金额': 3000.0,
            '累计申购': 3000.0,
            '累计赎回': 0.0
        }
        
        errors = InvestmentProductValidator.validate_product_info(valid_info)
        self.assertEqual(len(errors), 0)
    
    def test_validate_product_info_invalid(self):
        """测试无效产品信息验证"""
        invalid_info = {
            '个人金额': 1000.0,
            '公司金额': 2000.0,
            '总金额': 4000.0,  # 不匹配
            '累计申购': 3000.0
            # 缺少累计赎回
        }
        
        errors = InvestmentProductValidator.validate_product_info(invalid_info)
        self.assertGreater(len(errors), 0)
        
        # 检查具体错误
        error_messages = [error.lower() for error in errors]
        self.assertTrue(any('缺少必需字段' in msg for msg in error_messages))
        self.assertTrue(any('总金额计算错误' in msg for msg in error_messages))

class TestAuditLogger(unittest.TestCase):
    """测试审计日志器"""
    
    def setUp(self):
        """测试前准备"""
        self.logger = AuditLogger("test_logger", "test_logs")
    
    def test_logger_creation(self):
        """测试日志器创建"""
        self.assertIsNotNone(self.logger.logger)
        self.assertEqual(self.logger.logger.name, "test_logger")
    
    def test_log_levels(self):
        """测试日志级别"""
        # 这些方法应该不抛出异常
        self.logger.info("测试信息")
        self.logger.debug("测试调试")
        self.logger.warning("测试警告")
        self.logger.error("测试错误")
        self.logger.critical("测试严重错误")
    
    def test_log_transaction(self):
        """测试交易日志"""
        # 这个方法应该不抛出异常
        self.logger.log_transaction(
            row_idx=1,
            transaction_type="收入",
            amount=1000.0,
            fund_attribute="个人应收",
            personal_ratio=1.0,
            company_ratio=0.0,
            behavior="个人资金流入"
        )
    
    def test_log_balance_check(self):
        """测试余额检查日志"""
        # 测试余额匹配
        self.logger.log_balance_check(1, 1000.0, 1000.0, 0.0)
        
        # 测试余额不匹配
        self.logger.log_balance_check(2, 1000.0, 1100.0, 100.0)
    
    def tearDown(self):
        """测试后清理"""
        import shutil
        if os.path.exists("test_logs"):
            shutil.rmtree("test_logs")

if __name__ == '__main__':
    # 运行测试
    unittest.main(verbosity=2) 