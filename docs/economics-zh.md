# CHAOS 预测市场 — 经济模型

## 概述

CHAOS 预测市场使用 **LMSR（对数市场评分规则）** 自动做市商算法进行链上定价。该算法由 Robin Hanson 发明，所有定价、交易和赔付均为全自动 — 无需订单簿，无需人工做市商。

---

## LMSR 核心公式

```
成本函数:  C(qYes, qNo) = b × ln(exp(qYes/b) + exp(qNo/b))

买入成本 = C(买后状态) - C(买前状态)
卖出收益 = C(卖前状态) - C(卖后状态)

YES 价格 = exp(qYes/b) / (exp(qYes/b) + exp(qNo/b))
         = 1 / (1 + exp(-(qYes - qNo) / b))    ← sigmoid 函数
```

### 参数说明

| 参数 | 合约中的值 | 含义 |
|------|-----------|------|
| `qYes` | `market.yesShares` | 流通中的 YES 份额总量 |
| `qNo` | `market.noShares` | 流通中的 NO 份额总量 |
| `b` | `LMSR_B = 100e18` | **流动性参数** — 越大价格越稳定，越难被操纵 |

---

## 价格如何变动

```
初始状态:      qYes=0,   qNo=0    → YES = 50.0%,  NO = 50.0%
有人买入 YES:  qYes=50,  qNo=0    → YES = 62.2%,  NO = 37.8%
更多人买 YES:  qYes=200, qNo=0    → YES = 88.1%,  NO = 11.9%
```

**买入越多 → 价格越高 → 后续买入越贵。** 这就是 LMSR 的自动做市机制。

---

## 代币流转

```
用户钱包 (CHAOS 代币)
    │
    ├─ buyShares()  ──→ 代币转入合约 ──→ totalDeposited 增加
    │                                    yesShares/noShares 增加
    │                                    价格自动上移
    │
    ├─ sellShares() ←── 代币从合约转出 ←── totalDeposited 减少
    │                                     yesShares/noShares 减少
    │                                     价格自动下移
    │
    └─ claimWinnings() ←── 代币从合约转出 ←── 按持仓比例分配资金池
```

---

## 市场生命周期

```
创建 (Open) → 交易期 → 关闭 (Closed) → 解决 (Resolved) → 领奖
     │              │           │              │              │
     │ CHAOS 扫描    │ closeTime │ owner 调用    │ 用户调用       │
     │ 自动创建市场   │ 到期       │ resolveMarket│ claimWinnings │
     │              │           │              │              │
     │ buyShares()  │           │  取消路径:     │              │
     │ sellShares() │           │  cancelMarket │              │
     │              │           │  → 全额退款    │              │
```

---

## 自动化程度

| 环节 | 是否自动？ | 机制 |
|------|-----------|------|
| **市场创建** | ✅ 自动 | CHAOS 扫描 → 12 条规则 + LLM → `/api/market-seeds` → 数据库 |
| **定价** | ✅ 自动 | 链上 LMSR 计算，零人工干预 |
| **价格变动** | ✅ 自动 | 每笔交易后价格自动调整 |
| **市场关闭** | ✅ 自动 | `closeTime` 到期后自动阻止新交易 |
| **市场解决** | ⚠️ 半自动 | `/api/auto-resolve` 检查 CHAOS 数据判定结果，由 SSE sweep-hook 触发 |
| **奖金发放** | ⚠️ 用户触发 | 用户调用 `claimWinnings()`，但计算过程全自动 |

---

## 完整案例

```
市场: "BTC 本周会超过 $75K 吗？"

1. 初始状态
   资金池 = 0 CHAOS, YES = 50%, NO = 50%

2. Alice 买入 100 份 YES
   LMSR 计算成本 = 54.3 CHAOS
   资金池 = 54.3, YES = 62%, NO = 38%

3. Bob 买入 200 份 NO
   LMSR 计算成本 = 92.1 CHAOS
   资金池 = 146.4, YES = 45%, NO = 55%

4. Carol 买入 50 份 YES
   LMSR 计算成本 = 21.8 CHAOS
   资金池 = 168.2, YES = 48%, NO = 52%

5. Alice 卖出 50 份 YES（改变主意）
   LMSR 计算收益 = 24.1 CHAOS
   资金池 = 144.1, YES = 44%, NO = 56%

6. 到期，CHAOS 数据显示 BTC = $72K → 解决为 NO

7. 赔付:
   Bob 持有 200 份 NO（总 NO = 200）
   赔付 = (200/200) × 144.1 = 144.1 CHAOS ← Bob 拿走全部资金池

   Alice 剩余 50 份 YES → 亏损，无赔付
   Carol 50 份 YES → 亏损，无赔付
```

---

## 定价经济学

### 1. 买卖价差（做市商利润）

```
买入 100 份:  花费 ≈ 54 CHAOS
立刻卖出 100 份: 收回 ≈ 53.8 CHAOS
价差 ≈ 0.2 CHAOS（滑点/价差）
```

### 2. 大额交易成本非线性增长（防操纵）

```
买入 10 份:   花费 ≈ 5 CHAOS    (均价 0.50)
买入 100 份:  花费 ≈ 54 CHAOS   (均价 0.54)
买入 1000 份: 花费 ≈ 762 CHAOS  (均价 0.76)
→ 大户无法低成本操纵价格
```

### 3. 永远有流动性

即使价格到 99%，仍然可以买入 — 只是成本极高（接近 1:1 代币换份额）。

---

## 流动性参数 `b = 100` 的含义

- **做市商最大亏损** = `b × ln(2)` ≈ 69.3 个代币
- 这是为市场提供初始流动性的"成本"
- `b` 越大 → 价格越稳定，单笔交易越难移动价格
- 当前 `b=100e18` 适合中小规模预测市场

---

## 双轨交易系统

NewsPredict 运行**两套并行交易系统**：

```
┌─────────────────────────────────────────────┐
│           NewsPredict 交易层                  │
├─────────────────────┬───────────────────────┤
│   虚拟交易 (数据库)   │   链上交易 (智能合约)    │
│                     │                       │
│ POST /api/trades    │ buyShares() 链上调用    │
│ trade-executor.ts   │ ChaosPredictionMarket  │
│ PostgreSQL 记录      │ BSC 区块链记录          │
│ 虚拟余额（美元计价）   │ CHAOS 代币（真实资产）   │
│ LMSR (TypeScript)   │ LMSR (Solidity)        │
│                     │                       │
│ 用途: 免费体验        │ 用途: 真金白银交易       │
│ 无需钱包             │ 需要 Web3 钱包          │
└─────────────────────┴───────────────────────┘

OrderPanel 组件可切换:
  [虚拟积分] [链上交易 (CHAOS)]
```

---

## 收入模型

当前合约**没有内置手续费**，所有代币在用户之间流转。未来可选的收入方案：

| 方案 | 实现难度 | 说明 |
|------|----------|------|
| 交易手续费 | 低 | buyShares/sellShares 收取 0.5-2% 到国库 |
| 市场创建费 | 低 | createMarket 要求押金 |
| 解决手续费 | 低 | claimWinnings 扣除小额费用 |
| 代币通缩 | 已内置 | ChaosToken 可销毁（burn），减少供给推高价值 |

目前是**零手续费模式**，适合早期获客阶段。

---

## 偿付能力保证

Pro-rata（按比例）赔付模型确保合约**永远不会资不抵债**：

```
总赔付 = Σ (用户份额 / 赢方总份额 × totalDeposited)
       = (totalDeposited / 赢方总份额) × Σ 用户份额
       = (totalDeposited / 赢方总份额) × 赢方总份额
       = totalDeposited ✓
```

合约始终持有足够的代币来支付所有赢家。

---

## 智能合约安全

经过四轮安全审计，主要保护机制：

| 保护措施 | 实现方式 |
|----------|---------|
| 重入攻击防护 | 所有状态变更函数使用 `nonReentrant` |
| 整数溢出防护 | Solidity 0.8.24 内置检查 + `MAX_TOTAL_SHARES` 上限 |
| 价格操纵防护 | 链上 LMSR 定价，无链下信任 |
| 双重领取防护 | 每个持仓的 `claimed` 标志 |
| 滑点保护 | 买入 `maxCost` / 卖出 `minProceeds` |
| Gas 耗尽防护 | 循环迭代上限（128 次） |
| 紧急提款 | `emergencyWithdraw` + 已承诺资金保护 |
| 紧急暂停 | Token 级别 Pausable |
