#!/usr/bin/env python3
"""
Feature Engine Python 使用示例

展示如何使用 OfiCalculator, VolatilityCalculator, LiquidityCalculator, JumpCalculator

需要先构建 Rust 库:
    cd /path/to/nautilus_trader
    cargo build --package nautilus-stg --features python --release
    
然后安装 Python 包:
    pip install -e .
"""

import time
from typing import List, Dict, Optional

# 尝试不同的导入方式
try:
    # 方式 1: 通过 nautilus_pyo3 (推荐)
    from nautilus_trader.core import nautilus_pyo3
    stg = nautilus_pyo3.stg
    
    OfiCalculator = stg.OfiCalculator
    VolatilityCalculator = stg.VolatilityCalculator
    LiquidityCalculator = stg.LiquidityCalculator
    JumpCalculator = stg.JumpCalculator
    Side = stg.Side
    
    print("✓ 成功通过 nautilus_pyo3 导入")
    
except (ImportError, AttributeError) as e1:
    try:
        # 方式 2: 直接导入 (如果作为独立模块)
        import stg
        
        OfiCalculator = stg.OfiCalculator
        VolatilityCalculator = stg.VolatilityCalculator
        LiquidityCalculator = stg.LiquidityCalculator
        JumpCalculator = stg.JumpCalculator
        Side = stg.Side
        
        print("✓ 成功直接导入 stg 模块")
        
    except ImportError as e2:
        print("\n" + "="*70)
        print("❌ 导入错误")
        print("="*70)
        print("\n无法导入 Feature Engine 模块。")
        print("\n请按以下步骤操作:\n")
        print("1. 构建 Rust 库 (带 Python 支持):")
        print("   cd /Users/kuohungsheng/Desktop/nautilus_trader")
        print("   cargo build --package nautilus-stg --features python --release")
        print("\n2. 安装 Python 包:")
        print("   pip install -e .")
        print("\n3. 或者设置 PYTHONPATH:")
        print("   export PYTHONPATH=/path/to/nautilus_trader:$PYTHONPATH")
        print("\n原始错误:")
        print(f"  方式 1 错误: {e1}")
        print(f"  方式 2 错误: {e2}")
        print("="*70)
        raise SystemExit(1)


# ============================================================================
# 示例 1: 基本使用
# ============================================================================

def example_01_basic_usage():
    """示例 1: 基本使用 - 单个 Calculator"""
    print("\n" + "="*70)
    print("示例 1: 基本使用")
    print("="*70)
    
    # 创建 300ms 窗口的 OFI 计算器
    ofi = OfiCalculator(300)
    print(f"✓ 创建 OfiCalculator, 窗口: {ofi.window_ms}ms")
    
    # 模拟一些订单事件
    current_time = int(time.time() * 1e9)  # 当前时间（纳秒）
    
    print("\n添加订单事件:")
    
    # 买单
    ofi.add_order_event(
        timestamp_ns=current_time,
        side=Side.buy(),
        price=100.0,
        size=10.0,
        is_cancel=False
    )
    print(f"  ➜ 时间 {0:>6}ms: BUY  10.0 @ 100.0   | OFI: {ofi.ofi():>6.1f}")
    
    # 卖单
    ofi.add_order_event(
        timestamp_ns=current_time + 100_000_000,  # +100ms
        side=Side.sell(),
        price=100.10,
        size=5.0,
        is_cancel=False
    )
    print(f"  ➜ 时间 {100:>6}ms: SELL 5.0  @ 100.10 | OFI: {ofi.ofi():>6.1f}")
    
    # 取消单
    ofi.add_order_event(
        timestamp_ns=current_time + 200_000_000,  # +200ms
        side=Side.buy(),
        price=100.0,
        size=3.0,
        is_cancel=True
    )
    print(f"  ➜ 时间 {200:>6}ms: CANCEL BUY 3.0    | OFI: {ofi.ofi():>6.1f}")
    
    print(f"\n最终结果:")
    print(f"  • OFI:          {ofi.ofi():.2f}")
    print(f"  • Cancel Ratio: {ofi.cancel_ratio():.3f}")
    print(f"  • {ofi}")


# ============================================================================
# 示例 2: 所有 Calculator 的基本使用
# ============================================================================

def example_02_all_calculators():
    """示例 2: 所有 Calculator 的基本使用"""
    print("\n" + "="*70)
    print("示例 2: 所有 Calculator")
    print("="*70)
    
    # 创建所有 Calculator (1s 窗口)
    ofi = OfiCalculator(1000)
    vol = VolatilityCalculator(1000)
    liq = LiquidityCalculator(1000)
    jump = JumpCalculator(1000, threshold_bps=50.0)
    
    print("✓ 创建所有 Calculator (1s 窗口)\n")
    
    current_time = int(time.time() * 1e9)
    
    # 模拟市场数据
    prices = [100.0, 100.1, 100.05, 100.2, 100.15, 100.3, 100.25]
    
    print("添加市场数据:")
    for i, price in enumerate(prices):
        ts = current_time + i * 100_000_000  # 每 100ms 一个 tick
        
        # 更新 volatility 和 jump
        vol.update(ts, price)
        jump.update(ts, price)
        
        # 模拟订单流
        if i % 2 == 0:
            ofi.add_order_event(ts, Side.buy(), price, 10.0 + i, False)
        else:
            ofi.add_order_event(ts, Side.sell(), price, 5.0 + i, False)
        
        # 模拟成交
        liq.add_trade(ts, price, 50.0 + i * 10, Side.buy() if i % 2 == 0 else Side.sell())
        
        move = ((price - prices[i-1]) / prices[i-1] * 10000) if i > 0 else 0
        print(f"  Tick {i}: {price:>6.2f} (move: {move:>5.1f} bps)")
    
    print(f"\n计算结果:")
    print(f"  • OFI:           {ofi.ofi():>8.2f}")
    print(f"  • Cancel Ratio:  {ofi.cancel_ratio():>8.3f}")
    print(f"  • RVOL (bps):    {vol.rvol_bps():>8.2f}")
    print(f"  • Jump Count:    {jump.jump_count():>8}")
    print(f"  • Max Move:      {jump.max_move_bps():>8.2f} bps")
    print(f"  • Avg Trade Qty: {liq.avg_trade_qty():>8.2f}")


# ============================================================================
# 示例 3: 多窗口监控
# ============================================================================

def example_03_multi_window():
    """示例 3: 多窗口监控 - 期限结构分析"""
    print("\n" + "="*70)
    print("示例 3: 多窗口监控")
    print("="*70)
    
    # 创建多个窗口的 OFI 计算器
    windows = [200, 500, 1000]
    ofi_calcs = {w: OfiCalculator(w) for w in windows}
    vol_calcs = {w: VolatilityCalculator(w) for w in windows}
    
    print(f"✓ 创建多窗口 Calculator: {windows}ms\n")
    
    current_time = int(time.time() * 1e9)
    
    # 模拟 20 个事件
    print("添加 20 个市场事件...")
    for i in range(20):
        ts = current_time + i * 50_000_000  # 每 50ms
        price = 100.0 + (i % 5) * 0.05
        side = Side.buy() if i % 3 == 0 else Side.sell()
        size = 10.0 + i * 0.5
        
        # 更新所有窗口
        for calc in ofi_calcs.values():
            calc.add_order_event(ts, side, price, size, False)
        
        for calc in vol_calcs.values():
            calc.update(ts, price)
    
    print("✓ 完成\n")
    
    # 显示 OFI 期限结构
    print("OFI 期限结构:")
    print(f"  窗口大小 | OFI 值")
    print(f"  ---------|----------")
    for window in windows:
        ofi_value = ofi_calcs[window].ofi()
        print(f"  {window:>4} ms  | {ofi_value:>8.2f}")
    
    # 显示波动率期限结构
    print("\n波动率期限结构:")
    print(f"  窗口大小 | RVOL (bps)")
    print(f"  ---------|----------")
    for window in windows:
        rvol = vol_calcs[window].rvol_bps()
        print(f"  {window:>4} ms  | {rvol:>8.2f}")


# ============================================================================
# 示例 4: 实时市场监控类
# ============================================================================

class MarketMonitor:
    """实时市场监控器"""
    
    def __init__(self, window_ms: int = 1000):
        """初始化监控器
        
        Args:
            window_ms: 时间窗口（毫秒）
        """
        self.window_ms = window_ms
        self.ofi = OfiCalculator(window_ms)
        self.vol = VolatilityCalculator(window_ms)
        self.liq = LiquidityCalculator(window_ms)
        self.jump = JumpCalculator(window_ms, threshold_bps=50.0)
        
        self.last_update = 0
        self.update_count = 0
    
    def on_order_event(self, timestamp_ns: int, side: Side, price: float, 
                      size: float, is_cancel: bool):
        """处理订单事件"""
        self.ofi.add_order_event(timestamp_ns, side, price, size, is_cancel)
        self.last_update = timestamp_ns
        self.update_count += 1
    
    def on_trade(self, timestamp_ns: int, price: float, size: float, side: Side):
        """处理成交事件"""
        self.liq.add_trade(timestamp_ns, price, size, side)
        self.last_update = timestamp_ns
    
    def on_price_tick(self, timestamp_ns: int, price: float):
        """处理价格 tick"""
        self.vol.update(timestamp_ns, price)
        self.jump.update(timestamp_ns, price)
        self.last_update = timestamp_ns
    
    def get_features(self) -> Dict[str, float]:
        """获取所有特征"""
        return {
            'ofi': self.ofi.ofi(),
            'cancel_ratio': self.ofi.cancel_ratio(),
            'rvol_bps': self.vol.rvol_bps(),
            'jump_count': float(self.jump.jump_count()),
            'max_move_bps': self.jump.max_move_bps(),
            'avg_trade_qty': self.liq.avg_trade_qty(),
        }
    
    def check_signal(self) -> Optional[str]:
        """检查交易信号
        
        Returns:
            'BUY', 'SELL', 或 None
        """
        features = self.get_features()
        
        # 简单的信号逻辑示例
        if (features['ofi'] > 50 and 
            features['rvol_bps'] < 100 and 
            features['jump_count'] == 0):
            return 'BUY'
        
        elif (features['ofi'] < -50 and 
              features['rvol_bps'] < 100 and 
              features['jump_count'] == 0):
            return 'SELL'
        
        return None
    
    def print_status(self):
        """打印当前状态"""
        features = self.get_features()
        signal = self.check_signal()
        
        print(f"\n{'='*60}")
        print(f"Market Monitor Status (Window: {self.window_ms}ms)")
        print(f"{'='*60}")
        print(f"Updates:       {self.update_count}")
        print(f"{'─'*60}")
        print(f"OFI:           {features['ofi']:>10.2f}")
        print(f"Cancel Ratio:  {features['cancel_ratio']:>10.3f}")
        print(f"Volatility:    {features['rvol_bps']:>10.2f} bps")
        print(f"Jump Count:    {features['jump_count']:>10.0f}")
        print(f"Max Move:      {features['max_move_bps']:>10.2f} bps")
        print(f"Avg Trade Qty: {features['avg_trade_qty']:>10.2f}")
        print(f"{'─'*60}")
        if signal:
            print(f"Signal:        *** {signal} ***")
        else:
            print(f"Signal:        NEUTRAL")
        print(f"{'='*60}")


def example_04_market_monitor():
    """示例 4: 使用 MarketMonitor 类"""
    print("\n" + "="*70)
    print("示例 4: 实时市场监控器")
    print("="*70)
    
    # 创建监控器
    monitor = MarketMonitor(window_ms=1000)
    print("✓ 创建 MarketMonitor (1s 窗口)\n")
    
    current_time = int(time.time() * 1e9)
    
    # 模拟市场数据流
    print("模拟市场数据流...")
    
    # 阶段 1: 买压
    print("\n阶段 1: 强买压 (0-500ms)")
    for i in range(10):
        ts = current_time + i * 50_000_000
        price = 100.0 + i * 0.01
        monitor.on_order_event(ts, Side.buy(), price, 20.0 + i, False)
        monitor.on_price_tick(ts, price)
        monitor.on_trade(ts, price, 100.0, Side.buy())
    
    monitor.print_status()
    
    # 阶段 2: 卖压
    print("\n\n阶段 2: 强卖压 (500-1000ms)")
    for i in range(10, 20):
        ts = current_time + i * 50_000_000
        price = 100.1 - (i - 10) * 0.01
        monitor.on_order_event(ts, Side.sell(), price, 15.0 + i, False)
        monitor.on_price_tick(ts, price)
        monitor.on_trade(ts, price, 80.0, Side.sell())
    
    monitor.print_status()


# ============================================================================
# 示例 5: 多策略对比
# ============================================================================

def example_05_strategy_comparison():
    """示例 5: 不同窗口策略对比"""
    print("\n" + "="*70)
    print("示例 5: 多策略对比")
    print("="*70)
    
    # 创建三个不同窗口的监控器
    monitors = {
        'Fast (200ms)': MarketMonitor(200),
        'Medium (500ms)': MarketMonitor(500),
        'Slow (1000ms)': MarketMonitor(1000),
    }
    
    print("✓ 创建三个不同窗口的监控器\n")
    
    current_time = int(time.time() * 1e9)
    
    # 模拟相同的市场数据
    print("添加市场数据...")
    for i in range(30):
        ts = current_time + i * 50_000_000
        price = 100.0 + (i % 10) * 0.05
        side = Side.buy() if i < 15 else Side.sell()
        size = 10.0 + i
        
        for monitor in monitors.values():
            monitor.on_order_event(ts, side, price, size, False)
            monitor.on_price_tick(ts, price)
            monitor.on_trade(ts, price, 50.0 + i * 2, side)
    
    print("✓ 完成\n")
    
    # 对比结果
    print("\n策略对比:")
    print(f"{'指标':<15} | {'Fast (200ms)':<12} | {'Medium (500ms)':<15} | {'Slow (1000ms)':<12}")
    print("─" * 70)
    
    metrics = ['ofi', 'rvol_bps', 'jump_count', 'avg_trade_qty']
    metric_names = {
        'ofi': 'OFI',
        'rvol_bps': 'RVOL (bps)',
        'jump_count': 'Jump Count',
        'avg_trade_qty': 'Avg Trade Qty',
    }
    
    for metric in metrics:
        values = []
        for name, monitor in monitors.items():
            features = monitor.get_features()
            values.append(features[metric])
        
        print(f"{metric_names[metric]:<15} | {values[0]:>12.2f} | {values[1]:>15.2f} | {values[2]:>12.2f}")
    
    # 显示信号
    print("\n" + "─" * 70)
    print("交易信号:")
    for name, monitor in monitors.items():
        signal = monitor.check_signal() or "NEUTRAL"
        print(f"  • {name:<15}: {signal}")


# ============================================================================
# Main
# ============================================================================

def main():
    """运行所有示例"""
    print("\n")
    print("╔" + "═"*68 + "╗")
    print("║" + " "*20 + "Feature Engine Python 示例" + " "*22 + "║")
    print("╚" + "═"*68 + "╝")
    
    try:
        # 运行所有示例
        example_01_basic_usage()
        example_02_all_calculators()
        example_03_multi_window()
        example_04_market_monitor()
        example_05_strategy_comparison()
        
        print("\n" + "="*70)
        print("✓ 所有示例运行完成！")
        print("="*70)
        
    except Exception as e:
        print(f"\n❌ 错误: {e}")
        print("\n请确保已构建 Rust 库:")
        print("  cd /path/to/nautilus_trader")
        print("  cargo build --package nautilus-stg --features python --release")
        raise


if __name__ == "__main__":
    main()
