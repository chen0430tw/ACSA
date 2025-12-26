"""
SOSA - Spark Seed Self-Organizing Algorithm
火种源自组织算法

核心机制:
1. 稀疏马尔科夫链 - 高层模式状态转移
2. Binary-Twin双态数块 - 连续+离散特征
3. 时间窗口机制 - 事件聚合
4. 组合数编码 - 行为空间占用度
5. 探索-固化平衡 - 自适应决策

应用: 训练监控、异常检测、自动纠错

作者: 430 + GPT-5.1 Thinking
改造: chen0430tw (APT集成)
"""

import numpy as np
from typing import Dict, List, Tuple, Optional, Set
from dataclasses import dataclass, field
from collections import deque, defaultdict
import logging
import time

logger = logging.getLogger(__name__)


@dataclass
class Event:
    """事件记录"""
    timestamp: float
    event_type: str  # 事件类型: error/warning/metric/checkpoint
    severity: float  # 严重程度 [0, 1]
    attributes: Dict  # 附加属性
    value: Optional[float] = None  # 数值型事件的值


@dataclass
class BinaryTwin:
    """Binary-Twin 双态数块"""
    # 连续部分 x_cont
    avg_energy: float  # 平均局部势能 [0,1]
    diversity: float  # 行为多样性
    size_norm: float  # 窗口规模归一化
    
    # 离散部分 b_bits (3位)
    bit0: bool  # 是否存在高能行为 (>0.8)
    bit1: bool  # 行为模式 >= 3种
    bit2: bool  # 窗口事件数 >= 10
    
    def to_vector(self) -> np.ndarray:
        """转为向量表示"""
        cont = np.array([self.avg_energy, self.diversity, self.size_norm])
        disc = np.array([float(self.bit0), float(self.bit1), float(self.bit2)])
        return np.concatenate([cont, disc])
    
    def to_state_id(self) -> str:
        """转为状态ID (用于马尔科夫链)"""
        # 离散化连续值
        e_bin = int(self.avg_energy * 10)  # 0-10
        d_bin = int(self.diversity * 10)
        s_bin = int(self.size_norm * 10)
        
        # 组合离散位
        bits = (int(self.bit0) << 2) | (int(self.bit1) << 1) | int(self.bit2)
        
        return f"S_{e_bin}_{d_bin}_{s_bin}_{bits}"


class SparseMarkov:
    """稀疏马尔科夫链"""
    
    def __init__(self, num_states: int = 1000):
        """
        Args:
            num_states: 状态空间大小
        """
        self.num_states = num_states
        
        # 稀疏转移矩阵 {(s_from, s_to): probability}
        self.transitions: Dict[Tuple[str, str], float] = {}
        
        # 状态访问计数
        self.state_counts: Dict[str, int] = defaultdict(int)
        
        # 状态索引映射
        self.state_to_idx: Dict[str, int] = {}
        self.idx_to_state: Dict[int, str] = {}
        self.next_idx = 0
    
    def _get_state_idx(self, state_id: str) -> int:
        """获取或创建状态索引"""
        if state_id not in self.state_to_idx:
            if self.next_idx >= self.num_states:
                # 状态空间已满，使用最少访问的状态
                min_state = min(self.state_counts.items(), key=lambda x: x[1])[0]
                idx = self.state_to_idx[min_state]
                
                # 清理旧状态
                del self.state_to_idx[min_state]
                del self.state_counts[min_state]
                
                self.state_to_idx[state_id] = idx
                self.idx_to_state[idx] = state_id
            else:
                idx = self.next_idx
                self.state_to_idx[state_id] = idx
                self.idx_to_state[idx] = state_id
                self.next_idx += 1
        
        return self.state_to_idx[state_id]
    
    def update(self, s_from: str, s_to: str):
        """更新转移概率"""
        self.state_counts[s_from] += 1
        
        key = (s_from, s_to)
        self.transitions[key] = self.transitions.get(key, 0) + 1
    
    def get_transition_prob(self, s_from: str, s_to: str) -> float:
        """获取转移概率"""
        count_from = self.state_counts.get(s_from, 0)
        if count_from == 0:
            return 0.0
        
        count_transition = self.transitions.get((s_from, s_to), 0)
        return count_transition / count_from
    
    def step(self, current_state: str) -> Dict[str, float]:
        """
        马尔科夫步: 返回下一步状态分布
        
        Returns:
            {state_id: probability}
        """
        count_from = self.state_counts.get(current_state, 0)
        if count_from == 0:
            return {}
        
        # 收集所有可能的下一状态
        next_states = {}
        for (s_from, s_to), count in self.transitions.items():
            if s_from == current_state:
                next_states[s_to] = count / count_from
        
        return next_states
    
    def normalize_distribution(self, dist: Dict[str, float]) -> Dict[str, float]:
        """归一化分布"""
        total = sum(dist.values())
        if total == 0:
            return {}
        
        return {k: v / total for k, v in dist.items()}


class SOSA:
    """
    SOSA - 火种源自组织算法
    
    核心功能:
    - 时间窗口事件聚合
    - Binary-Twin特征提取
    - 稀疏马尔科夫链演化
    - 探索-固化平衡决策
    """
    
    def __init__(
        self,
        dt_window: float = 5.0,
        M_groups: int = 10,
        exploration_weight: float = 0.5
    ):
        """
        Args:
            dt_window: 时间窗口大小(秒)
            M_groups: 行为组数量
            exploration_weight: 探索权重 [0,1]
        """
        self.dt_window = dt_window
        self.M_groups = M_groups
        self.exploration_weight = exploration_weight
        
        # 马尔科夫链
        self.markov = SparseMarkov(num_states=1000)
        
        # 时间窗口缓冲
        self.window_buffer: deque = deque()
        
        # 当前状态
        self.current_state: Optional[str] = None
        
        # 历史记录
        self.state_history: List[Tuple[float, str, BinaryTwin]] = []
        
        # 行为组统计
        self.behavior_groups: Dict[int, Set[str]] = defaultdict(set)
        
        # 吸引子 (固化的模式)
        self.attractors: Dict[str, float] = {}
        
        logger.info(
            f"SOSA初始化: dt_window={dt_window}, "
            f"M_groups={M_groups}, exploration_weight={exploration_weight}"
        )
    
    # ==================== 事件处理 ====================
    
    def add_event(self, event: Event):
        """添加事件到缓冲区"""
        self.window_buffer.append(event)
        
        # 检查是否需要刷新窗口
        if len(self.window_buffer) > 0:
            oldest = self.window_buffer[0]
            newest = self.window_buffer[-1]
            
            if newest.timestamp - oldest.timestamp >= self.dt_window:
                self._flush_window()
    
    def _flush_window(self):
        """刷新时间窗口"""
        if len(self.window_buffer) == 0:
            return
        
        # 提取窗口事件
        window_events = list(self.window_buffer)
        self.window_buffer.clear()
        
        # 生成Binary-Twin
        twin = self._generate_binary_twin(window_events)
        
        # 转为状态ID
        new_state = twin.to_state_id()
        
        # 更新马尔科夫链
        if self.current_state is not None:
            self.markov.update(self.current_state, new_state)
        
        # 记录历史
        timestamp = window_events[-1].timestamp
        self.state_history.append((timestamp, new_state, twin))
        
        # 更新当前状态
        self.current_state = new_state
        
        # 更新行为组
        self._update_behavior_groups(window_events)
        
        # 检查是否形成吸引子
        self._check_attractor(new_state)
        
        logger.debug(f"窗口刷新: state={new_state}, events={len(window_events)}")
    
    def _generate_binary_twin(self, events: List[Event]) -> BinaryTwin:
        """生成Binary-Twin特征"""
        n = len(events)
        
        if n == 0:
            return BinaryTwin(
                avg_energy=0.0, diversity=0.0, size_norm=0.0,
                bit0=False, bit1=False, bit2=False
            )
        
        # 连续部分
        severities = [e.severity for e in events]
        avg_energy = np.mean(severities)
        
        # 多样性: 不同事件类型的比例
        types = set(e.event_type for e in events)
        diversity = len(types) / max(n, 1)
        
        # 规模归一化
        size_norm = min(n / 100.0, 1.0)  # 假设100为大窗口
        
        # 离散部分
        bit0 = any(s > 0.8 for s in severities)  # 高能事件
        bit1 = len(types) >= 3  # 多样模式
        bit2 = n >= 10  # 大窗口
        
        return BinaryTwin(
            avg_energy=avg_energy,
            diversity=diversity,
            size_norm=size_norm,
            bit0=bit0,
            bit1=bit1,
            bit2=bit2
        )
    
    def _update_behavior_groups(self, events: List[Event]):
        """更新行为组"""
        # 简化: 按事件类型分组
        for event in events:
            group_id = hash(event.event_type) % self.M_groups
            self.behavior_groups[group_id].add(event.event_type)
    
    def _check_attractor(self, state_id: str):
        """检查是否形成吸引子"""
        count = self.markov.state_counts.get(state_id, 0)
        
        # 频繁访问的状态成为吸引子
        if count > 10:
            self.attractors[state_id] = count / sum(self.markov.state_counts.values())
    
    # ==================== 组合数编码 ====================
    
    def compute_combination_occupancy(self) -> float:
        """
        计算组合占用度 c_r ∈ [0,1]
        
        c_r = 已占用行为组合数 / 总可能组合数
        1 - c_r = 探索自由度
        """
        # 统计非空的行为组
        occupied_groups = sum(1 for g in self.behavior_groups.values() if len(g) > 0)
        
        c_r = occupied_groups / self.M_groups
        
        return c_r
    
    # ==================== 决策: 探索 vs 固化 ====================
    
    def decide_next_action(self) -> Dict[str, float]:
        """
        决策下一步行动
        
        探索因子 = (1 - c_r) × (1 - 0.5×diversity) × (0.5 + 0.5×(1-size))
        
        Returns:
            {
                'exploration_factor': float,
                'recommended_state': str,
                'confidence': float
            }
        """
        if self.current_state is None:
            return {
                'exploration_factor': 1.0,
                'recommended_state': None,
                'confidence': 0.0
            }
        
        # 获取当前twin
        _, _, current_twin = self.state_history[-1]
        
        # 计算探索因子
        c_r = self.compute_combination_occupancy()
        
        explore_factor = (
            (1 - c_r) *
            (1 - 0.5 * current_twin.diversity) *
            (0.5 + 0.5 * (1 - current_twin.size_norm))
        )
        
        # 获取下一步状态分布
        next_dist = self.markov.step(self.current_state)
        
        if not next_dist:
            return {
                'exploration_factor': explore_factor,
                'recommended_state': None,
                'confidence': 0.0
            }
        
        # 混合策略: 探索 + 吸引子
        if self.attractors:
            # 找最强吸引子
            strongest_attractor = max(self.attractors.items(), key=lambda x: x[1])
            attractor_state, attractor_strength = strongest_attractor
            
            # 混合
            if attractor_state in next_dist:
                # 加权组合
                for state in next_dist:
                    if state == attractor_state:
                        next_dist[state] = (
                            explore_factor * next_dist[state] +
                            (1 - explore_factor) * attractor_strength
                        )
        
        # 归一化
        next_dist = self.markov.normalize_distribution(next_dist)
        
        # 选择最可能的状态
        if next_dist:
            recommended_state = max(next_dist.items(), key=lambda x: x[1])
            
            return {
                'exploration_factor': explore_factor,
                'recommended_state': recommended_state[0],
                'confidence': recommended_state[1],
                'c_r': c_r,
                'diversity': current_twin.diversity
            }
        
        return {
            'exploration_factor': explore_factor,
            'recommended_state': None,
            'confidence': 0.0
        }
    
    # ==================== 分析与诊断 ====================
    
    def get_statistics(self) -> Dict:
        """获取统计信息"""
        stats = {
            'num_states': len(self.markov.state_to_idx),
            'num_transitions': len(self.markov.transitions),
            'num_attractors': len(self.attractors),
            'current_state': self.current_state,
            'combination_occupancy': self.compute_combination_occupancy(),
            'history_length': len(self.state_history),
            'behavior_groups_active': sum(
                1 for g in self.behavior_groups.values() if len(g) > 0
            )
        }
        
        if self.state_history:
            recent_energies = [
                twin.avg_energy
                for _, _, twin in self.state_history[-10:]
            ]
            stats['recent_avg_energy'] = np.mean(recent_energies)
            stats['recent_max_energy'] = np.max(recent_energies)
        
        return stats
    
    def get_top_states(self, top_k: int = 10) -> List[Tuple[str, int]]:
        """获取访问最频繁的状态"""
        sorted_states = sorted(
            self.markov.state_counts.items(),
            key=lambda x: x[1],
            reverse=True
        )
        return sorted_states[:top_k]
    
    def detect_anomaly(self, threshold: float = 0.9) -> bool:
        """
        检测异常
        
        Args:
            threshold: 高能量阈值
        
        Returns:
            是否检测到异常
        """
        if not self.state_history:
            return False
        
        _, _, current_twin = self.state_history[-1]
        
        # 高能量 + 低多样性 = 异常
        if current_twin.avg_energy > threshold and current_twin.diversity < 0.3:
            return True
        
        # 急剧变化 = 异常
        if len(self.state_history) >= 2:
            _, _, prev_twin = self.state_history[-2]
            energy_jump = abs(current_twin.avg_energy - prev_twin.avg_energy)
            
            if energy_jump > 0.5:
                return True
        
        return False
    
    def print_report(self):
        """打印报告"""
        stats = self.get_statistics()
        
        print("=" * 60)
        print("SOSA 报告")
        print("=" * 60)
        
        print(f"\n状态空间:")
        print(f"  总状态数: {stats['num_states']}")
        print(f"  转移数: {stats['num_transitions']}")
        print(f"  吸引子数: {stats['num_attractors']}")
        
        print(f"\n当前状态: {stats['current_state']}")
        print(f"  组合占用度 c_r: {stats['combination_occupancy']:.3f}")
        print(f"  探索自由度: {1 - stats['combination_occupancy']:.3f}")
        
        if 'recent_avg_energy' in stats:
            print(f"\n近期能量:")
            print(f"  平均: {stats['recent_avg_energy']:.3f}")
            print(f"  最大: {stats['recent_max_energy']:.3f}")
        
        print(f"\n决策信息:")
        decision = self.decide_next_action()
        print(f"  探索因子: {decision['exploration_factor']:.3f}")
        print(f"  推荐状态: {decision.get('recommended_state')}")
        print(f"  置信度: {decision.get('confidence', 0):.3f}")
        
        print(f"\n异常检测:")
        is_anomaly = self.detect_anomaly()
        print(f"  当前异常: {'是' if is_anomaly else '否'}")
        
        print("\n访问最频繁的状态 (Top 5):")
        top_states = self.get_top_states(5)
        for state, count in top_states:
            print(f"  {state}: {count} 次")
        
        print("=" * 60)


if __name__ == "__main__":
    # 测试代码
    logging.basicConfig(level=logging.INFO)
    
    print("=== SOSA 算法测试 ===\n")
    
    # 创建SOSA实例
    sosa = SOSA(dt_window=2.0, M_groups=10)
    
    # 模拟训练事件
    print("模拟训练事件流...")
    
    base_time = time.time()
    
    # 正常训练
    for i in range(30):
        event = Event(
            timestamp=base_time + i * 0.5,
            event_type='metric',
            severity=0.2 + 0.1 * np.random.randn(),
            attributes={'loss': 1.0 - i * 0.02},
            value=1.0 - i * 0.02
        )
        sosa.add_event(event)
    
    # 注入异常
    print("\n注入异常事件...")
    for i in range(5):
        event = Event(
            timestamp=base_time + 30 * 0.5 + i * 0.5,
            event_type='error',
            severity=0.9,
            attributes={'error': 'NaN detected'},
            value=None
        )
        sosa.add_event(event)
    
    # 恢复正常
    print("恢复正常...")
    for i in range(20):
        event = Event(
            timestamp=base_time + 35 * 0.5 + i * 0.5,
            event_type='metric',
            severity=0.15,
            attributes={'loss': 0.5},
            value=0.5
        )
        sosa.add_event(event)
    
    # 刷新剩余事件
    sosa._flush_window()
    
    # 打印报告
    print("\n")
    sosa.print_report()
