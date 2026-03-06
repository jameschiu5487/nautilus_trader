#!/usr/bin/env python3
# -------------------------------------------------------------------------------------------------
#  Copyright (C) 2015-2026 Nautech Systems Pty Ltd. All rights reserved.
#  https://nautechsystems.io
#
#  Licensed under the GNU Lesser General Public License Version 3.0 (the "License");
#  You may not use this file except in compliance with the License.
#  You may obtain a copy of the License at https://www.gnu.org/licenses/lgpl-3.0.en.html
#
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.
# -------------------------------------------------------------------------------------------------
"""
使用 NautilusTrader 原生 Binance 适配器 + STG Feature Calculators

这个示例展示如何：
1. 使用 NautilusTrader 的 Binance 数据客户端接收实时市场数据
2. 使用 STG Feature Calculators 计算市场微观结构特征
3. 基于特征生成交易信号（演示用途）

运行:
    python examples/live/binance/binance_feature_monitor.py
"""

from nautilus_trader.adapters.binance import BINANCE
from nautilus_trader.adapters.binance import BinanceAccountType
from nautilus_trader.adapters.binance import BinanceDataClientConfig
from nautilus_trader.adapters.binance import BinanceLiveDataClientFactory
from nautilus_trader.common.actor import Actor
from nautilus_trader.config import ActorConfig
from nautilus_trader.config import InstrumentProviderConfig
from nautilus_trader.config import LoggingConfig
from nautilus_trader.config import TradingNodeConfig
from nautilus_trader.core.message import Event
from nautilus_trader.live.node import TradingNode
from nautilus_trader.model.data import OrderBookDelta
from nautilus_trader.model.data import QuoteTick
from nautilus_trader.model.data import TradeTick
from nautilus_trader.model.identifiers import InstrumentId
from nautilus_trader.model.identifiers import TraderId

try:
    from nautilus_trader.core import nautilus_pyo3
    stg = nautilus_pyo3.stg
except ImportError:
    print("\n❌ STG 模块导入失败！")
    print("\n请先构建 Python 包:")
    print("  cd /Users/kuohungsheng/Desktop/nautilus_trader")
    print("  python build.py\n")
    exit(1)


class FeatureMonitorConfig(ActorConfig, frozen=True):
    """Feature Monitor 配置"""
    instrument_id: str
    window_ms: int = 1000
    jump_threshold_bps: float = 50.0


class FeatureMonitor(Actor):
    """
    实时市场特征监控器
    
    订阅市场数据并计算微观结构特征：
    - OFI (Order Flow Imbalance)
    - Realized Volatility
    - Jump Detection
    - Liquidity Score
    """
    
    def __init__(self, config: FeatureMonitorConfig) -> None:
        super().__init__(config)
        
        # 配置
        self.instrument_id = InstrumentId.from_str(config.instrument_id)
        self.window_ms = config.window_ms
        
        # 创建 Feature Calculators
        self.ofi = stg.OfiCalculator(self.window_ms)
        self.vol = stg.VolatilityCalculator(self.window_ms)
        self.liq = stg.LiquidityCalculator(self.window_ms)
        self.jump = stg.JumpCalculator(self.window_ms, config.jump_threshold_bps)
        
        # 统计
        self.update_count = 0
        self.trade_count = 0
        self.delta_count = 0
        self.quote_count = 0
        
    def on_start(self) -> None:
        """Actor 启动时调用"""
        self.log.info(f"启动 FeatureMonitor: {self.instrument_id}")
        self.log.info(f"窗口大小: {self.window_ms}ms")
        
        # 订阅市场数据
        self.subscribe_order_book_deltas(self.instrument_id)
        self.subscribe_trade_ticks(self.instrument_id)
        self.subscribe_quote_ticks(self.instrument_id)
        
        self.log.info("已订阅市场数据流")
        self.log.info("⏱  等待数据到达...")
        self.log.info("📊 将在以下情况打印状态：")
        self.log.info("   - 首次接收到数据时")
        self.log.info("   - 每 100 次更新")
        self.log.info("   - 检测到价格跳跃时")
        
    def on_stop(self) -> None:
        """Actor 停止时调用"""
        self.log.info(f"停止 FeatureMonitor")
        self.print_final_stats()
        
    def on_order_book_delta(self, delta: OrderBookDelta) -> None:
        """
        处理订单簿增量更新
        
        Args:
            delta: 订单簿增量数据（价格层级的增删改）
        """
        self.delta_count += 1
        
        # 转换为 STG Side
        side = stg.Side.buy() if delta.order.side.name == "BUY" else stg.Side.sell()
        
        # 判断是否为取消
        is_cancel = (delta.action.name == "DELETE" or delta.order.size == 0)
        
        # 更新 OFI Calculator
        self.ofi.add_order_event(
            timestamp_ns=delta.ts_event,
            side=side,
            price=float(delta.order.price),
            size=float(delta.order.size) if not is_cancel else float(delta.order.size or 0),
            is_cancel=is_cancel,
        )
        
        self.update_count += 1
        
        # 首次数据到达时打印
        if self.update_count == 1:
            self.log.info("✓ 首次数据到达！开始计算特征...")
            self.print_features()
        
        # 每 100 次更新打印一次状态（更频繁，便于观察）
        elif self.update_count % 100 == 0:
            self.print_features()
    
    def on_trade_tick(self, tick: TradeTick) -> None:
        """
        处理成交数据
        
        Args:
            tick: 成交 tick 数据
        """
        self.trade_count += 1
        
        price = float(tick.price)
        size = float(tick.size)
        
        # 转换交易方向
        side = stg.Side.buy() if tick.aggressor_side.name == "BUYER" else stg.Side.sell()
        
        # 更新波动率和跳跃检测
        self.vol.update(tick.ts_event, price)
        self.jump.update(tick.ts_event, price)
        
        # 更新流动性（需要 4 个参数：timestamp_ns, price, size, side）
        self.liq.add_trade(tick.ts_event, price, size, side)
        
        self.update_count += 1
        
        # 首次成交数据时打印
        if self.trade_count == 1:
            self.log.info("✓ 首次成交数据到达！")
            self.print_features()
        # 每 100 次更新打印状态
        elif self.update_count % 100 == 0:
            self.print_features()
        
        # 检查是否有跳跃（立即打印）
        if self.jump.jump_count() > 0:
            self.log.warning(
                f"⚠️  检测到价格跳跃！"
                f"Max Move: {self.jump.max_move_bps():.2f} bps, "
                f"Jump Count: {self.jump.jump_count()}"
            )
            self.print_features()  # 跳跃时立即打印完整状态
    
    def on_quote_tick(self, tick: QuoteTick) -> None:
        """
        处理报价数据（买一卖一）
        
        Args:
            tick: 报价 tick 数据
        """
        self.quote_count += 1
        
        # 使用中间价更新波动率
        mid_price = (float(tick.bid_price) + float(tick.ask_price)) / 2
        self.vol.update(tick.ts_event, mid_price)
        self.jump.update(tick.ts_event, mid_price)
    
    def get_features(self) -> dict:
        """获取当前特征"""
        return {
            'ofi': self.ofi.ofi(),
            'cancel_ratio': self.ofi.cancel_ratio(),
            'rvol_bps': self.vol.rvol_bps(),
            'jump_count': self.jump.jump_count(),
            'max_move_bps': self.jump.max_move_bps(),
            'avg_trade_qty': self.liq.avg_trade_qty(),
        }
    
    def get_signal(self) -> str:
        """生成交易信号（示例）"""
        features = self.get_features()
        
        # 简单信号逻辑
        if features['jump_count'] > 0:
            return "⚠️  JUMP"
        elif features['rvol_bps'] > 5000:
            return "⚡ HIGH_VOL"
        elif features['ofi'] > 1000:
            return "🟢 BULLISH"
        elif features['ofi'] < -1000:
            return "🔴 BEARISH"
        else:
            return "⚪ NEUTRAL"
    
    def print_features(self) -> None:
        """打印当前特征"""
        features = self.get_features()
        signal = self.get_signal()
        
        self.log.info(
            f"\n{'='*70}\n"
            f"📊 Feature Status - {self.instrument_id}\n"
            f"{'='*70}\n"
            f"更新次数: {self.update_count:,} | "
            f"成交: {self.trade_count:,} | "
            f"订单簿增量: {self.delta_count:,} | "
            f"报价: {self.quote_count:,}\n"
            f"{'-'*70}\n"
            f"OFI:              {features['ofi']:>12.2f}\n"
            f"Cancel Ratio:     {features['cancel_ratio']:>12.3f}\n"
            f"RVOL (bps):       {features['rvol_bps']:>12.2f}\n"
            f"Jump Count:       {features['jump_count']:>12}\n"
            f"Max Move (bps):   {features['max_move_bps']:>12.2f}\n"
            f"Avg Trade Qty:    {features['avg_trade_qty']:>12.6f}\n"
            f"{'-'*70}\n"
            f"信号: {signal}\n"
            f"{'='*70}"
        )
    
    def print_final_stats(self) -> None:
        """打印最终统计"""
        self.log.info(
            f"\n{'='*70}\n"
            f"📈 最终统计 - {self.instrument_id}\n"
            f"{'='*70}\n"
            f"总更新次数:       {self.update_count:,}\n"
            f"成交 Ticks:       {self.trade_count:,}\n"
            f"订单簿增量:       {self.delta_count:,}\n"
            f"报价 Ticks:       {self.quote_count:,}\n"
            f"{'='*70}"
        )


# ============================================================================
# 主程序
# ============================================================================

# 配置交易对
SYMBOL = "BTCUSDT"
ACCOUNT_TYPE = BinanceAccountType.SPOT  # 或 BinanceAccountType.USDT_FUTURES

# 根据账户类型设置 instrument_id
if ACCOUNT_TYPE == BinanceAccountType.SPOT:
    instrument_id = f"{SYMBOL}.BINANCE"
else:
    instrument_id = f"{SYMBOL}-PERP.BINANCE"

# 配置交易节点
config_node = TradingNodeConfig(
    trader_id=TraderId("FEATURE-MONITOR-001"),
    logging=LoggingConfig(
        log_level="INFO",
        use_pyo3=True,
    ),
    data_clients={
        BINANCE: BinanceDataClientConfig(
            # 公开市场数据不需要 API Key
            api_key=None,
            api_secret=None,
            account_type=ACCOUNT_TYPE,
            instrument_provider=InstrumentProviderConfig(
                load_ids=frozenset([InstrumentId.from_str(instrument_id)])
            ),
        ),
    },
    timeout_connection=20.0,
    timeout_disconnection=10.0,
    timeout_post_stop=2.0,
)

# 创建交易节点
node = TradingNode(config=config_node)

# 配置 Feature Monitor
monitor_config = FeatureMonitorConfig(
    component_id="FeatureMonitor-1",
    instrument_id=instrument_id,
    window_ms=1000,  # 1s 窗口
    jump_threshold_bps=50.0,  # 50 bps 跳跃阈值
)

# 创建并添加 Actor
monitor = FeatureMonitor(config=monitor_config)
node.trader.add_actor(monitor)

# 注册 Binance 数据客户端工厂
node.add_data_client_factory(BINANCE, BinanceLiveDataClientFactory)
node.build()


# ============================================================================
# 运行节点（Ctrl+C 停止）
# ============================================================================

if __name__ == "__main__":
    print("\n" + "╔" + "═"*68 + "╗")
    print("║" + " "*10 + "NautilusTrader STG Feature Monitor" + " "*14 + "║")
    print("╚" + "═"*68 + "╝\n")
    print(f"交易对: {SYMBOL}")
    print(f"账户类型: {ACCOUNT_TYPE.name}")
    print(f"Instrument ID: {instrument_id}")
    print(f"窗口大小: 1000ms")
    print(f"\n按 Ctrl+C 停止...\n")
    
    try:
        node.run()
    finally:
        node.dispose()
