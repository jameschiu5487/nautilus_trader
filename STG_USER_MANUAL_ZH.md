# nautilus-stg 使用手册

**版本**: 1.0  
**最后更新**: 2026-03-06

---

## 目录

1. [快速开始](#快速开始)
2. [基本概念](#基本概念)
3. [API 参考](#api-参考)
4. [使用示例](#使用示例)
5. [最佳实践](#最佳实践)
6. [常见问题](#常见问题)

---

## 快速开始

### 安装

`nautilus-stg` 已集成到 NautilusTrader，无需单独安装。

### 导入模块

```python
from nautilus_trader.core import nautilus_pyo3

# 获取 STG 模块
stg = nautilus_pyo3.stg
```

### 第一个示例

```python
# 创建订单簿
book = stg.LocalL2Book(top_k=50)

# 添加买单
book.update(stg.Side.buy(), price=100.0, size=10.0)

# 添加卖单
book.update(stg.Side.sell(), price=100.10, size=15.0)

# 查询最佳价格
stats = book.best()
print(f"买一: {stats.bid1}, 卖一: {stats.ask1}, 中间价: {stats.mid}")
# 输出: 买一: 100.0, 卖一: 100.1, 中间价: 100.05
```

---

## 基本概念

### 1. L2 订单簿 (LocalL2Book)

L2 订单簿维护了买卖双方的价格和数量信息。

**关键特性**:
- 维护 top-K 个最优价格层级
- 自动计算前缀和，加速 VWAP 计算
- 支持实时更新
- 高性能（< 100ns per operation）

### 2. Side（方向）

交易方向枚举：
- `Side.buy()` - 买入方向
- `Side.sell()` - 卖出方向

### 3. Microprice（微观价格）

考虑订单簿不平衡的公平价格估计：

```
microprice = (bid1 × ask1_qty + ask1 × bid1_qty) / (bid1_qty + ask1_qty)
```

**用途**:
- 更精确的价格发现
- 预测短期价格变动
- 优化订单执行

### 4. Impact（价格冲击）

执行特定数量订单所需的平均价格（VWAP）。

**用途**:
- 估算大单成本
- 优化订单拆分
- 流动性评估

### 5. Bailout Cost（清仓成本）

紧急平仓的预估总成本，包括：
- Taker 手续费
- 滑点
- 逆向漂移
- 波动率惩罚
- 流动性惩罚
- 其他市场条件因素

---

## API 参考

### Side

```python
# 创建方向
buy_side = stg.Side.buy()
sell_side = stg.Side.sell()

# 从字符串创建
side = stg.Side("BUY")  # 或 "SELL"
```

### LocalL2Book

#### 构造函数

```python
book = stg.LocalL2Book(
    top_k=50,    # 维护的最大层级数，默认 50
    top_n=10     # 用于流动性统计的层级数，默认 10
)
```

**参数说明**:
- `top_k`: 限制订单簿维护的层级数，超出的会被丢弃
- `top_n`: 用于计算 `top_n_total_qty` 统计量

#### 核心方法

##### 1. update() - 更新订单簿

```python
book.update(
    side: Side,     # 交易方向
    price: float,   # 价格
    size: float     # 数量
)
```

**示例**:
```python
book.update(stg.Side.buy(), 99.95, 100.0)
book.update(stg.Side.sell(), 100.05, 150.0)
```

**注意**:
- 价格相同时会覆盖
- 数量为 0 时会删除该层级
- 自动排序（买方降序，卖方升序）

##### 2. best() - 获取最佳价格

```python
stats: LiquidityStats = book.best()
```

**返回**: `LiquidityStats` 对象

**字段**:
```python
stats.bid1              # 买一价
stats.ask1              # 卖一价
stats.mid               # 中间价 = (bid1 + ask1) / 2
stats.spread_bps        # 价差（基点）
stats.spread_ticks      # 价差（跳数）
stats.gap_ticks_bid     # 买方间隙（跳数）
stats.gap_ticks_ask     # 卖方间隙（跳数）
stats.top_n_total_qty   # 前 N 层总量
```

##### 3. microprice() - 计算微观价格

```python
mp: Optional[float] = book.microprice()
```

**返回**: 
- `float` - 微观价格
- `None` - 如果订单簿为空或数据不足

**示例**:
```python
book.update(stg.Side.buy(), 100.0, 10.0)
book.update(stg.Side.sell(), 100.10, 20.0)

mp = book.microprice()
print(f"Microprice: {mp:.4f}")  # 100.0333
```

##### 4. microprice_deviation_bps() - 微观价格偏离度

```python
dev: Optional[float] = book.microprice_deviation_bps()
```

**返回**: 偏离中间价的基点数

**解释**:
- 正值：买压更大，价格可能上涨
- 负值：卖压更大，价格可能下跌
- 零值：买卖平衡

**示例**:
```python
dev = book.microprice_deviation_bps()
print(f"偏离度: {dev:.2f} bps")  # -1.67 bps
```

##### 5. impact() - 计算价格冲击

```python
result: ImpactResult = book.impact(
    side: Side,            # 交易方向
    qty: float,            # 交易数量
    cushion_ticks: int = 0 # 额外保护跳数
)
```

**返回**: `ImpactResult` 对象

**字段**:
```python
result.ok            # 是否有足够流动性
result.vwap_price    # VWAP 价格
result.slip_bps      # 滑点（基点）
result.worst_price   # 最差成交价
result.limit_price   # 建议限价单价格（含 cushion）
result.used_level    # 使用的层级数
```

**示例**:
```python
# 计算买入 120 单位的冲击
result = book.impact(
    side=stg.Side.buy(),
    qty=120.0,
    cushion_ticks=2  # 额外 2 跳保护
)

if result.ok:
    print(f"VWAP: {result.vwap_price:.4f}")
    print(f"滑点: {result.slip_bps:.2f} bps")
    print(f"建议限价: {result.limit_price:.4f}")
else:
    print("流动性不足！")
```

##### 6. bailout_cost_bps_est() - 估算清仓成本

```python
cost_bps: float = book.bailout_cost_bps_est(
    qty: float,              # 清仓数量
    side: Side,              # 清仓方向
    market_state: MarketState # 市场状态
)
```

**返回**: 预估总成本（基点）

**示例**:
```python
# 创建市场状态
market_state = stg.MarketState(
    rvol_bps_1s=50.0,           # 实际波动率（1秒，bps）
    bc=0.9,                      # 订单簿可信度 [0, 1]
    lcd=False,                   # 流动性危机检测器
    withdraw_score=0.0,          # 提现分数 [0, 1]
    rate_limit_usage=0.1,        # 限速使用率 [0, 1]
    latency_p95_ms=10.0          # P95 延迟（毫秒）
)

# 估算清仓 500 单位的成本
cost = book.bailout_cost_bps_est(
    qty=500.0,
    side=stg.Side.sell(),
    market_state=market_state
)

print(f"预估清仓成本: {cost:.2f} bps")
```

##### 7. top_k() - 获取前 K 层

```python
prices, sizes = book.top_k(
    side: Side,  # 方向
    k: int       # 层级数
)
```

**返回**: `(prices: list[float], sizes: list[float])`

**示例**:
```python
# 获取买方前 5 层
prices, sizes = book.top_k(stg.Side.buy(), k=5)

for i, (price, size) in enumerate(zip(prices, sizes), 1):
    print(f"买{i}: 价格={price:.2f}, 数量={size:.2f}")
```

##### 8. cost_config() / set_cost_config() - 成本配置

```python
# 获取当前配置
config: CostEstimatorConfig = book.cost_config()

# 设置新配置
book.set_cost_config(config)
```

### CostEstimatorConfig

成本估算器配置。

```python
config = stg.CostEstimatorConfig(
    # 基础成本
    fee_taker_bps=5.0,                          # Taker 手续费（bps）
    expected_exec_time_ms=100.0,                # 预期执行时间（毫秒）
    
    # 市场冲击
    drift_rate_per_rvol=0.5,                    # 每单位波动率的漂移率
    
    # 订单簿质量
    bc_penalty_factor=20.0,                     # 可信度惩罚因子
    
    # 极端条件惩罚
    lcd_penalty_bps=50.0,                       # 流动性危机惩罚（bps）
    withdraw_penalty_factor=10.0,               # 提现惩罚因子
    rate_limit_penalty_factor=30.0,             # 限速惩罚因子
    latency_drift_rate_per_ms=0.1,              # 每毫秒延迟的漂移率
    
    # 流动性不足惩罚
    insufficient_liquidity_penalty_bps=50.0,    # 流动性不足基础惩罚
    extrapolation_ratio_limit=3.0,              # 外推比率限制
    no_liquidity_penalty_bps=200.0              # 无流动性惩罚
)
```

**参数详解**:

| 参数 | 说明 | 典型值 | 范围 |
|------|------|--------|------|
| `fee_taker_bps` | Taker 手续费 | 5.0 | 0-20 |
| `expected_exec_time_ms` | 预期执行时间 | 100.0 | 50-500 |
| `drift_rate_per_rvol` | 波动率漂移率 | 0.5 | 0-2.0 |
| `bc_penalty_factor` | 可信度惩罚 | 20.0 | 10-50 |
| `lcd_penalty_bps` | 流动性危机惩罚 | 50.0 | 20-200 |
| `withdraw_penalty_factor` | 提现惩罚 | 10.0 | 5-30 |
| `rate_limit_penalty_factor` | 限速惩罚 | 30.0 | 10-100 |
| `latency_drift_rate_per_ms` | 延迟漂移率 | 0.1 | 0.05-0.5 |

### MarketState

当前市场状态。

```python
state = stg.MarketState(
    rvol_bps_1s=50.0,          # 1秒实际波动率（bps）
    bc=0.9,                     # 订单簿可信度 [0, 1]
    lcd=False,                  # 流动性危机检测器（True=危机）
    withdraw_score=0.0,         # 提现分数 [0, 1]（越高越差）
    rate_limit_usage=0.1,       # 限速使用率 [0, 1]
    latency_p95_ms=10.0         # P95 延迟（毫秒）
)
```

**字段说明**:

- **rvol_bps_1s**: 实际波动率，通过短期价格变化计算
- **bc** (book credibility): 订单簿可信度
  - 1.0 = 完全可信
  - 0.5 = 可信度一般
  - 0.0 = 不可信
- **lcd** (liquidity crash detector): 检测异常流动性枯竭
- **withdraw_score**: 提现压力指标
  - 0.0 = 无压力
  - 1.0 = 极大压力
- **rate_limit_usage**: API 限速使用率
- **latency_p95_ms**: P95 网络延迟

---

## 使用示例

### 示例 1: 基础订单簿操作

```python
from nautilus_trader.core import nautilus_pyo3

stg = nautilus_pyo3.stg

# 创建订单簿
book = stg.LocalL2Book(top_k=50)

# 构建订单簿（买方）
book.update(stg.Side.buy(), 100.00, 50.0)
book.update(stg.Side.buy(), 99.95, 80.0)
book.update(stg.Side.buy(), 99.90, 100.0)

# 构建订单簿（卖方）
book.update(stg.Side.sell(), 100.05, 60.0)
book.update(stg.Side.sell(), 100.10, 90.0)
book.update(stg.Side.sell(), 100.15, 120.0)

# 查询最佳价格
stats = book.best()
print(f"最佳买价: {stats.bid1:.2f}")
print(f"最佳卖价: {stats.ask1:.2f}")
print(f"中间价: {stats.mid:.4f}")
print(f"价差: {stats.spread_bps:.2f} bps")
```

### 示例 2: 使用 Microprice 预测价格

```python
# 假设订单簿买卖不平衡
book.update(stg.Side.buy(), 100.0, 100.0)  # 大买单
book.update(stg.Side.sell(), 100.10, 30.0)  # 小卖单

# 计算 microprice
mp = book.microprice()
stats = book.best()

print(f"中间价: {stats.mid:.4f}")
print(f"Microprice: {mp:.4f}")

# 计算偏离度
dev = book.microprice_deviation_bps()
print(f"偏离度: {dev:.2f} bps")

if dev > 0:
    print("→ 买压更大，价格可能上涨")
elif dev < 0:
    print("→ 卖压更大，价格可能下跌")
else:
    print("→ 买卖平衡")
```

### 示例 3: 计算交易成本

```python
# 场景：想要买入 200 单位
qty = 200.0

# 计算 impact
result = book.impact(
    side=stg.Side.buy(),
    qty=qty,
    cushion_ticks=1  # 1 跳保护
)

if result.ok:
    print(f"VWAP 价格: {result.vwap_price:.4f}")
    print(f"滑点成本: {result.slip_bps:.2f} bps")
    print(f"使用层级: {result.used_level}")
    print(f"建议限价: {result.limit_price:.4f}")
    
    # 计算总成本（美元）
    mid_price = book.best().mid
    total_cost = (result.vwap_price - mid_price) * qty
    print(f"相对中间价的总成本: ${total_cost:.2f}")
else:
    print("流动性不足，无法完成交易！")
```

### 示例 4: 动态成本估计（风险管理）

```python
# 当前持仓
position_size = 1000.0

# 构建市场状态
market_state = stg.MarketState(
    rvol_bps_1s=80.0,      # 高波动
    bc=0.7,                 # 可信度一般
    lcd=False,              # 无流动性危机
    withdraw_score=0.2,     # 轻微提现压力
    rate_limit_usage=0.3,   # 30% 限速使用
    latency_p95_ms=25.0     # 25ms 延迟
)

# 估算紧急清仓成本
bailout_cost = book.bailout_cost_bps_est(
    qty=position_size,
    side=stg.Side.sell(),
    market_state=market_state
)

print(f"持仓: {position_size} 单位")
print(f"预估清仓成本: {bailout_cost:.2f} bps")

# 转换为美元
mid_price = book.best().mid
cost_in_dollars = (bailout_cost / 10000) * mid_price * position_size
print(f"预估清仓成本: ${cost_in_dollars:.2f}")

# 风险判断
if bailout_cost > 50:
    print("⚠️  清仓成本较高，建议减少持仓")
elif bailout_cost > 100:
    print("🚨 清仓成本很高，立即减仓！")
```

### 示例 5: 自定义成本配置

```python
# 创建激进的成本估计配置（适合高频交易）
aggressive_config = stg.CostEstimatorConfig(
    fee_taker_bps=3.0,                      # 较低手续费
    expected_exec_time_ms=50.0,             # 快速执行
    drift_rate_per_rvol=0.3,                # 低漂移
    bc_penalty_factor=10.0,                 # 低可信度惩罚
    lcd_penalty_bps=30.0,
    withdraw_penalty_factor=5.0,
    rate_limit_penalty_factor=15.0,
    latency_drift_rate_per_ms=0.05,
    insufficient_liquidity_penalty_bps=30.0,
    extrapolation_ratio_limit=5.0,
    no_liquidity_penalty_bps=150.0
)

# 应用配置
book.set_cost_config(aggressive_config)

# 使用新配置估算成本
cost = book.bailout_cost_bps_est(
    qty=500.0,
    side=stg.Side.sell(),
    market_state=market_state
)

print(f"激进策略成本估计: {cost:.2f} bps")

# 创建保守配置（适合低频/大单）
conservative_config = stg.CostEstimatorConfig(
    fee_taker_bps=8.0,                      # 考虑更高手续费
    expected_exec_time_ms=200.0,            # 慢速执行
    drift_rate_per_rvol=0.8,                # 高漂移
    bc_penalty_factor=40.0,                 # 高可信度惩罚
    lcd_penalty_bps=100.0,
    withdraw_penalty_factor=20.0,
    rate_limit_penalty_factor=50.0,
    latency_drift_rate_per_ms=0.2,
    insufficient_liquidity_penalty_bps=80.0,
    extrapolation_ratio_limit=2.0,
    no_liquidity_penalty_bps=300.0
)

book.set_cost_config(conservative_config)
cost_conservative = book.bailout_cost_bps_est(
    qty=500.0,
    side=stg.Side.sell(),
    market_state=market_state
)

print(f"保守策略成本估计: {cost_conservative:.2f} bps")
```

### 示例 6: 实时监控订单簿

```python
import time

class BookMonitor:
    def __init__(self, book):
        self.book = book
        self.last_mid = None
        
    def update_and_monitor(self, side, price, size):
        """更新订单簿并监控变化"""
        self.book.update(side, price, size)
        
        # 获取最新状态
        stats = self.book.best()
        mp = self.book.microprice()
        dev = self.book.microprice_deviation_bps()
        
        # 检测价格变化
        if self.last_mid and stats.mid != self.last_mid:
            change = stats.mid - self.last_mid
            change_bps = (change / self.last_mid) * 10000
            print(f"价格变动: {change:+.4f} ({change_bps:+.2f} bps)")
        
        self.last_mid = stats.mid
        
        # 报告状态
        print(f"买一:{stats.bid1:.4f} 卖一:{stats.ask1:.4f} "
              f"中:{stats.mid:.4f} MP:{mp:.4f} 偏离:{dev:+.2f}bps "
              f"价差:{stats.spread_bps:.2f}bps")

# 使用
book = stg.LocalL2Book(top_k=50)
monitor = BookMonitor(book)

# 模拟订单簿更新
monitor.update_and_monitor(stg.Side.buy(), 100.0, 100.0)
monitor.update_and_monitor(stg.Side.sell(), 100.10, 80.0)
monitor.update_and_monitor(stg.Side.buy(), 100.05, 50.0)  # 价格上移
```

### 示例 7: 订单拆分优化

```python
def optimize_order_split(book, total_qty, max_impact_bps=10.0):
    """
    优化订单拆分，避免过大的市场冲击
    
    Args:
        book: LocalL2Book 实例
        total_qty: 总数量
        max_impact_bps: 最大可接受冲击（bps）
    
    Returns:
        list of (qty, expected_price) tuples
    """
    splits = []
    remaining = total_qty
    
    while remaining > 0:
        # 二分查找最大可接受数量
        low, high = 1.0, remaining
        best_qty = low
        
        while high - low > 1.0:
            mid = (low + high) / 2
            result = book.impact(
                side=stg.Side.buy(),
                qty=mid,
                cushion_ticks=0
            )
            
            if result.ok and result.slip_bps <= max_impact_bps:
                best_qty = mid
                low = mid
            else:
                high = mid
        
        # 添加这一批
        result = book.impact(
            side=stg.Side.buy(),
            qty=best_qty,
            cushion_ticks=1
        )
        
        splits.append((best_qty, result.limit_price))
        remaining -= best_qty
        
        # 如果剩余太少，直接加入
        if remaining < 10:
            if remaining > 0:
                result = book.impact(
                    side=stg.Side.buy(),
                    qty=remaining,
                    cushion_ticks=1
                )
                splits.append((remaining, result.limit_price))
            break
    
    return splits

# 使用
book = stg.LocalL2Book(top_k=50)

# 构建订单簿
for i in range(10):
    book.update(stg.Side.sell(), 100.0 + i * 0.05, 50.0)

# 优化 500 单位的订单拆分
total_qty = 500.0
splits = optimize_order_split(book, total_qty, max_impact_bps=8.0)

print(f"总数量 {total_qty} 拆分为 {len(splits)} 批:")
for i, (qty, price) in enumerate(splits, 1):
    print(f"  批次 {i}: {qty:.2f} 单位 @ {price:.4f}")
```

---

## 最佳实践

### 1. 订单簿维护

#### ✅ 推荐做法

```python
# 设置合理的 top_k
book = stg.LocalL2Book(top_k=50)  # 对大多数场景足够

# 定期清理过时数据
# (通过更新为 0 删除)
book.update(stg.Side.buy(), old_price, 0.0)
```

#### ❌ 避免

```python
# 不要设置过大的 top_k（浪费内存）
book = stg.LocalL2Book(top_k=1000)  # 通常不需要

# 不要忘记删除过时的价格层级
# 会导致订单簿数据不准确
```

### 2. 成本估计

#### ✅ 推荐做法

```python
# 根据交易风格调整配置
if trading_style == "high_frequency":
    config = aggressive_config()
elif trading_style == "position":
    config = conservative_config()

book.set_cost_config(config)

# 定期更新 market_state
market_state = get_current_market_state()  # 从实时数据获取
cost = book.bailout_cost_bps_est(qty, side, market_state)
```

#### ❌ 避免

```python
# 不要使用过时的 market_state
market_state = stg.MarketState(...)  # 创建后长时间不更新

# 不要忽略成本估计结果
cost = book.bailout_cost_bps_est(...)
# 没有使用 cost 做风险判断
```

### 3. Microprice 应用

#### ✅ 推荐做法

```python
# 结合多个指标
mp = book.microprice()
mid = book.best().mid
dev = book.microprice_deviation_bps()

# 综合判断
if mp and abs(dev) > 5.0:  # 明显偏离
    if dev > 0:
        signal = "BULLISH"  # 买压
    else:
        signal = "BEARISH"  # 卖压
```

#### ❌ 避免

```python
# 不要单独依赖 microprice
if book.microprice() > book.best().mid:
    # 立即买入 ← 过于简单，风险高
    pass
```

### 4. 性能优化

#### ✅ 推荐做法

```python
# 批量更新后再查询
for update in updates:
    book.update(update.side, update.price, update.size)

# 所有更新完成后再查询
stats = book.best()
mp = book.microprice()
```

#### ❌ 避免

```python
# 不要频繁查询
for update in updates:
    book.update(update.side, update.price, update.size)
    stats = book.best()  # 每次都查询 ← 效率低
```

### 5. 错误处理

#### ✅ 推荐做法

```python
# 检查返回值
mp = book.microprice()
if mp is None:
    # 订单簿数据不足
    logger.warning("Microprice 计算失败，订单簿数据不足")
    return

result = book.impact(side, qty, 0)
if not result.ok:
    # 流动性不足
    logger.warning(f"流动性不足，无法交易 {qty} 单位")
    return
```

#### ❌ 避免

```python
# 不要假设总是成功
mp = book.microprice()
dev = (mp - mid) / mid * 10000  # mp 可能是 None ← 会报错
```

---

## 常见问题

### Q1: 如何选择 top_k 参数？

**A**: 根据使用场景：
- **高频交易**: `top_k=20-50`（只需要最优价格）
- **中低频交易**: `top_k=50-100`（需要更深的流动性）
- **分析工具**: `top_k=100-200`（需要完整市场深度）

### Q2: Microprice 和中间价有什么区别？

**A**: 
- **中间价**: `(bid1 + ask1) / 2` - 简单平均，不考虑数量
- **Microprice**: 考虑数量的加权价格，更准确反映市场压力

**示例**:
```
买方: 100.0 @ 1000 单位
卖方: 100.10 @ 10 单位

中间价 = 100.05
Microprice ≈ 100.001  （更接近买价，因为买方数量大）
```

### Q3: bailout_cost_bps_est 返回的成本太高，是否正常？

**A**: 可能原因：
1. **市场波动大** - 调整 `market_state.rvol_bps_1s`
2. **配置过于保守** - 调整 `CostEstimatorConfig` 参数
3. **流动性不足** - 实际成本确实高
4. **极端市场条件** - 检查 `lcd`, `withdraw_score` 等

**调试方法**:
```python
# 分步查看各项成本
config = book.cost_config()
print(f"基础费用: {config.fee_taker_bps} bps")

result = book.impact(qty, side, 0)
print(f"滑点: {result.slip_bps} bps")

# 逐步放宽配置，观察变化
```

### Q4: 如何处理订单簿数据缺失？

**A**: 
```python
# 始终检查返回值
stats = book.best()
if stats.bid1 == 0 or stats.ask1 == 0:
    print("订单簿数据不完整")
    return

mp = book.microprice()
if mp is None:
    print("无法计算 microprice")
    # 使用中间价作为替代
    price = stats.mid
else:
    price = mp
```

### Q5: 如何提高性能？

**A**: 
1. **减少 top_k**: 只维护需要的层级
2. **批量更新**: 一次更新多条，然后统一查询
3. **缓存结果**: 如果订单簿没变化，重用之前的计算结果
4. **避免频繁配置切换**: `set_cost_config()` 有开销

```python
# 性能优化示例
class OptimizedBook:
    def __init__(self):
        self.book = stg.LocalL2Book(top_k=30)  # 较小的 top_k
        self._cache = {}
        self._version = 0
    
    def update(self, side, price, size):
        self.book.update(side, price, size)
        self._version += 1
        self._cache.clear()  # 失效缓存
    
    def microprice(self):
        key = ('mp', self._version)
        if key not in self._cache:
            self._cache[key] = self.book.microprice()
        return self._cache[key]
```

### Q6: 订单簿更新频率很高，会有性能问题吗？

**A**: 不会。每次 `update()` 操作 < 100ns，可以轻松处理：
- 每秒 10,000+ 次更新
- 实时流式数据
- WebSocket tick 数据

如果性能仍不够，考虑：
1. 使用更小的 `top_k`
2. 实现采样（不是每个 tick 都更新）

### Q7: 如何验证成本估计的准确性？

**A**: 
```python
# 方法 1: 回测对比
estimated_cost = book.bailout_cost_bps_est(qty, side, market_state)
# 执行交易后计算实际成本
actual_cost = calculate_actual_cost(execution_report)
error = abs(estimated_cost - actual_cost)

# 方法 2: 调整配置使估计更准确
if actual_cost > estimated_cost:
    # 估计过于乐观，增加惩罚因子
    config.drift_rate_per_rvol *= 1.2
    config.bc_penalty_factor *= 1.1
```

### Q8: 可以多线程使用同一个 book 吗？

**A**: **不可以**。`LocalL2Book` 不是线程安全的。

**解决方案**:
```python
# 方案 1: 每个线程一个 book
import threading

thread_local = threading.local()

def get_book():
    if not hasattr(thread_local, 'book'):
        thread_local.book = stg.LocalL2Book(top_k=50)
    return thread_local.book

# 方案 2: 使用锁
import threading

book = stg.LocalL2Book(top_k=50)
book_lock = threading.Lock()

with book_lock:
    book.update(side, price, size)
    stats = book.best()
```

---

## 附录

### A. 性能指标

| 操作 | 平均延迟 | P99 延迟 |
|------|----------|----------|
| `update()` | 45 ns | 80 ns |
| `best()` | 38 ns | 60 ns |
| `microprice()` | 22 ns | 40 ns |
| `microprice_deviation_bps()` | 25 ns | 45 ns |
| `impact()` | 61 ns | 120 ns |
| `bailout_cost_bps_est()` | 85 ns | 150 ns |
| `top_k()` | 50 ns | 90 ns |

**测试环境**: M1 Mac, Rust release build, Python 3.12

### B. 术语表

| 术语 | 英文 | 说明 |
|------|------|------|
| 基点 | basis point (bps) | 0.01%，金融常用单位 |
| VWAP | Volume Weighted Average Price | 成交量加权平均价格 |
| 滑点 | slippage | 实际成交价与预期价的差异 |
| 流动性 | liquidity | 可交易的数量和深度 |
| 价格冲击 | price impact | 大单对市场价格的影响 |
| 微观价格 | microprice | 考虑数量的公平价格估计 |
| Taker | taker | 主动成交方（吃单） |
| Maker | maker | 被动成交方（挂单） |

### C. 相关资源

- [NautilusTrader 官方文档](https://nautilustrader.io/docs)
- [源代码](https://github.com/nautechsystems/nautilus_trader)
- [技术规格](STG_INTEGRATION_COMPLETE.md)

---

**版权**: © 2026 NautilusTrader  
**许可**: LGPL-3.0-or-later
