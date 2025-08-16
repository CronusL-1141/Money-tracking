"""
流水审计系统配置文件
"""

class Config:
    """系统配置类"""
    
    # 数值精度和阈值
    PRECISION = 2  # 数值精度
    EPSILON = 1e-8  # 浮点数比较阈值
    BALANCE_TOLERANCE = 0.01  # 余额验证容差
    
    # 投资产品前缀
    INVESTMENT_PREFIXES = ['理财', '投资', '保险', '关联银行卡', '资金池']
    
    # 资金属性关键词
    PERSONAL_KEYWORDS = ['个人', '个人应收', '个人应付']
    COMPANY_KEYWORDS = ['公司', '公司应收', '公司应付']
    
    # 大额交易阈值
    LARGE_AMOUNT_THRESHOLD = 1000000  # 100万
    
    # 显示设置
    MAX_DISPLAY_ROWS = 10  # 最大显示行数
    PROGRESS_INTERVAL = 1000  # 进度显示间隔
    
    # 文件设置
    DEFAULT_OUTPUT_FILE = "FIFO资金追踪结果.xlsx"
    DEFAULT_INPUT_FILE = "流水.xlsx"
    
    # 图表设置
    FIGURE_SIZE = (15, 12)
    FONT_SIZE = 16
    
    # 中文字体设置
    CHINESE_FONTS = ['SimHei', 'Microsoft YaHei']
    
    @classmethod
    def format_number(cls, value):
        """格式化数值，避免科学计数法"""
        if abs(value) < cls.EPSILON:
            return 0.0
        return round(value, cls.PRECISION)
    
    @classmethod
    def is_investment_product(cls, fund_attribute):
        """判断是否为投资产品"""
        if isinstance(fund_attribute, str) and '-' in fund_attribute:
            prefix = fund_attribute.split('-')[0]
            return prefix in cls.INVESTMENT_PREFIXES
        return False
    
    @classmethod
    def is_personal_fund(cls, fund_attribute):
        """判断是否为个人资金"""
        fund_str = str(fund_attribute).strip()
        return any(keyword in fund_str for keyword in cls.PERSONAL_KEYWORDS)
    
    @classmethod
    def is_company_fund(cls, fund_attribute):
        """判断是否为公司资金"""
        fund_str = str(fund_attribute).strip()
        return any(keyword in fund_str for keyword in cls.COMPANY_KEYWORDS) 