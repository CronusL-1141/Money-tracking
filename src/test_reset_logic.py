#!/usr/bin/env python3
"""测试资金池重置和盈亏计算逻辑"""

class PoolTracker:
    def __init__(self):
        self.pools = {}
    
    def process_transaction(self, pool_name, transaction_type, amount, description=""):
        """处理资金池交易"""
        if pool_name not in self.pools:
            self.pools[pool_name] = {
                'current_balance': 0,
                'cumulative_purchase': 0,
                'cumulative_redemption': 0,
                'reset_history': [],  # 记录每次重置时的盈亏
                'total_realized_profit': 0  # 累计已实现盈亏
            }
        
        pool = self.pools[pool_name]
        print(f"\n🔄 处理交易: {pool_name} - {transaction_type} ¥{amount:,.0f} {description}")
        print(f"   交易前余额: ¥{pool['current_balance']:,.0f}")
        
        if transaction_type == "申购":
            # 检查是否需要重置
            if pool['current_balance'] < 0:
                # 记录重置前的盈利
                realized_profit = abs(pool['current_balance'])
                pool['reset_history'].append({
                    'type': '重置盈利',
                    'amount': realized_profit,
                    'description': f"重置前实现盈利 ¥{realized_profit:,.0f}"
                })
                pool['total_realized_profit'] += realized_profit
                print(f"   💰 资金池重置！实现盈利: ¥{realized_profit:,.0f}")
                print(f"   📈 累计已实现盈利: ¥{pool['total_realized_profit']:,.0f}")
                pool['current_balance'] = 0  # 重置余额
            
            pool['current_balance'] += amount
            pool['cumulative_purchase'] += amount
            
        elif transaction_type == "赎回":
            pool['current_balance'] -= amount
            pool['cumulative_redemption'] += amount
            
        print(f"   交易后余额: ¥{pool['current_balance']:,.0f}")
        print(f"   累计申购: ¥{pool['cumulative_purchase']:,.0f}")
        print(f"   累计赎回: ¥{pool['cumulative_redemption']:,.0f}")
    
    def calculate_real_profit_loss(self, pool_name):
        """计算真实的盈亏状态"""
        if pool_name not in self.pools:
            return None
        
        pool = self.pools[pool_name]
        
        # 当前周期的盈亏
        current_profit_loss = 0
        if pool['current_balance'] < 0:
            # 当前有盈利
            current_profit_loss = abs(pool['current_balance'])
            status = "盈利"
        elif pool['current_balance'] > 0:
            # 当前有亏损（资金占用）
            current_profit_loss = -pool['current_balance']
            status = "亏损"
        else:
            status = "持平"
        
        # 总的真实盈亏 = 历史已实现盈利 + 当前盈亏
        total_real_profit = pool['total_realized_profit'] + current_profit_loss
        
        return {
            'status': status,
            'current_profit_loss': current_profit_loss,
            'historical_profit': pool['total_realized_profit'],
            'total_real_profit': total_real_profit,
            'current_balance': pool['current_balance'],
            'reset_count': len(pool['reset_history'])
        }

def test_scenarios():
    tracker = PoolTracker()
    
    print("=" * 80)
    print("📊 场外资金池重置盈亏计算测试")
    print("=" * 80)
    
    # 用户提到的场景1
    print("\n🎯 场景1：申购100w→赎回110w→申购100w→赎回110w")
    print("-" * 50)
    tracker.process_transaction("理财-A001", "申购", 1000000, "(第1次)")
    tracker.process_transaction("理财-A001", "赎回", 1100000, "(第1次，赚10w)")
    tracker.process_transaction("理财-A001", "申购", 1000000, "(第2次)")
    tracker.process_transaction("理财-A001", "赎回", 1100000, "(第2次，赚10w)")
    
    result1 = tracker.calculate_real_profit_loss("理财-A001")
    print(f"\n📊 最终分析:")
    print(f"   当前余额: ¥{result1['current_balance']:,.0f}")
    print(f"   重置次数: {result1['reset_count']}")
    print(f"   历史已实现盈利: ¥{result1['historical_profit']:,.0f}")
    print(f"   当前周期盈亏: ¥{result1['current_profit_loss']:,.0f}")
    print(f"   💎 真实总盈利: ¥{result1['total_real_profit']:,.0f}")
    
    # 用户提到的场景2（对比）
    print("\n\n🎯 场景2：申购100w→赎回110w→申购100w→赎回90w")
    print("-" * 50)
    tracker2 = PoolTracker()
    tracker2.process_transaction("理财-B001", "申购", 1000000, "(第1次)")
    tracker2.process_transaction("理财-B001", "赎回", 1100000, "(第1次，赚10w)")
    tracker2.process_transaction("理财-B001", "申购", 1000000, "(第2次)")
    tracker2.process_transaction("理财-B001", "赎回", 900000, "(第2次，亏10w)")
    
    result2 = tracker2.calculate_real_profit_loss("理财-B001")
    print(f"\n📊 最终分析:")
    print(f"   当前余额: ¥{result2['current_balance']:,.0f}")
    print(f"   重置次数: {result2['reset_count']}")
    print(f"   历史已实现盈利: ¥{result2['historical_profit']:,.0f}")
    print(f"   当前周期盈亏: ¥{result2['current_profit_loss']:,.0f}")
    print(f"   💎 真实总盈亏: ¥{result2['total_real_profit']:,.0f}")
    
    print("\n" + "=" * 80)
    print("💡 结论：需要跟踪资金池重置历史才能计算真实盈亏！")
    print("=" * 80)

if __name__ == "__main__":
    test_scenarios()
